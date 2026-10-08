# PureMath v0.1 Conformance Cases

These cases define the intended behavior of the prototype.

| Input | Expected result |
|---|---|
| `2 + 3 * 4` | `14` |
| `x := 10` | `10` |
| `f(x) := x^2 + 1` followed by `f(3)` | `10` |
| `1 / 3 + 1 / 3` | `\\frac{2}{3}` |
| `\\sum_{i=1}^{10} i` | `55` |
| `{1,2,3}` | `{1,2,3}` |
| `\\sqrt{9}` | `3` |
| `x + 1` with undefined `x` | symbolic `x + 1` |
| recursive factorial | exact integer result |

## Semantic invariants

1. `=` never mutates a binding.
2. `:=` never reassigns an existing value as an imperative operation.
3. Symbolic expressions are retained when exact reduction is unavailable.
4. Import reads definitions into an environment and detects recursive module import.
5. Function recursion is expressed through mathematical definitions, not loop syntax.
