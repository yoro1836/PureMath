from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
required = [
    "Cargo.toml",
    "src/main.rs",
    "README.md",
    "spec/SPEC-v0.1.md",
    "spec/IMPLEMENTATION-PLAN.md",
    "spec/ARCHITECTURE.md",
    "spec/SYNTAX-DECISIONS.md",
    "tests/CONFORMANCE.md",
    "examples/main.pmath",
    "examples/algebra.pmath",
    "examples/import_example.pmath",
]
missing = [x for x in required if not (ROOT / x).exists()]
if missing:
    raise SystemExit("missing required files: " + ", ".join(missing))

src = (ROOT / "src/main.rs").read_text()
checks = {
    ":= definition token": "Assign",
    "equality operator": "BinOp::Eq",
    "piecewise AST": "Expr::Piecewise",
    "finite sum AST": "Expr::Sum",
    "symbolic fallback": "Value::Symbolic",
    "module import": "Stmt::Import",
    "unit tests": "#[cfg(test)]",
}
missing_checks = [name for name, needle in checks.items() if needle not in src]
if missing_checks:
    raise SystemExit("missing implementation anchors: " + ", ".join(missing_checks))

print("PureMath layout/anchor checks: PASS")
