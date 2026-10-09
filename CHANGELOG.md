# Changelog

## 0.1.1

- Mathematical identifier model: adjacent letters multiply, names are single
  letters or Greek letters with subscripts, multi-letter names use
  `\operatorname{...}` / `\mathrm{...}`. **Breaking:** `fact(n) := ...` is
  now written `\operatorname{fact}(n) := ...`.
- `\diff{x}{E}` takes the variable first. **Breaking:** `\diff{E}{x}` is
  rejected with a hint.
- LaTeX set braces `\{ \}`, `\emptyset`, `\mid` in comprehensions.
- Relations `\lt \le \leq \gt \ge \geq \ne \neq`; `\times`, `\div`.
- Connectives `\land \lor \neg` with partial evaluation.
- Decimal literals as exact rationals (`0.1 + 0.2 = 0.3` is true).
- `x \mapsto E` function values, applicable directly.
- `dx` ends an integrand.
- `\left`, `\right`, sizing and spacing commands are ignored as typesetting.
- Builtins keep unknown arguments symbolic (`\det{A}` stays `\det(A)`).
- Wiki source lives in `wiki/` and is mirrored to the GitHub Wiki from `main`.
- Executable documentation and snapshot tests; CI runs on `dev` and `main`.

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
