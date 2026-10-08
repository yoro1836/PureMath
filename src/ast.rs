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
    Vector {
        elements: Vec<Expr>,
        span: Span,
    },
    Matrix {
        rows: Vec<Vec<Expr>>,
        span: Span,
    },
        elements: Vec<Expr>,
        span: Span,
    },
    Integral {
        var: String,
        lower: Option<Box<Expr>>,
        upper: Option<Box<Expr>>,
        body: Box<Expr>,
        span: Span,
    },
    Limit {
        var: String,
        target: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    Product {
        var: String,
        lower: Box<Expr>,
        upper: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    Sum {
        var: String,
        lower: Box<Expr>,
        upper: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    Abs {
        expr: Box<Expr>,
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
    Union,
    Intersect,
    Difference,
    In,
    Subset,
    SubsetEq,
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
            | Expr::Vector { span, .. }
            | Expr::Matrix { span, .. }
            | Expr::Integral { span, .. }
            | Expr::Limit { span, .. }
            | Expr::Product { span, .. }
            | Expr::Sum { span, .. }
            | Expr::Abs { span, .. }
            | Expr::Sqrt { span, .. }
            | Expr::Piecewise { span, .. }
            | Expr::Rational { span, .. } => *span,
        }
    }
}
