# PureMath Conformance Test Matrix v0.4

| ID | Feature | Example | Expected |
|---|---|---|---|
| C001 | arithmetic | `2 + 3 * 4` | `14` |
| C002 | definition | `x := 10` | `10` |
| C003 | equality | `x = 10` | proposition `true` |
| C004 | exact rational | `1/3 + 1/3` | `2/3` |
| C005 | function | `f(x) := x^2+1` | callable function |
| C006 | application | `f(3)` | `10` |
| C007 | recursion | `fact(5)` | `120` |
| C008 | piecewise | `cases` | correct branch |
| C009 | finite sum | `sum i=1..10` | `55` |
| C010 | set | `{1,2,3}` | finite set |
| C011 | symbolic | `x + 1` | symbolic expression |
| C012 | symbolic comparison | `x = 10` | symbolic equality |
| C013 | `\\cdot` | `x \\cdot y` | multiplication AST |
| C014 | comments | `x := 10 % c` | `10` |
| C015 | immutability | duplicate `x := ...` | error |
| C016 | recursion limit | infinite recursive call | bounded error |
| C017 | sum limit | excessive finite sum | bounded error |
| C018 | module | `\\import{algebra}` | definitions loaded |
| C019 | module cache | same import twice | module evaluated once |
| C020 | cycle | `a -> b -> a` | circular-dependency error |

The current Rust integration suite covers these behaviors where the prototype parser supports them.
