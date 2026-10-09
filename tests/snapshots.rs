//! Snapshot tests for `examples/*.pmath`.
//!
//! Each example is run through the CLI and its output is compared with a
//! checked-in snapshot. Set `UPDATE_SNAPSHOTS=1` to rewrite the snapshots
//! after an intended change, then review the diff.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn updating() -> bool {
    std::env::var_os("UPDATE_SNAPSHOTS").is_some_and(|value| value != "0")
}

fn examples() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(root().join("examples"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "pmath"))
        .collect();
    paths.sort();
    paths
}

fn run_cli(args: &[&str], file: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_puremath"))
        .args(args)
        .arg(file)
        .output()
        .unwrap();
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.status.success() {
        text.push_str(&format!("[exit {:?}]\n", output.status.code()));
        text.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    text
}

fn check_snapshots(kind: &str, extension: &str, args: &[&str]) {
    let dir = root().join("tests/snapshots").join(kind);
    if updating() {
        fs::create_dir_all(&dir).unwrap();
    }

    let mut failures = Vec::new();
    let mut expected_files = Vec::new();

    for example in examples() {
        let stem = example.file_stem().unwrap().to_string_lossy().into_owned();
        let snapshot = dir.join(format!("{}.{}", stem, extension));
        expected_files.push(snapshot.clone());
        let actual = run_cli(args, &example);

        if updating() {
            fs::write(&snapshot, &actual).unwrap();
            continue;
        }

        match fs::read_to_string(&snapshot) {
            Ok(expected) if expected == actual => {}
            Ok(expected) => failures.push(format!(
                "{} differs from {}\n--- expected ---\n{}--- actual ---\n{}",
                example.display(),
                snapshot.display(),
                expected,
                actual
            )),
            Err(_) => failures.push(format!("missing snapshot {}", snapshot.display())),
        }
    }

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries {
            let path = entry.unwrap().path();
            if !expected_files.contains(&path) {
                if updating() {
                    fs::remove_file(&path).unwrap();
                } else {
                    failures.push(format!("stale snapshot {}", path.display()));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{}\n\nRun with UPDATE_SNAPSHOTS=1 to accept intended changes.",
        failures.join("\n\n")
    );
}

#[test]
fn example_output_matches_snapshots() {
    check_snapshots("examples", "out", &[]);
}

#[test]
fn example_ast_matches_snapshots() {
    check_snapshots("ast", "ast", &["--ast"]);
}
