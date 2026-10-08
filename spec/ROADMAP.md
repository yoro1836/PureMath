# PureMath Roadmap

## Now: executable semantic kernel

- `:=`
- `=` / relations
- arithmetic
- exact rationals
- functions
- recursion
- piecewise definitions
- finite sums
- sets
- symbolic fallback
- REPL

## Next: semantic richness

- explicit mathematical domains
- set membership
- subsets
- products / tuples
- sequences
- vectors / matrices
- function spaces
- typed relations

## Then: symbolic mathematics

- canonical forms
- algebraic simplification
- substitution
- differentiation
- integration
- limits
- solving
- linear algebra

## Then: mathematical ecosystem

- PureMath mathematical standard library
- probability and statistics
- random variables and pseudo-random sequences
- sampling and combinatorics
- namespaces
- dependency graph
- versioned packages
- documentation generation
- LaTeX rendering

Mathematical functionality should be implemented as Core semantics or library definitions whenever it can be stated independently of the execution environment.

## Runtime boundary

Introduce explicit interfaces only for capabilities that depend on external state:

- stdout / \\print
- stdin / \\read
- files
- networking
- current time
- OS/environment access
- OS entropy
- external processes

## Finally: compilation and proof

- Math IR
- optimizer
- WASM
- native code generation
- theorem/proof layer
