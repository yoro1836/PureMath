use crate::ast::{BinOp, Expr, UnaryOp};
use crate::diagnostics::Span;

/// Small, semantics-preserving normalization rules used by the prototype.
/// This is intentionally not a CAS; it is a separate layer that can grow later.
pub fn simplify(expr: Expr) -> Expr {
    match expr {
        Expr::Unary { op: UnaryOp::Neg, expr: inner, span } => {
            let inner = simplify(*inner);
            if let Expr::Unary { op: UnaryOp::Neg, expr: nested, .. } = inner {
                return simplify(*nested);
            }
            Expr::Unary { op: UnaryOp::Neg, expr: Box::new(inner), span }
        }
        Expr::Binary { op, lhs, rhs, span } => {
            let lhs = simplify(*lhs);
            let rhs = simplify(*rhs);

            match op {
                BinOp::Add if is_zero(&rhs) => lhs,
                BinOp::Add if is_zero(&lhs) => rhs,
                BinOp::Sub if is_zero(&rhs) => lhs,
                BinOp::Mul if is_zero(&lhs) || is_zero(&rhs) => integer(0, span),
                BinOp::Mul if is_one(&lhs) => rhs,
                BinOp::Mul if is_one(&rhs) => lhs,
                BinOp::Div if is_one(&rhs) => lhs,
                BinOp::Pow if is_one(&rhs) => lhs,
                _ => Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), span },
            }
        }
        Expr::Rational { numerator, denominator, span } => Expr::Rational {
            numerator: Box::new(simplify(*numerator)),
            denominator: Box::new(simplify(*denominator)),
            span,
        },
        Expr::Call { callee, args, span } => Expr::Call {
            callee: Box::new(simplify(*callee)),
            args: args.into_iter().map(simplify).collect(),
            span,
        },
        Expr::Set { elements, span } => Expr::Set { elements: elements.into_iter().map(simplify).collect(), span },
        Expr::Sum { var, lower, upper, body, span } => Expr::Sum {
            var,
            lower: Box::new(simplify(*lower)),
            upper: Box::new(simplify(*upper)),
            body: Box::new(simplify(*body)),
            span,
        },
        Expr::Sqrt { expr: inner, span } => Expr::Sqrt { expr: Box::new(simplify(*inner)), span },
        Expr::Piecewise { branches, span } => Expr::Piecewise {
            branches: branches.into_iter().map(|branch| crate::ast::PiecewiseBranch {
                value: simplify(branch.value),
                condition: simplify(branch.condition),
                span: branch.span,
            }).collect(),
            span,
        },
        other => other,
    }
}

fn integer(value: i128, span: Span) -> Expr { Expr::Integer(value, span) }

fn is_integer(expr: &Expr, expected: i128) -> bool {
    matches!(expr, Expr::Integer(value, _) if *value == expected)
}

fn is_zero(expr: &Expr) -> bool { is_integer(expr, 0) }
fn is_one(expr: &Expr) -> bool { is_integer(expr, 1) }
