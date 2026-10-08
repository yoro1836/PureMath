use super::prelude::*;
use super::Evaluator;

use super::arithmetic::*;
use super::elementary::*;
use super::linear_algebra::*;
use super::sets::*;
use super::symbolic::*;

impl<'a> Evaluator<'a> {
    pub(super) fn eval_expr(
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
                        return super::builtins::eval_builtin(self, name, args, *span, locals);
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
                var, target, body, ..
            } => self.eval_limit(var, target, body, expr, locals),
            Expr::Abs { expr: inner, span } => match self.eval_expr(inner, locals)? {
                Value::Rational(v) => Ok(Value::Rational(
                    Rational::new(
                        v.num
                            .checked_abs()
                            .ok_or_else(|| Diagnostic::new("integer overflow in absolute value"))?,
                        v.den,
                    )
                    .map_err(Diagnostic::new)?,
                )),
                Value::Symbolic(v) => Ok(Value::Symbolic(Expr::Abs {
                    expr: Box::new(v),
                    span: *span,
                })),
                other => Err(Diagnostic::at(
                    format!("cannot take absolute value of {}", other),
                    *span,
                )),
            },
            Expr::Sqrt { expr: inner, span } => match self.eval_expr(inner, locals)? {
                Value::Rational(v) if v.num >= 0 => {
                    if let Some(root) = exact_sqrt_rational(&v) {
                        Ok(Value::Rational(root))
                    } else {
                        Ok(Value::Symbolic(Expr::Sqrt {
                            expr: Box::new(value_to_rational_expr(&v, *span)),
                            span: *span,
                        }))
                    }
                }
                Value::Symbolic(v) => Ok(Value::Symbolic(Expr::Sqrt {
                    expr: Box::new(v),
                    span: *span,
                })),
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
                    return Err(Diagnostic::new(
                        "finite product exceeds evaluation term limit",
                    ));
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
            | BinOp::SubsetEq => match compare(op, &a, &b) {
                Ok(value) => Ok(Value::Bool(value)),
                Err(_) => Ok(Value::Symbolic(Expr::Binary {
                    op,
                    lhs: Box::new(value_to_expr(&a, lhs)),
                    rhs: Box::new(value_to_expr(&b, rhs)),
                    span,
                })),
            },
            BinOp::Union | BinOp::Intersect | BinOp::Difference => {
                set_binary(op, &a, &b, lhs, rhs, span)
            }
            BinOp::Add | BinOp::Sub => Ok(structured_add_sub(op, &a, &b, lhs, rhs, span)),
            BinOp::Mul => Ok(structured_mul(&a, &b, lhs, rhs, span)),
            BinOp::Div => numeric_or_symbolic(BinOp::Div, a, b, lhs, rhs, |x, y| x.div(y)),
            BinOp::Pow => match (a, b) {
                (Value::Rational(x), Value::Rational(y)) if y.is_integer() => {
                    x.powi(y.num).map(Value::Rational).map_err(Diagnostic::new)
                }
                (Value::Matrix(matrix), Value::Rational(exp))
                    if exp.is_integer() && exp.num >= 0 =>
                {
                    matrix_pow(&matrix, exp.num)
                        .map(Value::Matrix)
                        .map_err(Diagnostic::new)
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
        let antiderivative = integrate_expr(body, var).ok_or_else(|| {
            Diagnostic::new("integral is outside the exact polynomial integration subset")
        })?;
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
}
