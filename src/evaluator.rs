use crate::ast::{BinOp, Expr, Program, Stmt};
use crate::diagnostics::Diagnostic;
use crate::env::Environment;
use crate::simplifier;
use crate::value::{exact_integer_sqrt, value_to_expr, FunctionValue, Rational, Value};
use std::collections::HashMap;
use std::path::Path;

pub struct Evaluator<'a> {
    env: &'a mut Environment,
    call_depth: usize,
    max_call_depth: usize,
    max_sum_terms: usize,
}

impl<'a> Evaluator<'a> {
    pub const DEFAULT_MAX_CALL_DEPTH: usize = 256;
    pub const DEFAULT_MAX_SUM_TERMS: usize = 100_000;

    pub fn new(env: &'a mut Environment) -> Self {
        Self {
            env,
            call_depth: 0,
            max_call_depth: Self::DEFAULT_MAX_CALL_DEPTH,
            max_sum_terms: Self::DEFAULT_MAX_SUM_TERMS,
        }
    }

    pub fn with_limits(
        env: &'a mut Environment,
        max_call_depth: usize,
        max_sum_terms: usize,
    ) -> Self {
        Self {
            env,
            call_depth: 0,
            max_call_depth,
            max_sum_terms,
        }
    }

    pub fn execute_program(mut self, program: &Program) -> Result<Value, Diagnostic> {
        let mut last = Value::Unit;
        for stmt in &program.statements {
            last = self.execute(stmt, Path::new("."))?;
        }
        Ok(last)
    }

    pub fn execute(&mut self, stmt: &Stmt, base_dir: &Path) -> Result<Value, Diagnostic> {
        match stmt {
            Stmt::Definition {
                name, params, body, ..
            } => {
                if params.is_empty() {
                    let value = self.eval_expr(body, &HashMap::new())?;
                    self.env.define_value(name.clone(), value.clone())?;
                    Ok(value)
                } else {
                    self.env.define_function(
                        name.clone(),
                        FunctionValue {
                            params: params.clone(),
                            body: body.clone(),
                        },
                    )?;
                    Ok(self.env.get(name).cloned().unwrap_or(Value::Unit))
                }
            }
            Stmt::Expression(expr) => self.eval_expr(expr, &HashMap::new()),
            Stmt::Import { module, .. } => {
                crate::module::load_module(module, self.env, base_dir)?;
                Ok(Value::Unit)
            }
            Stmt::Print { span, .. } => Err(Diagnostic::at(
                "print is a runtime operation; execute it through puremath::Runtime",
                *span,
            )),
        }
    }

    pub(crate) fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, Diagnostic> {
        self.eval_expr(expr, &HashMap::new())
    }

    fn eval_expr(
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
            Expr::Sum {
                var,
                lower,
                upper,
                body,
                ..
            } => self.eval_sum(var, lower, upper, body, expr, locals),
            Expr::Sqrt { expr: inner, .. } => match self.eval_expr(inner, locals)? {
                Value::Rational(v) if v.is_integer() && v.num >= 0 => {
                    let n = v.num as u128;
                    let r = exact_integer_sqrt(n);
                    if r * r == n {
                        Ok(Value::Rational(Rational::integer(r as i128)))
                    } else {
                        Ok(Value::Symbolic(expr.clone()))
                    }
                }
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
            BinOp::Eq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                match compare(op, &a, &b) {
                    Ok(value) => Ok(Value::Bool(value)),
                    Err(_) => Ok(Value::Symbolic(Expr::Binary {
                        op,
                        lhs: Box::new(value_to_expr(&a, lhs)),
                        rhs: Box::new(value_to_expr(&b, rhs)),
                        span,
                    })),
                }
            }
            BinOp::Add => numeric_or_symbolic(BinOp::Add, a, b, lhs, rhs, |x, y| x.add(y)),
            BinOp::Sub => numeric_or_symbolic(BinOp::Sub, a, b, lhs, rhs, |x, y| x.sub(y)),
            BinOp::Mul => numeric_or_symbolic(BinOp::Mul, a, b, lhs, rhs, |x, y| x.mul(y)),
            BinOp::Div => numeric_or_symbolic(BinOp::Div, a, b, lhs, rhs, |x, y| x.div(y)),
            BinOp::Pow => match (a, b) {
                (Value::Rational(x), Value::Rational(y)) if y.is_integer() => {
                    x.powi(y.num).map(Value::Rational).map_err(Diagnostic::new)
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
}

fn numeric_or_symbolic<F>(
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

fn compare(op: BinOp, a: &Value, b: &Value) -> Result<bool, String> {
    match (a, b) {
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
    }
}
