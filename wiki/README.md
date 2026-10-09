# PureMath Wiki Source

This directory is the source of the GitHub Wiki. When a change to `wiki/`
reaches `main`, the `Wiki` workflow mirrors this directory to the Wiki, so
edits made directly on the Wiki are overwritten. Edit the pages here and
send them through `dev` → review → `main` like any other change.

This file is not published.

- `wiki/`: user-facing language documentation
- `spec/`: semantic and architectural specification
- `examples/`: executable source examples

Every ```latex block in these pages is executed by `tests/docs.rs`; blocks
that do not run yet are listed in `tests/docs_known_failures.txt`.

The pages describe the current prototype and should not be read as a claim of
full KaTeX compatibility or a finished production language. The formal
semantic baseline remains `spec/SPEC-v0.1.md`.
