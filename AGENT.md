# Agent Guidelines

Rules for AI coding agents working in this repository.

## Branches

- Do not create feature branches.
- Commit and push all work to `dev`.
- `dev` is reviewed by the maintainer and then merged into `main`.
- Never push directly to `main`.

## Commit identity

Commits are authored and committed as the maintainer:

```bash
git config user.name yoro1836
git config user.email ijaeyong46@outlook.kr
```

## Attribution

Do not add `Co-Authored-By:` trailers. Mark AI assistance with an
`Assisted-by:` trailer at the end of the commit message instead:

```text
Short summary of the change

Optional body explaining why.

Assisted-by: Claude Code
```

## Before pushing

Run the same checks as CI:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Snapshot and documentation tests

- `tests/snapshots.rs` compares `examples/*.pmath` output and `--ast` output
  with `tests/snapshots/`. After an intended change, regenerate with
  `UPDATE_SNAPSHOTS=1 cargo test --test snapshots` and review the diff.
- `tests/docs.rs` runs every ```latex block in README.md, and in the Wiki
  when `PUREMATH_WIKI_DIR` points at a Wiki checkout. Blocks that do not run
  yet are listed in `tests/docs_known_failures.txt`. That list must only
  shrink: remove entries when you fix them, and do not add entries to hide
  a regression.
