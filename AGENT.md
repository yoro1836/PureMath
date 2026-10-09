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
