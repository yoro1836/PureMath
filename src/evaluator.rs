use crate::ast::{BinOp, Expr, Program, Stmt};
use crate::diagnostics::Diagnostic;
use crate::env::Environment;
use crate::simplifier;
use crate::value::{exact_integer_sqrt, value_to_expr, FunctionValue, Rational, Value};
use std::collections::HashMap;
use std::path::Path;

pub struct Evaluator<'a> {
    env: &'a mut Environment,
    call_depth: usize,
    max_call_depth: usize,
    max_sum_terms: usize,
}

impl<'a> Evaluator<'a> {
    pub const DEFAULT_MAX_CALL_DEPTH: usize = 256;
    pub const DEFAULT_MAX_SUM_TERMS: usize = 100_000;

    pub fn new(env: &'a mut Environment) -> Self {
        Self {
            env,
            call_depth: 0,
            max_call_depth: Self::DEFAULT_MAX_CALL_DEPTH,
            max_sum_terms: Self::DEFAULT_MAX_SUM_TERMS,
        }
    }

    pub fn with_limits(
        env: &'a mut Environment,
        max_call_depth: usize,
        max_sum_terms: usize,
    ) -> Self {
        Self {
            env,
            call_depth: 0,
            max_call_depth,
            max_sum_terms,
        }
    }

    pub fn execute_program(mut self, program: &Program) -> Result<Value, Diagnostic> {
        let mut last = Value::Unit;
        for stmt in &program.statements {
            last = self.execute(stmt, Path::new("."))?;
        }
        Ok(last)
    }

    pub fn execute(&mut self, stmt: &Stmt, base_dir: &Path) -> Result<Value, Diagnostic> {
        match stmt {
            Stmt::Definition {
                name, params, body, ..
            } => {
                if params.is_empty() {
                    let value = self.eval_expr(body, &HashMap::new())?;
                    self.env.define_value(name.clone(), value.clone())?;
                    Ok(value)
                } else {
                    self.env.define_function(
                        name.clone(),
                        FunctionValue {
                            params: params.clone(),
                            body: body.clone(),
                        },
                    )?;
                    Ok(self.env.get(name).cloned().unwrap_or(Value::Unit))
                }
            }
            Stmt::Expression(expr) => self.eval_expr(expr, &HashMap::new()),
            Stmt::Import { module, .. } => {
                crate::module::load_module(module, self.env, base_dir)?;
                Ok(Value::Unit)
            }
            Stmt::Print { span, .. } => Err(Diagnostic::at(
                "print is a runtime operation; execute it through puremath::Runtime",
                *span,
            )),
        }
    }

    pub(crate) fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, Diagnostic> {
        self.eval_expr(expr, &HashMap::new())
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        locals: &HashMap<String, Value>,
    ) -> Result<Value, Diagnostic> {
        match expr {
            Expr::Integer(n, _) => Ok(Value::Rational(Rational::integer(*n))),
            Expr::Rational {
                numerator,
                denominator,
                ..
            } => {
                let a = self.eval_expr(numerator, locals)?;
                let b = self.eval_expr(denominator, locals)?;
                match (a, b) {
                    (Value::Rational(x), Value::Rational(y)) => {
                        x.div(&y).map(Value::Rational).map_err(Diagnostic::new)
                    }
                    (va, vb) => Ok(Value::Symbolic(Expr::Rational {
                        numerator: Box::new(value_to_expr(&va, numerator)),
                        denominator: Box::new(value_to_expr(&vb, denominator)),
                        span: expr.span(),
                    })),
                }
            }
            Expr::Symbol { name, span } => Ok(locals
                .get(name)
                .cloned()
                .or_else(|| self.env.get(name).cloned())
                .unwrap_or_else(|| {
                    Value::Symbolic(Expr::Symbol {
                        name: name.clone(),
                        span: *span,
                    })
                })),
            Expr::Unary {
                expr: inner, span, ..
            } => match self.eval_expr(inner, locals)? {
                Value::Rational(v) => Rational::integer(-1)
                    .mul(&v)
                    .map(Value::Rational)
                    .map_err(Diagnostic::new),
                Value::Symbolic(v) => Ok(Value::Symbolic(Expr::Unary {
                    op: crate::ast::UnaryOp::Neg,
                    expr: Box::new(v),
                    span: *span,
                })),
                other => Err(Diagnostic::at(format!("cannot negate {}", other), *span)),
            },
            Expr::Binary { op, lhs, rhs, span } => self.eval_binary(*op, lhs, rhs, *span, locals),
            Expr::Call { callee, args, span } => {
                if let Expr::Symbol { name, .. } = callee.as_ref() {
                    if name.starts_with('\\') {
                        return self.eval_builtin(name, args, *span, locals);
                    }
                }
                let callable = self.eval_expr(callee, locals)?;
                match callable {
                    Value::Function(function) => {
                        if function.params.len() != args.len() {
                            return Err(Diagnostic::at(
                                format!(
                                    "arity mismatch: expected {}, got {}",
                                    function.params.len(),
                                    args.len()
                                ),
                                *span,
                            ));
                        }
                        if self.call_depth >= self.max_call_depth {
                            return Err(Diagnostic::at(
                                "maximum function call depth exceeded",
                                *span,
                            ));
                        }
                        let mut child = HashMap::new();
                        for (param, arg) in function.params.iter().zip(args) {
                            child.insert(param.clone(), self.eval_expr(arg, locals)?);
                        }
                        self.call_depth += 1;
                        let result = self.eval_expr(&function.body, &child);
                        self.call_depth -= 1;
                        result
                    }
                    Value::Symbolic(callee_expr) => Ok(Value::Symbolic(Expr::Call {
                        callee: Box::new(callee_expr),
                        args: args.clone(),
                        span: *span,
                    })),
                    other => Err(Diagnostic::at(format!("{} is not callable", other), *span)),
                }
            }
            Expr::Set { elements, .. } => Ok(Value::Set(
                elements
                    .iter()
                    .map(|e| self.eval_expr(e, locals))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Expr::Vector { elements, .. } => Ok(Value::Vector(
                elements
                    .iter()
                    .map(|e| self.eval_expr(e, locals))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Expr::Matrix { rows, .. } => Ok(Value::Matrix(
                rows.iter()
                    .map(|row| {
                        row.iter()
                            .map(|e| self.eval_expr(e, locals))
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Expr::Product {
                var,
                lower,
                upper,
                body,
                ..
            } => self.eval_product(var, lower, upper, body, expr, locals),
            Expr::Sum {
                var,
                lower,
                upper,
                body,
                ..
            } => self.eval_sum(var, lower, upper, body, expr, locals),
            Expr::Integral {
                var,
                lower,
                upper,
                body,
                ..
            } => self.eval_integral(var, lower.as_deref(), upper.as_deref(), body, expr, locals),
            Expr::Limit {
                var,
                target,
                body,
                ..
            } => self.eval_limit(var, target, body, expr, locals),
            Expr::Abs { expr: inner, span } => match self.eval_expr(inner, locals)? {
                Value::Rational(v) => Ok(Value::Rational(Rational::new(v.num.checked_abs().ok_or_else(|| Diagnostic::new("integer overflow in absolute value"))?, v.den)
                    .map_err(Diagnostic::new)?)),
                Value::Symbolic(v) => Ok(Value::Symbolic(Expr::Abs {
                    expr: Box::new(v),
                    span: *span,
                })),
                other => Err(Diagnostic::at(format!("cannot take absolute value of {}", other), *span)),
            },
            Expr::Sqrt { expr: inner, .. } => match self.eval_expr(inner, locals)? {
                Value::Rational(v) if v.is_integer() && v.num >= 0 => {
                    let n = v.num as u128;
                    let r = exact_integer_sqrt(n);
                    if r * r == n {
                        Ok(Value::Rational(Rational::integer(r as i128)))
                    } else {
                        Ok(Value::Symbolic(expr.clone()))
                    }
                }
                _ => Ok(Value::Symbolic(expr.clone())),
            },
            Expr::Piecewise { branches, .. } => {
                for branch in branches {
                    match self.eval_expr(&branch.condition, locals)? {
                        Value::Bool(true) => return self.eval_expr(&branch.value, locals),
                        Value::Bool(false) => continue,
                        _ => return Ok(Value::Symbolic(expr.clone())),
                    }
                }
                Err(Diagnostic::new(
                    "piecewise definition has no matching branch",
                ))
            }
            Expr::Opaque { .. } => Ok(Value::Symbolic(expr.clone())),
        }
    }

    fn eval_product(
        &mut self,
        var: &str,
        lower: &Expr,
        upper: &Expr,
        body: &Expr,
        original: &Expr,
        locals: &HashMap<String, Value>,
    ) -> Result<Value, Diagnostic> {
        let lo = self.eval_expr(lower, locals)?;
        let hi = self.eval_expr(upper, locals)?;
        match (lo, hi) {
            (Value::Rational(l), Value::Rational(h)) if l.is_integer() && h.is_integer() => {
                if l.num > h.num {
                    return Ok(Value::Rational(Rational::integer(1)));
                }
                let mut acc = Rational::integer(1);
                let mut local = locals.clone();
                let term_count = h
                    .num
                    .checked_sub(l.num)
                    .and_then(|n| usize::try_from(n).ok())
                    .and_then(|n| n.checked_add(1));
                if term_count.is_none_or(|count| count > self.max_sum_terms) {
                    return Err(Diagnostic::new("finite product exceeds evaluation term limit"));
                }
                for i in l.num..=h.num {
                    local.insert(var.to_owned(), Value::Rational(Rational::integer(i)));
                    match self.eval_expr(body, &local)? {
                        Value::Rational(v) => acc = acc.mul(&v).map_err(Diagnostic::new)?,
                        _ => return Ok(Value::Symbolic(original.clone())),
                    }
                }
                Ok(Value::Rational(acc))
            }
            _ => Ok(Value::Symbolic(original.clone())),
        }
    }

    fn eval_sum(
        &mut self,
        var: &str,
        lower: &Expr,
        upper: &Expr,
        body: &Expr,
        original: &Expr,
        locals: &HashMap<String, Value>,
    ) -> Result<Value, Diagnostic> {
        let lo = self.eval_expr(lower, locals)?;
        let hi = self.eval_expr(upper, locals)?;
        match (lo, hi) {
            (Value::Rational(l), Value::Rational(h)) if l.is_integer() && h.is_integer() => {
                if l.num > h.num {
                    return Ok(Value::Rational(Rational::integer(0)));
                }
                let mut acc = Rational::integer(0);
                let mut local = locals.clone();
                let term_count = h
                    .num
                    .checked_sub(l.num)
                    .and_then(|n| usize::try_from(n).ok())
                    .and_then(|n| n.checked_add(1));
                if term_count.is_none_or(|count| count > self.max_sum_terms) {
                    return Err(Diagnostic::new("finite sum exceeds evaluation term limit"));
                }
                for i in l.num..=h.num {
                    local.insert(var.to_owned(), Value::Rational(Rational::integer(i)));
                    match self.eval_expr(body, &local)? {
                        Value::Rational(v) => acc = acc.add(&v).map_err(Diagnostic::new)?,
                        _ => return Ok(Value::Symbolic(original.clone())),
                    }
                }
                Ok(Value::Rational(acc))
            }
            _ => Ok(Value::Symbolic(original.clone())),
        }
    }

    fn eval_binary(
        &mut self,
        op: BinOp,
        lhs: &Expr,
        rhs: &Expr,
        span: crate::diagnostics::Span,
        locals: &HashMap<String, Value>,
    ) -> Result<Value, Diagnostic> {
        let a = self.eval_expr(lhs, locals)?;
        let b = self.eval_expr(rhs, locals)?;
        match op {
            BinOp::Eq
            | BinOp::Lt
            | BinOp::Le
            | BinOp::Gt
            | BinOp::Ge
            | BinOp::In
            | BinOp::Subset
            | BinOp::SubsetEq => {
                match compare(op, &a, &b) {
                    Ok(value) => Ok(Value::Bool(value)),
                    Err(_) => Ok(Value::Symbolic(Expr::Binary {
                        op,
                        lhs: Box::new(value_to_expr(&a, lhs)),
                        rhs: Box::new(value_to_expr(&b, rhs)),
                        span,
                    })),
                }
            }
            BinOp::Union | BinOp::Intersect | BinOp::Difference => {
                set_binary(op, &a, &b, lhs, rhs, span)
            }
            BinOp::Add | BinOp::Sub => structured_add_sub(op, &a, &b, lhs, rhs, span),
            BinOp::Mul => structured_mul(&a, &b, lhs, rhs, span),
            BinOp::Div => numeric_or_symbolic(BinOp::Div, a, b, lhs, rhs, |x, y| x.div(y)),
            BinOp::Pow => match (a, b) {
                (Value::Rational(x), Value::Rational(y)) if y.is_integer() => {
                    x.powi(y.num).map(Value::Rational).map_err(Diagnostic::new)
                }
                (Value::Matrix(matrix), Value::Rational(exp)) if exp.is_integer() && exp.num >= 0 => {
                    matrix_pow(&matrix, exp.num).map(Value::Matrix).map_err(Diagnostic::new)
                }
                (va, vb) => Ok(Value::Symbolic(Expr::Binary {
                    op,
                    lhs: Box::new(value_to_expr(&va, lhs)),
                    rhs: Box::new(value_to_expr(&vb, rhs)),
                    span,
                })),
            },
        }
    }

    fn eval_integral(
        &mut self,
        var: &str,
        lower: Option<&Expr>,
        upper: Option<&Expr>,
        body: &Expr,
        original: &Expr,
        locals: &HashMap<String, Value>,
    ) -> Result<Value, Diagnostic> {
        let antiderivative = integrate_expr(body, var)
            .ok_or_else(|| Diagnostic::new("integral is outside the exact polynomial integration subset"))?;
        match (lower, upper) {
            (Some(lo), Some(hi)) => {
                let lo_value = self.eval_expr(lo, locals)?;
                let hi_value = self.eval_expr(hi, locals)?;
                let mut lo_locals = locals.clone();
                let mut hi_locals = locals.clone();
                match (lo_value, hi_value) {
                    (Value::Rational(lo), Value::Rational(hi)) => {
                        lo_locals.insert(var.to_owned(), Value::Rational(lo));
                        hi_locals.insert(var.to_owned(), Value::Rational(hi));
                        let a = self.eval_expr(&antiderivative, &lo_locals)?;
                        let b = self.eval_expr(&antiderivative, &hi_locals)?;
                        match (b, a) {
                            (Value::Rational(b), Value::Rational(a)) => {
                                Ok(Value::Rational(b.sub(&a).map_err(Diagnostic::new)?))
                            }
                            _ => Ok(Value::Symbolic(original.clone())),
                        }
                    }
                    _ => Ok(Value::Symbolic(original.clone())),
                }
            }
            _ => Ok(Value::Symbolic(antiderivative)),
        }
    }

    fn eval_limit(
        &mut self,
        var: &str,
        target: &Expr,
        body: &Expr,
        original: &Expr,
        locals: &HashMap<String, Value>,
    ) -> Result<Value, Diagnostic> {
        let target = self.eval_expr(target, locals)?;
        match target {
            Value::Rational(value) => {
                let mut child = locals.clone();
                child.insert(var.to_owned(), Value::Rational(value));
                match self.eval_expr(body, &child) {
                    Ok(value) => Ok(value),
                    Err(_) => Ok(Value::Symbolic(original.clone())),
                }
            }
            _ => Ok(Value::Symbolic(original.clone())),
        }
    }

    fn eval_builtin(
        &mut self,
        name: &str,
        args: &[Expr],
        span: crate::diagnostics::Span,
        locals: &HashMap<String, Value>,
    ) -> Result<Value, Diagnostic> {
        let symbolic = |name: &str, values: &[Value]| {
            Value::Symbolic(Expr::Call {
                callee: Box::new(Expr::Symbol {
                    name: name.to_owned(),
                    span,
                }),
                args: values
                    .iter()
                    .zip(args.iter())
                    .map(|(value, expr)| value_to_expr(value, expr))
                    .collect(),
                span,
            })
        };

        let require = |count: usize| {
            if args.len() == count {
                Ok(())
            } else {
                Err(Diagnostic::at(
                    format!("{} expects {} argument(s), got {}", name, count, args.len()),
                    span,
                ))
            }
        };

        match name {
            "\factorial" => {
                require(1)?;
                match self.eval_expr(&args[0], locals)? {
                    Value::Rational(n) if n.is_integer() && n.num >= 0 => {
                        factorial(n.num).map(Value::Rational).map_err(Diagnostic::new)
                    }
                    _ => Err(Diagnostic::at("factorial requires a non-negative integer", span)),
                }
            }
            "\binom" | "\choose" => {
                require(2)?;
                let a = self.eval_expr(&args[0], locals)?;
                let b = self.eval_expr(&args[1], locals)?;
                match (a, b) {
                    (Value::Rational(n), Value::Rational(k))
                        if n.is_integer() && k.is_integer() && n.num >= 0 && k.num >= 0 =>
                    {
                        binom(n.num, k.num)
                            .map(Value::Rational)
                            .map_err(Diagnostic::new)
                    }
                    _ => Err(Diagnostic::at("binomial coefficient requires non-negative integers", span)),
                }
            }
            "\perm" | "\permutation" => {
                require(2)?;
                let a = self.eval_expr(&args[0], locals)?;
                let b = self.eval_expr(&args[1], locals)?;
                match (a, b) {
                    (Value::Rational(n), Value::Rational(k))
                        if n.is_integer() && k.is_integer() && n.num >= 0 && k.num >= 0 =>
                    {
                        perm(n.num, k.num).map(Value::Rational).map_err(Diagnostic::new)
                    }
                    _ => Err(Diagnostic::at("permutation requires non-negative integers", span)),
                }
            }
            "\gcd" => {
                require(2)?;
                let a = self.eval_expr(&args[0], locals)?;
                let b = self.eval_expr(&args[1], locals)?;
                match (a, b) {
                    (Value::Rational(a), Value::Rational(b)) if a.is_integer() && b.is_integer() => {
                        Ok(Value::Rational(Rational::integer(gcd_i128(a.num, b.num))))
                    }
                    _ => Err(Diagnostic::at("gcd requires integers", span)),
                }
            }
            "\lcm" => {
                require(2)?;
                let a = self.eval_expr(&args[0], locals)?;
                let b = self.eval_expr(&args[1], locals)?;
                match (a, b) {
                    (Value::Rational(a), Value::Rational(b)) if a.is_integer() && b.is_integer() => {
                        lcm_i128(a.num, b.num)
                            .map(|n| Value::Rational(Rational::integer(n)))
                            .map_err(Diagnostic::new)
                    }
                    _ => Err(Diagnostic::at("lcm requires integers", span)),
                }
            }
            "\floor" | "\ceil" => {
                require(1)?;
                match self.eval_expr(&args[0], locals)? {
                    Value::Rational(r) => {
                        let q = r.num / r.den;
                        let rem = r.num % r.den;
                        let n = if name == "\floor" {
                            if r.num < 0 && rem != 0 { q - 1 } else { q }
                        } else if r.num > 0 && rem != 0 {
                            q + 1
                        } else {
                            q
                        };
                        Ok(Value::Rational(Rational::integer(n)))
                    }
                    _ => Err(Diagnostic::at(format!("{} requires an exact rational", name), span)),
                }
            }
            "\min" | "\max" => {
                if args.is_empty() {
                    return Err(Diagnostic::at(format!("{} requires at least one argument", name), span));
                }
                let values = args.iter().map(|arg| self.eval_expr(arg, locals)).collect::<Result<Vec<_>, _>>()?;
                let mut best: Option<Rational> = None;
                for value in values {
                    match value {
                        Value::Rational(r) => {
                            best = Some(match best {
                                None => r,
                                Some(current) => {
                                    let less = r.num.checked_mul(current.den).ok_or_else(|| Diagnostic::new("comparison overflow"))?
                                        .cmp(&current.num.checked_mul(r.den).ok_or_else(|| Diagnostic::new("comparison overflow"))?);
                                    if (name == "\min" && less.is_lt()) || (name == "\max" && less.is_gt()) { r } else { current }
                                }
                            });
                        }
                        _ => return Err(Diagnostic::at(format!("{} requires exact numeric arguments", name), span)),
                    }
                }
                Ok(Value::Rational(best.expect("non-empty min/max")))
            }
            "\dot" => {
                require(2)?;
                let a = self.eval_expr(&args[0], locals)?;
                let b = self.eval_expr(&args[1], locals)?;
                match (a, b) {
                    (Value::Vector(a), Value::Vector(b)) if a.len() == b.len() => {
                        let mut out = Rational::integer(0);
                        for (x, y) in a.iter().zip(&b) {
                            let (Value::Rational(x), Value::Rational(y)) = (x, y) else {
                                return Ok(symbolic(name, &[Value::Vector(a.clone()), Value::Vector(b.clone())]));
                            };
                            let product = x.mul(y).map_err(Diagnostic::new)?;
                            out = out.add(&product).map_err(Diagnostic::new)?;
                        }
                        Ok(Value::Rational(out))
                    }
                    _ => Err(Diagnostic::at("dot product requires vectors of equal length", span)),
                }
            }
            "\norm" => {
                require(1)?;
                let value = self.eval_expr(&args[0], locals)?;
                match value {
                    Value::Vector(xs) => {
                        let mut sum = Rational::integer(0);
                        for x in &xs {
                            let Value::Rational(x) = x else {
                                return Ok(Value::Symbolic(Expr::Call {
                                    callee: Box::new(Expr::Symbol { name: name.to_owned(), span }),
                                    args: vec![args[0].clone()],
                                    span,
                                }));
                            };
                            let sq = x.mul(x).map_err(Diagnostic::new)?;
                            sum = sum.add(&sq).map_err(Diagnostic::new)?;
                        }
                        self.eval_expr(&Expr::Sqrt { expr: Box::new(value_to_rational_expr(&sum, span)), span }, locals)
                    }
                    Value::Rational(x) => self.eval_expr(&Expr::Sqrt { expr: Box::new(value_to_rational_expr(&x.mul(&x).map_err(Diagnostic::new)?, span)), span }, locals),
                    _ => Err(Diagnostic::at("norm requires a vector or number", span)),
                }
            }
            "\det" => {
                require(1)?;
                match self.eval_expr(&args[0], locals)? {
                    Value::Matrix(matrix) => {
                        determinant(&matrix).map(Value::Rational).map_err(Diagnostic::new)
                    }
                    _ => Err(Diagnostic::at("determinant requires a matrix", span)),
                }
            }
            "\transpose" | "\trans" => {
                require(1)?;
                match self.eval_expr(&args[0], locals)? {
                    Value::Matrix(matrix) => Ok(Value::Matrix(transpose(&matrix))),
                    _ => Err(Diagnostic::at("transpose requires a matrix", span)),
                }
            }
            "\trace" => {
                require(1)?;
                match self.eval_expr(&args[0], locals)? {
                    Value::Matrix(matrix) => trace_matrix(&matrix).map(Value::Rational).map_err(Diagnostic::new),
                    _ => Err(Diagnostic::at("trace requires a matrix", span)),
                }
            }
            "\mean" | "\variance" | "\stdev" => {
                require(1)?;
                let value = self.eval_expr(&args[0], locals)?;
                let xs = match value {
                    Value::Vector(xs) => xs,
                    Value::Set(xs) => xs,
                    _ => return Err(Diagnostic::at(format!("{} requires a finite set or vector", name), span)),
                };
                let numbers = xs
                    .into_iter()
                    .map(|value| match value {
                        Value::Rational(r) => Ok(r),
                        _ => Err(Diagnostic::new("statistics requires exact numeric data")),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                match name {
                    "\mean" => mean(&numbers).map(Value::Rational).map_err(Diagnostic::new),
                    "\variance" => variance(&numbers).map(Value::Rational).map_err(Diagnostic::new),
                    _ => {
                        let v = variance(&numbers).map_err(Diagnostic::new)?;
                        self.eval_expr(&Expr::Sqrt {
                            expr: Box::new(value_to_rational_expr(&v, span)),
                            span,
                        }, locals)
                    }
                }
            }
            "\diff" | "\derivative" => {
                require(2)?;
                let var = match &args[1] {
                    Expr::Symbol { name, .. } => name.clone(),
                    _ => return Err(Diagnostic::at("derivative variable must be a symbol", span)),
                };
                let derivative = differentiate(&args[0], &var)
                    .ok_or_else(|| Diagnostic::at("derivative is outside the supported symbolic subset", span))?;
                let derivative = simplifier::simplify(derivative);
                self.eval_expr(&derivative, locals)
            }
            "\subs" | "\substitute" => {
                require(2)?;
                let (var, replacement) = match &args[0] {
                    Expr::Binary { op: BinOp::Eq, lhs, rhs, .. } => {
                        let Expr::Symbol { name, .. } = lhs.as_ref() else {
                            return Err(Diagnostic::at("substitution must use x=value", span));
                        };
                        let replacement = self.eval_expr(rhs, locals)?;
                        (name.clone(), value_to_expr(&replacement, rhs))
                    }
                    _ => return Err(Diagnostic::at("substitution must use x=value", span)),
                };
                let body = substitute(&args[1], &var, &replacement);
                self.eval_expr(&body, locals)
            }
            "\solve" => {
                require(2)?;
                let var = match &args[1] {
                    Expr::Symbol { name, .. } => name.clone(),
                    _ => return Err(Diagnostic::at("solve variable must be a symbol", span)),
                };
                let equation = match &args[0] {
                    Expr::Binary { op: BinOp::Eq, lhs, rhs, .. } => Expr::Binary {
                        op: BinOp::Sub,
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                        span,
                    },
                    _ => return Err(Diagnostic::at("solve expects an equation such as x^2 = 4", span)),
                };
                let coeffs = polynomial_coefficients(&equation, &var, self, locals)?;
                solve_polynomial(&coeffs, &var, span, self)
            }
            "\sin" | "\cos" | "\tan" | "\ln" | "\log" | "\exp" => {
                require(1)?;
                let value = self.eval_expr(&args[0], locals)?;
                if let Some(result) = elementary_exact(name, &value, span) {
                    Ok(result)
                } else {
                    Ok(symbolic(name, &[value]))
                }
            }
            "\range" => {
                require(2)?;
                let a = self.eval_expr(&args[0], locals)?;
                let b = self.eval_expr(&args[1], locals)?;
                match (a, b) {
                    (Value::Rational(a), Value::Rational(b)) if a.is_integer() && b.is_integer() => {
                        let mut values = Vec::new();
                        if a.num <= b.num {
                            let count = b.num.checked_sub(a.num)
                                .and_then(|n| usize::try_from(n).ok())
                                .and_then(|n| n.checked_add(1))
                                .ok_or_else(|| Diagnostic::new("range is too large"))?;
                            if count > self.max_sum_terms {
                                return Err(Diagnostic::new("range exceeds evaluation term limit"));
                            }
                            for i in a.num..=b.num {
                                values.push(Value::Rational(Rational::integer(i)));
                            }
                        }
                        Ok(Value::Set(values))
                    }
                    _ => Err(Diagnostic::at("range requires integer bounds", span)),
                }
            }
            "\tuple" => {
                Ok(Value::Vector(args.iter().map(|arg| self.eval_expr(arg, locals)).collect::<Result<Vec<_>, _>>()?))
            }
            _ => {
                let values = args.iter().map(|arg| self.eval_expr(arg, locals)).collect::<Result<Vec<_>, _>>()?;
                Ok(symbolic(name, &values))
            }
        }
    }

}


fn numeric_or_symbolic<F>(
    op: BinOp,
    a: Value,
    b: Value,
    lhs: &Expr,
    rhs: &Expr,
    f: F,
) -> Result<Value, Diagnostic>
where
    F: Fn(&Rational, &Rational) -> Result<Rational, String>,
{
    match (&a, &b) {
        (Value::Rational(x), Value::Rational(y)) => {
            f(x, y).map(Value::Rational).map_err(Diagnostic::new)
        }
        _ => Ok(Value::Symbolic(simplifier::simplify(Expr::Binary {
            op,
            lhs: Box::new(value_to_expr(&a, lhs)),
            rhs: Box::new(value_to_expr(&b, rhs)),
            span: crate::diagnostics::Span::new(lhs.span().start, rhs.span().end),
        }))),
    }
}

fn compare(op: BinOp, a: &Value, b: &Value) -> Result<bool, String> {
    match (a, b) {
        (Value::Rational(x), Value::Rational(y)) => {
            let left = x.num.checked_mul(y.den).ok_or("comparison overflow")?;
            let right = y.num.checked_mul(x.den).ok_or("comparison overflow")?;
            Ok(match op {
                BinOp::Eq => left == right,
                BinOp::Lt => left < right,
                BinOp::Le => left <= right,
                BinOp::Gt => left > right,
                BinOp::Ge => left >= right,
                _ => unreachable!(),
            })
        }
        (Value::Bool(x), Value::Bool(y)) if op == BinOp::Eq => Ok(x == y),
        _ => Err("comparison requires compatible exact values".into()),
    }
}
