# Syntax Reference

This page is a practical reference for the current prototype. It is not a complete formal grammar.

## Definitions and equality

| Syntax | Meaning |
|---|---|
| `x := E` | Define immutable name `x` |
| `f(x) := E` | Define a function |
| `x = y` | Mathematical equality |

## Names

| Syntax | Meaning |
|---|---|
| `x`, `\alpha` | Single-letter or Greek name |
| `x_1`, `x_{12}`, `x_{max}`, `\theta_0` | Subscripted name (`x_1` = `x_{1}`) |
| `\operatorname{fact}`, `\mathrm{fact}` | Multi-letter name |
| `xy`, `2x`, `2(x+1)`, `(a+b)(a-b)` | Implicit multiplication |

Standard words such as `sin`, `gcd`, `det`, `in` and `pi` stay whole without a backslash; other letter runs are products of single-letter names.

## Core operators

| Syntax | Operation |
|---|---|
| `+` | Addition |
| `-` | Subtraction / negation |
| `*`, `\times`, juxtaposition (`2x`) | Multiplication |
| `/`, `\div` | Division |
| `^` | Power |
| `=` | Equality |
| `\neq` / `\ne` | Inequality |
| `<`, `\lt` | Less than |
| `<=`, `\le`, `\leq` | Less than or equal |
| `>`, `\gt` | Greater than |
| `>=`, `\ge`, `\geq` | Greater than or equal |
| `\land` / `\wedge` | Conjunction |
| `\lor` / `\vee` | Disjunction |
| `\neg` / `\lnot` | Negation |
| `\in` / `in` | Membership |
| `\subset` / `subset` | Proper subset |
| `\subseteq` / `subseteq` | Subset or equal |
| `\cup` / `cup` | Union |
| `\cap` / `cap` | Intersection |
| `\setminus` / `setminus` | Set difference |
| `\cdot` / `cdot` | Dot/scalar multiplication notation |
| `x \mapsto E` | Function value |

Relations bind tighter than `\land`, which binds tighter than `\lor`:

```latex
\print{1 < 2 \land 2 \le 3 \lor 1 \neq 1}
\print{\neg 1 = 2}
```

## Literals and sets

```latex
\print{0.1 + 0.2 = 0.3}
A := \{1, 2, 3\}
\print{\{x \in A \mid x \ge 2\}}
\print{\emptyset \subseteq A}
```

Decimal literals are exact rationals: `3.7` is `\frac{37}{10}`. Set literals use `\{ \}` as in LaTeX (plain `{ }` also works), and comprehensions separate the condition with `\mid` or `|`.

`\left`, `\right`, `\big` and spacing commands such as `\,` only affect typesetting and are ignored.

## Function values

```latex
s := x \mapsto x^2
\print{s(3)}
\print{(t \mapsto t + 1)(4)}
```

## Braced commands

Common one-argument commands:

```latex
\sqrt{16}
\abs{-3}
\vec{1,2,3}
\set{1,2,3}
\tuple{1,2,3}
\norm{\vec{3,4}}
\det{A}
\transpose{A}
\inverse{A}
\rank{A}
\card{A}
\trace{A}
\mean{\vec{1,2,3}}
\variance{\vec{1,2,3}}
\stdev{\vec{1,2,3}}
\sin{0}
\cos{0}
\tan{0}
\ln{e}
\log{100}
\exp{1}
\factorial{5}
\floor{3.7}
\ceil{3.2}
```

Common multi-argument commands:

```latex
\frac{1}{2}
\dot{u}{v}
\subs{x=3}{x^2}
\solve{x^2 - 4 = 0}{x}
\binom{5}{2}
\perm{5}{2}
\gcd{84}{30}
\lcm{12}{18}
```

## Indexed commands

Indexed aggregation uses mathematical subscript/superscript syntax:

```latex
\sum_{i=1}^{10} i
\prod_{i=1}^{5} i
```

The backslash may be omitted in the current surface:

```latex
sum_{i=1}^{10} i
prod_{i=1}^{5} i
```

## Integrals

Integrals take optional bounds and an optional differential:

```latex
\print{\int_{0}^{1} x^2 \,dx}
\print{\int_{0}^{3} 3t^2 dt}
\print{\int x^2 dx}
```

Inside an integrand, `d` followed by a name is the differential; it ends the integrand and names the integration variable. Without it, the variable is inferred from the integrand.

## Limits

Limits use an indexed target and a `to` relation:

```latex
\lim_{x \to 0} \frac{\sin{x}}{x}
```

## Cases

Piecewise definitions use:

```latex
\begin{cases}
1 & x = 0 \\
x \cdot f(x-1) & x > 0
\end{cases}
```

The current parser recognizes both the LaTeX command spelling and the corresponding backslash-free command spelling where supported.

## Parenthesized calls

Ordinary function calls use parentheses:

```text
f(3)
```

Several built-in names can also be called without a leading backslash:

```text
sin(0)
gcd(84, 30)
```

This does **not** yet mean that every braced command has a universal comma-separated parenthesized spelling. The parser currently has command-specific syntax in addition to generic function-call syntax.

For example, the canonical derivative form remains:

```latex
\diff{x}{x^2 + 1}
```

The variable comes first, as `d/dx` is read; `\diff{x^2 + 1}{x}` is rejected with a hint.

## Optional command backslashes

Where a command is recognized in a command/operator position, its leading backslash can be omitted.

```latex
\sqrt{16}
sqrt{16}

\sin{0}
sin{0}

\diff{x}{x^2+1}
diff{x}{x^2+1}

2 \in \{1,2,3\}
2 in \{1,2,3\}
```

LaTeX-style backslashes remain valid.

## Imports

Modules are imported with:

```latex
\import{algebra}
```

A `.pmath` file is treated as a declarative mathematical module.

## Output

Program output is explicit:

```latex
\print{x + 2}
```

A standalone expression in a source file is not implicitly printed to stdout.

## AST

The CLI can inspect the parsed Mathematical AST:

```bash
puremath --ast examples/main.pmath
```

The AST is semantic rather than a direct copy of surface LaTeX syntax.