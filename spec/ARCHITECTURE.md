# PureMath Architecture v0.1 (Development)

## 1. Semantic pipeline

```text
.pmath source
    ↓
Lexer
    ↓
Surface tokens + source spans
    ↓
Parser
    ↓
Mathematical AST
    ↓
Elaboration / domain checking        [Phase 2]
    ↓
Math IR                              [Phase 3+]
    ├── Evaluator
    ├── Symbolic Engine
    └── Compiler backends
```

The renderer is intentionally outside the semantic pipeline:

```text
Mathematical AST ──→ LaTeX renderer
```

Rendering must never define meaning.

## 2. Module responsibilities

`lexer.rs` converts source characters into a compact token stream and records source spans.

`parser.rs` converts those tokens into PureMath's AST. It does not perform evaluation.

`ast.rs` contains language-semantic nodes such as `Definition`, `Call`, `Piecewise`, and `Sum`.

`value.rs` contains exact runtime values and symbolic values.

`env.rs` owns mathematical name bindings. Definitions are immutable and duplicate names are rejected.

`evaluator.rs` performs reduction while preserving expressions that cannot yet be evaluated.

`module.rs` loads declarative `.pmath` modules, tracks active imports, detects cycles, and caches completed modules.

`render.rs` converts AST/value expressions back into LaTeX-like notation.

`repl.rs` provides the interactive development loop.

## 3. Important boundaries

### Mathematical Core

PureMath meaning lives here. Examples: functions, equality, sets, sums, recursive definitions.

### Meta Layer

Module/file organization such as `\\import{algebra}`.

### Mathematical Standard Library

Reusable mathematical capabilities belong here when they have direct mathematical semantics but are too large or specialized to be primitive Core syntax.

Examples include:

```text
probability
statistics
random variables
pseudo-random sequences
sampling
combinatorics
advanced numerical methods
```

A library implementation may use optimized runtime facilities internally, but its observable semantics remain mathematical.

### Runtime Layer

Only capabilities whose meaning depends on the external execution environment belong here:

```text
stdout / \\print
stdin / \\read
files
networking
current time
OS/environment APIs
OS entropy
external processes
```

Runtime capabilities must be explicit at the language boundary and must not silently enter Mathematical Core evaluation.

For example, a PRNG defined by a mathematical recurrence is a Core/library construct. An OS entropy source used to seed that recurrence is a Runtime capability.

## 4. Why this structure matters

The parser can grow toward a larger KaTeX-compatible surface without forcing the evaluator to understand renderer-specific constructs.

Likewise, the evaluator can acquire symbolic transformations without changing the source grammar.
