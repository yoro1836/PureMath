use super::prelude::*;
use super::Evaluator;

use super::algebra::*;
use super::elementary::*;
use super::linear_algebra::*;
use super::number_theory::*;
use super::statistics::*;
use super::symbolic::*;

pub(super) fn bare_builtin_name(name: &str) -> Option<&'static str> {
    match name {
        "factorial" => Some("\\factorial"),
        "binom" => Some("\\binom"),
        "choose" => Some("\\choose"),
        "perm" => Some("\\perm"),
        "permutation" => Some("\\permutation"),
        "gcd" => Some("\\gcd"),
        "lcm" => Some("\\lcm"),
        "floor" => Some("\\floor"),
        "ceil" => Some("\\ceil"),
        "min" => Some("\\min"),
        "max" => Some("\\max"),
        "dot" => Some("\\dot"),
        "norm" => Some("\\norm"),
        "det" => Some("\\det"),
        "transpose" => Some("\\transpose"),
        "trans" => Some("\\trans"),
        "inverse" => Some("\\inverse"),
        "inv" => Some("\\inv"),
        "rank" => Some("\\rank"),
        "card" => Some("\\card"),
        "cardinality" => Some("\\cardinality"),
        "trace" => Some("\\trace"),
        "mean" => Some("\\mean"),
        "variance" => Some("\\variance"),
        "stdev" => Some("\\stdev"),
        "diff" => Some("\\diff"),
        "derivative" => Some("\\derivative"),
        "subs" => Some("\\subs"),
        "substitute" => Some("\\substitute"),
        "solve" => Some("\\solve"),
        "sin" => Some("\\sin"),
        "cos" => Some("\\cos"),
        "tan" => Some("\\tan"),
        "ln" => Some("\\ln"),
        "log" => Some("\\log"),
        "exp" => Some("\\exp"),
        "range" => Some("\\range"),
        "tuple" => Some("\\tuple"),
        _ => None,
    }
}

pub(super) fn eval_builtin(
    evaluator: &mut Evaluator<'_>,
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
        "\\factorial" => {
            require(1)?;
            match evaluator.eval_expr(&args[0], locals)? {
                Value::Rational(n) if n.is_integer() && n.num >= 0 => factorial(n.num)
                    .map(Value::Rational)
                    .map_err(Diagnostic::new),
                _ => Err(Diagnostic::at(
                    "factorial requires a non-negative integer",
                    span,
                )),
            }
        }
        "\\binom" | "\\choose" => {
            require(2)?;
            let a = evaluator.eval_expr(&args[0], locals)?;
            let b = evaluator.eval_expr(&args[1], locals)?;
            match (a, b) {
                (Value::Rational(n), Value::Rational(k))
                    if n.is_integer() && k.is_integer() && n.num >= 0 && k.num >= 0 =>
                {
                    binom(n.num, k.num)
                        .map(Value::Rational)
                        .map_err(Diagnostic::new)
                }
                _ => Err(Diagnostic::at(
                    "binomial coefficient requires non-negative integers",
                    span,
                )),
            }
        }
        "\\perm" | "\\permutation" => {
            require(2)?;
            let a = evaluator.eval_expr(&args[0], locals)?;
            let b = evaluator.eval_expr(&args[1], locals)?;
            match (a, b) {
                (Value::Rational(n), Value::Rational(k))
                    if n.is_integer() && k.is_integer() && n.num >= 0 && k.num >= 0 =>
                {
                    perm(n.num, k.num)
                        .map(Value::Rational)
                        .map_err(Diagnostic::new)
                }
                _ => Err(Diagnostic::at(
                    "permutation requires non-negative integers",
                    span,
                )),
            }
        }
        "\\gcd" => {
            require(2)?;
            let a = evaluator.eval_expr(&args[0], locals)?;
            let b = evaluator.eval_expr(&args[1], locals)?;
            match (a, b) {
                (Value::Rational(a), Value::Rational(b)) if a.is_integer() && b.is_integer() => {
                    Ok(Value::Rational(Rational::integer(gcd_i128(a.num, b.num))))
                }
                _ => Err(Diagnostic::at("gcd requires integers", span)),
            }
        }
        "\\lcm" => {
            require(2)?;
            let a = evaluator.eval_expr(&args[0], locals)?;
            let b = evaluator.eval_expr(&args[1], locals)?;
            match (a, b) {
                (Value::Rational(a), Value::Rational(b)) if a.is_integer() && b.is_integer() => {
                    lcm_i128(a.num, b.num)
                        .map(|n| Value::Rational(Rational::integer(n)))
                        .map_err(Diagnostic::new)
                }
                _ => Err(Diagnostic::at("lcm requires integers", span)),
            }
        }
        "\\floor" | "\\ceil" => {
            require(1)?;
            match evaluator.eval_expr(&args[0], locals)? {
                Value::Rational(r) => {
                    let q = r.num / r.den;
                    let rem = r.num % r.den;
                    let n = if name == "\\floor" {
                        if r.num < 0 && rem != 0 {
                            q - 1
                        } else {
                            q
                        }
                    } else if r.num > 0 && rem != 0 {
                        q + 1
                    } else {
                        q
                    };
                    Ok(Value::Rational(Rational::integer(n)))
                }
                _ => Err(Diagnostic::at(
                    format!("{} requires an exact rational", name),
                    span,
                )),
            }
        }
        "\\min" | "\\max" => {
            if args.is_empty() {
                return Err(Diagnostic::at(
                    format!("{} requires at least one argument", name),
                    span,
                ));
            }
            let values = args
                .iter()
                .map(|arg| evaluator.eval_expr(arg, locals))
                .collect::<Result<Vec<_>, _>>()?;
            let mut best: Option<Rational> = None;
            for value in values {
                match value {
                    Value::Rational(r) => {
                        best = Some(match best {
                            None => r,
                            Some(current) => {
                                let less =
                                    r.num
                                        .checked_mul(current.den)
                                        .ok_or_else(|| Diagnostic::new("comparison overflow"))?
                                        .cmp(&current.num.checked_mul(r.den).ok_or_else(|| {
                                            Diagnostic::new("comparison overflow")
                                        })?);
                                if (name == "\\min" && less.is_lt())
                                    || (name == "\\max" && less.is_gt())
                                {
                                    r
                                } else {
                                    current
                                }
                            }
                        });
                    }
                    _ => {
                        return Err(Diagnostic::at(
                            format!("{} requires exact numeric arguments", name),
                            span,
                        ))
                    }
                }
            }
            Ok(Value::Rational(best.expect("non-empty min/max")))
        }
        "\\dot" => {
            require(2)?;
            let a = evaluator.eval_expr(&args[0], locals)?;
            let b = evaluator.eval_expr(&args[1], locals)?;
            match (a, b) {
                (Value::Vector(a), Value::Vector(b)) if a.len() == b.len() => {
                    let mut out = Rational::integer(0);
                    for (x, y) in a.iter().zip(&b) {
                        let (Value::Rational(x), Value::Rational(y)) = (x, y) else {
                            return Ok(symbolic(
                                name,
                                &[Value::Vector(a.clone()), Value::Vector(b.clone())],
                            ));
                        };
                        let product = x.mul(y).map_err(Diagnostic::new)?;
                        out = out.add(&product).map_err(Diagnostic::new)?;
                    }
                    Ok(Value::Rational(out))
                }
                _ => Err(Diagnostic::at(
                    "dot product requires vectors of equal length",
                    span,
                )),
            }
        }
        "\\norm" => {
            require(1)?;
            let value = evaluator.eval_expr(&args[0], locals)?;
            match value {
                Value::Vector(xs) => {
                    let mut sum = Rational::integer(0);
                    for x in &xs {
                        let Value::Rational(x) = x else {
                            return Ok(Value::Symbolic(Expr::Call {
                                callee: Box::new(Expr::Symbol {
                                    name: name.to_owned(),
                                    span,
                                }),
                                args: vec![args[0].clone()],
                                span,
                            }));
                        };
                        let sq = x.mul(x).map_err(Diagnostic::new)?;
                        sum = sum.add(&sq).map_err(Diagnostic::new)?;
                    }
                    evaluator.eval_expr(
                        &Expr::Sqrt {
                            expr: Box::new(value_to_rational_expr(&sum, span)),
                            span,
                        },
                        locals,
                    )
                }
                Value::Rational(x) => evaluator.eval_expr(
                    &Expr::Sqrt {
                        expr: Box::new(value_to_rational_expr(
                            &x.mul(&x).map_err(Diagnostic::new)?,
                            span,
                        )),
                        span,
                    },
                    locals,
                ),
                _ => Err(Diagnostic::at("norm requires a vector or number", span)),
            }
        }
        "\\det" => {
            require(1)?;
            match evaluator.eval_expr(&args[0], locals)? {
                Value::Matrix(matrix) => determinant(&matrix)
                    .map(Value::Rational)
                    .map_err(Diagnostic::new),
                _ => Err(Diagnostic::at("determinant requires a matrix", span)),
            }
        }
        "\\transpose" | "\\trans" => {
            require(1)?;
            match evaluator.eval_expr(&args[0], locals)? {
                Value::Matrix(matrix) => Ok(Value::Matrix(transpose(&matrix))),
                _ => Err(Diagnostic::at("transpose requires a matrix", span)),
            }
        }
        "\\inverse" | "\\inv" => {
            require(1)?;
            match evaluator.eval_expr(&args[0], locals)? {
                Value::Matrix(matrix) => inverse_matrix(&matrix)
                    .map(Value::Matrix)
                    .map_err(Diagnostic::new),
                _ => Err(Diagnostic::at("inverse requires a matrix", span)),
            }
        }
        "\\rank" => {
            require(1)?;
            match evaluator.eval_expr(&args[0], locals)? {
                Value::Matrix(matrix) => Ok(Value::Rational(Rational::integer(
                    rank_matrix(&matrix).map_err(Diagnostic::new)? as i128,
                ))),
                _ => Err(Diagnostic::at("rank requires a matrix", span)),
            }
        }
        "\\card" | "\\cardinality" => {
            require(1)?;
            match evaluator.eval_expr(&args[0], locals)? {
                Value::Set(values) | Value::Vector(values) => {
                    Ok(Value::Rational(Rational::integer(values.len() as i128)))
                }
                Value::Matrix(rows) => Ok(Value::Rational(Rational::integer(
                    rows.iter().map(Vec::len).sum::<usize>() as i128,
                ))),
                _ => Err(Diagnostic::at(
                    "cardinality requires a set, vector, or matrix",
                    span,
                )),
            }
        }
        "\\trace" => {
            require(1)?;
            match evaluator.eval_expr(&args[0], locals)? {
                Value::Matrix(matrix) => trace_matrix(&matrix)
                    .map(Value::Rational)
                    .map_err(Diagnostic::new),
                _ => Err(Diagnostic::at("trace requires a matrix", span)),
            }
        }
        "\\mean" | "\\variance" | "\\stdev" => {
            require(1)?;
            let value = evaluator.eval_expr(&args[0], locals)?;
            let xs = match value {
                Value::Vector(xs) => xs,
                Value::Set(xs) => xs,
                _ => {
                    return Err(Diagnostic::at(
                        format!("{} requires a finite set or vector", name),
                        span,
                    ))
                }
            };
            let numbers = xs
                .into_iter()
                .map(|value| match value {
                    Value::Rational(r) => Ok(r),
                    _ => Err(Diagnostic::new("statistics requires exact numeric data")),
                })
                .collect::<Result<Vec<_>, _>>()?;
            match name {
                "\\mean" => mean(&numbers).map(Value::Rational).map_err(Diagnostic::new),
                "\\variance" => variance(&numbers)
                    .map(Value::Rational)
                    .map_err(Diagnostic::new),
                _ => {
                    let v = variance(&numbers).map_err(Diagnostic::new)?;
                    evaluator.eval_expr(
                        &Expr::Sqrt {
                            expr: Box::new(value_to_rational_expr(&v, span)),
                            span,
                        },
                        locals,
                    )
                }
            }
        }
        "\\diff" | "\\derivative" => {
            require(2)?;

            // Accept both \diff{expr}{var} (the original prototype form)
            // and the natural mathematical \diff{var}{expr} form.
            //
            // When exactly one argument is a Symbol, use the Symbol as the
            // differentiation variable. This keeps both forms compatible.
            let (expr, var) = match (&args[0], &args[1]) {
                (Expr::Symbol { .. }, Expr::Symbol { .. }) => (
                    &args[0],
                    match &args[1] {
                        Expr::Symbol { name, .. } => name.clone(),
                        _ => unreachable!(),
                    },
                ),
                (Expr::Symbol { name, .. }, _) => (&args[1], name.clone()),
                (_, Expr::Symbol { name, .. }) => (&args[0], name.clone()),
                _ => {
                    return Err(Diagnostic::at(
                        "derivative requires a symbolic variable",
                        span,
                    ))
                }
            };

            let derivative = differentiate(expr, &var).ok_or_else(|| {
                Diagnostic::at("derivative is outside the supported symbolic subset", span)
            })?;
            let derivative = simplifier::simplify(derivative);

            // Differentiation is a symbolic transformation. Do not evaluate
            // the resulting expression against the current environment:
            // x := 12 must not turn d/dx (x^3 + 2x) into 434.
            if contains_symbol(&derivative, &var) {
                Ok(Value::Symbolic(derivative))
            } else {
                evaluator.eval_expr(&derivative, locals)
            }
        }
        "\\subs" | "\\substitute" => {
            require(2)?;
            let (var, replacement) = match &args[0] {
                Expr::Binary {
                    op: BinOp::Eq,
                    lhs,
                    rhs,
                    ..
                } => {
                    let Expr::Symbol { name, .. } = lhs.as_ref() else {
                        return Err(Diagnostic::at("substitution must use x=value", span));
                    };
                    let replacement = evaluator.eval_expr(rhs, locals)?;
                    (name.clone(), value_to_expr(&replacement, rhs))
                }
                _ => return Err(Diagnostic::at("substitution must use x=value", span)),
            };
            let body = substitute(&args[1], &var, &replacement);
            evaluator.eval_expr(&body, locals)
        }
        "\\solve" => {
            require(2)?;
            let var = match &args[1] {
                Expr::Symbol { name, .. } => name.clone(),
                _ => return Err(Diagnostic::at("solve variable must be a symbol", span)),
            };
            let equation = match &args[0] {
                Expr::Binary {
                    op: BinOp::Eq,
                    lhs,
                    rhs,
                    ..
                } => Expr::Binary {
                    op: BinOp::Sub,
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                    span,
                },
                _ => {
                    return Err(Diagnostic::at(
                        "solve expects an equation such as x^2 = 4",
                        span,
                    ))
                }
            };
            let coeffs = polynomial_coefficients(&equation, &var, evaluator, locals)?;
            solve_polynomial(&coeffs, &var, span, evaluator)
        }
        "\\sin" | "\\cos" | "\\tan" | "\\ln" | "\\log" | "\\exp" => {
            require(1)?;
            let value = evaluator.eval_expr(&args[0], locals)?;
            if let Some(result) = elementary_exact(name, &value, span) {
                Ok(result)
            } else {
                Ok(symbolic(name, &[value]))
            }
        }
        "\\range" => {
            require(2)?;
            let a = evaluator.eval_expr(&args[0], locals)?;
            let b = evaluator.eval_expr(&args[1], locals)?;
            match (a, b) {
                (Value::Rational(a), Value::Rational(b)) if a.is_integer() && b.is_integer() => {
                    let mut values = Vec::new();
                    if a.num <= b.num {
                        let count = b
                            .num
                            .checked_sub(a.num)
                            .and_then(|n| usize::try_from(n).ok())
                            .and_then(|n| n.checked_add(1))
                            .ok_or_else(|| Diagnostic::new("range is too large"))?;
                        if count > evaluator.max_sum_terms {
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
        "\\tuple" => Ok(Value::Vector(
            args.iter()
                .map(|arg| evaluator.eval_expr(arg, locals))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        _ => {
            let values = args
                .iter()
                .map(|arg| evaluator.eval_expr(arg, locals))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(symbolic(name, &values))
        }
    }
}
