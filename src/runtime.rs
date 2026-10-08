use crate::ast::{Program, Stmt};
use crate::diagnostics::Diagnostic;
use crate::env::Environment;
use crate::evaluator::Evaluator;
use crate::value::Value;
use std::io::{self, Write};
use std::path::Path;

/// Side-effectful execution boundary for capabilities provided by the host runtime.
///
/// Mathematical evaluation remains in Evaluator. Runtime operations such as
/// printing are handled here so the mathematical core does not own stdout.
pub struct Runtime<'env, W: Write> {
    evaluator: Evaluator<'env>,
    output: W,
}

impl<'env, W: Write> Runtime<'env, W> {
    pub fn new(env: &'env mut Environment, output: W) -> Self {
        Self {
            evaluator: Evaluator::new(env),
            output,
        }
    }

    pub fn execute(&mut self, stmt: &Stmt, base_dir: &Path) -> Result<Value, Diagnostic> {
        match stmt {
            Stmt::Print { expr, .. } => {
                let value = self.evaluator.evaluate_expr(expr)?;
                writeln!(self.output, "{}", value)
                    .map_err(|error| Diagnostic::new(format!("print failed: {}", error)))?;
                Ok(Value::Unit)
            }
            _ => self.evaluator.execute(stmt, base_dir),
        }
    }

    pub fn execute_program(
        &mut self,
        program: &Program,
        base_dir: &Path,
    ) -> Result<Value, Diagnostic> {
        let mut last = Value::Unit;
        for stmt in &program.statements {
            last = self.execute(stmt, base_dir)?;
        }
        Ok(last)
    }

    pub fn output_mut(&mut self) -> &mut W {
        &mut self.output
    }
}

impl<'env> Runtime<'env, io::Stdout> {
    pub fn stdout(env: &'env mut Environment) -> Self {
        Self::new(env, io::stdout())
    }
}
