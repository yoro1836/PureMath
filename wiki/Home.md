# PureMath Wiki

> **Mathematics as a Language.**

PureMath is an experimental programming language in which mathematical objects, relations, and definitions form the semantic core of the language.

It is not an imperative language with mathematical-looking syntax. The goal is to make mathematical notation executable while keeping the meaning of core constructs grounded in mathematics.

## Start here

- [[Getting Started]]
- [[Language Guide]]
- [[Syntax Reference]]
- [[Mathematical Constructs]]
- [[Execution and Modules]]
- [[Examples]]
- [[Design Philosophy]]

## Core idea

PureMath treats a mathematical definition as a program construct.

```latex
x := 10
f(x) := x^2 + 1
```

Here `:=` introduces an immutable mathematical definition. It is not mutable assignment.

Mathematical equality is written with `=`:

```text
x = 10
```

The language deliberately separates this from definition.

## Current prototype

The current prototype supports:

- exact integer and rational arithmetic
- symbolic expressions and simplification
- definitions and function application
- recursive piecewise definitions
- sets and set relations
- vectors and matrices
- number theory and combinatorics helpers
- elementary functions and constants
- statistics
- symbolic substitution and differentiation
- exact polynomial integration
- direct-substitution limits
- degree-2 polynomial solving
- modules and imports
- explicit runtime output with `\\print{...}`
- CLI execution, AST inspection, and REPL

The implementation is still a prototype. Full KaTeX compatibility is a surface-language goal, not a claim about the current parser.

## Design boundary

The Mathematical Core intentionally does not use conventional imperative control constructs such as:

```text
if
for
while
switch
goto
break
continue
return
class
struct
object
mutable assignment
```

Equivalent computational behavior can be expressed through mathematical definitions, functions, recursion, piecewise definitions, sums, products, and other mathematical constructs.

For the formal semantic boundary, see `spec/SPEC-v0.1.md` in the repository.
