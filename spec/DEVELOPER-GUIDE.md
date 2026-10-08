# PureMath Developer Guide v0.1 (Development)

## Adding a new mathematical construct

Do not start by editing the evaluator.

Use this order:

```text
1. Define mathematical meaning.
2. Decide whether it passes the Core syntax admission test.
3. Add AST node if its semantics are genuinely distinct.
4. Add lexer tokens/commands.
5. Add parser production.
6. Add evaluator behavior.
7. Add renderer support.
8. Add conformance tests.
9. Update the specification.
```

## Adding syntax without changing semantics

When a new notation is only another spelling of an existing concept, prefer a surface-parser alias that produces the existing AST node.

For example, `*` and `\\cdot` both map to `BinOp::Mul` in the prototype.

## Do not put symbolic algebra in the parser

The parser should produce structure. It should not expand, factor, differentiate, or otherwise transform expressions.

```text
Parser:        x^2 + 2x + 1
Symbolic layer: normalize/expand/factor as requested
```

## Do not let rendering leak into semantics

The renderer consumes AST/value objects. The AST must never contain KaTeX renderer state.

## Error handling

Use `Diagnostic` with a `Span` for source-related errors. Avoid panicking on malformed user programs.

## Future namespace model

The next module revision should replace the current flat import environment with:

```text
Module
 ├── public definitions
 ├── private definitions
 └── namespace
```

Import resolution should construct a dependency graph rather than execute arbitrary file-level code.
