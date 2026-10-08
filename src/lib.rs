pub mod ast;
pub mod diagnostics;
pub mod env;
pub mod evaluator;
pub mod lexer;
pub mod module;
pub mod parser;
pub mod render;
pub mod repl;
pub mod runtime;
pub mod simplifier;
pub mod value;

pub use ast::{BinOp, Expr, Program, Stmt};
pub use diagnostics::{Diagnostic, Span};
pub use env::Environment;
pub use evaluator::Evaluator;
pub use parser::Parser;
pub use runtime::Runtime;

pub fn parse(source: &str) -> Result<Program, Diagnostic> {
    let tokens = lexer::lex(source).map_err(|error| error.with_source(source))?;
    Parser::new(tokens)
        .parse_program()
        .map_err(|error| error.with_source(source))
}

pub fn evaluate(source: &str, env: &mut Environment) -> Result<value::Value, Diagnostic> {
    let program = parse(source)?;
    let evaluator = Evaluator::new(env);
    evaluator
        .execute_program(&program)
        .map_err(|error| error.with_source(source))
}
