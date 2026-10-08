use crate::env::Environment;
use crate::evaluator::Evaluator;
use crate::parser::Parser;
use crate::lexer;
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
        if stdin.read_line(&mut line).is_err() { break; }
        let line = line.trim();
        if line.is_empty() { continue; }
        if matches!(line, ":quit" | ":q") { break; }
        if line == ":ast" { println!("AST debug mode is available through the library API."); continue; }

        match eval_line(line, &mut env) {
            Ok(value) => println!("{}", value),
            Err(err) => eprintln!("error: {}", err),
        }
    }
}

fn eval_line(line: &str, env: &mut Environment) -> Result<crate::value::Value, crate::diagnostics::Diagnostic> {
    let tokens = lexer::lex(line)?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;
    let mut evaluator = Evaluator::new(env);
    let mut last = crate::value::Value::Unit;
    for stmt in &program.statements { last = evaluator.execute(stmt, Path::new("."))?; }
    Ok(last)
}
