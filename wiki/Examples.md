# Examples

These examples use the current prototype's implemented mathematical surface.

## Basic arithmetic

```latex
2 + 3
7 \\cdot 6
\\frac{1}{2} + \\frac{1}{3}
\\sqrt{16}
```

## Definitions and functions

```latex
x := 10
f(x) := x^2 + 1

\\print{f(3)}
```

## Recursive factorial

```latex
fact(n) :=
\\begin{cases}
1 & n = 0 \\\\
n \\cdot fact(n-1) & n > 0
\\end{cases}

\\print{fact(5)}
```

## Sets

```latex
A := \\{1,2,3\\}
B := \\{3,4,5\\}

\\print{2 \\in A}
\\print{A \\cup B}
\\print{A \\cap B}
```

## Vectors

```latex
u := \\vec{1,2,3}
v := \\vec{4,5,6}

d := \\dot{u}{v}

\\print{d}
```

## Matrices

```latex
A :=
\\begin{pmatrix}
1 & 2 \\\\
3 & 4
\\end{pmatrix}

\\print{\\det{A}}
\\print{\\transpose{A}}
```

## Summation

```latex
s := \\sum_{i=1}^{10} i
\\print{s}
```

## Differentiation

```latex
f(x) := x^3 + 2x

d := \\diff{x}{x^3 + 2x}

\\print{d}
```

The derivative is kept symbolic when necessary.

## Symbolic substitution

```latex
expr := x^2 + 1
result := \\subs{x=3}{expr}

\\print{result}
```

## Solving a quadratic

```latex
roots := \\solve{x^2 - 4 = 0}{x}
\\print{roots}
```

## Importing a module

Suppose `math.pmath` defines reusable values:

```latex
pi2 := 2 \\cdot pi
```

Another module can import it:

```latex
\\import{math}
\\print{pi2}
```

Module resolution is relative to the importing module.

## Current feature tour

The repository includes:

```text
examples/main.pmath
examples/math.pmath
examples/algebra.pmath
examples/recursion.pmath
examples/import_example.pmath
examples/all_features.pmath
```

`examples/all_features.pmath` is the broadest single executable tour of the current prototype.
