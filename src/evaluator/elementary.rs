use super::prelude::*;

pub(super) fn exact_sqrt_rational(value: &Rational) -> Option<Rational> {
    if value.num < 0 || value.den <= 0 {
        return None;
    }
    let numerator = value.num as u128;
    let denominator = value.den as u128;
    let n = exact_integer_sqrt(numerator);
    let d = exact_integer_sqrt(denominator);
    if n.checked_mul(n)? == numerator && d.checked_mul(d)? == denominator {
        Rational::new(n as i128, d as i128).ok()
    } else {
        None
    }
}

pub(super) fn value_to_rational_expr(value: &Rational, span: crate::diagnostics::Span) -> Expr {
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

pub(super) fn call_expr(name: &str, args: Vec<Expr>, span: crate::diagnostics::Span) -> Expr {
    Expr::Call {
        callee: Box::new(Expr::Symbol {
            name: name.to_owned(),
            span,
        }),
        args,
        span,
    }
}

pub(super) fn elementary_exact(
    name: &str,
    value: &Value,
    span: crate::diagnostics::Span,
) -> Option<Value> {
    match name {
        "\\sin" => {
            if matches!(value, Value::Rational(r) if r.num == 0) {
                return Some(Value::Rational(Rational::integer(0)));
            }
        }
        "\\cos" => {
            if matches!(value, Value::Rational(r) if r.num == 0) {
                return Some(Value::Rational(Rational::integer(1)));
            }
        }
        "\\tan" => {
            if matches!(value, Value::Rational(r) if r.num == 0) {
                return Some(Value::Rational(Rational::integer(0)));
            }
        }
        "\\ln" => {
            if matches!(value, Value::Rational(r) if r.num == r.den) {
                return Some(Value::Rational(Rational::integer(0)));
            }
        }
        "\\exp" => {
            if matches!(value, Value::Rational(r) if r.num == 0) {
                return Some(Value::Rational(Rational::integer(1)));
            }
        }
        "\\log" => {
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
        ("\\sin", Expr::Symbol { name, .. }) if name == "\\pi" => {
            Some(Value::Rational(Rational::integer(0)))
        }
        ("\\cos", Expr::Symbol { name, .. }) if name == "\\pi" => {
            Some(Value::Rational(Rational::integer(-1)))
        }
        ("\\ln", Expr::Symbol { name, .. }) if name == "\\e" => {
            Some(Value::Rational(Rational::integer(1)))
        }
        ("\\exp", Expr::Integer(1, _)) => Some(Value::Symbolic(Expr::Symbol {
            name: "\\e".to_owned(),
            span,
        })),
        _ => None,
    }
}
