use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use puremath::{Environment, Runtime};

enum CliCommand {
    Help,
    Version,
    Repl,
    Run { path: String, ast: bool },
}

fn usage() -> &'static str {
    "Usage: puremath [OPTIONS] [FILE]

Options:
    -h, --help           Show this help message
    -V, --version        Show the version
        --ast FILE       Parse FILE and print its AST

Without FILE, PureMath starts the REPL.
"
}

fn parse_args<I>(mut args: I) -> Result<CliCommand, String>
where
    I: Iterator<Item = String>,
{
    let Some(first) = args.next() else {
        return Ok(CliCommand::Repl);
    };

    match first.as_str() {
        "-h" | "--help" => {
            if args.next().is_some() {
                return Err("--help does not take arguments".into());
            }
            Ok(CliCommand::Help)
        }
        "-V" | "--version" => {
            if args.next().is_some() {
                return Err("--version does not take arguments".into());
            }
            Ok(CliCommand::Version)
        }
        "--ast" => {
            let path = args
                .next()
                .ok_or_else(|| "--ast requires a FILE argument".to_string())?;
            if args.next().is_some() {
                return Err("--ast accepts exactly one FILE argument".into());
            }
            Ok(CliCommand::Run { path, ast: true })
        }
        "--" => {
            let path = args
                .next()
                .ok_or_else(|| "-- requires a FILE argument".to_string())?;
            if args.next().is_some() {
                return Err("expected exactly one FILE argument".into());
            }
            Ok(CliCommand::Run { path, ast: false })
        }
        _ if first.starts_with('-') => Err(format!("unknown option: {}", first)),
        path => {
            if args.next().is_some() {
                return Err("expected exactly one FILE argument".into());
            }
            Ok(CliCommand::Run {
                path: path.to_string(),
                ast: false,
            })
        }
    }
}

fn main() {
    let command = match parse_args(env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("error: {}", error);
            eprintln!();
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };

    match command {
        CliCommand::Help => print!("{}", usage()),
        CliCommand::Version => println!("PureMath {}", env!("CARGO_PKG_VERSION")),
        CliCommand::Repl => puremath::repl::run(),
        CliCommand::Run { path, ast } => {
            if let Err(error) = run_file(Path::new(&path), ast) {
                eprintln!("error: {}", error);
                std::process::exit(1);
            }
        }
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
    let mut runtime = Runtime::new(&mut env, io::stdout());

    for statement in &program.statements {
        let value = runtime
            .execute(statement, base_dir)
            .map_err(|error| error.to_string())?;

        if matches!(statement, puremath::Stmt::Expression(_)) {
            println!("{}", value);
        }
    }

    runtime
        .output_mut()
        .flush()
        .map_err(|error| format!("stdout flush failed: {}", error))?;

    Ok(())
}
