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