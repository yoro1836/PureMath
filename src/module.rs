use crate::diagnostics::Diagnostic;
use crate::env::Environment;
use crate::evaluator::Evaluator;
use crate::parser::Parser;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_module(module: &str, env: &mut Environment, base_dir: &Path) -> Result<(), Diagnostic> {
    let file = resolve_module_path(module, base_dir);
    let canonical = fs::canonicalize(&file)
        .map_err(|e| Diagnostic::new(format!("cannot import {}: {}", file.display(), e)))?;

    if env.loaded_modules.contains(&canonical) {
        return Ok(());
    }
    if env.loading.contains(&canonical) {
        return Err(Diagnostic::new(format!(
            "circular module dependency involving {}",
            module
        )));
    }

    env.loading.push(canonical.clone());
    let source = fs::read_to_string(&canonical)
        .map_err(|e| Diagnostic::new(format!("cannot read {}: {}", canonical.display(), e)))?;
    let result = (|| {
        let tokens = crate::lexer::lex(&source)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;
        let parent = canonical.parent().unwrap_or(base_dir);
        let mut evaluator = Evaluator::new(env);
        for stmt in &program.statements {
            evaluator.execute(stmt, parent)?;
        }
        Ok::<(), Diagnostic>(())
    })();
    env.loading.pop();

    result?;
    env.loaded_modules.insert(canonical);
    Ok(())
}

fn resolve_module_path(module: &str, base_dir: &Path) -> PathBuf {
    let mut path = base_dir.join(module);
    if path.extension().is_none() {
        path.set_extension("pmath");
    }
    path
}
