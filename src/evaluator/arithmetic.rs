use super::linear_algebra::*;
use super::prelude::*;

pub(super) fn numeric_or_symbolic<F>(
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

pub(super) fn compare(op: BinOp, a: &Value, b: &Value) -> Result<bool, String> {
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
            let contained = left
                .iter()
                .all(|value| right.iter().any(|item| values_equal(value, item)));
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
            _ => Err("comparison requires compatible exact values".into()),
        },
        _ => Err("unsupported comparison".into()),
    }
}

pub(super) fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Rational(x), Value::Rational(y)) => {
            x.num.checked_mul(y.den) == y.num.checked_mul(x.den)
        }
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Set(x), Value::Set(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|value| y.iter().any(|item| values_equal(value, item)))
        }
        (Value::Vector(x), Value::Vector(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| values_equal(a, b))
        }
        (Value::Matrix(x), Value::Matrix(y)) => {
            x.len() == y.len()
                && x.iter().zip(y).all(|(ra, rb)| {
                    ra.len() == rb.len() && ra.iter().zip(rb).all(|(a, b)| values_equal(a, b))
                })
        }
        _ => a == b,
    }
}

pub(super) fn symbolic_binary(
    op: BinOp,
    a: &Value,
    b: &Value,
    lhs: &Expr,
    rhs: &Expr,
    span: crate::diagnostics::Span,
) -> Value {
    Value::Symbolic(simplifier::simplify(Expr::Binary {
        op,
        lhs: Box::new(value_to_expr(a, lhs)),
        rhs: Box::new(value_to_expr(b, rhs)),
        span,
    }))
}

pub(super) fn structured_add_sub(
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
            result
                .map(Value::Rational)
                .unwrap_or_else(|_| symbolic_binary(op, a, b, lhs, rhs, span))
        }
        (Value::Vector(x), Value::Vector(y)) if x.len() == y.len() => {
            let mut out = Vec::with_capacity(x.len());
            for (vx, vy) in x.iter().zip(y) {
                let (Value::Rational(rx), Value::Rational(ry)) = (vx, vy) else {
                    return symbolic_binary(op, a, b, lhs, rhs, span);
                };
                let value = if op == BinOp::Add {
                    rx.add(ry)
                } else {
                    rx.sub(ry)
                };
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
                    let value = if op == BinOp::Add {
                        rx.add(ry)
                    } else {
                        rx.sub(ry)
                    };
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

pub(super) fn structured_mul(
    a: &Value,
    b: &Value,
    lhs: &Expr,
    rhs: &Expr,
    span: crate::diagnostics::Span,
) -> Value {
    match (a, b) {
        (Value::Rational(x), Value::Rational(y)) => x
            .mul(y)
            .map(Value::Rational)
            .unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span)),
        (Value::Rational(s), Value::Vector(v)) | (Value::Vector(v), Value::Rational(s)) => {
            scale_vector(v, s)
                .map(Value::Vector)
                .unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        (Value::Rational(s), Value::Matrix(m)) | (Value::Matrix(m), Value::Rational(s)) => {
            scale_matrix(m, s)
                .map(Value::Matrix)
                .unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        (Value::Vector(left), Value::Vector(right)) if left.len() == right.len() => {
            dot_vectors(left, right)
                .map(Value::Rational)
                .unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        (Value::Matrix(matrix), Value::Vector(vector))
            if !matrix.is_empty() && matrix[0].len() == vector.len() =>
        {
            matrix_vector_mul(matrix, vector)
                .map(Value::Vector)
                .unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        (Value::Matrix(left), Value::Matrix(right))
            if !left.is_empty() && !right.is_empty() && left[0].len() == right.len() =>
        {
            matrix_matrix_mul(left, right)
                .map(Value::Matrix)
                .unwrap_or_else(|_| symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span))
        }
        _ => symbolic_binary(BinOp::Mul, a, b, lhs, rhs, span),
    }
}
