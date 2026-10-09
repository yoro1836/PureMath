# PureMath Prototype Status

Current milestone: **v0.1.1** — every documented code block runs. See
`spec/ROADMAP.md` for what comes next.

## Implemented

- Modular, dependency-free Rust crate with a public library API
- CLI (`--ast`, `--help`, `--version`) and REPL
- Source spans and diagnostics with source context
- Multi-statement files, `%` comments
- Mathematical identifier model: single-letter and Greek names, subscripts,
  implicit multiplication, `\operatorname{...}` for multi-letter names
- Exact rational arithmetic, decimal literals as exact rationals
- Immutable definitions; functions, recursion, piecewise definitions
- Function values with `\mapsto`
- Relations (`=`, `\neq`, `<`, `\le`, `>`, `\ge`) and connectives
  (`\land`, `\lor`, `\neg`) with symbolic fallback
- Finite sums and products
- Finite sets with `\{ \}`, set operations, relations and comprehensions
- Vectors and matrices with exact linear algebra
- Number theory, combinatorics, statistics, elementary functions
- Symbolic preservation, including builtins over unknown arguments
- Substitution, differentiation, exact polynomial integration (with `dx`),
  direct-substitution limits, degree-2 polynomial solving
- Declarative module imports with cycle detection and caching
- Runtime boundary: explicit `\print`
- Evaluation resource limits
- Integration, CLI, snapshot and executable-documentation tests

## Not implemented yet

- Arbitrary-precision integers (values are `i128`)
- Canonical finite sets (deduplication, decided set equality), tuples
- Domains, type/domain inference, `\forall` / `\exists`
- Full KaTeX grammar
- Complex numbers
- General symbolic algebra (like terms, factoring, radical simplification)
- General integration, limits and equation solving
- Namespaces and explicit exports
- Math IR and the LLVM backend
