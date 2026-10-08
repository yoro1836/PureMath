# PureMath Module Architecture v0.1

The prototype is deliberately split into semantic components so that adding mathematical notation does not turn the interpreter into a monolithic parser/evaluator.

```text
src/
├── main.rs        CLI entry point
├── lib.rs         public API
├── diagnostics.rs source spans + diagnostics
├── lexer.rs       LaTeX-like tokenization
├── parser.rs      surface grammar -> Mathematical AST
├── ast.rs         semantic syntax tree
├── value.rs       exact/symbolic runtime values
├── env.rs         immutable mathematical environment
├── evaluator.rs   reduction/evaluation
├── module.rs      declarative module loading
├── render.rs      AST -> LaTeX output
└── repl.rs        interactive shell
```

## Design rule

No module is allowed to make the Mathematical AST depend on a rendering library. Rendering is a consumer of the AST, not its semantic source.

## Evaluation boundary

```text
Source
  ↓
Lexer
  ↓
Parser
  ↓
Mathematical AST
  ↓
Evaluator
  ├── Exact value
  ├── Proposition
  └── Symbolic expression
```

Future compiler work inserts an elaboration/Math IR layer between AST and execution:

```text
Mathematical AST
      ↓
Elaboration / domain checking
      ↓
Math IR
  ┌───┼─────────────┐
  ↓   ↓             ↓
Eval Simplify    Compiler
```
