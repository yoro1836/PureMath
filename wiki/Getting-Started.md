# Getting Started

## Requirements

PureMath is implemented in Rust.

Build the prototype with:

```bash
cargo test
cargo build --release
```

The release binary is:

```bash
./target/release/puremath
```

## REPL

Running PureMath without a file starts the REPL:

```bash
./target/release/puremath
```

Enter a mathematical expression:

```text
2 + 3
```

The REPL displays the evaluated result.

## Running a file

PureMath source files use the `.pmath` extension.

```bash
./target/release/puremath examples/main.pmath
```

A file is evaluated as a declarative mathematical module. Expressions are evaluated for their program effects, but ordinary expressions are not implicitly written to stdout.

Use explicit runtime output instead:

```latex
x := 10
\print{x + 2}
```

## CLI options

```bash
./target/release/puremath --help
./target/release/puremath --version
./target/release/puremath --ast examples/main.pmath
```

`--ast` parses a source file and displays its mathematical AST.

Unknown or multiple file arguments are rejected by the CLI.

## First program

Create `hello.pmath`:

```latex
x := 10

f(x) := x^2 + 1

\print{f(3)}
```

Run it:

```bash
./target/release/puremath hello.pmath
```

The important distinction is:

```text
:=   definition
=    mathematical equality
```

Definitions are immutable within an environment. A second definition of the same name is rejected.

## LaTeX-style surface syntax

PureMath accepts a LaTeX-like syntax for many mathematical constructs:

```latex
\frac{1}{2}
\sqrt{16}
\sum_{i=1}^{10} i
2 \in \{1,2,3\}
```

For several command names, the leading backslash is optional:

```latex
\sin{0}
sin{0}

\diff{x}{x^2 + 1}
diff{x}{x^2 + 1}

\sum_{i=1}^{10} i
sum_{i=1}^{10} i
```

The backslash-free form is an additional surface spelling of the same semantic operation.

## Names

Variables are single letters, Greek letters or subscripted names (`x`, `\alpha`, `x_1`, `x_{max}`), and adjacent letters multiply: `2xy` is `2 \cdot x \cdot y`. Multi-letter names are written `\operatorname{name}`. See [Language Guide](Language-Guide#names-and-scope).