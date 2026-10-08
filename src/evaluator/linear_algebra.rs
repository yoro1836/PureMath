use super::prelude::*;

pub(super) fn matrix_shape(m: &[Vec<Value>]) -> (usize, usize) {
    (m.len(), m.first().map_or(0, Vec::len))
}

pub(super) fn scale_vector(v: &[Value], s: &Rational) -> Result<Vec<Value>, String> {
    v.iter()
        .map(|value| match value {
            Value::Rational(x) => x.mul(s).map(Value::Rational),
            _ => Err("vector contains a non-exact element".into()),
        })
        .collect()
}

pub(super) fn scale_matrix(m: &[Vec<Value>], s: &Rational) -> Result<Vec<Vec<Value>>, String> {
    m.iter()
        .map(|row| {
            row.iter()
                .map(|value| match value {
                    Value::Rational(x) => x.mul(s).map(Value::Rational),
                    _ => Err("matrix contains a non-exact element".into()),
                })
                .collect()
        })
        .collect()
}

pub(super) fn dot_vectors(a: &[Value], b: &[Value]) -> Result<Rational, String> {
    let mut out = Rational::integer(0);
    for (x, y) in a.iter().zip(b) {
        let (Value::Rational(x), Value::Rational(y)) = (x, y) else {
            return Err("vector contains a non-exact element".into());
        };
        let product = x.mul(y)?;
        out = out.add(&product)?;
    }
    Ok(out)
}

pub(super) fn matrix_vector_mul(m: &[Vec<Value>], v: &[Value]) -> Result<Vec<Value>, String> {
    m.iter()
        .map(|row| {
            if row.len() != v.len() {
                return Err("matrix/vector dimensions do not match".into());
            }
            let value = dot_vectors(row, v)?;
            Ok(Value::Rational(value))
        })
        .collect()
}

pub(super) fn matrix_matrix_mul(
    a: &[Vec<Value>],
    b: &[Vec<Value>],
) -> Result<Vec<Vec<Value>>, String> {
    if a.is_empty() || b.is_empty() || a[0].len() != b.len() {
        return Err("matrix dimensions do not match".into());
    }
    let rows = a.len();
    let cols = b[0].len();
    let mut out = vec![vec![Value::Rational(Rational::integer(0)); cols]; rows];
    for i in 0..rows {
        for j in 0..cols {
            let mut value = Rational::integer(0);
            for k in 0..b.len() {
                let (Value::Rational(x), Value::Rational(y)) = (&a[i][k], &b[k][j]) else {
                    return Err("matrix contains a non-exact element".into());
                };
                value = value.add(&x.mul(y)?)?;
            }
            out[i][j] = Value::Rational(value);
        }
    }
    Ok(out)
}

pub(super) fn transpose(m: &[Vec<Value>]) -> Vec<Vec<Value>> {
    if m.is_empty() {
        return Vec::new();
    }
    (0..m[0].len())
        .map(|j| m.iter().map(|row| row[j].clone()).collect())
        .collect()
}

pub(super) fn trace_matrix(m: &[Vec<Value>]) -> Result<Rational, String> {
    if m.is_empty() || m.len() != m[0].len() {
        return Err("trace requires a non-empty square matrix".into());
    }
    let mut out = Rational::integer(0);
    for i in 0..m.len() {
        let Value::Rational(value) = &m[i][i] else {
            return Err("matrix contains a non-exact element".into());
        };
        out = out.add(value)?;
    }
    Ok(out)
}

pub(super) fn inverse_matrix(m: &[Vec<Value>]) -> Result<Vec<Vec<Value>>, String> {
    if m.is_empty() || m.len() != m[0].len() {
        return Err("inverse requires a non-empty square matrix".into());
    }
    let n = m.len();
    let mut aug = vec![vec![Rational::integer(0); n * 2]; n];
    for i in 0..n {
        if m[i].len() != n {
            return Err("inverse requires a square matrix".into());
        }
        for j in 0..n {
            let Value::Rational(value) = &m[i][j] else {
                return Err("matrix contains a non-exact element".into());
            };
            aug[i][j] = value.clone();
        }
        aug[i][n + i] = Rational::integer(1);
    }

    for col in 0..n {
        let pivot = (col..n).find(|&row| aug[row][col].num != 0);
        let Some(pivot) = pivot else {
            return Err("matrix is singular".into());
        };
        if pivot != col {
            aug.swap(pivot, col);
        }
        let pivot_value = aug[col][col].clone();
        for j in 0..2 * n {
            aug[col][j] = aug[col][j].div(&pivot_value)?;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row][col].clone();
            if factor.num == 0 {
                continue;
            }
            for j in 0..2 * n {
                let product = factor.mul(&aug[col][j])?;
                aug[row][j] = aug[row][j].sub(&product)?;
            }
        }
    }
    Ok((0..n)
        .map(|i| {
            (0..n)
                .map(|j| Value::Rational(aug[i][n + j].clone()))
                .collect()
        })
        .collect())
}

pub(super) fn rank_matrix(m: &[Vec<Value>]) -> Result<usize, String> {
    if m.is_empty() || m[0].is_empty() {
        return Ok(0);
    }
    let rows = m.len();
    let cols = m[0].len();
    let mut a = Vec::with_capacity(rows);
    for row in m {
        if row.len() != cols {
            return Err("matrix rows must have equal length".into());
        }
        a.push(
            row.iter()
                .map(|value| match value {
                    Value::Rational(r) => Ok(r.clone()),
                    _ => Err("matrix contains a non-exact element".into()),
                })
                .collect::<Result<Vec<Rational>, String>>()?,
        );
    }

    let mut rank = 0;
    for col in 0..cols {
        let Some(pivot) = (rank..rows).find(|&row| a[row][col].num != 0) else {
            continue;
        };
        a.swap(rank, pivot);
        let pivot_value = a[rank][col].clone();
        for row in (rank + 1)..rows {
            let factor = a[row][col].div(&pivot_value)?;
            for j in col..cols {
                a[row][j] = a[row][j].sub(&factor.mul(&a[rank][j])?)?;
            }
        }
        rank += 1;
        if rank == rows {
            break;
        }
    }
    Ok(rank)
}

pub(super) fn determinant(m: &[Vec<Value>]) -> Result<Rational, String> {
    if m.is_empty() || m.len() != m[0].len() {
        return Err("determinant requires a non-empty square matrix".into());
    }
    let n = m.len();
    let mut a = Vec::with_capacity(n);
    for row in m {
        if row.len() != n {
            return Err("determinant requires a square matrix".into());
        }
        a.push(
            row.iter()
                .map(|value| match value {
                    Value::Rational(r) => Ok(r.clone()),
                    _ => Err("matrix contains a non-exact element".into()),
                })
                .collect::<Result<Vec<Rational>, String>>()?,
        );
    }
    let mut det = Rational::integer(1);
    for col in 0..n {
        let pivot = (col..n).find(|&row| a[row][col].num != 0);
        let Some(pivot) = pivot else {
            return Ok(Rational::integer(0));
        };
        if pivot != col {
            a.swap(pivot, col);
            det = Rational::integer(-1).mul(&det)?;
        }
        let pivot_value = a[col][col].clone();
        det = det.mul(&pivot_value)?;
        for row in (col + 1)..n {
            let factor = a[row][col].div(&pivot_value)?;
            for j in col..n {
                let product = factor.mul(&a[col][j])?;
                a[row][j] = a[row][j].sub(&product)?;
            }
        }
    }
    Ok(det)
}

pub(super) fn matrix_pow(matrix: &[Vec<Value>], exp: i128) -> Result<Vec<Vec<Value>>, String> {
    if matrix.is_empty() || matrix.len() != matrix[0].len() {
        return Err("matrix power requires a non-empty square matrix".into());
    }
    let n = matrix.len();
    let mut result = vec![vec![Value::Rational(Rational::integer(0)); n]; n];
    for i in 0..n {
        result[i][i] = Value::Rational(Rational::integer(1));
    }
    let mut base = matrix.to_vec();
    let mut e = exp as u128;
    while e > 0 {
        if e & 1 == 1 {
            result = matrix_matrix_mul(&result, &base)?;
        }
        e >>= 1;
        if e > 0 {
            base = matrix_matrix_mul(&base, &base)?;
        }
    }
    Ok(result)
}
