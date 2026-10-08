use super::prelude::*;

pub(super) fn factorial(n: i128) -> Result<Rational, String> {
    let mut out = 1i128;
    for i in 2..=n {
        out = out.checked_mul(i).ok_or("integer overflow in factorial")?;
    }
    Ok(Rational::integer(out))
}

pub(super) fn binom(n: i128, k: i128) -> Result<Rational, String> {
    if k > n {
        return Ok(Rational::integer(0));
    }
    let k = k.min(n - k);
    let mut out = Rational::integer(1);
    for i in 1..=k {
        out = out.mul(&Rational::new(n - k + i, i)?)?;
    }
    Ok(out)
}

pub(super) fn perm(n: i128, k: i128) -> Result<Rational, String> {
    if k > n {
        return Ok(Rational::integer(0));
    }
    let mut out = 1i128;
    for i in 0..k {
        out = out
            .checked_mul(n - i)
            .ok_or("integer overflow in permutation")?;
    }
    Ok(Rational::integer(out))
}

pub(super) fn gcd_i128(mut a: i128, mut b: i128) -> i128 {
    a = a.checked_abs().unwrap_or(i128::MAX);
    b = b.checked_abs().unwrap_or(i128::MAX);
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

pub(super) fn lcm_i128(a: i128, b: i128) -> Result<i128, String> {
    if a == 0 || b == 0 {
        return Ok(0);
    }
    let g = gcd_i128(a, b);
    (a / g)
        .checked_mul(b.checked_abs().ok_or("integer overflow")?)
        .ok_or("integer overflow".into())
}
