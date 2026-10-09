# Language Guide

## Definitions

Use `:=` to define an immutable mathematical name.

```text
x := 10
r := 3
```

Redefinition is an error in the same environment:

```text
x := 10
x := 20
```

This prevents `:=` from becoming hidden mutable assignment.

## Equality

Use `=` for mathematical equality.

```text
x = 10
```

Equality can evaluate to a proposition when the operands can be compared exactly. When comparison cannot currently be decided, PureMath preserves the equality symbolically.

## Functions

A function is defined by giving its argument and body:

```latex
f(x) := x^2 + 1
f(3)
```

Functions can also be represented as values with mapping notation:

```latex
s := x \mapsto x^2
```

Function application is ordinary mathematical application.

## Piecewise definitions

Use a cases environment for a piecewise mathematical definition:

```latex
f(x) :=
\begin{cases}
x^2 & x \ge 0 \\
-x  & x < 0
\end{cases}
```

A branch is selected only when its condition can be decided in the current environment. Otherwise the piecewise expression may remain symbolic.

## Recursion

Recursive definitions are expressed mathematically rather than with a loop.

```latex
\operatorname{fact}(n) :=
\begin{cases}
1 & n = 0 \\
n \cdot \operatorname{fact}(n-1) & n > 0
\end{cases}
```

The evaluator may execute the recursion operationally, but the language meaning comes from the mathematical definition.

## Arithmetic

Supported arithmetic includes:

```text
2 + 3
7 - 4
6 * 5
20 / 4
2^8
```

Exact rational values are preserved rather than immediately converted to floating point.

For example:

```latex
\frac{1}{2} + \frac{1}{3}
```

is represented as an exact rational result.

## Symbols and symbolic values

Not every valid expression has to reduce to a number.

```text
x + 1
```

may remain symbolic.

Symbolic evaluation is a valid result state, not an evaluation failure. This is important for calculus and algebraic manipulation.

## Exactness

The prototype distinguishes at least these semantic result classes:

- Exact rational values
- Boolean propositions
- Finite sets
- Functions
- Symbolic expressions
- Unit/opaque values

The evaluator reduces an expression only as far as its current semantic engine allows.

## Names and scope

Identifiers represent mathematical names and references in the current environment.

Names follow mathematical convention:

- A variable is a single letter or Greek letter, optionally with a subscript: `x`, `x_1`, `x_{12}`, `x_{max}`, `\alpha`, `\theta_0`. An unbraced subscript is one character, as in LaTeX, and `x_1` and `x_{1}` are the same name.
- Adjacent letters are a product: `xy` means `x \cdot y`, exactly as LaTeX renders it. Juxtaposed factors multiply: `2x`, `2(x+1)`, `(a+b)(a-b)`, `2\pi`.
- Multi-letter names are upright operator names: `\operatorname{fact}` or `\mathrm{fact}` (the same name).
- Standard function and operator words such as `sin`, `gcd`, `det`, `in` and `pi` are recognised as whole words; any other run of letters is split, so `sinx` is `s \cdot i \cdot n \cdot x`.
- `f(x)` applies `f` when `f` is a function or still unknown. When a name denotes a number, vector or matrix, `a(b+1)` is a product.
- A number cannot follow a name: write `x_2` or `2x`, not `x2`.

```latex
a := 3
b_1 := 4
\print{2ab_1}
\print{a(b_1 + 1)}
```

Definitions and imports populate that environment. The current prototype intentionally keeps module environments simple; namespace objects and explicit exports are planned for a later revision.

## Sets

Finite set literals use braces:

```latex
A := \{1,2,3\}
```

Membership and set relations use mathematical notation:

```latex
2 \in A
A \subseteq \{1,2,3,4\}
A \subset \{1,2,3,4\}
```

Set operations include union, intersection, and set difference:

```latex
A \cup B
A \cap B
A \setminus B
```

## Vectors

Vector literals use the vector construct:

```latex
\vec{1,2,3}
```

The prototype supports vector addition, scalar multiplication, dot products, and norms.

```latex
\dot{\vec{1,2,3}}{\vec{4,5,6}}
\norm{\vec{3,4}}
```

## Matrices

Matrices use a pmatrix-style environment:

```latex
\begin{pmatrix}
1 & 2 \\
3 & 4
\end{pmatrix}
```

The current engine supports matrix addition and multiplication, powers, transpose, determinant, inverse, rank, trace, matrix-vector multiplication, and exact rational inversion.

## Number theory and combinatorics

The prototype includes:

```latex
\factorial{5}
\binom{5}{2}
\perm{5}{2}
\gcd{84}{30}
\lcm{12}{18}
\floor{3.7}
\ceil{3.2}
```

Aliases such as `choose`, `permutation`, and `cardinality` are also recognized in the current command surface where implemented.

## Elementary functions

Supported elementary operations include:

```latex
\sqrt{16}
\abs{-3}
\sin{0}
\cos{0}
\tan{0}
\ln{e}
\log{100}
\exp{1}
```

Constants include `pi` and `e`.

## Statistics

The current prototype provides:

```latex
\mean{\vec{1,2,3}}
\variance{\vec{1,2,3}}
\stdev{\vec{1,2,3}}
```

The exact input representation should follow the vector/set forms accepted by the current parser.