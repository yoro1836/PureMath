use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rational {
    num: i128,
    den: i128,
}

impl Rational {
    fn new(mut num: i128, mut den: i128) -> Result<Self, String> {
        if den == 0 {
            return Err("division by zero".into());
        }
        if den < 0 {
            num = -num;
            den = -den;
        }
        let g = gcd(num.unsigned_abs(), den.unsigned_abs()) as i128;
        Ok(Self {
            num: num / g,
            den: den / g,
        })
    }
    fn integer(n: i128) -> Self {
        Self { num: n, den: 1 }
    }
    fn is_integer(&self) -> bool {
        self.den == 1
    }
    fn add(&self, rhs: &Self) -> Result<Self, String> {
        Rational::new(self.num * rhs.den + rhs.num * self.den, self.den * rhs.den)
    }
    fn sub(&self, rhs: &Self) -> Result<Self, String> {
        Rational::new(self.num * rhs.den - rhs.num * self.den, self.den * rhs.den)
    }
    fn mul(&self, rhs: &Self) -> Result<Self, String> {
        Rational::new(self.num * rhs.num, self.den * rhs.den)
    }
    fn div(&self, rhs: &Self) -> Result<Self, String> {
        Rational::new(self.num * rhs.den, self.den * rhs.num)
    }
    fn powi(&self, exp: i128) -> Result<Self, String> {
        if exp < 0 {
            return Rational::integer(1).div(&self.powi(-exp)?);
        }
        let mut base = self.clone();
        let mut e = exp as u128;
        let mut out = Rational::integer(1);
        while e > 0 {
            if e & 1 == 1 {
                out = out.mul(&base)?;
            }
            e >>= 1;
            if e > 0 {
                base = base.mul(&base)?;
            }
        }
        Ok(out)
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.max(1)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Integer(i128),
    Rational(Box<Expr>, Box<Expr>),
    Symbol(String),
    UnaryNeg(Box<Expr>),
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Set(Vec<Expr>),
    Sum {
        var: String,
        lower: Box<Expr>,
        upper: Box<Expr>,
        body: Box<Expr>,
    },
    Sqrt(Box<Expr>),
    Piecewise(Vec<(Expr, Expr)>),
    Opaque(String),
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

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Definition {
        name: String,
        params: Vec<String>,
        body: Expr,
    },
    Expression(Expr),
    Import(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Rational(Rational),
    Bool(bool),
    Set(Vec<Value>),
    Function(Rc<FunctionValue>),
    Symbolic(Expr),
    Unit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionValue {
    pub params: Vec<String>,
    pub body: Expr,
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "\\frac{{{}}}{{{}}}", self.num, self.den)
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Rational(r) => write!(f, "{}", r),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Set(xs) => write!(
                f,
                "{{{}}}",
                xs.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Value::Function(fun) => write!(f, "function({})", fun.params.join(", ")),
            Value::Symbolic(e) => write!(f, "{}", render_expr(e)),
            Value::Unit => Ok(()),
        }
    }
}

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    loading: Vec<PathBuf>,
}

impl Environment {
    fn define_value(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
    fn define_function(&mut self, name: String, f: FunctionValue) {
        self.values.insert(name, Value::Function(Rc::new(f)));
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Int(i128),
    Ident(String),
    Command(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Underscore,
    Ampersand,
    RowSep,
    Assign,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
}

fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            c if c.is_whitespace() => i += 1,
            '\\' => {
                if i + 1 < chars.len() && chars[i + 1] == '\\' {
                    out.push(Token::RowSep);
                    i += 2;
                    continue;
                }
                i += 1;
                let start = i;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }
                if start == i {
                    return Err("expected LaTeX command after backslash".into());
                }
                out.push(Token::Command(chars[start..i].iter().collect()));
            }
            ':' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                out.push(Token::Assign);
                i += 2;
            }
            '=' => {
                out.push(Token::Eq);
                i += 1;
            }
            '<' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                out.push(Token::Le);
                i += 2;
            }
            '>' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                out.push(Token::Ge);
                i += 2;
            }
            '<' => {
                out.push(Token::Lt);
                i += 1;
            }
            '>' => {
                out.push(Token::Gt);
                i += 1;
            }
            '+' => {
                out.push(Token::Plus);
                i += 1;
            }
            '-' => {
                out.push(Token::Minus);
                i += 1;
            }
            '*' => {
                out.push(Token::Star);
                i += 1;
            }
            '/' => {
                out.push(Token::Slash);
                i += 1;
            }
            '^' => {
                out.push(Token::Caret);
                i += 1;
            }
            '(' => {
                out.push(Token::LParen);
                i += 1;
            }
            ')' => {
                out.push(Token::RParen);
                i += 1;
            }
            '{' => {
                out.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                out.push(Token::RBrace);
                i += 1;
            }
            ',' => {
                out.push(Token::Comma);
                i += 1;
            }
            '_' => {
                out.push(Token::Underscore);
                i += 1;
            }
            '&' => {
                out.push(Token::Ampersand);
                i += 1;
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                out.push(Token::Int(
                    s.parse().map_err(|_| "integer literal overflow")?,
                ));
            }
            c if c.is_ascii_alphabetic() => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                out.push(Token::Ident(chars[start..i].iter().collect()));
            }
            _ => return Err(format!("unexpected character: {}", chars[i])),
        }
    }
    Ok(out)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn take(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn expect(&mut self, want: Token) -> Result<(), String> {
        match self.take() {
            Some(t) if t == want => Ok(()),
            other => Err(format!("expected {:?}, got {:?}", want, other)),
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        if matches!(self.peek(), Some(Token::Command(c)) if c == "import") {
            self.take();
            return Ok(Stmt::Import(self.parse_group_name()?));
        }
        if let Some(Token::Ident(name)) = self.peek().cloned() {
            let save = self.pos;
            self.take();
            let mut params = Vec::new();
            if matches!(self.peek(), Some(Token::LParen)) {
                self.take();
                if !matches!(self.peek(), Some(Token::RParen)) {
                    loop {
                        match self.take() {
                            Some(Token::Ident(p)) => params.push(p),
                            other => {
                                return Err(format!("expected parameter name, got {:?}", other))
                            }
                        }
                        if matches!(self.peek(), Some(Token::Comma)) {
                            self.take();
                            continue;
                        }
                        break;
                    }
                }
                self.expect(Token::RParen)?;
            }
            if matches!(self.peek(), Some(Token::Assign)) {
                self.take();
                let body = self.parse_expr(0)?;
                if self.peek().is_some() {
                    return Err(format!(
                        "unexpected tokens after definition: {:?}",
                        &self.tokens[self.pos..]
                    ));
                }
                return Ok(Stmt::Definition { name, params, body });
            }
            self.pos = save;
        }
        let expr = self.parse_expr(0)?;
        if self.peek().is_some() {
            return Err(format!("unexpected tokens: {:?}", &self.tokens[self.pos..]));
        }
        Ok(Stmt::Expression(expr))
    }

    fn parse_group_name(&mut self) -> Result<String, String> {
        self.expect(Token::LBrace)?;
        let out = match self.take() {
            Some(Token::Ident(s)) | Some(Token::Command(s)) => s,
            other => return Err(format!("expected name, got {:?}", other)),
        };
        self.expect(Token::RBrace)?;
        Ok(out)
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_prefix()?;
        loop {
            let (op, lbp, rbp) = match self.peek() {
                Some(Token::Plus) => (BinOp::Add, 10, 11),
                Some(Token::Minus) => (BinOp::Sub, 10, 11),
                Some(Token::Star) => (BinOp::Mul, 20, 21),
                Some(Token::Slash) => (BinOp::Div, 20, 21),
                Some(Token::Caret) => (BinOp::Pow, 30, 30),
                Some(Token::Eq) => (BinOp::Eq, 5, 6),
                Some(Token::Lt) => (BinOp::Lt, 5, 6),
                Some(Token::Le) => (BinOp::Le, 5, 6),
                Some(Token::Gt) => (BinOp::Gt, 5, 6),
                Some(Token::Ge) => (BinOp::Ge, 5, 6),
                _ => break,
            };
            if lbp < min_bp {
                break;
            }
            self.take();
            let rhs = self.parse_expr(rbp)?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        let mut expr = match self.take() {
            Some(Token::Int(n)) => Expr::Integer(n),
            Some(Token::Ident(s)) => Expr::Symbol(s),
            Some(Token::Minus) => Expr::UnaryNeg(Box::new(self.parse_expr(25)?)),
            Some(Token::LParen) => {
                let e = self.parse_expr(0)?;
                self.expect(Token::RParen)?;
                e
            }
            Some(Token::LBrace) => {
                let mut elems = Vec::new();
                if !matches!(self.peek(), Some(Token::RBrace)) {
                    loop {
                        elems.push(self.parse_expr(0)?);
                        if matches!(self.peek(), Some(Token::Comma)) {
                            self.take();
                            continue;
                        }
                        break;
                    }
                }
                self.expect(Token::RBrace)?;
                Expr::Set(elems)
            }
            Some(Token::Command(c)) if c == "frac" => {
                let a = self.parse_group_expr()?;
                let b = self.parse_group_expr()?;
                Expr::Rational(Box::new(a), Box::new(b))
            }
            Some(Token::Command(c)) if c == "sqrt" => {
                Expr::Sqrt(Box::new(self.parse_group_expr()?))
            }
            Some(Token::Command(c)) if c == "sum" => self.parse_sum()?,
            Some(Token::Command(c)) if c == "begin" => self.parse_cases()?,
            Some(Token::Command(c)) => Expr::Opaque(format!("\\{}", c)),
            other => return Err(format!("expected expression, got {:?}", other)),
        };
        loop {
            if matches!(self.peek(), Some(Token::LParen)) {
                self.take();
                let mut args = Vec::new();
                if !matches!(self.peek(), Some(Token::RParen)) {
                    loop {
                        args.push(self.parse_expr(0)?);
                        if matches!(self.peek(), Some(Token::Comma)) {
                            self.take();
                            continue;
                        }
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else {
                break;
            }
        }
        if matches!(self.peek(), Some(Token::Caret)) {
            self.take();
            let rhs = self.parse_expr(30)?;
            expr = Expr::Binary {
                op: BinOp::Pow,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_group_expr(&mut self) -> Result<Expr, String> {
        self.expect(Token::LBrace)?;
        let e = self.parse_expr(0)?;
        self.expect(Token::RBrace)?;
        Ok(e)
    }

    fn parse_sum(&mut self) -> Result<Expr, String> {
        self.expect(Token::Underscore)?;
        self.expect(Token::LBrace)?;
        let var = match self.take() {
            Some(Token::Ident(s)) => s,
            other => return Err(format!("expected summation variable, got {:?}", other)),
        };
        self.expect(Token::Eq)?;
        let lower = self.parse_expr(0)?;
        self.expect(Token::RBrace)?;
        self.expect(Token::Caret)?;
        let upper = self.parse_group_expr()?;
        let body = self.parse_expr(0)?;
        Ok(Expr::Sum {
            var,
            lower: Box::new(lower),
            upper: Box::new(upper),
            body: Box::new(body),
        })
    }

    fn parse_cases(&mut self) -> Result<Expr, String> {
        let name = self.parse_group_name()?;
        if name != "cases" {
            return Err(format!("unsupported environment: {}", name));
        }
        let mut rows = Vec::new();
        loop {
            if matches!(self.peek(), Some(Token::Command(c)) if c == "end") {
                self.take();
                let end_name = self.parse_group_name()?;
                if end_name != "cases" {
                    return Err("mismatched cases environment".into());
                }
                break;
            }
            let body = self.parse_expr(0)?;
            self.expect(Token::Ampersand)?;
            let cond = self.parse_expr(0)?;
            rows.push((cond, body));
            if matches!(self.peek(), Some(Token::RowSep)) {
                self.take();
            } else if !matches!(self.peek(), Some(Token::Command(c)) if c == "end") {
                return Err("expected row separator or \\end{cases}".into());
            }
        }
        Ok(Expr::Piecewise(rows))
    }
}

fn parse_line(src: &str) -> Result<Stmt, String> {
    let t = lex(src)?;
    if t.is_empty() {
        Err("empty input".into())
    } else {
        Parser::new(t).parse_statement()
    }
}

fn render_expr(e: &Expr) -> String {
    match e {
        Expr::Integer(n) => n.to_string(),
        Expr::Rational(a, b) => format!("\\frac{{{}}}{{{}}}", render_expr(a), render_expr(b)),
        Expr::Symbol(s) => s.clone(),
        Expr::UnaryNeg(x) => format!("-{}", parenthesize(x)),
        Expr::Binary { op, lhs, rhs } => {
            let sym = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "\\cdot ",
                BinOp::Div => "/",
                BinOp::Pow => "^",
                BinOp::Eq => "=",
                BinOp::Lt => "<",
                BinOp::Le => "\\le",
                BinOp::Gt => ">",
                BinOp::Ge => "\\ge",
            };
            if *op == BinOp::Div {
                format!("\\frac{{{}}}{{{}}}", render_expr(lhs), render_expr(rhs))
            } else if *op == BinOp::Pow {
                format!("{}^{{{}}}", parenthesize(lhs), render_expr(rhs))
            } else {
                format!("{} {} {}", render_expr(lhs), sym, render_expr(rhs))
            }
        }
        Expr::Call { callee, args } => format!(
            "{}({})",
            render_expr(callee),
            args.iter().map(render_expr).collect::<Vec<_>>().join(",")
        ),
        Expr::Set(xs) => format!(
            "\\{{{}\\}}",
            xs.iter().map(render_expr).collect::<Vec<_>>().join(",")
        ),
        Expr::Sum {
            var,
            lower,
            upper,
            body,
        } => format!(
            "\\sum_{{{}={}}}^{{{}}} {}",
            var,
            render_expr(lower),
            render_expr(upper),
            render_expr(body)
        ),
        Expr::Sqrt(x) => format!("\\sqrt{{{}}}", render_expr(x)),
        Expr::Piecewise(rows) => {
            let body = rows
                .iter()
                .map(|(c, e)| format!("{} & {}", render_expr(e), render_expr(c)))
                .collect::<Vec<_>>()
                .join(" \\\\ ");
            format!("\\begin{{cases}} {} \\end{{cases}}", body)
        }
        Expr::Opaque(s) => s.clone(),
    }
}
fn parenthesize(e: &Expr) -> String {
    match e {
        Expr::Integer(_) | Expr::Symbol(_) | Expr::Call { .. } | Expr::Sqrt(_) => render_expr(e),
        _ => format!("({})", render_expr(e)),
    }
}

fn eval_expr(
    expr: &Expr,
    env: &mut Environment,
    locals: &HashMap<String, Value>,
) -> Result<Value, String> {
    match expr {
        Expr::Integer(n) => Ok(Value::Rational(Rational::integer(*n))),
        Expr::Rational(a, b) => match (eval_expr(a, env, locals)?, eval_expr(b, env, locals)?) {
            (Value::Rational(x), Value::Rational(y)) => Ok(Value::Rational(x.div(&y)?)),
            _ => Ok(Value::Symbolic(expr.clone())),
        },
        Expr::Symbol(name) => locals
            .get(name)
            .cloned()
            .or_else(|| env.values.get(name).cloned())
            .or_else(|| Some(Value::Symbolic(Expr::Symbol(name.clone()))))
            .ok_or_else(|| format!("undefined name: {}", name)),
        Expr::UnaryNeg(x) => match eval_expr(x, env, locals)? {
            Value::Rational(v) => Ok(Value::Rational(Rational::integer(-1).mul(&v)?)),
            Value::Symbolic(v) => Ok(Value::Symbolic(Expr::UnaryNeg(Box::new(v)))),
            other => Err(format!("cannot negate {}", other)),
        },
        Expr::Binary { op, lhs, rhs } => eval_binary(*op, lhs, rhs, env, locals),
        Expr::Call { callee, args } => match eval_expr(callee, env, locals)? {
            Value::Function(f) => {
                if f.params.len() != args.len() {
                    return Err(format!(
                        "arity mismatch: expected {}, got {}",
                        f.params.len(),
                        args.len()
                    ));
                }
                let mut child = HashMap::new();
                for (p, a) in f.params.iter().zip(args) {
                    child.insert(p.clone(), eval_expr(a, env, locals)?);
                }
                eval_expr(&f.body, env, &child)
            }
            Value::Symbolic(c) => Ok(Value::Symbolic(Expr::Call {
                callee: Box::new(c),
                args: args.clone(),
            })),
            other => Err(format!("{} is not callable", other)),
        },
        Expr::Set(xs) => Ok(Value::Set(
            xs.iter()
                .map(|x| eval_expr(x, env, locals))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Expr::Sum {
            var,
            lower,
            upper,
            body,
        } => {
            let lo = eval_expr(lower, env, locals)?;
            let hi = eval_expr(upper, env, locals)?;
            match (lo, hi) {
                (Value::Rational(l), Value::Rational(h)) if l.is_integer() && h.is_integer() => {
                    let mut acc = Rational::integer(0);
                    let mut local = locals.clone();
                    if l.num <= h.num {
                        for i in l.num..=h.num {
                            local.insert(var.clone(), Value::Rational(Rational::integer(i)));
                            match eval_expr(body, env, &local)? {
                                Value::Rational(v) => acc = acc.add(&v)?,
                                _ => return Ok(Value::Symbolic(expr.clone())),
                            }
                        }
                    }
                    Ok(Value::Rational(acc))
                }
                _ => Ok(Value::Symbolic(expr.clone())),
            }
        }
        Expr::Sqrt(x) => match eval_expr(x, env, locals)? {
            Value::Rational(v) if v.is_integer() && v.num >= 0 => {
                let n = v.num as u128;
                let r = integer_sqrt(n);
                if r * r == n {
                    Ok(Value::Rational(Rational::integer(r as i128)))
                } else {
                    Ok(Value::Symbolic(Expr::Sqrt(Box::new(Expr::Integer(v.num)))))
                }
            }
            _ => Ok(Value::Symbolic(expr.clone())),
        },
        Expr::Piecewise(rows) => {
            for (cond, body) in rows {
                match eval_expr(cond, env, locals)? {
                    Value::Bool(true) => return eval_expr(body, env, locals),
                    Value::Bool(false) => continue,
                    _ => return Ok(Value::Symbolic(expr.clone())),
                }
            }
            Err("no piecewise branch matched".into())
        }
        Expr::Opaque(_) => Ok(Value::Symbolic(expr.clone())),
    }
}

fn integer_sqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = (x + n / x) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

fn eval_binary(
    op: BinOp,
    lhs: &Expr,
    rhs: &Expr,
    env: &mut Environment,
    locals: &HashMap<String, Value>,
) -> Result<Value, String> {
    let a = eval_expr(lhs, env, locals)?;
    let b = eval_expr(rhs, env, locals)?;
    match op {
        BinOp::Eq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            compare(op, &a, &b).map(Value::Bool)
        }
        BinOp::Add => numeric_or_symbolic(op, a, b, lhs, rhs, |x, y| x.add(y)),
        BinOp::Sub => numeric_or_symbolic(op, a, b, lhs, rhs, |x, y| x.sub(y)),
        BinOp::Mul => numeric_or_symbolic(op, a, b, lhs, rhs, |x, y| x.mul(y)),
        BinOp::Div => numeric_or_symbolic(op, a, b, lhs, rhs, |x, y| x.div(y)),
        BinOp::Pow => match (a, b) {
            (Value::Rational(x), Value::Rational(y)) if y.is_integer() => {
                Ok(Value::Rational(x.powi(y.num)?))
            }
            (va, vb) => Ok(Value::Symbolic(Expr::Binary {
                op,
                lhs: Box::new(value_to_expr(va, lhs.clone())),
                rhs: Box::new(value_to_expr(vb, rhs.clone())),
            })),
        },
    }
}
fn numeric_or_symbolic<F>(
    op: BinOp,
    a: Value,
    b: Value,
    lhs: &Expr,
    rhs: &Expr,
    f: F,
) -> Result<Value, String>
where
    F: Fn(&Rational, &Rational) -> Result<Rational, String>,
{
    match (a, b) {
        (Value::Rational(x), Value::Rational(y)) => Ok(Value::Rational(f(&x, &y)?)),
        (va, vb) => Ok(Value::Symbolic(Expr::Binary {
            op,
            lhs: Box::new(value_to_expr(va, lhs.clone())),
            rhs: Box::new(value_to_expr(vb, rhs.clone())),
        })),
    }
}
fn value_to_expr(v: Value, fallback: Expr) -> Expr {
    match v {
        Value::Rational(r) if r.den == 1 => Expr::Integer(r.num),
        Value::Rational(r) => Expr::Rational(
            Box::new(Expr::Integer(r.num)),
            Box::new(Expr::Integer(r.den)),
        ),
        Value::Symbolic(e) => e,
        _ => fallback,
    }
}
fn compare(op: BinOp, a: &Value, b: &Value) -> Result<bool, String> {
    match (a, b) {
        (Value::Rational(x), Value::Rational(y)) => {
            let l = x.num * y.den;
            let r = y.num * x.den;
            Ok(match op {
                BinOp::Eq => l == r,
                BinOp::Lt => l < r,
                BinOp::Le => l <= r,
                BinOp::Gt => l > r,
                BinOp::Ge => l >= r,
                _ => unreachable!(),
            })
        }
        (Value::Bool(x), Value::Bool(y)) if op == BinOp::Eq => Ok(x == y),
        _ => Err("comparison requires compatible exact values".into()),
    }
}

fn execute(stmt: Stmt, env: &mut Environment, base_dir: &Path) -> Result<Value, String> {
    match stmt {
        Stmt::Definition { name, params, body } => {
            if params.is_empty() {
                let v = eval_expr(&body, env, &HashMap::new())?;
                env.define_value(name, v.clone());
                Ok(v)
            } else {
                env.define_function(name.clone(), FunctionValue { params, body });
                Ok(env.values.get(&name).cloned().unwrap_or(Value::Unit))
            }
        }
        Stmt::Expression(e) => eval_expr(&e, env, &HashMap::new()),
        Stmt::Import(module) => {
            load_module(&module, env, base_dir)?;
            Ok(Value::Unit)
        }
    }
}
fn load_module(module: &str, env: &mut Environment, base_dir: &Path) -> Result<(), String> {
    let mut file = base_dir.join(module);
    if file.extension().is_none() {
        file.set_extension("pmath");
    }
    let canonical =
        fs::canonicalize(&file).map_err(|e| format!("cannot import {}: {}", file.display(), e))?;
    if env.loading.contains(&canonical) {
        return Err(format!("circular module dependency involving {}", module));
    }
    env.loading.push(canonical.clone());
    let src = fs::read_to_string(&canonical)
        .map_err(|e| format!("cannot read {}: {}", canonical.display(), e))?;
    for (line_no, line) in src.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('%') {
            continue;
        }
        let stmt = parse_line(line)
            .map_err(|e| format!("{}:{}: {}", canonical.display(), line_no + 1, e))?;
        execute(stmt, env, canonical.parent().unwrap_or(base_dir))?;
    }
    env.loading.pop();
    Ok(())
}
fn run_file(path: &Path) -> Result<(), String> {
    let src =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    let mut env = Environment::default();
    let base = path.parent().unwrap_or(Path::new("."));
    for (line_no, line) in src.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('%') {
            continue;
        }
        let stmt =
            parse_line(line).map_err(|e| format!("{}:{}: {}", path.display(), line_no + 1, e))?;
        let v = execute(stmt, &mut env, base)?;
        println!("{}", v);
    }
    Ok(())
}
fn repl() {
    let mut env = Environment::default();
    println!("PureMath prototype 0.1");
    println!("Type :quit to exit.");
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }
        let line = line.trim();
        if line == ":quit" || line == ":q" {
            break;
        }
        if line.is_empty() {
            continue;
        }
        match parse_line(line).and_then(|s| execute(s, &mut env, Path::new("."))) {
            Ok(v) => println!("{}", v),
            Err(e) => println!("error: {}", e),
        }
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if let Err(e) = run_file(Path::new(&args[1])) {
            eprintln!("error: {}", e);
            std::process::exit(1)
        }
    } else {
        repl()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn exec(src: &str, env: &mut Environment) -> Value {
        execute(parse_line(src).unwrap(), env, Path::new(".")).unwrap()
    }
    #[test]
    fn arithmetic() {
        let mut e = Environment::default();
        assert_eq!(exec("2 + 3 * 4", &mut e).to_string(), "14");
    }
    #[test]
    fn defs_and_calls() {
        let mut e = Environment::default();
        exec("f(x) := x^2 + 1", &mut e);
        assert_eq!(exec("f(3)", &mut e).to_string(), "10");
    }
    #[test]
    fn exact_fraction() {
        let mut e = Environment::default();
        assert_eq!(exec("1 / 3 + 1 / 3", &mut e).to_string(), "\\frac{2}{3}");
    }
    #[test]
    fn finite_sum() {
        let mut e = Environment::default();
        assert_eq!(exec("\\sum_{i=1}^{10} i", &mut e).to_string(), "55");
    }
    #[test]
    fn set_literal() {
        let mut e = Environment::default();
        assert_eq!(exec("{1,2,3}", &mut e).to_string(), "\\{1,2,3\\}");
    }
    #[test]
    fn piecewise_recursion() {
        let mut e = Environment::default();
        exec(
            r"fact(n) := \begin{cases} 1 & n = 0 \\ n * fact(n-1) & n > 0 \end{cases}",
            &mut e,
        );
        assert_eq!(exec("fact(5)", &mut e).to_string(), "120");
    }
    #[test]
    fn symbolic_preservation() {
        let mut e = Environment::default();
        assert_eq!(exec("x + 1", &mut e).to_string(), "x + 1");
    }
    #[test]
    fn equality_is_not_assignment() {
        let mut e = Environment::default();
        exec("x := 10", &mut e);
        assert_eq!(exec("x = 10", &mut e).to_string(), "true");
        assert_eq!(exec("x", &mut e).to_string(), "10");
    }
}
