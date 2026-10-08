use std::fs;
use std::process::Command;

fn puremath() -> Command {
    Command::new(env!("CARGO_BIN_EXE_puremath"))
}

#[test]
fn help_does_not_start_repl() {
    let output = puremath().arg("--help").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: puremath"));
    assert!(!stdout.contains("> "));
}

#[test]
fn short_help_works() {
    let output = puremath().arg("-h").output().unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Options:"));
}

#[test]
fn version_does_not_start_repl() {
    let output = puremath().arg("--version").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PureMath 0.1.0-dev"));
    assert!(!stdout.contains("> "));
}

#[test]
fn file_argument_executes_program() {
    let path = std::env::temp_dir().join(format!("puremath-cli-{}.pmath", std::process::id()));
    fs::write(&path, "x := 10\nx + 2\n").unwrap();

    let output = puremath().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "12\n");
}

#[test]
fn explicit_print_writes_to_stdout() {
    let path =
        std::env::temp_dir().join(format!("puremath-cli-print-{}.pmath", std::process::id()));
    fs::write(&path, "x := 10\n\\print{x + 2}\n").unwrap();

    let output = puremath().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "12\n");
}

#[test]
fn ast_mode_requires_and_reads_file() {
    let path = std::env::temp_dir().join(format!("puremath-cli-ast-{}.pmath", std::process::id()));
    fs::write(&path, "x := 10\n").unwrap();

    let output = puremath()
        .args(["--ast", path.to_str().unwrap()])
        .output()
        .unwrap();
    let _ = fs::remove_file(&path);

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Program"));
}

#[test]
fn missing_ast_file_is_an_error() {
    let output = puremath().arg("--ast").output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--ast requires a FILE"));
}

#[test]
fn unknown_option_is_an_error() {
    let output = puremath()
        .arg("--definitely-not-an-option")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown option"));
}

#[test]
fn extra_file_argument_is_an_error() {
    let output = puremath()
        .args(["first.pmath", "second.pmath"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("expected exactly one FILE"));
}
