# Examples

These examples use the current prototype's implemented mathematical surface.

## Basic arithmetic

```latex
2 + 3
7 \cdot 6
\frac{1}{2} + \frac{1}{3}
\sqrt{16}
```

## Definitions and functions

```latex
x := 10
f(x) := x^2 + 1

\print{f(3)}
```

## Recursive factorial

```latex
\operatorname{fact}(n) :=
\begin{cases}
1 & n = 0 \\
n \cdot \operatorname{fact}(n-1) & n > 0
\end{cases}

\print{\operatorname{fact}(5)}
```

## Names and implicit multiplication

```latex
a := 3
b_1 := 4
\alpha := 2

\print{2ab_1}
\print{\alpha(a + 1)}
\print{(a + 1)(b_1 - 1)}
```

`2ab_1` is `2 \cdot a \cdot b_1`. Multi-letter names use `\operatorname`, as in the factorial example above.

## Sets

```latex
A := \{1,2,3\}
B := \{3,4,5\}

\print{2 \in A}
\print{A \cup B}
\print{A \cap B}
```

## Vectors

```latex
u := \vec{1,2,3}
v := \vec{4,5,6}

d := \dot{u}{v}

\print{d}
```

## Matrices

```latex
A :=
\begin{pmatrix}
1 & 2 \\
3 & 4
\end{pmatrix}

\print{\det{A}}
\print{\transpose{A}}
```

## Summation

```latex
s := \sum_{i=1}^{10} i
\print{s}
```

## Differentiation

```latex
f(x) := x^3 + 2x

d := \diff{x}{x^3 + 2x}

\print{d}
```

The derivative is kept symbolic when necessary.

## Symbolic substitution

```latex
p := x^2 + 1
r := \subs{x=3}{p}

\print{r}
```

## Solving a quadratic

```latex
R := \solve{x^2 - 4 = 0}{x}
\print{R}
```

## Importing a module

Suppose `algebra.pmath` defines reusable functions:

```latex
\operatorname{square}(x) := x^2
\operatorname{cube}(x) := x^3
```

Another module can import it:

```latex
\import{algebra}
\print{\operatorname{square}(12)}
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