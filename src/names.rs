//! Mathematical identifier model.
//!
//! PureMath follows mathematical convention: a variable is a single letter,
//! optionally decorated (Greek letters, subscripts), and adjacent letters
//! denote a product (`xy` is `x \cdot y`). Multi-letter names are written as
//! upright operator names (`\operatorname{fact}`, `\mathrm{fact}`).
//!
//! Only the standard operator and command words below are recognised as
//! whole words without a backslash; every other run of letters is split into
//! single-letter names by the lexer.

/// Words that stay whole when written without a leading backslash.
pub fn is_reserved_word(word: &str) -> bool {
    matches!(
        word,
        // structural commands
        "begin" | "end" | "print" | "import" | "frac" | "sqrt" | "abs" | "vec" | "set" | "tuple"
            | "sum" | "prod" | "lim" | "int" | "operatorname" | "mathrm"
            // relations and set operators
            | "in" | "subset" | "subseteq" | "cup" | "cap" | "setminus" | "cdot" | "to"
            // constants
            | "pi"
            // standard functions
            | "sin" | "cos" | "tan" | "ln" | "log" | "exp"
            | "factorial" | "binom" | "choose" | "perm" | "permutation" | "gcd" | "lcm"
            | "floor" | "ceil" | "min" | "max" | "range"
            | "dot" | "norm" | "det" | "transpose" | "trans" | "inverse" | "inv" | "rank"
            | "card" | "cardinality" | "trace"
            | "mean" | "variance" | "stdev"
            | "diff" | "derivative" | "subs" | "substitute" | "solve"
    )
}

/// Greek letters usable as variable names. `\pi` is excluded: it is a constant.
pub fn is_greek_letter(command: &str) -> bool {
    matches!(
        command,
        "alpha"
            | "beta"
            | "gamma"
            | "delta"
            | "epsilon"
            | "varepsilon"
            | "zeta"
            | "eta"
            | "theta"
            | "vartheta"
            | "iota"
            | "kappa"
            | "lambda"
            | "mu"
            | "nu"
            | "xi"
            | "rho"
            | "varrho"
            | "sigma"
            | "varsigma"
            | "tau"
            | "upsilon"
            | "phi"
            | "varphi"
            | "chi"
            | "psi"
            | "omega"
            | "Gamma"
            | "Delta"
            | "Theta"
            | "Lambda"
            | "Xi"
            | "Sigma"
            | "Upsilon"
            | "Phi"
            | "Psi"
            | "Omega"
    )
}

/// Commands that introduce a multi-letter upright name.
pub fn is_operator_name_command(command: &str) -> bool {
    matches!(command, "operatorname" | "mathrm")
}

/// Infix words and commands; these never start an implicit multiplication operand.
pub fn is_infix_word(word: &str) -> bool {
    matches!(
        word,
        "in" | "subset"
            | "subseteq"
            | "cup"
            | "cap"
            | "setminus"
            | "cdot"
            | "to"
            | "end"
            | "right"
            | "mid"
            | "times"
            | "le"
            | "leq"
            | "ge"
            | "geq"
            | "ne"
            | "neq"
            | "lt"
            | "gt"
            | "land"
            | "lor"
            | "mapsto"
    )
}
