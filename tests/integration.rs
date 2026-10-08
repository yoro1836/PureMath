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
    assert_eq!(
        evaluate(r"\frac{1}{3} + \frac{1}{3}", &mut env)
            .unwrap()
            .to_string(),
        r"\frac{2}{3}"
    );
}

#[test]
fn finite_sum() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\sum_{i=1}^{10} i", &mut env)
            .unwrap()
            .to_string(),
        "55"
    );
}

#[test]
fn recursive_piecewise() {
    let mut env = Environment::new();
    evaluate(
        r"fact(n) := \begin{cases} 1 & n = 0 \\ n \cdot fact(n-1) & n > 0 \end{cases}",
        &mut env,
    )
    .unwrap();
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
    assert_eq!(
        evaluate("x := 10 % comment", &mut env).unwrap().to_string(),
        "10"
    );
}

#[test]
fn parser_produces_multiple_statements() {
    let program = parse("x := 10\nf(x) := x + 1\nf(x)").unwrap();
    assert_eq!(program.statements.len(), 3);
}

#[test]
fn latex_surface_tokens_include_cdot() {
    let tokens = lexer::lex(r"x \cdot y").unwrap();
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.kind, puremath::lexer::TokenKind::Command(c) if c == "cdot")));
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
    assert!(evaluator
        .execute(&program.statements[0], std::path::Path::new("."))
        .is_err());
}

#[test]
fn print_is_runtime_output() {
    let mut env = Environment::new();
    assert!(evaluate(r"\print{2 + 3}", &mut env).is_err());

    let program = parse(r"\print{2 + 3}").unwrap();
    let mut output = Vec::new();
    let mut runtime = puremath::Runtime::new(&mut env, &mut output);
    let value = runtime
        .execute_program(&program, std::path::Path::new("."))
        .unwrap();

    assert_eq!(value, puremath::value::Value::Unit);
    assert_eq!(String::from_utf8(output).unwrap(), "5\n");
}

#[test]
fn file_definitions_are_not_implicitly_printed() {
    let mut env = Environment::new();
    let program = parse("x := 10\nf(x) := x + 1\n\\print{f(x)}").unwrap();
    let mut output = Vec::new();
    let mut runtime = puremath::Runtime::new(&mut env, &mut output);
    runtime
        .execute_program(&program, std::path::Path::new("."))
        .unwrap();

    assert_eq!(String::from_utf8(output).unwrap(), "11\n");
}

#[test]
fn finite_product() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\prod_{i=1}^{5} i", &mut env)
            .unwrap()
            .to_string(),
        "120"
    );
}

#[test]
fn absolute_value_is_exact() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\abs{-\frac{3}{2}}", &mut env)
            .unwrap()
            .to_string(),
        r"\frac{3}{2}"
    );
}

#[test]
fn empty_product_is_one() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\prod_{i=5}^{1} i", &mut env)
            .unwrap()
            .to_string(),
        "1"
    );
}

#[test]
fn set_relations_and_operations() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"{1,2} \cup {2,3}", &mut env).unwrap().to_string(),
        r"\{1,2,3\}"
    );
    assert_eq!(
        evaluate(r"2 \in {1,2,3}", &mut env).unwrap().to_string(),
        "true"
    );
    assert_eq!(
        evaluate(r"{1,2} \subseteq {1,2,3}", &mut env)
            .unwrap()
            .to_string(),
        "true"
    );
    assert_eq!(
        evaluate(r"{1,2,3} \setminus {2}", &mut env)
            .unwrap()
            .to_string(),
        r"\{1,3\}"
    );
}

#[test]
fn vectors_and_dot_product() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\vec{1,2,3} + \vec{4,5,6}", &mut env)
            .unwrap()
            .to_string(),
        r"\left(5, 7, 9\right)"
    );
    assert_eq!(
        evaluate(r"\dot{\vec{1,2,3}}{\vec{4,5,6}}", &mut env)
            .unwrap()
            .to_string(),
        "32"
    );
}

#[test]
fn matrix_operations() {
    let mut env = Environment::new();
    evaluate(
        r"A := \begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}",
        &mut env,
    )
    .unwrap();
    assert_eq!(evaluate(r"\det{A}", &mut env).unwrap().to_string(), "-2");
    assert_eq!(
        evaluate(r"\transpose{A}", &mut env).unwrap().to_string(),
        r"\begin{pmatrix} 1 & 3 \\ 2 & 4 \end{pmatrix}"
    );
    assert_eq!(
        evaluate(r"A^2", &mut env).unwrap().to_string(),
        r"\begin{pmatrix} 7 & 10 \\ 15 & 22 \end{pmatrix}"
    );
}

#[test]
fn derivative_accepts_both_argument_orders_and_stays_symbolic() {
    let mut env = Environment::new();

    assert_eq!(
        evaluate(r"\diff{x}{x^2 + 1}", &mut env)
            .unwrap()
            .to_string(),
        r"2 \cdot x"
    );
    assert_eq!(
        evaluate(r"\diff{x^3 + 2*x}{x}", &mut env)
            .unwrap()
            .to_string(),
        r"3 \cdot x^{2} + 2"
    );

    evaluate("x := 12", &mut env).unwrap();
    assert_eq!(
        evaluate(r"\diff{x^3 + 2*x}{x}", &mut env)
            .unwrap()
            .to_string(),
        r"3 \cdot x^{2} + 2"
    );
}

#[test]
fn bare_command_aliases_work_without_backslashes() {
    let mut env = Environment::new();

    assert_eq!(
        evaluate(r"diff{x}{x^2 + 1}", &mut env).unwrap().to_string(),
        r"2 \cdot x"
    );
    assert_eq!(
        evaluate(r"diff{x^3 + 2*x}{x}", &mut env)
            .unwrap()
            .to_string(),
        r"3 \cdot x^{2} + 2"
    );
    assert_eq!(evaluate(r"sin{0}", &mut env).unwrap().to_string(), "0");
    assert_eq!(evaluate(r"sqrt{16}", &mut env).unwrap().to_string(), "4");
    assert_eq!(
        evaluate(r"frac{1}{2} + frac{1}{2}", &mut env)
            .unwrap()
            .to_string(),
        "1"
    );
    assert_eq!(
        evaluate(r"sum_{i=1}^{3} i", &mut env).unwrap().to_string(),
        "6"
    );
    assert_eq!(
        evaluate(r"2 in {1,2,3}", &mut env).unwrap().to_string(),
        "true"
    );
    assert_eq!(
        evaluate(r"x cdot y", &mut env).unwrap().to_string(),
        r"x \cdot y"
    );
}

#[test]
fn bare_indexed_commands_tokenize_without_whitespace() {
    let tokens = lexer::lex(r"sum_{i=1}^{3} i").unwrap();
    assert!(tokens
        .iter()
        .any(|t| matches!(&t.kind, puremath::lexer::TokenKind::Ident(c) if c == "sum")));
    assert!(tokens
        .iter()
        .any(|t| matches!(t.kind, puremath::lexer::TokenKind::Underscore)));
}

#[test]
fn bare_calculus_bounds_work_without_backslashes() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"int_{0}^{3} x^2", &mut env).unwrap().to_string(),
        "9"
    );
    assert_eq!(
        evaluate(r"lim_{x to 2} x^2", &mut env).unwrap().to_string(),
        "4"
    );
}

#[test]
fn bare_parenthesized_builtins_work_without_backslashes() {
    let mut env = Environment::new();

    assert_eq!(evaluate("sin(0)", &mut env).unwrap().to_string(), "0");
    assert_eq!(evaluate("gcd(84,30)", &mut env).unwrap().to_string(), "6");
}

#[test]
fn bare_begin_environment_works_without_backslash() {
    let mut env = Environment::new();
    evaluate(
        r"fact(n) := begin{cases} 1 & n = 0 \\ n * fact(n-1) & n > 0 \\ end{cases}",
        &mut env,
    )
    .unwrap();
    assert_eq!(evaluate("fact(5)", &mut env).unwrap().to_string(), "120");
}

#[test]
fn bare_runtime_command_works_without_backslash() {
    let mut env = Environment::new();
    let program = parse(r"print{2 + 3}").unwrap();
    let mut output = Vec::new();
    let mut runtime = puremath::Runtime::new(&mut env, &mut output);
    runtime
        .execute_program(&program, std::path::Path::new("."))
        .unwrap();

    assert_eq!(String::from_utf8(output).unwrap(), "5\n");
}

#[test]
fn calculus_primitives() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\diff{x^3 + 2*x}{x}", &mut env)
            .unwrap()
            .to_string(),
        r"3 \cdot x^{2} + 2"
    );
    assert_eq!(
        evaluate(r"\int_{0}^{3} x^2", &mut env).unwrap().to_string(),
        "9"
    );
    assert_eq!(
        evaluate(r"\lim_{x\to 2} x^2", &mut env)
            .unwrap()
            .to_string(),
        "4"
    );
}

#[test]
fn solving_and_combinatorics() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\solve{x^2 - 5*x + 6 = 0}{x}", &mut env)
            .unwrap()
            .to_string(),
        r"\{2,3\}"
    );
    assert_eq!(evaluate(r"5!", &mut env).unwrap().to_string(), "120");
    assert_eq!(
        evaluate(r"\binom{5}{2}", &mut env).unwrap().to_string(),
        "10"
    );
    assert_eq!(
        evaluate(r"\gcd{84}{30}", &mut env).unwrap().to_string(),
        "6"
    );
}

#[test]
fn statistics_and_integer_functions() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\mean{\vec{1,2,3,4}}", &mut env)
            .unwrap()
            .to_string(),
        r"\frac{5}{2}"
    );
    assert_eq!(
        evaluate(r"\variance{\vec{1,2,3,4}}", &mut env)
            .unwrap()
            .to_string(),
        r"\frac{5}{4}"
    );
    assert_eq!(
        evaluate(r"\floor{\frac{-7}{3}}", &mut env)
            .unwrap()
            .to_string(),
        "-3"
    );
    assert_eq!(
        evaluate(r"\ceil{\frac{-7}{3}}", &mut env)
            .unwrap()
            .to_string(),
        "-2"
    );
    assert_eq!(evaluate(r"\sin{0}", &mut env).unwrap().to_string(), "0");
    assert_eq!(evaluate(r"\cos{0}", &mut env).unwrap().to_string(), "1");
}

#[test]
fn matrix_inverse_rank_and_trace() {
    let mut env = Environment::new();
    evaluate(
        r"A := \begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}",
        &mut env,
    )
    .unwrap();
    assert_eq!(
        evaluate(r"\inverse{A}", &mut env).unwrap().to_string(),
        r"\begin{pmatrix} -2 & 1 \\ \frac{3}{2} & -\frac{1}{2} \end{pmatrix}"
    );
    assert_eq!(evaluate(r"\rank{A}", &mut env).unwrap().to_string(), "2");
    assert_eq!(evaluate(r"\trace{A}", &mut env).unwrap().to_string(), "5");
}

#[test]
fn substitution_and_round_trip_calculus() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\subs{x=3}{x^2 + 1}", &mut env)
            .unwrap()
            .to_string(),
        "10"
    );
    assert_eq!(
        evaluate(r"\lim_{x\to 2} x^2", &mut env)
            .unwrap()
            .to_string(),
        "4"
    );
}

#[test]
fn broader_number_and_statistics_functions() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"\perm{5}{2}", &mut env).unwrap().to_string(),
        "20"
    );
    assert_eq!(
        evaluate(r"\lcm{12}{18}", &mut env).unwrap().to_string(),
        "36"
    );
    assert_eq!(
        evaluate(r"\range{3}{5}", &mut env).unwrap().to_string(),
        r"\{3,4,5\}"
    );
    assert_eq!(
        evaluate(r"\stdev{\vec{1,2,3,4}}", &mut env)
            .unwrap()
            .to_string(),
        r"\sqrt{\frac{5}{4}}"
    );
}

#[test]
fn finite_set_comprehension_evaluates_and_renders() {
    let mut env = Environment::new();
    assert_eq!(
        evaluate(r"{x in {1,2,3} | x > 1}", &mut env)
            .unwrap()
            .to_string(),
        r"\\{2,3\\}"
    );
    assert_eq!(
        evaluate(r"{x in {1,2,3} | x > 3}", &mut env)
            .unwrap()
            .to_string(),
        r"\\{\\}"
    );
}
