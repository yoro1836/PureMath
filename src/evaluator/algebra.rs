use super::elementary::{call_expr, value_to_rational_expr};
use super::prelude::*;
use super::symbolic::contains_symbol;
use super::Evaluator;

pub(super) fn polynomial_coefficients(
    expr: &Expr,
    var: &str,
    evaluator: &mut Evaluator<'_>,
    locals: &HashMap<String, Value>,
) -> Result<Vec<Rational>, Diagnostic> {
    let mut out = polynomial(expr, var, evaluator, locals).ok_or_else(|| {
        Diagnostic::new("solve currently supports polynomial equations of degree at most 2")
    })?;
    while out.len() > 1 && out.last().is_some_and(|x| x.num == 0) {
        out.pop();
    }
    Ok(out)
}

pub(super) fn polynomial(
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
        Expr::Symbol { name, .. } if name == var => {
            Some(vec![Rational::integer(0), Rational::integer(1)])
        }
        Expr::Symbol { .. } => match evaluator.eval_expr(expr, locals).ok()? {
            Value::Rational(r) => Some(vec![r]),
            _ => None,
        },
        Expr::Unary {
            op: crate::ast::UnaryOp::Neg,
            expr,
            ..
        } => {
            let mut p = polynomial(expr, var, evaluator, locals)?;
            for c in &mut p {
                c.num = c.num.checked_neg()?;
            }
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
                        out[i] = if *op == BinOp::Add {
                            av.add(&bv).ok()?
                        } else {
                            av.sub(&bv).ok()?
                        };
                    }
                    Some(out)
                }
                BinOp::Mul => {
                    if a.len() + b.len() > 4 {
                        return None;
                    }
                    let mut out = vec![Rational::integer(0); a.len() + b.len() - 1];
                    for (i, x) in a.iter().enumerate() {
                        for (j, y) in b.iter().enumerate() {
                            let product = x.mul(y).ok()?;
                            out[i + j] = out[i + j].add(&product).ok()?;
                        }
                    }
                    Some(out)
                }
                BinOp::Div if b.len() == 1 => out_div(&a, &b[0]),
                BinOp::Pow => {
                    if b.len() != 1 || !b[0].is_integer() || b[0].num < 0 || b[0].num > 2 {
                        return None;
                    }
                    let mut out = vec![Rational::integer(1)];
                    for _ in 0..b[0].num {
                        let mut next = vec![Rational::integer(0); out.len() + a.len() - 1];
                        for (i, x) in out.iter().enumerate() {
                            for (j, y) in a.iter().enumerate() {
                                next[i + j] = next[i + j].add(&x.mul(y).ok()?).ok()?;
                            }
                        }
                        if next.len() > 3 {
                            return None;
                        }
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

pub(super) fn out_div(a: &[Rational], b: &Rational) -> Option<Vec<Rational>> {
    a.iter().map(|x| x.div(b).ok()).collect()
}

pub(super) fn solve_polynomial(
    coeffs: &[Rational],
    var: &str,
    span: crate::diagnostics::Span,
    evaluator: &mut Evaluator<'_>,
) -> Result<Value, Diagnostic> {
    let zero = |r: &Rational| r.num == 0;
    let c = coeffs
        .first()
        .cloned()
        .unwrap_or_else(|| Rational::integer(0));
    let b = coeffs
        .get(1)
        .cloned()
        .unwrap_or_else(|| Rational::integer(0));
    let a = coeffs
        .get(2)
        .cloned()
        .unwrap_or_else(|| Rational::integer(0));
    if zero(&a) {
        if zero(&b) {
            return Ok(Value::Symbolic(call_expr(
                "\\solve",
                vec![
                    value_to_rational_expr(&c, span),
                    Expr::Symbol {
                        name: var.to_owned(),
                        span,
                    },
                ],
                span,
            )));
        }
        let root = b
            .mul(&Rational::integer(-1))
            .and_then(|x| c.div(&x))
            .map_err(Diagnostic::new)?;
        return Ok(Value::Set(vec![Value::Rational(root)]));
    }
    let b2 = b.mul(&b).map_err(Diagnostic::new)?;
    let four_ac = a
        .mul(&c)
        .and_then(|x| Rational::integer(4).mul(&x))
        .map_err(Diagnostic::new)?;
    let discriminant = b2.sub(&four_ac).map_err(Diagnostic::new)?;
    if discriminant.num < 0 {
        return Ok(Value::Symbolic(call_expr("\\solve", vec![], span)));
    }
    let sqrt_expr = Expr::Sqrt {
        expr: Box::new(value_to_rational_expr(&discriminant, span)),
        span,
    };
    let neg_b = Expr::Unary {
        op: crate::ast::UnaryOp::Neg,
        expr: Box::new(value_to_rational_expr(&b, span)),
        span,
    };
    let denom = Expr::Binary {
        op: BinOp::Mul,
        lhs: Box::new(Expr::Integer(2, span)),
        rhs: Box::new(value_to_rational_expr(&a, span)),
        span,
    };
    let plus = Expr::Binary {
        op: BinOp::Div,
        lhs: Box::new(Expr::Binary {
            op: BinOp::Add,
            lhs: Box::new(neg_b.clone()),
            rhs: Box::new(sqrt_expr.clone()),
            span,
        }),
        rhs: Box::new(denom.clone()),
        span,
    };
    let minus = Expr::Binary {
        op: BinOp::Div,
        lhs: Box::new(Expr::Binary {
            op: BinOp::Sub,
            lhs: Box::new(neg_b),
            rhs: Box::new(sqrt_expr),
            span,
        }),
        rhs: Box::new(denom),
        span,
    };
    let p = evaluator.eval_expr(&plus, &HashMap::new())?;
    let m = evaluator.eval_expr(&minus, &HashMap::new())?;
    let mut roots = vec![p, m];
    roots.sort_by_key(|value| value.to_string());
    Ok(Value::Set(roots))
}
