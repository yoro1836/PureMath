# PureMath Module Architecture v0.1

The prototype is deliberately split into semantic components so that adding mathematical notation does not turn the interpreter into a monolithic parser/evaluator.

```text
src/
├── ast.rs            semantic nodes
├── diagnostics.rs    spans and diagnostics with source context
├── env.rs            immutable name environment
├── evaluator/        exact + symbolic evaluation
│   ├── mod.rs            Evaluator, statements, resource limits
│   ├── expressions.rs    expression reduction, integrals, limits
│   ├── arithmetic.rs     exact arithmetic and comparison
│   ├── builtins.rs       builtin dispatch
│   ├── algebra.rs        polynomial solving
│   ├── symbolic.rs       differentiation, integration, substitution
│   ├── elementary.rs     elementary functions and constants
│   ├── linear_algebra.rs vectors and matrices
│   ├── number_theory.rs  combinatorics and integer functions
│   ├── sets.rs           set operations
│   └── statistics.rs     mean, variance, standard deviation
├── lexer.rs          LaTeX-like tokenization
├── lib.rs            public crate API
├── main.rs           CLI
├── module.rs         declarative imports
├── names.rs          mathematical identifier model
├── parser.rs         surface -> AST
├── render.rs         AST -> LaTeX output
├── repl.rs           REPL
├── runtime.rs        runtime capabilities (\print)
├── simplifier.rs     small symbolic normalization layer
└── value.rs          exact/symbolic values
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
