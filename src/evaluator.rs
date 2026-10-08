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
    match op {
        BinOp::In => {
            let Value::Set(set) = b else {
                return Err("membership requires a set on the right".into());
            };
            Ok(set.iter().any(|value| values_equal(a, value)))
        }
        BinOp::Subset | BinOp::SubsetEq => {
            let (Value::Set(left), Value::Set(right)) = (a, b) else {
                return Err("subset comparison requires sets".into());
            };
            let contained = left.iter().all(|value| right.iter().any(|item| values_equal(value, item)));
            if op == BinOp::Subset {
                Ok(contained && left.len() < right.len())
            } else {
                Ok(contained)
            }
        }
        BinOp::Eq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => match (a, b) {
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
            _ if op == BinOp::Eq => Ok(values_equal(a, b)),
            _ => Err("comparison requires compatible exact values".into()),
        },
        _ => Err("unsupported comparison".into()),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Rational(x), Value::Rational(y)) => {
            x.num.checked_mul(y.den) == y.num.checked_mul(x.den)
        }
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Set(x), Value::Set(y)) => {
            x.len() == y.len() && x.iter().all(|value| y.iter().any(|item| values_equal(value, item)))
        }
        (Value::Vector(x), Value::Vector(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| values_equal(a, b))
        }
        (Value::Matrix(x), Value::Matrix(y)) => {
            x.len() == y.len()
                && x.iter().zip(y).all(|(ra, rb)| {
                    ra.len() == rb.len()
                        && ra.iter().zip(rb).all(|(a, b)| values_equal(a, b))
                })
        }
        _ => a == b,
    }
}

fn symbolic_binary(op: BinOp, a: &Value, b: &Value, lhs: &Expr, rhs: &Expr, span: crate::diagnostics::Span) -> Value {
    Value::Symbolic(simplifier::simplify(Expr::Binary {
        op,
        lhs: Box::new(value_to_expr(a, lhs)),
        rhs: Box::new(value_to_expr(b, rhs)),
        span,
    }))
}

fn structured_add_sub(
    op: BinOp,
    a: &Value,
    b: &Value,
    lhs: &Expr,
    rhs: &Expr,
    span: crate::diagnostics::Span,
) -> Value {
    match (a, b) {
        (Value::Rational(x), Value::Rational(y)) => {
            let result = if op == BinOp::Add { x.add(y) } else { x.sub(y) };
            result.map(Value::Rational).unwrap_or_else(|_| symbolic_binary(op, a, b, lhs, rhs, span))
        }
        (Value::Vector(x), Value::Vector(y)) if x.len() == y.len() => {
            let mut out = Vec::with_capacity(x.len());
            for (vx, vy) in x.iter().zip(y) {
                let (Value::Rational(rx), Value::Rational(ry)) = (vx, vy) else {
                    return symbolic_binary(op, a, b, lhs, rhs, span);
                };
                let value = if op == BinOp::Add { rx.add(ry) } else { rx.sub(ry) };
                match value {
                    Ok(v) => out.push(Value::Rational(v)),
                    Err(_) => return symbolic_binary(op, a, b, lhs, rhs, span),
                }
            }
            Value::Vector(out)
        }
        (Value::Matrix(x), Value::Matrix(y)) if matrix_shape(x) == matrix_shape(y) => {
            let mut out = Vec::with_capacity(x.len());
            for (ra, rb) in x.iter().zip(y) {
                let mut row = Vec::with_capacity(ra.len());
                for (vx, vy) in ra.iter().zip(rb) {
                    let (Value::Rational(rx), Value::Rational(ry)) = (vx, vy) else {
                        return symbolic_binary(op, a, b, lhs, rhs, span);
                    };
                    let value = if op == BinOp::Add { rx.add(ry) } else { rx.sub(ry) };
                    match value {
                        Ok(v) => row.push(Value::Rational(v)),
                        Err(_) => return symbolic_binary(op, a, b, lhs, rhs, span),
                    }
                }
                out.push(row);
            }
            Value::Matrix(out)
        }
        _ => symbolic_binary(op, a, b, lhs, rhs, span),
    }
}

fn structured_mul(
    a: &Value,
    b: &Value,
    lhs: &Expr,
    rhs: &Expr,
    span: crate::diagnostics::Span,
) -> Value {
    match (a, b) {
        (Value::Rational(x), Value::Rational(y)) => {
            x.mul(y).map(Value::Rational).unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        (Value::Rational(s), Value::Vector(v)) | (Value::Vector(v), Value::Rational(s)) => {
            scale_vector(v, s).map(Value::Vector).unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        (Value::Rational(s), Value::Matrix(m)) | (Value::Matrix(m), Value::Rational(s)) => {
            scale_matrix(m, s).map(Value::Matrix).unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        (Value::Vector(a), Value::Vector(b)) if a.len() == b.len() => {
            dot_vectors(a, b).map(Value::Rational).unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        (Value::Matrix(a), Value::Vector(b)) if !a.is_empty() && a[0].len() == b.len() => {
            matrix_vector_mul(a, b).map(Value::Vector).unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        (Value::Matrix(a), Value::Matrix(b)) if !a.is_empty() && !b.is_empty() && a[0].len() == b.len() => {
            matrix_matrix_mul(a, b).map(Value::Matrix).unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        _ => symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span),
    }
}

fn set_binary(
    op: BinOp,
    a: &Value,
    b: &Value,
    lhs: &Expr,
    rhs: &Expr,
    span: crate::diagnostics::Span,
) -> Result<Value, Diagnostic> {
    let (Value::Set(left), Value::Set(right)) = (a, b) else {
        return Ok(symbolic_binary(op, a, b, lhs, rhs, span));
    };
    let mut out = match op {
        BinOp::Union => left.clone(),
        BinOp::Intersect => Vec::new(),
        BinOp::Difference => Vec::new(),
        _ => unreachable!(),
    };
    match op {
        BinOp::Union => {
            for value in right {
                if !out.iter().any(|item| values_equal(item, value)) {
                    out.push(value.clone());
                }
            }
        }
        BinOp::Intersect => {
            for value in left {
                if right.iter().any(|item| values_equal(value, item))
                    && !out.iter().any(|item| values_equal(item, value))
                {
                    out.push(value.clone());
                }
            }
        }
        BinOp::Difference => {
            for value in left {
                if !right.iter().any(|item| values_equal(value, item)) {
                    out.push(value.clone());
                }
            }
        }
        _ => {}
    }
    Ok(Value::Set(out))
}

fn matrix_shape(m: &[Vec<Value>]) -> (usize, usize) {
    (m.len(), m.first().map_or(0, Vec::len))
}

fn scale_vector(v: &[Value], s: &Rational) -> Result<Vec<Value>, String> {
    v.iter()
        .map(|value| match value {
            Value::Rational(x) => x.mul(s).map(Value::Rational),
            _ => Err("vector contains a non-exact element".into()),
        })
        .collect()
}

fn scale_matrix(m: &[Vec<Value>], s: &Rational) -> Result<Vec<Vec<Value>>, String> {
    m.iter()
        .map(|row| {
            row.iter()
                .map(|value| match value {
                    Value::Rational(x) => x.mul(s).map(Value::Rational),
                    _ => Err("matrix contains a non-exact element".into()),
                })
                .collect()
        })
        .collect()
}

fn dot_vectors(a: &[Value], b: &[Value]) -> Result<Rational, String> {
    let mut out = Rational::integer(0);
    for (x, y) in a.iter().zip(b) {
        let (Value::Rational(x), Value::Rational(y)) = (x, y) else {
            return Err("vector contains a non-exact element".into());
        };
        let product = x.mul(y)?;
        out = out.add(&product)?;
    }
    Ok(out)
}

fn matrix_vector_mul(m: &[Vec<Value>], v: &[Value]) -> Result<Vec<Value>, String> {
    m.iter()
        .map(|row| {
            if row.len() != v.len() {
                return Err("matrix/vector dimensions do not match".into());
            }
            let value = dot_vectors(row, v)?;
            Ok(Value::Rational(value))
        })
        .collect()
}

fn matrix_matrix_mul(a: &[Vec<Value>], b: &[Vec<Value>]) -> Result<Vec<Vec<Value>>, String> {
    if a.is_empty() || b.is_empty() || a[0].len() != b.len() {
        return Err("matrix dimensions do not match".into());
    }
    let rows = a.len();
    let cols = b[0].len();
    let mut out = vec![vec![Value::Rational(Rational::integer(0)); cols]; rows];
    for i in 0..rows {
        for j in 0..cols {
            let mut value = Rational::integer(0);
            for k in 0..b.len() {
                let (Value::Rational(x), Value::Rational(y)) = (&a[i][k], &b[k][j]) else {
                    return Err("matrix contains a non-exact element".into());
                };
                value = value.add(&x.mul(y)?)?;
            }
            out[i][j] = Value::Rational(value);
        }
    }
    Ok(out)
}

fn transpose(m: &[Vec<Value>]) -> Vec<Vec<Value>> {
    if m.is_empty() {
        return Vec::new();
    }
    (0..m[0].len())
        .map(|j| m.iter().map(|row| row[j].clone()).collect())
        .collect()
}

fn trace_matrix(m: &[Vec<Value>]) -> Result<Rational, String> {
    if m.is_empty() || m.len() != m[0].len() {
        return Err("trace requires a non-empty square matrix".into());
    }
    let mut out = Rational::integer(0);
    for i in 0..m.len() {
        let Value::Rational(value) = &m[i][i] else {
            return Err("matrix contains a non-exact element".into());
        };
        out = out.add(value)?;
    }
    Ok(out)
}

fn determinant(m: &[Vec<Value>]) -> Result<Rational, String> {
    if m.is_empty() || m.len() != m[0].len() {
        return Err("determinant requires a non-empty square matrix".into());
    }
    let n = m.len();
    let mut a = Vec::with_capacity(n);
    for row in m {
        if row.len() != n {
            return Err("determinant requires a square matrix".into());
        }
        a.push(row.iter().map(|value| match value {
            Value::Rational(r) => Ok(r.clone()),
            _ => Err("matrix contains a non-exact element".into()),
        }).collect::<Result<Vec<_>, _>>()?);
    }
    let mut det = Rational::integer(1);
    for col in 0..n {
        let pivot = (col..n).find(|&row| a[row][col].num != 0);
        let Some(pivot) = pivot else {
            return Ok(Rational::integer(0));
        };
        if pivot != col {
            a.swap(pivot, col);
            det = Rational::integer(-1).mul(&det)?;
        }
        let pivot_value = a[col][col].clone();
        det = det.mul(&pivot_value)?;
        for row in (col + 1)..n {
            let factor = a[row][col].div(&pivot_value)?;
            for j in col..n {
                let product = factor.mul(&a[col][j])?;
                a[row][j] = a[row][j].sub(&product)?;
            }
        }
    }
    Ok(det)
}

fn matrix_pow(matrix: &[Vec<Value>], exp: i128) -> Result<Vec<Vec<Value>>, String> {
    if matrix.is_empty() || matrix.len() != matrix[0].len() {
        return Err("matrix power requires a non-empty square matrix".into());
    }
    let n = matrix.len();
    let mut result = vec![vec![Value::Rational(Rational::integer(0)); n]; n];
    for i in 0..n {
        result[i][i] = Value::Rational(Rational::integer(1));
    }
    let mut base = matrix.to_vec();
    let mut e = exp as u128;
    while e > 0 {
        if e & 1 == 1 {
            result = matrix_matrix_mul(&result, &base)?;
        }
        e >>= 1;
        if e > 0 {
            base = matrix_matrix_mul(&base, &base)?;
        }
    }
    Ok(result)
}

fn factorial(n: i128) -> Result<Rational, String> {
    let mut out = 1i128;
    for i in 2..=n {
        out = out.checked_mul(i).ok_or("integer overflow in factorial")?;
    }
    Ok(Rational::integer(out))
}

fn binom(n: i128, k: i128) -> Result<Rational, String> {
    if k > n {
        return Ok(Rational::integer(0));
    }
    let k = k.min(n - k);
    let mut out = Rational::integer(1);
    for i in 1..=k {
        out = out.mul(&Rational::new(n - k + i, i)?)?;
    }
    Ok(out)
}

fn perm(n: i128, k: i128) -> Result<Rational, String> {
    if k > n {
        return Ok(Rational::integer(0));
    }
    let mut out = 1i128;
    for i in 0..k {
        out = out.checked_mul(n - i).ok_or("integer overflow in permutation")?;
    }
    Ok(Rational::integer(out))
}

fn gcd_i128(mut a: i128, mut b: i128) -> i128 {
    a = a.checked_abs().unwrap_or(i128::MAX);
    b = b.checked_abs().unwrap_or(i128::MAX);
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

fn lcm_i128(a: i128, b: i128) -> Result<i128, String> {
    if a == 0 || b == 0 {
        return Ok(0);
    }
    let g = gcd_i128(a, b);
    (a / g).checked_mul(b.checked_abs().ok_or("integer overflow")?).ok_or("integer overflow".into())
}

fn mean(xs: &[Rational]) -> Result<Rational, String> {
    if xs.is_empty() {
        return Err("mean of empty data is undefined".into());
    }
    let mut sum = Rational::integer(0);
    for x in xs {
        sum = sum.add(x)?;
    }
    sum.div(&Rational::integer(xs.len() as i128))
}

fn variance(xs: &[Rational]) -> Result<Rational, String> {
    if xs.is_empty() {
        return Err("variance of empty data is undefined".into());
    }
    let mean = mean(xs)?;
    let mut sum = Rational::integer(0);
    for x in xs {
        let d = x.sub(&mean)?;
        sum = sum.add(&d.mul(&d)?)?;
    }
    sum.div(&Rational::integer(xs.len() as i128))
}

fn value_to_rational_expr(value: &Rational, span: crate::diagnostics::Span) -> Expr {
    if value.den == 1 {
        Expr::Integer(value.num, span)
    } else {
        Expr::Rational {
            numerator: Box::new(Expr::Integer(value.num, span)),
            denominator: Box::new(Expr::Integer(value.den, span)),
            span,
        }
    }
}

fn call_expr(name: &str, args: Vec<Expr>, span: crate::diagnostics::Span) -> Expr {
    Expr::Call {
        callee: Box::new(Expr::Symbol { name: name.to_owned(), span }),
        args,
        span,
    }
}

fn elementary_exact(name: &str, value: &Value, span: crate::diagnostics::Span) -> Option<Value> {
    match name {
        "\sin" => {
            if matches!(value, Value::Rational(r) if r.num == 0) {
                return Some(Value::Rational(Rational::integer(0)));
            }
        }
        "\cos" => {
            if matches!(value, Value::Rational(r) if r.num == 0) {
                return Some(Value::Rational(Rational::integer(1)));
            }
        }
        "\tan" => {
            if matches!(value, Value::Rational(r) if r.num == 0) {
                return Some(Value::Rational(Rational::integer(0)));
            }
        }
        "\ln" => {
            if matches!(value, Value::Rational(r) if r.num == r.den) {
                return Some(Value::Rational(Rational::integer(0)));
            }
        }
        "\exp" => {
            if matches!(value, Value::Rational(r) if r.num == 0) {
                return Some(Value::Rational(Rational::integer(1)));
            }
        }
        "\log" => {
            if matches!(value, Value::Rational(r) if r.num == r.den) {
                return Some(Value::Rational(Rational::integer(0)));
            }
        }
        _ => {}
    }
    let Value::Symbolic(expr) = value else {
        return None;
    };
    match (name, expr) {
        ("\\sin", Expr::Symbol { name, .. }) if name == "\\pi" => Some(Value::Rational(Rational::integer(0))),
        ("\\cos", Expr::Symbol { name, .. }) if name == "\\pi" => Some(Value::Rational(Rational::integer(-1))),
        ("\\ln", Expr::Symbol { name, .. }) if name == "\\e" => Some(Value::Rational(Rational::integer(1))),
        ("\\exp", Expr::Integer(1, _)) => Some(Value::Symbolic(Expr::Symbol { name: "\\e".to_owned(), span })),
        _ => None,
    }
}

fn contains_symbol(expr: &Expr, var: &str) -> bool {
    match expr {
        Expr::Symbol { name, .. } => name == var,
        Expr::Unary { expr, .. } => contains_symbol(expr, var),
        Expr::Binary { lhs, rhs, .. } => contains_symbol(lhs, var) || contains_symbol(rhs, var),
        Expr::Call { callee, args, .. } => contains_symbol(callee, var) || args.iter().any(|arg| contains_symbol(arg, var)),
        Expr::Rational { numerator, denominator, .. } => contains_symbol(numerator, var) || contains_symbol(denominator, var),
        Expr::Set { elements, .. } | Expr::Vector { elements, .. } => elements.iter().any(|e| contains_symbol(e, var)),
        Expr::Matrix { rows, .. } => rows.iter().flatten().any(|e| contains_symbol(e, var)),
        Expr::Integral { body, lower, upper, .. } => {
            contains_symbol(body, var)
                || lower.as_deref().is_some_and(|e| contains_symbol(e, var))
                || upper.as_deref().is_some_and(|e| contains_symbol(e, var))
        }
        Expr::Limit { body, target, .. } => contains_symbol(body, var) || contains_symbol(target, var),
        Expr::Product { body, lower, upper, .. } | Expr::Sum { body, lower, upper, .. } => {
            contains_symbol(body, var) || contains_symbol(lower, var) || contains_symbol(upper, var)
        }
        Expr::Abs { expr, .. } | Expr::Sqrt { expr, .. } => contains_symbol(expr, var),
        Expr::Piecewise { branches, .. } => branches.iter().any(|b| contains_symbol(&b.value, var) || contains_symbol(&b.condition, var)),
        Expr::Integer(..) | Expr::Opaque { .. } => false,
    }
}

fn differentiate(expr: &Expr, var: &str) -> Option<Expr> {
    let span = expr.span();
    let zero = || Expr::Integer(0, span);
    let one = || Expr::Integer(1, span);
    match expr {
        Expr::Integer(..) | Expr::Rational { .. } if !contains_symbol(expr, var) => Some(zero()),
        Expr::Symbol { name, .. } => Some(if name == var { one() } else { zero() }),
        Expr::Unary { expr, .. } => differentiate(expr, var).map(|d| Expr::Unary {
            op: crate::ast::UnaryOp::Neg,
            expr: Box::new(d),
            span,
        }),
        Expr::Binary { op, lhs, rhs, .. } => {
            let dl = differentiate(lhs, var)?;
            let dr = differentiate(rhs, var)?;
            match op {
                BinOp::Add | BinOp::Sub => Some(Expr::Binary {
                    op: *op,
                    lhs: Box::new(dl),
                    rhs: Box::new(dr),
                    span,
                }),
                BinOp::Mul => Some(Expr::Binary {
                    op: BinOp::Add,
                    lhs: Box::new(Expr::Binary {
                        op: BinOp::Mul,
                        lhs: Box::new(dl),
                        rhs: rhs.clone(),
                        span,
                    }),
                    rhs: Box::new(Expr::Binary {
                        op: BinOp::Mul,
                        lhs: lhs.clone(),
                        rhs: Box::new(dr),
                        span,
                    }),
                    span,
                }),
                BinOp::Div => Some(Expr::Binary {
                    op: BinOp::Div,
                    lhs: Box::new(Expr::Binary {
                        op: BinOp::Sub,
                        lhs: Box::new(Expr::Binary {
                            op: BinOp::Mul,
                            lhs: Box::new(dl),
                            rhs: rhs.clone(),
                            span,
                        }),
                        rhs: Box::new(Expr::Binary {
                            op: BinOp::Mul,
                            lhs: lhs.clone(),
                            rhs: Box::new(dr),
                            span,
                        }),
                        span,
                    }),
                    rhs: Box::new(Expr::Binary {
                        op: BinOp::Pow,
                        lhs: rhs.clone(),
                        rhs: Box::new(Expr::Integer(2, span)),
                        span,
                    }),
                    span,
                }),
                BinOp::Pow => {
                    let Expr::Integer(n, _) = rhs.as_ref() else { return None; };
                    let coefficient = Expr::Integer(*n, span);
                    Some(Expr::Binary {
                        op: BinOp::Mul,
                        lhs: Box::new(coefficient),
                        rhs: Box::new(Expr::Binary {
                            op: BinOp::Mul,
                            lhs: Box::new(Expr::Binary {
                                op: BinOp::Pow,
                                lhs: lhs.clone(),
                                rhs: Box::new(Expr::Integer(n - 1, span)),
                                span,
                            }),
                            rhs: Box::new(dl),
                            span,
                        }),
                        span,
                    })
                }
                _ => None,
            }
        }
        Expr::Call { callee, args, .. } if args.len() == 1 => {
            let Expr::Symbol { name, .. } = callee.as_ref() else { return None; };
            let dx = differentiate(&args[0], var)?;
            let x = args[0].clone();
            match name.as_str() {
                "\sin" => Some(Expr::Binary { op: BinOp::Mul, lhs: Box::new(call_expr("\cos", vec![x], span)), rhs: Box::new(dx), span }),
                "\cos" => Some(Expr::Unary { op: crate::ast::UnaryOp::Neg, expr: Box::new(Expr::Binary { op: BinOp::Mul, lhs: Box::new(call_expr("\sin", vec![x], span)), rhs: Box::new(dx), span }), span }),
                "\tan" => Some(Expr::Binary { op: BinOp::Div, lhs: Box::new(dx), rhs: Box::new(Expr::Binary { op: BinOp::Pow, lhs: Box::new(call_expr("\cos", vec![x], span)), rhs: Box::new(Expr::Integer(2, span)), span }), span }),
                "\exp" => Some(Expr::Binary { op: BinOp::Mul, lhs: Box::new(call_expr("\exp", vec![x], span)), rhs: Box::new(dx), span }),
                "\ln" | "\log" => Some(Expr::Binary { op: BinOp::Div, lhs: Box::new(dx), rhs: Box::new(x), span }),
                "\sqrt" => Some(Expr::Binary { op: BinOp::Div, lhs: Box::new(dx), rhs: Box::new(Expr::Binary { op: BinOp::Mul, lhs: Box::new(Expr::Integer(2, span)), rhs: Box::new(call_expr("\sqrt", vec![x], span)), span }), span }),
                _ => None,
            }
        }
        Expr::Abs { .. } | Expr::Sqrt { .. } | Expr::Set { .. } | Expr::Vector { .. } | Expr::Matrix { .. } => None,
        _ => None,
    }
}

fn substitute(expr: &Expr, var: &str, replacement: &Expr) -> Expr {
    match expr {
        Expr::Symbol { name, .. } if name == var => replacement.clone(),
        Expr::Unary { op, expr, span } => Expr::Unary { op: *op, expr: Box::new(substitute(expr, var, replacement)), span: *span },
        Expr::Binary { op, lhs, rhs, span } => Expr::Binary { op: *op, lhs: Box::new(substitute(lhs, var, replacement)), rhs: Box::new(substitute(rhs, var, replacement)), span: *span },
        Expr::Call { callee, args, span } => Expr::Call { callee: Box::new(substitute(callee, var, replacement)), args: args.iter().map(|arg| substitute(arg, var, replacement)).collect(), span: *span },
        Expr::Rational { numerator, denominator, span } => Expr::Rational { numerator: Box::new(substitute(numerator, var, replacement)), denominator: Box::new(substitute(denominator, var, replacement)), span: *span },
        Expr::Set { elements, span } => Expr::Set { elements: elements.iter().map(|e| substitute(e, var, replacement)).collect(), span: *span },
        Expr::Vector { elements, span } => Expr::Vector { elements: elements.iter().map(|e| substitute(e, var, replacement)).collect(), span: *span },
        Expr::Matrix { rows, span } => Expr::Matrix { rows: rows.iter().map(|row| row.iter().map(|e| substitute(e, var, replacement)).collect()).collect(), span: *span },
        Expr::Abs { expr, span } => Expr::Abs { expr: Box::new(substitute(expr, var, replacement)), span: *span },
        Expr::Sqrt { expr, span } => Expr::Sqrt { expr: Box::new(substitute(expr, var, replacement)), span: *span },
        Expr::Product { var: bound, lower, upper, body, span } if bound != var => Expr::Product { var: bound.clone(), lower: Box::new(substitute(lower, var, replacement)), upper: Box::new(substitute(upper, var, replacement)), body: Box::new(substitute(body, var, replacement)), span: *span },
        Expr::Sum { var: bound, lower, upper, body, span } if bound != var => Expr::Sum { var: bound.clone(), lower: Box::new(substitute(lower, var, replacement)), upper: Box::new(substitute(upper, var, replacement)), body: Box::new(substitute(body, var, replacement)), span: *span },
        Expr::Integral { var: bound, lower, upper, body, span } if bound != var => Expr::Integral { var: bound.clone(), lower: lower.as_deref().map(|e| Box::new(substitute(e, var, replacement))), upper: upper.as_deref().map(|e| Box::new(substitute(e, var, replacement))), body: Box::new(substitute(body, var, replacement)), span: *span },
        Expr::Limit { var: bound, target, body, span } if bound != var => Expr::Limit { var: bound.clone(), target: Box::new(substitute(target, var, replacement)), body: Box::new(substitute(body, var, replacement)), span: *span },
        Expr::Piecewise { branches, span } => Expr::Piecewise { branches: branches.iter().map(|b| crate::ast::PiecewiseBranch { value: substitute(&b.value, var, replacement), condition: substitute(&b.condition, var, replacement), span: b.span }).collect(), span: *span },
        _ => expr.clone(),
    }
}

fn integrate_expr(expr: &Expr, var: &str) -> Option<Expr> {
    let span = expr.span();
    match expr {
        Expr::Integer(_, _) | Expr::Rational { .. } if !contains_symbol(expr, var) => {
            Some(Expr::Binary { op: BinOp::Mul, lhs: Box::new(expr.clone()), rhs: Box::new(Expr::Symbol { name: var.to_owned(), span }), span })
        }
        Expr::Symbol { name, .. } if name == var => {
            Some(Expr::Binary { op: BinOp::Div, lhs: Box::new(Expr::Binary { op: BinOp::Pow, lhs: Box::new(expr.clone()), rhs: Box::new(Expr::Integer(2, span)), span }), rhs: Box::new(Expr::Integer(2, span)), span })
        }
        Expr::Symbol { .. } => Some(Expr::Binary { op: BinOp::Mul, lhs: Box::new(expr.clone()), rhs: Box::new(Expr::Symbol { name: var.to_owned(), span }), span }),
        Expr::Unary { expr, .. } => integrate_expr(expr, var).map(|inner| Expr::Unary { op: crate::ast::UnaryOp::Neg, expr: Box::new(inner), span }),
        Expr::Binary { op: BinOp::Add, lhs, rhs, .. } | Expr::Binary { op: BinOp::Sub, lhs, rhs, .. } => {
            Some(Expr::Binary { op: *op, lhs: Box::new(integrate_expr(lhs, var)?), rhs: Box::new(integrate_expr(rhs, var)?), span })
        }
        Expr::Binary { op: BinOp::Mul, lhs, rhs, .. } => {
            if !contains_symbol(lhs, var) {
                let inner = integrate_expr(rhs, var)?;
                Some(Expr::Binary { op: BinOp::Mul, lhs: lhs.clone(), rhs: Box::new(inner), span })
            } else if !contains_symbol(rhs, var) {
                let inner = integrate_expr(lhs, var)?;
                Some(Expr::Binary { op: BinOp::Mul, lhs: rhs.clone(), rhs: Box::new(inner), span })
            } else {
                None
            }
        }
        Expr::Binary { op: BinOp::Pow, lhs, rhs, .. } if matches!(lhs.as_ref(), Expr::Symbol { name, .. } if name == var) => {
            let Expr::Integer(n, _) = rhs.as_ref() else { return None; };
            if *n == -1 {
                Some(call_expr("\ln", vec![lhs.as_ref().clone()], span))
            } else {
                let next = n.checked_add(1)?;
                Some(Expr::Binary {
                    op: BinOp::Div,
                    lhs: Box::new(Expr::Binary { op: BinOp::Pow, lhs: lhs.clone(), rhs: Box::new(Expr::Integer(next, span)), span }),
                    rhs: Box::new(Expr::Integer(next, span)),
                    span,
                })
            }
        }
        Expr::Binary { op: BinOp::Div, lhs, rhs, .. } if !contains_symbol(rhs, var) => {
            let inner = integrate_expr(lhs, var)?;
            Some(Expr::Binary { op: BinOp::Div, lhs: Box::new(inner), rhs: rhs.clone(), span })
        }
        _ => None,
    }
}

fn polynomial_coefficients(
    expr: &Expr,
    var: &str,
    evaluator: &mut Evaluator<'_>,
    locals: &HashMap<String, Value>,
) -> Result<Vec<Rational>, Diagnostic> {
    let mut out = polynomial(expr, var, evaluator, locals)
        .ok_or_else(|| Diagnostic::new("solve currently supports polynomial equations of degree at most 2"))?;
    while out.len() > 1 && out.last().is_some_and(|x| x.num == 0) {
        out.pop();
    }
    Ok(out)
}

fn polynomial(
    expr: &Expr,
    var: &str,
    evaluator: &mut Evaluator<'_>,
    locals: &HashMap<String, Value>,
) -> Option<Vec<Rational>> {
    match expr {
        Expr::Integer(n, _) => Some(vec![Rational::integer(*n)]),
        Expr::Rational { .. } if !contains_symbol(expr, var) => {
            match evaluator.eval_expr(expr, locals).ok()? {
                Value::Rational(r) => Some(vec![r]),
                _ => None,
            }
        }
        Expr::Symbol { name, .. } if name == var => Some(vec![Rational::integer(0), Rational::integer(1)]),
        Expr::Symbol { .. } => {
            match evaluator.eval_expr(expr, locals).ok()? {
                Value::Rational(r) => Some(vec![r]),
                _ => None,
            }
        }
        Expr::Unary { expr, .. } => {
            let mut p = polynomial(expr, var, evaluator, locals)?;
            for c in &mut p { c.num = c.num.checked_neg()?; }
            Some(p)
        }
        Expr::Binary { op, lhs, rhs, .. } => {
            let a = polynomial(lhs, var, evaluator, locals)?;
            let b = polynomial(rhs, var, evaluator, locals)?;
            match op {
                BinOp::Add | BinOp::Sub => {
                    let n = a.len().max(b.len());
                    let mut out = vec![Rational::integer(0); n];
                    for i in 0..n {
                        let av = a.get(i).cloned().unwrap_or_else(|| Rational::integer(0));
                        let bv = b.get(i).cloned().unwrap_or_else(|| Rational::integer(0));
                        out[i] = if *op == BinOp::Add { av.add(&bv).ok()? } else { av.sub(&bv).ok()? };
                    }
                    Some(out)
                }
                BinOp::Mul => {
                    if a.len() + b.len() > 4 { return None; }
                    let mut out = vec![Rational::integer(0); a.len()+b.len()-1];
                    for (i, x) in a.iter().enumerate() {
                        for (j, y) in b.iter().enumerate() {
                            let product = x.mul(y).ok()?;
                            out[i+j] = out[i+j].add(&product).ok()?;
                        }
                    }
                    Some(out)
                }
                BinOp::Div if b.len() == 1 => {
                    out_div(&a, &b[0])
                }
                BinOp::Pow => {
                    if b.len() != 1 || !b[0].is_integer() || b[0].num < 0 || b[0].num > 2 { return None; }
                    let mut out = vec![Rational::integer(1)];
                    for _ in 0..b[0].num {
                        let mut next = vec![Rational::integer(0); out.len()+a.len()-1];
                        for (i, x) in out.iter().enumerate() {
                            for (j, y) in a.iter().enumerate() {
                                next[i+j] = next[i+j].add(&x.mul(y).ok()?).ok()?;
                            }
                        }
                        if next.len() > 3 { return None; }
                        out = next;
                    }
                    Some(out)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn out_div(a: &[Rational], b: &Rational) -> Option<Vec<Rational>> {
    a.iter().map(|x| x.div(b).ok()).collect()
}

fn solve_polynomial(
    coeffs: &[Rational],
    var: &str,
    span: crate::diagnostics::Span,
    evaluator: &mut Evaluator<'_>,
) -> Result<Value, Diagnostic> {
    let zero = |r: &Rational| r.num == 0;
    let c = coeffs.first().cloned().unwrap_or_else(|| Rational::integer(0));
    let b = coeffs.get(1).cloned().unwrap_or_else(|| Rational::integer(0));
    let a = coeffs.get(2).cloned().unwrap_or_else(|| Rational::integer(0));
    if zero(&a) {
        if zero(&b) {
            return Ok(Value::Symbolic(call_expr("\solve", vec![
                value_to_rational_expr(&c, span),
                Expr::Symbol { name: var.to_owned(), span },
            ], span)));
        }
        let root = b.mul(&Rational::integer(-1)).and_then(|x| c.div(&x)).map_err(Diagnostic::new)?;
        return Ok(Value::Set(vec![Value::Rational(root)]));
    }
    let b2 = b.mul(&b).map_err(Diagnostic::new)?;
    let four_ac = a.mul(&c).and_then(|x| Rational::integer(4).mul(&x)).map_err(Diagnostic::new)?;
    let discriminant = b2.sub(&four_ac).map_err(Diagnostic::new)?;
    if discriminant.num < 0 {
        return Ok(Value::Symbolic(call_expr("\solve", vec![], span)));
    }
    let sqrt_expr = Expr::Sqrt {
        expr: Box::new(value_to_rational_expr(&discriminant, span)),
        span,
    };
    let neg_b = Expr::Unary { op: crate::ast::UnaryOp::Neg, expr: Box::new(value_to_rational_expr(&b, span)), span };
    let denom = Expr::Binary { op: BinOp::Mul, lhs: Box::new(Expr::Integer(2, span)), rhs: Box::new(value_to_rational_expr(&a, span)), span };
    let plus = Expr::Binary { op: BinOp::Div, lhs: Box::new(Expr::Binary { op: BinOp::Add, lhs: Box::new(neg_b.clone()), rhs: Box::new(sqrt_expr.clone()), span }), rhs: Box::new(denom.clone()), span };
    let minus = Expr::Binary { op: BinOp::Div, lhs: Box::new(Expr::Binary { op: BinOp::Sub, lhs: Box::new(neg_b), rhs: Box::new(sqrt_expr), span }), rhs: Box::new(denom), span };
    let p = evaluator.eval_expr(&plus, &HashMap::new())?;
    let m = evaluator.eval_expr(&minus, &HashMap::new())?;
    Ok(Value::Set(vec![p, m]))
}
