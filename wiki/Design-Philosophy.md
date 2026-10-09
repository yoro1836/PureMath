# Design Philosophy

## Mathematics is the semantic core

PureMath is built around a strict distinction:

> Being implementable using mathematics is not the same as directly expressing a mathematical concept.

The Mathematical Core admits constructs because the constructs themselves have mathematical semantics, not merely because an imperative feature can be encoded with them.

## Direct correspondence

A Core construct should directly denote a mathematical object, relation, operation, function, sequence, transformation, or other mathematical entity.

Examples include:

```latex
f(x) := x^2 + 1
\sum_{i=1}^{n} i
\{1\} \subseteq \{1,2\}
g(n) := \begin{cases} 1 & n = 0 \\ n \cdot g(n-1) & n > 0 \end{cases}
```

## No hidden mutable state

Core definitions use immutable bindings:

```text
x := E
```

The same environment cannot silently replace the meaning of an existing name with a later assignment.

This is why conventional mutable assignment is intentionally outside the Mathematical Core.

## Imperative constructs are not the semantic model

The prototype intentionally excludes conventional Core syntax such as:

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

This is a semantic boundary, not a claim that equivalent algorithms cannot be computed.

For example, iteration can often be represented as:

- finite sums/products
- recursively defined functions
- sequences
- mathematical transformations

## Turing completeness is not the admission criterion

A construct does not belong in the Mathematical Core merely because it increases computational expressiveness.

A language can encode a construct using existing primitives without that construct being a natural mathematical notation.

PureMath therefore does not use Turing completeness as its Core-syntax admission test.

## Mathematical values can stay symbolic

Exact and symbolic results are first-class.

If an operation cannot currently be reduced without changing its mathematical meaning, PureMath preserves the expression rather than forcing an arbitrary floating-point approximation.

```text
x + 1
\sqrt{2}
\sin{x}
```

may therefore remain symbolic.

## Runtime is a boundary

External side effects are separated from mathematical semantics.

For example:

```latex
\print{E}
```

has a mathematical argument `E`, but the act of writing output is a Runtime Interface operation.

The same boundary applies to external files, networking, clocks, OS APIs, entropy, and processes.

## Surface syntax vs semantic AST

PureMath aims for a LaTeX-compatible surface, but its semantic AST is independent of KaTeX or any particular renderer.

The implementation does not claim that every valid KaTeX document is automatically valid PureMath code.

The intended pipeline is:

```text
LaTeX-like source
      ↓
PureMath parser
      ↓
Mathematical AST
      ↓
semantic validation / evaluation
```

## Composition and minimality

Core constructs should compose naturally with other mathematical expressions.

The design favors a relatively small semantic core over a large set of programming conveniences.

Future architectural work includes a dedicated Math IR and deeper domain checking without moving mathematical meaning into parser-specific implementation machinery.