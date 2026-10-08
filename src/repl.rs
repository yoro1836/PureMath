use crate::env::Environment;
use crate::lexer;
use crate::parser::Parser;
use crate::runtime::Runtime;
use std::io::{self, Write};
use std::path::Path;

pub fn run() {
    let mut env = Environment::new();
    println!("PureMath v0.1 development prototype");
    println!("Type :quit to exit.");

    let stdin = io::stdin();
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if matches!(line, ":quit" | ":q") {
            break;
        }
        if line == ":ast" {
            println!("AST debug mode is available through the library API.");
            continue;
        }

        match eval_line(line, &mut env) {
            Ok(value) if !matches!(value, crate::value::Value::Unit) => println!("{}", value),
            Ok(crate::value::Value::Unit) => {},
            Err(err) => eprintln!("error: {}", err),
        }
    }
}

fn eval_line(
    line: &str,
    env: &mut Environment,
) -> Result<crate::value::Value, crate::diagnostics::Diagnostic> {
    let tokens = lexer::lex(line)?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;
    let mut runtime = Runtime::stdout(env);
    let mut last = crate::value::Value::Unit;
    for stmt in &program.statements {
        last = runtime.execute(stmt, Path::new("."))?;
    }
    Ok(last)
}
