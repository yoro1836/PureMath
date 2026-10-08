use super::elementary::call_expr;
use super::prelude::*;

pub(super) fn contains_symbol(expr: &Expr, var: &str) -> bool {
    match expr {
        Expr::Symbol { name, .. } => name == var,
        Expr::Unary { expr, .. } => contains_symbol(expr, var),
        Expr::Binary { lhs, rhs, .. } => contains_symbol(lhs, var) || contains_symbol(rhs, var),
        Expr::Call { callee, args, .. } => {
            contains_symbol(callee, var) || args.iter().any(|arg| contains_symbol(arg, var))
        }
        Expr::Rational {
            numerator,
            denominator,
            ..
        } => contains_symbol(numerator, var) || contains_symbol(denominator, var),
        Expr::Set { elements, .. } | Expr::Vector { elements, .. } => {
            elements.iter().any(|e| contains_symbol(e, var))
        }
        Expr::SetComprehension { var: bound, domain, condition, .. } => {
            contains_symbol(domain, var)
                || (bound != var && contains_symbol(condition, var))
        }
        Expr::Matrix { rows, .. } => rows.iter().flatten().any(|e| contains_symbol(e, var)),
        Expr::Integral {
            body, lower, upper, ..
        } => {
            contains_symbol(body, var)
                || lower.as_deref().is_some_and(|e| contains_symbol(e, var))
                || upper.as_deref().is_some_and(|e| contains_symbol(e, var))
        }
        Expr::Limit { body, target, .. } => {
            contains_symbol(body, var) || contains_symbol(target, var)
        }
        Expr::Product {
            body, lower, upper, ..
        }
        | Expr::Sum {
            body, lower, upper, ..
        } => {
            contains_symbol(body, var) || contains_symbol(lower, var) || contains_symbol(upper, var)
        }
        Expr::Abs { expr, .. } | Expr::Sqrt { expr, .. } => contains_symbol(expr, var),
        Expr::Piecewise { branches, .. } => branches
            .iter()
            .any(|b| contains_symbol(&b.value, var) || contains_symbol(&b.condition, var)),
        Expr::Integer(..) | Expr::Opaque { .. } => false,
    }
}

pub(super) fn differentiate(expr: &Expr, var: &str) -> Option<Expr> {
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
                    let Expr::Integer(n, _) = rhs.as_ref() else {
                        return None;
                    };
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
            let Expr::Symbol { name, .. } = callee.as_ref() else {
                return None;
            };
            let dx = differentiate(&args[0], var)?;
            let x = args[0].clone();
            match name.as_str() {
                "\\sin" => Some(Expr::Binary {
                    op: BinOp::Mul,
                    lhs: Box::new(call_expr("\\cos", vec![x], span)),
                    rhs: Box::new(dx),
                    span,
                }),
                "\\cos" => Some(Expr::Unary {
                    op: crate::ast::UnaryOp::Neg,
                    expr: Box::new(Expr::Binary {
                        op: BinOp::Mul,
                        lhs: Box::new(call_expr("\\sin", vec![x], span)),
                        rhs: Box::new(dx),
                        span,
                    }),
                    span,
                }),
                "\\tan" => Some(Expr::Binary {
                    op: BinOp::Div,
                    lhs: Box::new(dx),
                    rhs: Box::new(Expr::Binary {
                        op: BinOp::Pow,
                        lhs: Box::new(call_expr("\\cos", vec![x], span)),
                        rhs: Box::new(Expr::Integer(2, span)),
                        span,
                    }),
                    span,
                }),
                "\\exp" => Some(Expr::Binary {
                    op: BinOp::Mul,
                    lhs: Box::new(call_expr("\\exp", vec![x], span)),
                    rhs: Box::new(dx),
                    span,
                }),
                "\\ln" | "\\log" => Some(Expr::Binary {
                    op: BinOp::Div,
                    lhs: Box::new(dx),
                    rhs: Box::new(x),
                    span,
                }),
                "\\sqrt" => Some(Expr::Binary {
                    op: BinOp::Div,
                    lhs: Box::new(dx),
                    rhs: Box::new(Expr::Binary {
                        op: BinOp::Mul,
                        lhs: Box::new(Expr::Integer(2, span)),
                        rhs: Box::new(call_expr("\\sqrt", vec![x], span)),
                        span,
                    }),
                    span,
                }),
                _ => None,
            }
        }
        Expr::Abs { .. }
        | Expr::Sqrt { .. }
        | Expr::Set { .. }
        | Expr::Vector { .. }
        | Expr::Matrix { .. } => None,
        _ => None,
    }
}

pub(super) fn substitute(expr: &Expr, var: &str, replacement: &Expr) -> Expr {
    match expr {
        Expr::Symbol { name, .. } if name == var => replacement.clone(),
        Expr::Unary { op, expr, span } => Expr::Unary {
            op: *op,
            expr: Box::new(substitute(expr, var, replacement)),
            span: *span,
        },
        Expr::Binary { op, lhs, rhs, span } => Expr::Binary {
            op: *op,
            lhs: Box::new(substitute(lhs, var, replacement)),
            rhs: Box::new(substitute(rhs, var, replacement)),
            span: *span,
        },
        Expr::Call { callee, args, span } => Expr::Call {
            callee: Box::new(substitute(callee, var, replacement)),
            args: args
                .iter()
                .map(|arg| substitute(arg, var, replacement))
                .collect(),
            span: *span,
        },
        Expr::Rational {
            numerator,
            denominator,
            span,
        } => Expr::Rational {
            numerator: Box::new(substitute(numerator, var, replacement)),
            denominator: Box::new(substitute(denominator, var, replacement)),
            span: *span,
        },
        Expr::Set { elements, span } => Expr::Set {
            elements: elements
                .iter()
                .map(|e| substitute(e, var, replacement))
                .collect(),
            span: *span,
        },
        Expr::SetComprehension { var: bound, domain, condition, span } => {
            Expr::SetComprehension {
                var: bound.clone(),
                domain: Box::new(substitute(domain, var, replacement)),
                condition: Box::new(if bound == var {
                    condition.as_ref().clone()
                } else {
                    substitute(condition, var, replacement)
                }),
                span: *span,
            }
        },
        Expr::Vector { elements, span } => Expr::Vector {
            elements: elements
                .iter()
                .map(|e| substitute(e, var, replacement))
                .collect(),
            span: *span,
        },
        Expr::Matrix { rows, span } => Expr::Matrix {
            rows: rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|e| substitute(e, var, replacement))
                        .collect()
                })
                .collect(),
            span: *span,
        },
        Expr::Abs { expr, span } => Expr::Abs {
            expr: Box::new(substitute(expr, var, replacement)),
            span: *span,
        },
        Expr::Sqrt { expr, span } => Expr::Sqrt {
            expr: Box::new(substitute(expr, var, replacement)),
            span: *span,
        },
        Expr::Product {
            var: bound,
            lower,
            upper,
            body,
            span,
        } if bound != var => Expr::Product {
            var: bound.clone(),
            lower: Box::new(substitute(lower, var, replacement)),
            upper: Box::new(substitute(upper, var, replacement)),
            body: Box::new(substitute(body, var, replacement)),
            span: *span,
        },
        Expr::Sum {
            var: bound,
            lower,
            upper,
            body,
            span,
        } if bound != var => Expr::Sum {
            var: bound.clone(),
            lower: Box::new(substitute(lower, var, replacement)),
            upper: Box::new(substitute(upper, var, replacement)),
            body: Box::new(substitute(body, var, replacement)),
            span: *span,
        },
        Expr::Integral {
            var: bound,
            lower,
            upper,
            body,
            span,
        } if bound != var => Expr::Integral {
            var: bound.clone(),
            lower: lower
                .as_deref()
                .map(|e| Box::new(substitute(e, var, replacement))),
            upper: upper
                .as_deref()
                .map(|e| Box::new(substitute(e, var, replacement))),
            body: Box::new(substitute(body, var, replacement)),
            span: *span,
        },
        Expr::Limit {
            var: bound,
            target,
            body,
            span,
        } if bound != var => Expr::Limit {
            var: bound.clone(),
            target: Box::new(substitute(target, var, replacement)),
            body: Box::new(substitute(body, var, replacement)),
            span: *span,
        },
        Expr::Piecewise { branches, span } => Expr::Piecewise {
            branches: branches
                .iter()
                .map(|b| crate::ast::PiecewiseBranch {
                    value: substitute(&b.value, var, replacement),
                    condition: substitute(&b.condition, var, replacement),
                    span: b.span,
                })
                .collect(),
            span: *span,
        },
        _ => expr.clone(),
    }
}

pub(super) fn integrate_expr(expr: &Expr, var: &str) -> Option<Expr> {
    let span = expr.span();
    match expr {
        Expr::Integer(_, _) | Expr::Rational { .. } if !contains_symbol(expr, var) => {
            Some(Expr::Binary {
                op: BinOp::Mul,
                lhs: Box::new(expr.clone()),
                rhs: Box::new(Expr::Symbol {
                    name: var.to_owned(),
                    span,
                }),
                span,
            })
        }
        Expr::Symbol { name, .. } if name == var => Some(Expr::Binary {
            op: BinOp::Div,
            lhs: Box::new(Expr::Binary {
                op: BinOp::Pow,
                lhs: Box::new(expr.clone()),
                rhs: Box::new(Expr::Integer(2, span)),
                span,
            }),
            rhs: Box::new(Expr::Integer(2, span)),
            span,
        }),
        Expr::Symbol { .. } => Some(Expr::Binary {
            op: BinOp::Mul,
            lhs: Box::new(expr.clone()),
            rhs: Box::new(Expr::Symbol {
                name: var.to_owned(),
                span,
            }),
            span,
        }),
        Expr::Unary { expr, .. } => integrate_expr(expr, var).map(|inner| Expr::Unary {
            op: crate::ast::UnaryOp::Neg,
            expr: Box::new(inner),
            span,
        }),
        Expr::Binary {
            op: BinOp::Add,
            lhs,
            rhs,
            ..
        }
        | Expr::Binary {
            op: BinOp::Sub,
            lhs,
            rhs,
            ..
        } => Some(Expr::Binary {
            op: if matches!(expr, Expr::Binary { op: BinOp::Add, .. }) {
                BinOp::Add
            } else {
                BinOp::Sub
            },
            lhs: Box::new(integrate_expr(lhs, var)?),
            rhs: Box::new(integrate_expr(rhs, var)?),
            span,
        }),
        Expr::Binary {
            op: BinOp::Mul,
            lhs,
            rhs,
            ..
        } => {
            if !contains_symbol(lhs, var) {
                let inner = integrate_expr(rhs, var)?;
                Some(Expr::Binary {
                    op: BinOp::Mul,
                    lhs: lhs.clone(),
                    rhs: Box::new(inner),
                    span,
                })
            } else if !contains_symbol(rhs, var) {
                let inner = integrate_expr(lhs, var)?;
                Some(Expr::Binary {
                    op: BinOp::Mul,
                    lhs: rhs.clone(),
                    rhs: Box::new(inner),
                    span,
                })
            } else {
                None
            }
        }
        Expr::Binary {
            op: BinOp::Pow,
            lhs,
            rhs,
            ..
        } if matches!(lhs.as_ref(), Expr::Symbol { name, .. } if name == var) => {
            let Expr::Integer(n, _) = rhs.as_ref() else {
                return None;
            };
            if *n == -1 {
                Some(call_expr("\\ln", vec![lhs.as_ref().clone()], span))
            } else {
                let next = n.checked_add(1)?;
                Some(Expr::Binary {
                    op: BinOp::Div,
                    lhs: Box::new(Expr::Binary {
                        op: BinOp::Pow,
                        lhs: lhs.clone(),
                        rhs: Box::new(Expr::Integer(next, span)),
                        span,
                    }),
                    rhs: Box::new(Expr::Integer(next, span)),
                    span,
                })
            }
        }
        Expr::Binary {
            op: BinOp::Div,
            lhs,
            rhs,
            ..
        } if !contains_symbol(rhs, var) => {
            let inner = integrate_expr(lhs, var)?;
            Some(Expr::Binary {
                op: BinOp::Div,
                lhs: Box::new(inner),
                rhs: rhs.clone(),
                span,
            })
        }
        _ => None,
    }
}
