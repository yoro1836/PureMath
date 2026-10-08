# PureMath

> Mathematics as a Language.

PureMath is an experimental programming language in which mathematical objects and relations form the semantic core of the language.

It is not a general-purpose imperative language with mathematical-looking syntax.

## Current prototype

The Rust crate is intentionally dependency-free and modular.

Supported surface subset:

```latex
x := 10
f(x) := x^2 + 1
f(3)

fact(n) :=
\begin{cases}
1 & n = 0 \\
n \cdot fact(n-1) & n > 0
\end{cases}

\frac{1}{3} + \frac{1}{3}
\sum_{i=1}^{10} i
A := \{1,2,3\}
\sqrt{16}
\import{algebra}
```

The implementation deliberately does **not** claim to parse all of KaTeX. KaTeX-compatible notation is a surface goal; PureMath maintains its own semantic AST.

Definitions are immutable: a name cannot be defined twice in the same environment. This is intentional and prevents `:=` from becoming hidden reassignment.

## CLI

Build and run the prototype:

```bash
cargo test
cargo build --release
```

Start the REPL by running without a file:

```bash
./target/release/puremath
```

Useful CLI options:

```bash
./target/release/puremath --help
./target/release/puremath --version
./target/release/puremath examples/main.pmath
./target/release/puremath --ast examples/main.pmath
```

The `--help` and `--version` options exit immediately; they do not start the REPL.

## Architecture

```text
source
  ↓
lexer
  ↓
parser
  ↓
Mathematical AST
  ↓
evaluator
  ├─ exact values
  ├─ propositions
  └─ symbolic expressions
```

The next architectural step is elaboration/domain checking and a dedicated Math IR.

Mathematical features are not moved into the runtime merely because their implementations may use runtime services. Probability, random variables, pseudo-random sequences, statistics, and similar constructs remain mathematical semantics or standard-library functionality.

The runtime boundary is reserved for environment-dependent capabilities such as `\\print`, input, files, networking, clocks, OS APIs, entropy sources, and external processes.

## Source layout

```text
src/
├── ast.rs          semantic nodes
├── diagnostics.rs  spans/errors
├── env.rs          immutable name environment
├── evaluator.rs    exact + symbolic evaluation
├── lexer.rs        LaTeX-like tokenization
├── lib.rs          public crate API
├── main.rs         CLI
├── module.rs       declarative imports
├── parser.rs       surface -> AST
├── render.rs       AST -> LaTeX-like output
├── repl.rs         REPL
├── simplifier.rs   small symbolic normalization layer
└── value.rs        exact/symbolic values
```
