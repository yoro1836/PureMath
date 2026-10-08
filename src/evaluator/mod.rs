#![allow(clippy::needless_range_loop)]

mod prelude {
    pub(crate) use crate::ast::{BinOp, Expr, Program, Stmt};
    pub(crate) use crate::diagnostics::Diagnostic;
    pub(crate) use crate::env::Environment;
    pub(crate) use crate::simplifier;
    pub(crate) use crate::value::{
        exact_integer_sqrt, value_to_expr, FunctionValue, Rational, Value,
    };
    pub(crate) use std::collections::HashMap;
    pub(crate) use std::path::Path;
}
use prelude::*;

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
}

mod algebra;
mod arithmetic;
mod builtins;
mod elementary;
mod expressions;
mod linear_algebra;
mod number_theory;
mod sets;
mod statistics;
mod symbolic;
