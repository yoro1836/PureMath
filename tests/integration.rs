use puremath::{evaluate, lexer, parse, Environment};

#[test]
fn arithmetic() {
    let mut env = Environment::new();
    assert_eq!(evaluate("2 + 3 * 4", &mut env).unwrap().to_string(), "14");
}

#[test]
fn definitions_and_function_application() {
    let mut env = Environment::new();
    evaluate("f(x) := x^2 + 1", &mut env).unwrap();
    assert_eq!(evaluate("f(3)", &mut env).unwrap().to_string(), "10");
}

#[test]
fn equality_is_a_proposition() {
    let mut env = Environment::new();
    evaluate("x := 10", &mut env).unwrap();
    assert_eq!(evaluate("x = 10", &mut env).unwrap().to_string(), "true");
}

#[test]
fn rational_arithmetic_is_exact() {
    let mut env = Environment::new();
    assert_eq!(evaluate(r"\frac{1}{3} + \frac{1}{3}", &mut env).unwrap().to_string(), r"\frac{2}{3}");
}

#[test]
fn finite_sum() {
    let mut env = Environment::new();
    assert_eq!(evaluate(r"\sum_{i=1}^{10} i", &mut env).unwrap().to_string(), "55");
}

#[test]
fn recursive_piecewise() {
    let mut env = Environment::new();
    evaluate(r"fact(n) := \begin{cases} 1 & n = 0 \\ n \cdot fact(n-1) & n > 0 \end{cases}", &mut env).unwrap();
    assert_eq!(evaluate("fact(5)", &mut env).unwrap().to_string(), "120");
}

#[test]
fn symbolic_values_are_preserved() {
    let mut env = Environment::new();
    assert_eq!(evaluate("x + 1", &mut env).unwrap().to_string(), "x + 1");
}

#[test]
fn comments_are_ignored() {
    let mut env = Environment::new();
    assert_eq!(evaluate("x := 10 % comment", &mut env).unwrap().to_string(), "10");
}

#[test]
fn parser_produces_multiple_statements() {
    let program = parse("x := 10\nf(x) := x + 1\nf(x)").unwrap();
    assert_eq!(program.statements.len(), 3);
}

#[test]
fn latex_surface_tokens_include_cdot() {
    let tokens = lexer::lex(r"x \cdot y").unwrap();
    assert!(tokens.iter().any(|t| matches!(&t.kind, puremath::lexer::TokenKind::Command(c) if c == "cdot")));
}

#[test]
fn multiline_program_executes() {
    let mut env = Environment::new();
    let program = r"x := 10
f(x) := x^2 + 1
f(3)";
    assert_eq!(evaluate(program, &mut env).unwrap().to_string(), "10");
}

#[test]
fn symbolic_comparison_is_preserved() {
    let mut env = Environment::new();
    assert_eq!(evaluate("x = 10", &mut env).unwrap().to_string(), "x = 10");
}

#[test]
fn negative_power_precedence() {
    let mut env = Environment::new();
    assert_eq!(evaluate("-2^2", &mut env).unwrap().to_string(), "-4");
}

#[test]
fn duplicate_definition_is_rejected() {
    let mut env = Environment::new();
    evaluate("x := 1", &mut env).unwrap();
    assert!(evaluate("x := 2", &mut env).is_err());
}

#[test]
fn module_import_loads_definitions_once() {
    use std::fs;
    use std::path::PathBuf;

    let dir = std::env::temp_dir().join(format!("puremath-test-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let module = dir.join("algebra.pmath");
    let main = dir.join("main.pmath");
    fs::write(&module, "square(x) := x^2\n").unwrap();
    fs::write(&main, "\\import{algebra}\nsquare(7)\n").unwrap();

    let source = fs::read_to_string(&main).unwrap();
    let tokens = lexer::lex(&source).unwrap();
    let mut parser = puremath::Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut env = Environment::new();
    let mut evaluator = puremath::Evaluator::new(&mut env);
    let base = PathBuf::from(&dir);
    let mut last = puremath::value::Value::Unit;
    for stmt in &program.statements {
        last = evaluator.execute(stmt, &base).unwrap();
    }
    assert_eq!(last.to_string(), "49");
    let _ = fs::remove_file(module);
    let _ = fs::remove_file(main);
    let _ = fs::remove_dir(dir);
}

#[test]
fn symbolic_simplifier_removes_identity() {
    let mut env = Environment::new();
    assert_eq!(evaluate("x + 0", &mut env).unwrap().to_string(), "x");
    assert_eq!(evaluate("x * 1", &mut env).unwrap().to_string(), "x");
}

#[test]
fn recursive_call_depth_is_bounded() {
    let mut env = Environment::new();
    evaluate("loop(x) := loop(x)", &mut env).unwrap();
    let program = puremath::parse("loop(0)").unwrap();
    let mut evaluator = puremath::Evaluator::with_limits(&mut env, 8, 100);
    assert!(evaluator.execute(&program.statements[0], std::path::Path::new("." )).is_err());
}
