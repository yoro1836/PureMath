use crate::ast::{BinOp, Expr, UnaryOp};

pub fn expr(node: &Expr) -> String {
    expr_inner(node)
}

fn expr_inner(node: &Expr) -> String {
    match node {
        Expr::Integer(n, _) => n.to_string(),
        Expr::Rational {
            numerator,
            denominator,
            ..
        } => format!(
            "\\frac{{{}}}{{{}}}",
            expr_inner(numerator),
            expr_inner(denominator)
        ),
        Expr::Symbol { name, .. } => name.clone(),
        Expr::Unary {
            op: UnaryOp::Neg,
            expr: inner,
            ..
        } => format!("-{}", parenthesize(inner)),
        Expr::Binary { op, lhs, rhs, .. } => match op {
            BinOp::Div => format!("\\frac{{{}}}{{{}}}", expr_inner(lhs), expr_inner(rhs)),
            BinOp::Pow => format!("{}^{{{}}}", parenthesize(lhs), expr_inner(rhs)),
            BinOp::Mul => format!("{} \\cdot {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Add => format!("{} + {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Sub => format!("{} - {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Eq => format!("{} = {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Lt => format!("{} < {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Le => format!("{} \\le {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Gt => format!("{} > {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Ge => format!("{} \\ge {}", parenthesize(lhs), parenthesize(rhs)),
        },
        Expr::Call { callee, args, .. } => format!(
            "{}({})",
            expr_inner(callee),
            args.iter().map(expr_inner).collect::<Vec<_>>().join(", ")
        ),
        Expr::Set { elements, .. } => {
            let body = elements
                .iter()
                .map(expr_inner)
                .collect::<Vec<_>>()
                .join(", ");
            format!("\\{{{}\\}}", body)
        }
        Expr::Product {
            var,
            lower,
            upper,
            body,
            ..
        } => format!(
            "\\\\prod_{{{}={}}}^{{{}}} {}",
            var,
            expr_inner(lower),
            expr_inner(upper),
            expr_inner(body)
        ),
        Expr::Sum {
            var,
            lower,
            upper,
            body,
            ..
        } => format!(
            "\\sum_{{{}={}}}^{{{}}} {}",
            var,
            expr_inner(lower),
            expr_inner(upper),
            expr_inner(body)
        ),
        Expr::Abs { expr: inner, .. } => format!("\\\\left|{}\\\\right|", expr_inner(inner)),
        Expr::Sqrt { expr: inner, .. } => format!("\\sqrt{{{}}}", expr_inner(inner)),
        Expr::Piecewise { branches, .. } => {
            let rows = branches
                .iter()
                .map(|b| format!("{} & {}", expr_inner(&b.value), expr_inner(&b.condition)))
                .collect::<Vec<_>>()
                .join(" \\\\ ");
            format!("\\begin{{cases}} {} \\end{{cases}}", rows)
        }
        Expr::Opaque { text, .. } => text.clone(),
    }
}

fn parenthesize(node: &Expr) -> String {
    match node {
        Expr::Integer(_, _) | Expr::Symbol { .. } | Expr::Call { .. } | Expr::Sqrt { .. } => {
            expr_inner(node)
        }
        _ => format!("({})", expr_inner(node)),
    }
}
