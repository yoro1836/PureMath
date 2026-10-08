use crate::diagnostics::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Definition {
        name: String,
        params: Vec<String>,
        body: Expr,
        span: Span,
    },
    Expression(Expr),
    Import {
        module: String,
        span: Span,
    },
    Print {
        expr: Expr,
        span: Span,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Integer(i128, Span),
    Rational {
        numerator: Box<Expr>,
        denominator: Box<Expr>,
        span: Span,
    },
    Symbol {
        name: String,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Set {
        elements: Vec<Expr>,
        span: Span,
    },
    Sum {
        var: String,
        lower: Box<Expr>,
        upper: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    Sqrt {
        expr: Box<Expr>,
        span: Span,
    },
    Piecewise {
        branches: Vec<PiecewiseBranch>,
        span: Span,
    },
    Opaque {
        text: String,
        span: Span,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PiecewiseBranch {
    pub value: Expr,
    pub condition: Expr,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Integer(_, span)
            | Expr::Symbol { span, .. }
            | Expr::Opaque { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Call { span, .. }
            | Expr::Set { span, .. }
            | Expr::Sum { span, .. }
            | Expr::Sqrt { span, .. }
            | Expr::Piecewise { span, .. }
            | Expr::Rational { span, .. } => *span,
        }
    }
}
