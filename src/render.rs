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
            BinOp::Pow => format!("{}^{{{}}}", format_child(lhs, 30, false), expr_inner(rhs)),
            BinOp::Mul => format!(
                "{} \\cdot {}",
                format_child(lhs, 20, false),
                format_child(rhs, 20, true)
            ),
            BinOp::Add => format!(
                "{} + {}",
                format_child(lhs, 10, false),
                format_child(rhs, 10, true)
            ),
            BinOp::Sub => format!(
                "{} - {}",
                format_child(lhs, 10, false),
                format_child(rhs, 10, true)
            ),
            BinOp::Eq => format!(
                "{} = {}",
                format_child(lhs, 5, false),
                format_child(rhs, 5, true)
            ),
            BinOp::Ne => format!(
                "{} \\neq {}",
                format_child(lhs, 5, false),
                format_child(rhs, 5, true)
            ),
            BinOp::Lt => format!(
                "{} < {}",
                format_child(lhs, 5, false),
                format_child(rhs, 5, true)
            ),
            BinOp::Le => format!("{} \\le {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Gt => format!(
                "{} > {}",
                format_child(lhs, 5, false),
                format_child(rhs, 5, true)
            ),
            BinOp::Ge => format!("{} \\ge {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Union => format!("{} \\cup {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Intersect => format!("{} \\cap {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Difference => format!("{} \\setminus {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::In => format!("{} \\in {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::Subset => format!("{} \\subset {}", parenthesize(lhs), parenthesize(rhs)),
            BinOp::SubsetEq => format!("{} \\subseteq {}", parenthesize(lhs), parenthesize(rhs)),
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
        Expr::SetComprehension {
            var,
            domain,
            condition,
            ..
        } => format!(
            "\\{{{} \\in {} \\mid {}\\}}",
            var,
            expr_inner(domain),
            expr_inner(condition)
        ),
        Expr::Vector { elements, .. } => format!(
            "\\left({}\\right)",
            elements
                .iter()
                .map(expr_inner)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Matrix { rows, .. } => {
            let body = rows
                .iter()
                .map(|row| row.iter().map(expr_inner).collect::<Vec<_>>().join(" & "))
                .collect::<Vec<_>>()
                .join(" \\\\ ");
            format!("\\begin{{pmatrix}} {} \\end{{pmatrix}}", body)
        }
        Expr::Integral {
            var,
            lower,
            upper,
            body,
            ..
        } => match (lower, upper) {
            (Some(lower), Some(upper)) => format!(
                "\\int_{{{}}}^{{{}}} {} \\,d{}",
                expr_inner(lower),
                expr_inner(upper),
                expr_inner(body),
                var
            ),
            _ => format!("\\int {} \\,d{}", expr_inner(body), var),
        },
        Expr::Limit {
            var, target, body, ..
        } => format!(
            "\\lim_{{{} \\to {}}} {}",
            var,
            expr_inner(target),
            expr_inner(body)
        ),
        Expr::Product {
            var,
            lower,
            upper,
            body,
            ..
        } => format!(
            "\\prod_{{{}={}}}^{{{}}} {}",
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
        Expr::Abs { expr: inner, .. } => format!("\\left|{}\\right|", expr_inner(inner)),
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

fn format_child(node: &Expr, parent_precedence: u8, right_side: bool) -> String {
    let text = expr_inner(node);
    let precedence = match node {
        Expr::Binary { op, .. } => match op {
            BinOp::Eq
            | BinOp::Ne
            | BinOp::Lt
            | BinOp::Le
            | BinOp::Gt
            | BinOp::Ge
            | BinOp::In
            | BinOp::Subset
            | BinOp::SubsetEq => 5,
            BinOp::Union => 7,
            BinOp::Add | BinOp::Sub => 10,
            BinOp::Intersect | BinOp::Difference => 8,
            BinOp::Mul | BinOp::Div => 20,
            BinOp::Pow => 30,
        },
        Expr::Unary { .. } => 25,
        _ => 40,
    };
    if precedence < parent_precedence
        || (right_side
            && precedence == parent_precedence
            && matches!(
                node,
                Expr::Binary {
                    op: BinOp::Sub | BinOp::Div,
                    ..
                }
            ))
    {
        format!("({})", text)
    } else {
        text
    }
}

fn parenthesize(node: &Expr) -> String {
    match node {
        Expr::Integer(_, _)
        | Expr::Symbol { .. }
        | Expr::Call { .. }
        | Expr::Sqrt { .. }
        | Expr::Abs { .. } => expr_inner(node),
        _ => format!("({})", expr_inner(node)),
    }
}
