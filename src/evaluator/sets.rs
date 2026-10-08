use super::arithmetic::{symbolic_binary, values_equal};
use super::prelude::*;

pub(super) fn set_binary(
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
