# Changelog

## 0.1-development

- Split the interpreter into lexer/parser/AST/value/environment/evaluator/module/render/REPL modules.
- Added source spans and structured diagnostics.
- Added multi-statement parsing in a single source buffer.
- Added `%` comments.
- Added `\cdot` as a surface multiplication operator.
- Preserved unresolved names as symbolic values.
- Added integration tests outside the main binary.
- Separated the CLI from the interpreter library API.
- Added a dedicated module architecture document.
- Kept the project dependency-free to make the semantic prototype easy to audit.
