# PureMath Prototype Status

## Implemented in v0.1 development prototype

- Modular Rust crate
- Public library API
- CLI + REPL
- Structured source spans
- Multi-statement files
- `%` comments
- Exact rational arithmetic
- Function definitions/application
- Recursive piecewise functions
- Finite sums
- Finite sets
- Symbolic preservation
- Minimal symbolic simplification
- Declarative module imports
- Module cycle detection/cache
- Evaluation resource limits
- Integration tests
- AST dump mode

## Intentionally not implemented yet

- Full KaTeX grammar
- Unicode/math-font identifier model
- Type/domain inference
- Set comprehension
- quantified logic
- complex numbers as exact values
- matrix/vector semantics
- general symbolic differentiation/integration
- namespace/export syntax
- explicit runtime effect system
- Math IR
- WASM/native compiler backend

## Verification note

The execution environment used for this package does not contain `cargo` or `rustc`, so `cargo test` could not be executed locally. The project is kept dependency-free and includes a Rust integration suite intended to run directly under a normal Rust 2021 toolchain.
