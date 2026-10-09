use crate::ast::Expr;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rational {
    pub num: i128,
    pub den: i128,
}

impl Rational {
    pub fn new(mut num: i128, mut den: i128) -> Result<Self, String> {
        if den == 0 {
            return Err("division by zero".into());
        }
        if den < 0 {
            num = num
                .checked_neg()
                .ok_or("integer overflow while normalizing rational")?;
            den = den
                .checked_neg()
                .ok_or("integer overflow while normalizing rational")?;
        }
        let g = gcd(num.unsigned_abs(), den.unsigned_abs()) as i128;
        Ok(Self {
            num: num / g,
            den: den / g,
        })
    }
    pub fn integer(n: i128) -> Self {
        Self { num: n, den: 1 }
    }
    pub fn is_integer(&self) -> bool {
        self.den == 1
    }
    pub fn add(&self, rhs: &Self) -> Result<Self, String> {
        let left = self.num.checked_mul(rhs.den).ok_or("integer overflow")?;
        let right = rhs.num.checked_mul(self.den).ok_or("integer overflow")?;
        Self::new(
            left.checked_add(right).ok_or("integer overflow")?,
            self.den.checked_mul(rhs.den).ok_or("integer overflow")?,
        )
    }
    pub fn sub(&self, rhs: &Self) -> Result<Self, String> {
        let left = self.num.checked_mul(rhs.den).ok_or("integer overflow")?;
        let right = rhs.num.checked_mul(self.den).ok_or("integer overflow")?;
        Self::new(
            left.checked_sub(right).ok_or("integer overflow")?,
            self.den.checked_mul(rhs.den).ok_or("integer overflow")?,
        )
    }
    pub fn mul(&self, rhs: &Self) -> Result<Self, String> {
        Self::new(
            self.num.checked_mul(rhs.num).ok_or("integer overflow")?,
            self.den.checked_mul(rhs.den).ok_or("integer overflow")?,
        )
    }
    pub fn div(&self, rhs: &Self) -> Result<Self, String> {
        if rhs.num == 0 {
            return Err("division by zero".into());
        }
        Self::new(
            self.num.checked_mul(rhs.den).ok_or("integer overflow")?,
            self.den.checked_mul(rhs.num).ok_or("integer overflow")?,
        )
    }
    pub fn powi(&self, exp: i128) -> Result<Self, String> {
        if exp < 0 {
            let positive = exp.checked_neg().ok_or("integer overflow in exponent")?;
            return Self::integer(1).div(&self.powi(positive)?);
        }
        let mut base = self.clone();
        let mut e = exp as u128;
        let mut out = Self::integer(1);
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
pub struct FunctionValue {
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Rational(Rational),
    Bool(bool),
    Set(Vec<Value>),
    Vector(Vec<Value>),
    Matrix(Vec<Vec<Value>>),
    Function(Rc<FunctionValue>),
    Symbolic(Expr),
    Unit,
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else if self.num < 0 {
            write!(
                f,
                "-\\frac{{{}}}{{{}}}",
                self.num.checked_abs().ok_or(fmt::Error)?,
                self.den
            )
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
            Value::Set(xs) => {
                let body = xs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                write!(f, "\\{{{}\\}}", body)
            }
            Value::Vector(xs) => {
                let body = xs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "\\left({}\\right)", body)
            }
            Value::Matrix(rows) => {
                let body = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(" & ")
                    })
                    .collect::<Vec<_>>()
                    .join(" \\\\ ");
                write!(f, "\\begin{{pmatrix}} {} \\end{{pmatrix}}", body)
            }
            Value::Function(fun) => {
                write!(f, "{}", crate::render::function(&fun.params, &fun.body))
            }
            Value::Symbolic(e) => write!(f, "{}", crate::render::expr(e)),
            Value::Unit => Ok(()),
        }
    }
}

pub fn value_to_expr(value: &Value, fallback: &Expr) -> Expr {
    match value {
        Value::Rational(r) if r.den == 1 => Expr::Integer(r.num, fallback.span()),
        Value::Rational(r) => Expr::Rational {
            numerator: Box::new(Expr::Integer(r.num, fallback.span())),
            denominator: Box::new(Expr::Integer(r.den, fallback.span())),
            span: fallback.span(),
        },
        Value::Symbolic(expr) => expr.clone(),
        Value::Vector(xs) | Value::Set(xs) if xs.iter().all(is_expressible) => {
            let elements = xs.iter().map(|x| value_to_expr(x, fallback)).collect();
            if matches!(value, Value::Vector(_)) {
                Expr::Vector {
                    elements,
                    span: fallback.span(),
                }
            } else {
                Expr::Set {
                    elements,
                    span: fallback.span(),
                }
            }
        }
        Value::Matrix(rows) if rows.iter().flatten().all(is_expressible) => Expr::Matrix {
            rows: rows
                .iter()
                .map(|row| row.iter().map(|x| value_to_expr(x, fallback)).collect())
                .collect(),
            span: fallback.span(),
        },
        _ => fallback.clone(),
    }
}

/// Whether `value_to_expr` can rebuild the value without its fallback.
fn is_expressible(value: &Value) -> bool {
    match value {
        Value::Rational(_) | Value::Symbolic(_) => true,
        Value::Vector(xs) | Value::Set(xs) => xs.iter().all(is_expressible),
        Value::Matrix(rows) => rows.iter().flatten().all(is_expressible),
        _ => false,
    }
}

pub fn exact_integer_sqrt(n: u128) -> u128 {
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
