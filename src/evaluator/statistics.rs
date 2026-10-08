use super::prelude::*;

pub(super) fn mean(xs: &[Rational]) -> Result<Rational, String> {
    if xs.is_empty() {
        return Err("mean of empty data is undefined".into());
    }
    let mut sum = Rational::integer(0);
    for x in xs {
        sum = sum.add(x)?;
    }
    sum.div(&Rational::integer(xs.len() as i128))
}

pub(super) fn variance(xs: &[Rational]) -> Result<Rational, String> {
    if xs.is_empty() {
        return Err("variance of empty data is undefined".into());
    }
    let mean = mean(xs)?;
    let mut sum = Rational::integer(0);
    for x in xs {
        let d = x.sub(&mean)?;
        sum = sum.add(&d.mul(&d)?)?;
    }
    sum.div(&Rational::integer(xs.len() as i128))
}
