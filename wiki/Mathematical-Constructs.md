# Mathematical Constructs

PureMath's main design choice is to express computation using mathematical constructs rather than imperative control flow.

## Summation

Finite sums are indexed mathematical aggregations:

```latex
\\sum_{i=1}^{10} i
```

The prototype directly evaluates finite integer bounds.

A sum that cannot currently be evaluated exactly is preserved symbolically.

A summation is not a loop statement.

## Product

Finite products use:

```latex
\\prod_{i=1}^{5} i
```

As with sums, finite integer bounds are evaluated directly when possible.

## Recursion

Recursion is expressed through a mathematical definition.

```latex
fib(n) :=
\\begin{cases}
0 & n = 0 \\\\
1 & n = 1 \\\\
fib(n-1) + fib(n-2) & n > 1
\\end{cases}
```

This is semantically a recursive definition, not a `while` or `for` construct.

## Piecewise functions

A cases expression is a mathematical piecewise definition:

```latex
f(x) :=
\\begin{cases}
x^2 & x \\ge 0 \\\\
-x & x < 0
\\end{cases}
```

The evaluator chooses a branch only when the corresponding condition is decidable.

## Differentiation

Symbolic differentiation is available for the supported symbolic subset.

```latex
\\diff{x}{x^3 + 2x}
```

The result is simplified before evaluation.

PureMath preserves a symbolic derivative when reducing it to a number would destroy the meaning of differentiation.

## Substitution

Symbolic substitution is available through the substitution command:

```latex
\\subs{x=3}{x^2 + 1}
```

The exact surface argument order follows the current parser/evaluator implementation.

## Integration

The current prototype supports exact polynomial integration for its implemented polynomial subset.

```latex
\\int_{0}^{1} x^2
```

This is intentionally narrower than a general-purpose symbolic integration system.

## Limits

The current prototype supports direct-substitution limits for its supported subset:

```latex
\\lim_{x \\to 0} x^2
```

More advanced limit algorithms belong to future semantic-engine work.

## Solving equations

The prototype includes degree-2 polynomial solving:

```latex
\\solve{x^2 - 4 = 0}{x}
```

This should be understood as a deliberately bounded symbolic capability rather than a general theorem prover.

## Sets and relations

Finite sets are first-class mathematical values:

```latex
A := \\{1,2,3\\}
```

They compose with membership and set relations:

```latex
2 \\in A
A \\subseteq \\{1,2,3,4\\}
```

## Linear algebra

Vectors and matrices are structured mathematical values.

Vectors support:

- addition
- scalar multiplication
- dot product
- norm

Matrices support:

- addition
- multiplication
- powers
- transpose
- determinant
- inverse
- rank
- trace
- matrix-vector multiplication
- scaling

Exact rational arithmetic is retained by the current matrix implementation.

## Mathematical core vs runtime

Operations that are mathematically meaningful remain part of the Mathematical Core even when their implementation uses runtime machinery internally.

Environment-dependent capabilities such as output, input, files, networking, clocks, OS APIs, entropy sources, and external processes belong at the Runtime Interface boundary.

This keeps mathematical meaning independent from a particular execution environment.
