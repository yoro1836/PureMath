# PureMath Roadmap

This roadmap is ordered by dependency: each milestone makes the next one
sound. Semantics come before surface convenience, and the compiler comes only
after evaluator semantics and domain information are stable.

## Current state (v0.1.1)

The prototype parses a LaTeX-like subset into a semantic AST and evaluates it
with exact rationals, finite sets, vectors and matrices, propositions, function
values, and a small symbolic layer. Every documented code block runs.

Known gaps:

- Finite sets keep duplicates (`{1,1,2}`) and set equality is not decided
  (`{1,2} = {2,1}` stays symbolic).
- Integers are fixed `i128` (`2^200`, `\factorial{40}` overflow).
- `\solve{x^2+1=0}{x}` renders as `\solve()`; radicals are not simplified
  (`\sqrt{8}`); like terms are not collected (`x - x`).
- `x \in \mathbb{N}` is accepted silently without meaning.
- Free names in function bodies resolve at call time; this is unspecified.
- Documentation blocks are checked for running, not for their results.

## M0 — Foundation

- [x] Restore green CI (rustfmt, clippy); run CI on `dev` as well as `main`.
- [ ] Require CI on `main` (repository settings).
- [x] Wiki source lives in `wiki/` and is mirrored to the GitHub Wiki from
  `main`.
- [x] Executable documentation: `latex` code blocks from README and Wiki run
  as tests (`tests/docs.rs`); current drift is recorded in
  `tests/docs_known_failures.txt`, which may only shrink.
- [x] Snapshot tests for `examples/*.pmath` output and `--ast` output.

## v0.1.1 — Everything documented runs

- [x] Surface syntax: `\{ \}`, `\emptyset`, `\le \ge \leq \geq \neq \lt \gt`,
  `\times`, `\div`, `\left \right` and spacing commands, `\mid`,
  `\land \lor \neg`.
- [x] Decimal literals as exact rationals (`3.7` = 37/10).
- [x] Mathematical identifier model: single-letter and Greek names with
  subscripts, implicit multiplication (`xy`, `2x`, `(a+b)(a-b)`), and
  `\operatorname{...}` / `\mathrm{...}` for multi-letter names.
- [x] `\mapsto` function values.
- [x] One canonical `\diff` argument order: `\diff{x}{E}`.
- [x] Builtins keep unknown arguments symbolic (`\det{A}` → `\det(A)`).
- [x] `dx` ends an integrand instead of multiplying it.
- [x] Fix Wiki and README escaping.
- [x] Refresh STATUS, SPEC §12 and MODULE-ARCHITECTURE.

Exit criterion: `tests/docs_known_failures.txt` is empty. **Met.**

## v0.2 — Sound values

- Dependency-free arbitrary-precision integers and rationals.
- Canonical finite sets: deduplication, order-independent equality, decided
  set equality.
- Tuples `(a, b)`.
- `\forall` / `\exists` over finite sets.
- `\times` as the Cartesian product of sets.
- Constant and radical normalization (`\ln{e} = 1`, `\sqrt{8} = 2\sqrt{2}`).
- Specify name-binding rules for function bodies.

## v0.3 — Domains and elaboration

- Domain values `\mathbb{N}, \mathbb{Z}, \mathbb{Q}, \mathbb{R}, \mathbb{C}`.
- `x \in \mathbb{N}` as an assumption; `f : A \to B` as a signature.
- An elaboration pass between AST and evaluation: name resolution, domain
  checking, builtin resolution (replacing string-name builtin dispatch).
- Domain violations become diagnostics; domain assumptions feed
  simplification (e.g. `x \ge 0` gives `\sqrt{x^2} = x`).
- Set comprehension over domains (`\{x \in \mathbb{N} \mid x < 10\}`).

The domain information produced here is the type information the LLVM
backend later depends on.

## v0.4 — Symbolic engine v1

- Canonical multivariate polynomial form with rational coefficients: like
  terms, expansion, integer-polynomial factoring.
- Rational function simplification.
- Differentiation over all elementary functions.
- Integration: table-based, linearity, simple substitution.
- Limits for `0/0` forms.
- Solving: rational root theorem, simplified radicals, linear systems via
  matrices.
- Every rewrite carries the domain assumptions it requires.

## v0.5 — Modules and standard library

- Namespaces, `\export`, qualified names.
- Move statistics and number-theory helpers into `.pmath` standard modules.
- Probability, random variables and pseudo-random sequences as library
  semantics; only OS entropy for seeding is a Runtime capability.
- Dependency graph, versioned packages, documentation generation.

Follow-ups from the identifier model: `\mathbf{v}` / `\mathcal{A}` /
accent decorations as names, a `\DeclareMathOperator`-style shorthand for
user operator names, and `f'` as derivative notation.

## v0.6 — Tooling

- REPL: multi-line input, `:ast`, `:env`.
- Rendered output (KaTeX/HTML) for programs and diagnostics.
- Editor syntax highlighting; minimal language server.

## v0.7 — Math IR and LLVM backend

```text
Mathematical AST
    ↓
Elaboration (domains)
    ↓
Math IR
    ↓
Optimizer
    ├── Interpreter backend
    └── LLVM backend
```

- Math IR: explicit, domain-annotated, side-effect boundary preserved.
- The interpreter is re-based on Math IR first, so both backends share one
  semantics and can be tested against each other.
- LLVM backend compiles the decidable, fully elaborated subset: functions over
  `\mathbb{Z}` and `\mathbb{Q}` (with an exact arbitrary-precision runtime
  library), finite sums/products, piecewise definitions and recursion.
- Symbolic-only expressions stay on the interpreter path; compilation never
  silently replaces exact semantics with approximation. A `\mathbb{R}`
  floating-point lowering, if added, must be an explicit opt-in.
- Emitting textual LLVM IR keeps the compiler crate dependency-free; bindings
  such as `inkwell` remain an option if in-process JIT is needed.
- Differential testing: interpreter and compiled results must agree on the
  conformance suite.

## Long term

- Theorem/proof layer.

## Runtime boundary

Explicit interfaces exist only for capabilities that depend on external state:

- stdout / `\print`
- stdin / `\read`
- files
- networking
- current time
- OS/environment access
- OS entropy
- external processes

Mathematical functionality is implemented as Core semantics or library
definitions whenever it can be stated independently of the execution
environment.

## Open decisions

1. ~~Identifier model~~ — decided: mathematical convention (`xy` is
   `x \cdot y`; multi-letter names via `\operatorname`). See
   `SYNTAX-DECISIONS.md`.
2. Free-name binding in function bodies: at definition or at call time.
3. ~~Canonical `\diff` argument order~~ — decided: `\diff{x}{E}`.
4. Arbitrary-precision arithmetic: self-implemented (dependency-free) or an
   external crate.
5. Function application by juxtaposition: whether `\sin 2x` means
   `\sin(2x)`. Currently rejected with a hint to write `\sin{x}`.
