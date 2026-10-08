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
