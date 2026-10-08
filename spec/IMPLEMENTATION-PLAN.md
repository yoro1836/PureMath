# PureMath Implementation Plan v0.1 (Development)

## Phase 0 — Freeze semantics

- `:=` is definition.
- `=` is mathematical equality.
- Names are references, not mutable storage.
- Functions are mathematical functions.
- `cases` is piecewise definition.
- `sum` is indexed aggregation.
- Symbolic results are first-class evaluation results.

## Phase 1 — Interpreter core

Implemented in this prototype:

- modular Rust crate
- lexer with source spans
- parser producing Mathematical AST
- exact rational arithmetic
- function application
- recursive definitions
- piecewise evaluation
- finite sums and products
- finite sets and set relations
- vectors and matrices
- symbolic preservation
- substitution
- basic differentiation / integration / limits
- low-degree polynomial solving
- combinatorics, number theory, and statistics helpers
- simple module imports
- REPL

## Phase 1.1 — Immediate hardening

- Add real source snippets to diagnostics.
- Add AST pretty-printer and parser snapshot tests.
- Add explicit duplicate-definition policy.
- Add import namespace objects rather than flat imports.
- Add recursion depth / resource limits.
- Separate mathematical environment from runtime capability environment.

## Phase 2 — Domains and types

Introduce domain values and judgments such as:

```text
x ∈ ℕ
f : A → B
```

The type layer should be mathematical rather than class/object based.

## Phase 3 — Symbolic engine

Start with normalization and rewrite rules. Every rewrite must carry domain assumptions when required.

Priority:

1. constant folding
2. rational normalization
3. algebraic identities
4. substitution
5. polynomial normalization
6. differentiation
7. integration
8. limits

## Phase 4 — Standard library

Move reusable mathematics into `.pmath` modules.

## Phase 5 — Math IR / compiler

Only after evaluator semantics are stable:

```text
Mathematical AST
    ↓
Elaboration
    ↓
Math IR
    ↓
Optimizer
    ├── Interpreter backend
    ├── WASM backend
    └── Native/LLVM backend
```

## Phase 1.2 — Runtime boundary

Implement the minimum host-side interface without adding non-mathematical constructs to Mathematical Core:

- `\\print{E}` writes an evaluated value to stdout.
- runtime execution is separated from `Evaluator`.
- pure library evaluation rejects runtime statements.
- declarative module loading rejects runtime side effects.

Future input, filesystem, networking, clock, OS, entropy, and process interfaces follow the same boundary.