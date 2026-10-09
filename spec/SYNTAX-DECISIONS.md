# PureMath Syntax Decisions v0.1

## `:=` vs `=`

```text
x := 10
```

adds a definition.

```text
x = 10
```

is an equality proposition.

## Names and implicit multiplication

PureMath follows mathematical convention for names:

- A variable is a single letter (`x`) or Greek letter (`\alpha`), optionally
  with a subscript (`x_1`, `x_{12}`, `x_{max}`, `\theta_0`). An unbraced
  subscript is one character, as in LaTeX; `x_1` and `x_{1}` are the same name.
- Adjacent letters are a product: `xy` is `x \cdot y`. This is how LaTeX and
  KaTeX render the source, so the visible formula and the program agree.
- Juxtaposed factors multiply: `2x`, `2(x+1)`, `(a+b)(a-b)`, `2\pi`.
- Multi-letter names are upright operator names: `\operatorname{fact}` or
  `\mathrm{fact}`, which denote the same name.
- Standard function, operator and command words (`sin`, `gcd`, `det`, `in`,
  `pi`, `sum`, ...) are recognised as whole words without a backslash. Any
  other run of letters is split: `sinx` is `s \cdot i \cdot n \cdot x`.
- `f(x)` applies `f` when `f` is a function or unknown. When a name denotes a
  number, vector or matrix, `a(b+1)` is the product `a \cdot (b+1)`.
- A number cannot directly follow a name (`x2` is an error); write `x_2` or
  `2x`.
- `f'` is reserved for derivative notation and is not a separate name.

## Core control flow

No `if`, `while`, `for`, `switch`, `goto`, `break`, or `return` exists in Mathematical Core.

Mathematical alternatives include piecewise functions, recursion, sums, products, and indexed families.

## Surface syntax vs semantics

LaTeX/KaTeX notation is a presentation-oriented surface syntax. The semantic AST is owned by PureMath.

## Symbolic preservation

An expression that cannot currently be reduced is preserved as an expression. Lack of a numeric evaluator is not permission to discard mathematical structure.

## Mathematical functionality vs runtime functionality

The boundary is semantic, not based on implementation technique.

A feature belongs to Mathematical Core or the mathematical standard library when it directly denotes a mathematical concept. This includes probability, statistics, random variables, pseudo-random sequences, and mathematically defined sampling.

A feature belongs to the Runtime Interface when its meaning requires external execution state or an external side effect, such as output, input, filesystem access, networking, the current clock, OS APIs, OS entropy, or external processes.

Therefore an implementation may use runtime machinery to accelerate or seed a mathematical operation without turning that mathematical operation into a Runtime primitive.