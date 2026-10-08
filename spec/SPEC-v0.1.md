# PureMath Language Specification v0.1 (Prototype)

Status: experimental implementation baseline.

This document freezes the semantics that the prototype is trying to preserve. It is not a promise that the current parser accepts all KaTeX syntax.

## 1. Language identity

PureMath treats mathematical objects, relations, and definitions as program constructs. It must not be interpreted as imperative syntax with mathematical glyphs substituted for keywords.

The central distinction is:

```text
computationally expressible by mathematics  !=  mathematically justified syntax
```

A construct is admitted to Mathematical Core because the construct itself denotes a directly definable mathematical object/relation/operation, not because it can encode an imperative feature.

## 2. Three-layer model

### 2.1 Mathematical Core

Contains constructs with mathematical semantics independent of program execution.

Examples:

```latex
f(x) := x^2 + 1
A \subseteq B
\sum_{i=1}^{n} i
\begin{cases} ... \end{cases}
```

### 2.2 Meta Syntax

Controls organization of mathematical modules.

```latex
\import{algebra}
```

This layer is intentionally not claimed to be mathematical notation.

### 2.3 Runtime Interface

The Runtime Interface contains capabilities whose meaning depends on the execution environment or on externally observable state.

Examples:

```text
stdout / \\print
stdin / \\read
files
networking
current time
OS/environment APIs
OS entropy sources
external processes
```

These capabilities are explicitly separated from Core evaluation because they introduce external state, side effects, or implementation-environment dependence.

A capability is **not** Runtime merely because its implementation may use runtime machinery. If its meaning is independently definable as a mathematical object, relation, operation, function, distribution, sequence, or transformation, it belongs to Mathematical Core or the mathematical standard library.

For example:

```text
pseudo-random sequences
probability distributions
random variables
sampling algorithms
statistics
```

have mathematical semantics and therefore are not excluded from Core merely because an implementation may use a runtime-provided seed.

An operating-system entropy source used to obtain a seed is Runtime; the resulting mathematical random process is Core/math-library semantics.

## 3. Syntax admission rule

A proposed Core construct `X` must satisfy all mandatory conditions:

A. `X` directly denotes an independently definable mathematical concept.

B. The correspondence is direct; an arbitrary encoding/decoding pipeline is not sufficient.

C. Existing standard mathematical notation does not already express the same concept naturally enough.

D. The construct does not inherently require hidden mutable state or execution sequencing.

E. The construct composes naturally with other mathematical expressions.

F. It adds semantic expressive value rather than only programming convenience.

Therefore, Turing-completeness is not a Core-syntax admission criterion.

## 4. Definitions and equality

### 4.1 Definition

```text
x := E
f(x) := E
```

`:=` adds an immutable name binding to the current mathematical environment.

It is never assignment to mutable storage.

### 4.2 Equality

```text
x = y
```

`=` denotes mathematical equality and evaluates to a proposition when both sides can be compared exactly. If comparison cannot currently be decided, the equality is preserved symbolically.

## 5. Names and immutability

Identifiers denote mathematical names/references. They are not mutable storage locations.

The prototype rejects a second definition of an existing name in the same environment.

```text
x := 1
x := 2     % error
```

This rule prevents `:=` from becoming accidental reassignment.

## 6. Functions

```latex
f(x) := x^2 + 1
f(3)
```

A function definition denotes a mathematical function; a call denotes function application.

The long-term domain annotation is:

```latex
f : A \to B
```

Functions may be passed as values when their domains support it.

## 7. Piecewise functions

```latex
f(x) :=
\begin{cases}
x^2 & x \ge 0 \\
-x & x < 0
\end{cases}
```

The construct is semantically a piecewise mathematical definition, not an imperative conditional instruction.

A branch is selected only when its condition is decidable in the current environment. Otherwise the whole piecewise expression may remain symbolic.

## 8. Recursive definitions

Recursive definitions are mathematical definitions whose body may refer to the defined function.

```latex
fact(n) :=
\begin{cases}
1 & n = 0 \\
n \cdot fact(n-1) & n > 0
\end{cases}
```

The evaluator uses recursion operationally, but recursion is justified by the mathematical definition rather than by being a substitute encoding for `while`.

## 9. Indexed aggregation

Finite sums are mathematical indexed aggregations:

```latex
\sum_{i=1}^{10} i
```

They are not loop statements.

The prototype directly evaluates finite integer bounds. A non-evaluable sum is preserved symbolically.

## 10. Sets

Finite set literals are Core values:

```latex
A := \{1,2,3\}
```

Set comprehension is planned but not yet required by the prototype parser.


## 10.1 Indexed products and absolute value

Finite products are mathematical indexed aggregations:

```latex
\prod_{i=1}^{5} i
```

They evaluate exactly when the bounds are finite integers and the body produces exact rational values. An empty product evaluates to `1`.

Absolute value is a mathematical operation:

```latex
\abs{-\frac{3}{2}}
```

For exact rational values, it evaluates exactly rather than through floating-point approximation.

## 11. Exact and symbolic values

The evaluator distinguishes at minimum:

```text
Exact rational
Boolean proposition
Finite set
Function
Symbolic expression
Unit
```

A symbolic value is a valid evaluation result, not a failure mode.

Examples:

```latex
x + 1
x = 10
\sqrt{2}
\sin(x)
```

may remain symbolic until a future semantic engine can reduce them further.

## 12. Mathematical AST

The current AST includes:

```text
Program
Definition
Import
Expression
  Integer
  Rational
  Symbol
  Unary
  Binary
  Call
  Set
  Sum
  Sqrt
  Piecewise
  Opaque
```

Every expression node carries a source span.

The AST is semantic and is intentionally independent from any KaTeX renderer.

## 13. Parser boundary

```text
source
  ↓
lexer
  ↓
parser
  ↓
Mathematical AST
```

The current surface parser supports a small LaTeX-like subset. Full KaTeX compatibility is a surface-language goal, not the semantic AST definition.

## 14. Evaluation model

```text
Mathematical AST
       ↓
Evaluator
       ↓
Exact / Proposition / Symbolic / Unit
```

Evaluation is reduction under an environment of immutable mathematical definitions.

### Symbol preservation rule

When a subexpression cannot be evaluated without losing mathematical meaning, preserve it as an expression instead of forcing an error or floating-point approximation.

## 15. Prototype safety limits

The evaluator has bounded defaults for:

- recursive function call depth
- finite-sum term count

These are implementation safeguards and are not part of mathematical meaning.

## 16. Rendering

Rendered output is derived from the Mathematical AST/value and must never define semantics.

Example:

```text
Value::Symbolic(AST)
        ↓
LaTeX renderer
        ↓
\\sqrt{x}
```

## 17. Module semantics

A `.pmath` file is a declarative module.

Importing a module:

1. resolves the module path relative to the importing module,
2. detects active dependency cycles,
3. parses the full source,
4. evaluates its definitions into the importing environment,
5. caches successful loads.

Namespace objects and explicit exports are planned for the next module revision; the current prototype uses a flat environment intentionally to keep the semantic baseline small.

## 18. Future elaboration

Before a production compiler, insert:

```text
Mathematical AST
      ↓
Elaboration / domain checking
      ↓
Math IR
```

This layer will handle domain information such as:

```latex
x \in \mathbb{N}
f : A \to B
```

without making those concepts part of the parser's operational machinery.

## 19. Forbidden Core constructs

The following are intentionally absent from Mathematical Core:

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

Their absence is not an inability to compute equivalent algorithms. It is a semantic boundary decision.

## 21. Explicit output

Output is a Runtime Interface operation, not a Mathematical Core operation.

```latex
\\print{E}
```

evaluates the mathematical expression `E`, writes its resulting value to standard output, and produces `Unit`.

The mathematical evaluator does not perform the side effect directly. Runtime execution supplies the output capability.

An imported declarative module must not use runtime output during module loading.

When executing a `.pmath` file, expression statements are evaluated for their effects on the mathematical environment but their values are not implicitly written to stdout. Program output requires an explicit `\\print{E}` operation. The REPL is a separate interactive interface and may display the result of each entered expression.
