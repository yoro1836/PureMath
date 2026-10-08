# Execution and Modules

## Evaluation model

The current execution pipeline is conceptually:

```text
source
  ↓
lexer
  ↓
parser
  ↓
Mathematical AST
  ↓
evaluator
  ↓
Exact / Proposition / Symbolic / Unit result
```

The AST carries mathematical meaning rather than preserving a one-to-one representation of the original surface spelling.

## Exact evaluation

The evaluator prefers exact values.

For example:

```latex
\\frac{1}{2} + \\frac{1}{3}
```

is handled as exact rational arithmetic instead of immediately becoming a floating-point approximation.

## Symbolic preservation

A symbolic expression is a legitimate result:

```text
x + 1
```

The evaluator does not force every expression into a numeric approximation.

Symbol preservation is especially important for calculus and algebra operations.

## Runtime output

Output is explicitly separated from Mathematical Core evaluation.

```latex
\\print{x + 2}
```

`\\print` is a Runtime Interface operation. A source file does not implicitly print every evaluated expression.

The REPL is different: it can display the result of each entered expression as an interactive interface.

## Modules

A `.pmath` source file is a declarative module.

Imports use:

```latex
\\import{algebra}
```

The current module loader:

1. resolves the module path relative to the importing module,
2. detects active dependency cycles,
3. parses the module,
4. evaluates its definitions into the importing environment,
5. caches successful loads.

The prototype intentionally uses a flat environment. Namespaces and explicit exports are planned for a later module revision.

## Runtime boundary

The Runtime Interface is reserved for capabilities whose meaning depends on external state or the execution environment.

Examples include:

```text
stdout / \\print
stdin / future input facilities
files
networking
current time
OS/environment APIs
entropy sources
external processes
```

A capability is not moved out of the Mathematical Core merely because its implementation uses runtime services. The deciding question is whether the mathematical meaning itself depends on external state.

## CLI

Useful commands:

```bash
puremath
puremath file.pmath
puremath --ast file.pmath
puremath --help
puremath --version
```

The current prototype also rejects unknown CLI forms rather than silently interpreting them as source files.

## AST inspection

Use `--ast` when investigating parser behavior:

```bash
puremath --ast examples/main.pmath
```

This is useful when a surface expression parses successfully but does not evaluate as expected, because the AST reveals the semantic structure handed to the evaluator.
