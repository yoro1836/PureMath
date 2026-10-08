use std::env;
use std::fs;
use std::path::Path;

use puremath::{Environment, Evaluator};

fn main() {
    let mut args = env::args().skip(1);
    let ast = matches!(args.next().as_deref(), Some("--ast"));
    let path = if ast { args.next() } else { args.next() };

    match path {
        Some(path) => {
            if let Err(error) = run_file(Path::new(&path), ast) {
                eprintln!("error: {}", error);
                std::process::exit(1);
            }
        }
        None => puremath::repl::run(),
    }
}

fn run_file(path: &Path, ast: bool) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {}", path.display(), error))?;

    let program = puremath::parse(&source).map_err(|error| error.to_string())?;

    if ast {
        println!("{:#?}", program);
        return Ok(());
    }

    let mut env = Environment::new();
    let base_dir = path.parent().unwrap_or(Path::new("."));
    let mut evaluator = Evaluator::new(&mut env);

    for statement in &program.statements {
        let value = evaluator
            .execute(statement, base_dir)
            .map_err(|error| error.to_string())?;
        println!("{}", value);
    }

    Ok(())
}
