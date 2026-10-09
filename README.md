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

\operatorname{fact}(n) :=
\begin{cases}
1 & n = 0 \\
n \cdot \operatorname{fact}(n-1) & n > 0
\end{cases}

\frac{1}{3} + \frac{1}{3}
\sum_{i=1}^{10} i
\prod_{i=1}^{5} i
A := \{1,2,3\}
A \cup \{4\}
2 \in A
\vec{1,2,3}
M := \begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}
\sqrt{16}
\abs{-3}
\det{M}
\diff{x}{x^3}
\int_{0}^{1} x^2
\lim_{x\to 0} x^2
\solve{x^2 - 4 = 0}{x}
\binom{5}{2}
\mean{\vec{1,2,3}}
\print{f(3)}
\import{algebra}
```

The implementation deliberately does **not** claim to parse all of KaTeX. KaTeX-compatible notation is a surface goal; PureMath maintains its own semantic AST.

### Names

Names follow mathematical convention. A variable is a single letter or Greek letter, optionally subscripted, and adjacent factors multiply:

```latex
a := 3
b_1 := 4
\alpha := 2
x_{max} := 10
\print{2ab_1}
\print{\alpha(a + 1)}
\print{(a + 1)(b_1 - 1)}
```

`xy` is `x \cdot y`, never a name `xy`, exactly as LaTeX renders it. Multi-letter names are written as upright operator names, `\operatorname{fact}` or `\mathrm{fact}` (the same name). Standard function and operator words such as `sin`, `gcd`, `det`, `in` and `pi` are recognised as whole words. A number may not follow a name (`x2`); write `x_2` or `2x`.

### Optional command backslashes

LaTeX-style command backslashes are optional for PureMath command names. Both forms are accepted and map to the same semantic operations:

```latex
\diff{x}{x^2 + 1}
diff{x}{x^2 + 1}

\sin{0}
sin{0}

\sum_{i=1}^{10} i
sum_{i=1}^{10} i

2 \in \{1,2,3\}
2 in \{1,2,3\}
```

The backslash form remains valid for LaTeX compatibility. Only standard command and operator words are recognized without a backslash; any other run of letters is a product of single-letter names (see [Names](#names)).

The broad prototype currently includes exact indexed sums/products, finite sets and set relations, vectors and matrices, elementary exact functions, number theory/combinatorics helpers, statistics, substitution, symbolic differentiation, exact polynomial integration, direct-substitution limits, and degree-2 polynomial solving.

For a single executable tour of the implemented mathematical surface, run:

```bash
./target/release/puremath examples/all_features.pmath
```


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

The runtime boundary is reserved for environment-dependent capabilities such as `\print`, input, files, networking, clocks, OS APIs, entropy sources, and external processes.

## Source layout

```text
src/
├── ast.rs            semantic nodes
├── diagnostics.rs    spans and diagnostics with source context
├── env.rs            immutable name environment
├── evaluator/        exact + symbolic evaluation
│   ├── mod.rs            Evaluator, statements, resource limits
│   ├── expressions.rs    expression reduction, integrals, limits
│   ├── arithmetic.rs     exact arithmetic and comparison
│   ├── builtins.rs       builtin dispatch
│   ├── algebra.rs        polynomial solving
│   ├── symbolic.rs       differentiation, integration, substitution
│   ├── elementary.rs     elementary functions and constants
│   ├── linear_algebra.rs vectors and matrices
│   ├── number_theory.rs  combinatorics and integer functions
│   ├── sets.rs           set operations
│   └── statistics.rs     mean, variance, standard deviation
├── lexer.rs          LaTeX-like tokenization
├── lib.rs            public crate API
├── main.rs           CLI
├── module.rs         declarative imports
├── names.rs          mathematical identifier model
├── parser.rs         surface -> AST
├── render.rs         AST -> LaTeX output
├── repl.rs           REPL
├── runtime.rs        runtime capabilities (\print)
├── simplifier.rs     small symbolic normalization layer
└── value.rs          exact/symbolic values
```

### Explicit output

`\print{E}` is the explicit stdout interface for programs:

```latex
x := 10
\print{x + 2}
```

Definitions, imports, and expression statements do not produce implicit output when a `.pmath` file is executed. Use `\print` for program output. The REPL separately displays evaluated expression results for interactive use.