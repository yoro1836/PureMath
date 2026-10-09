//! Executable documentation tests.
//!
//! Every ```latex code block in README.md (and in the GitHub Wiki when
//! `PUREMATH_WIKI_DIR` points at a checkout of it) is executed as a PureMath
//! program. A block must run without error unless it is listed in
//! `tests/docs_known_failures.txt`.
//!
//! The known-failures list tracks documentation drift. It must only shrink:
//! a listed block that now runs, or an entry whose block no longer exists,
//! also fails the test so the list stays accurate.

use puremath::{Environment, Runtime};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const KNOWN_FAILURES: &str = "tests/docs_known_failures.txt";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

struct Block {
    file: String,
    line: usize,
    id: String,
    source: String,
}

/// FNV-1a; identifies a block by its content so entries survive unrelated edits.
fn content_id(source: &str) -> String {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in source.trim().bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    format!("{:08x}", hash)
}

fn latex_blocks(path: &Path) -> Vec<Block> {
    let file = path.file_name().unwrap().to_string_lossy().into_owned();
    let text = fs::read_to_string(path).unwrap();
    let mut blocks = Vec::new();
    let mut current: Option<(usize, String)> = None;

    for (index, line) in text.lines().enumerate() {
        let fence = line.trim_start();
        match current.take() {
            None => {
                if fence.starts_with("```") && fence.trim_start_matches('`').trim() == "latex" {
                    current = Some((index + 1, String::new()));
                }
            }
            Some((start, mut source)) => {
                if fence.starts_with("```") {
                    blocks.push(Block {
                        file: file.clone(),
                        line: start,
                        id: content_id(&source),
                        source,
                    });
                } else {
                    source.push_str(line);
                    source.push('\n');
                    current = Some((start, source));
                }
            }
        }
    }
    blocks
}

fn run_block(source: &str) -> Result<(), String> {
    let program = puremath::parse(source).map_err(|error| error.to_string())?;
    let mut env = Environment::new();
    let mut runtime = Runtime::new(&mut env, Vec::new());
    let base_dir = root().join("examples");
    for statement in &program.statements {
        runtime
            .execute(statement, &base_dir)
            .map_err(|error| error.with_source(source).to_string())?;
    }
    Ok(())
}

/// Maps "file id" to the recorded reason.
fn known_failures() -> BTreeMap<(String, String), String> {
    let text = fs::read_to_string(root().join(KNOWN_FAILURES)).unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut parts = line.splitn(3, char::is_whitespace);
            let file = parts.next().unwrap_or_default().to_string();
            let id = parts.next().unwrap_or_default().to_string();
            let reason = parts.next().unwrap_or_default().trim().to_string();
            ((file, id), reason)
        })
        .collect()
}

fn check_documents(paths: &[PathBuf]) {
    let known = known_failures();
    let mut problems = Vec::new();
    let mut seen = Vec::new();

    for path in paths {
        for block in latex_blocks(path) {
            let key = (block.file.clone(), block.id.clone());
            seen.push(key.clone());
            let listed = known.contains_key(&key);
            match run_block(&block.source) {
                Ok(()) if listed => problems.push(format!(
                    "{}:{} now runs; remove `{} {}` from {}",
                    block.file, block.line, block.file, block.id, KNOWN_FAILURES
                )),
                Ok(()) => {}
                Err(_) if listed => {}
                Err(error) => {
                    let first_line = error.lines().next().unwrap_or_default().to_string();
                    problems.push(format!(
                        "{}:{} fails: {}\n{}\nIf this is accepted drift, add to {}:\n{} {} {}",
                        block.file,
                        block.line,
                        error,
                        block.source,
                        KNOWN_FAILURES,
                        block.file,
                        block.id,
                        first_line
                    ));
                }
            }
        }
    }

    let files: Vec<String> = paths
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    for (file, id) in known.keys() {
        if files.contains(file) && !seen.contains(&(file.clone(), id.clone())) {
            problems.push(format!(
                "stale entry `{} {}` in {}: no such block",
                file, id, KNOWN_FAILURES
            ));
        }
    }

    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

#[test]
fn readme_code_blocks_run() {
    check_documents(&[root().join("README.md")]);
}

#[test]
fn wiki_code_blocks_run() {
    let Some(dir) = std::env::var_os("PUREMATH_WIKI_DIR") else {
        eprintln!("PUREMATH_WIKI_DIR is not set; skipping Wiki code blocks");
        return;
    };
    let mut pages: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    pages.sort();
    assert!(!pages.is_empty(), "PUREMATH_WIKI_DIR contains no pages");
    check_documents(&pages);
}
