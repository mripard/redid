use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, Euclid, FromPrimitive, Num};

fn round_up<T>(number: &T, multiple: &T) -> T
where
    T: Copy + Num + CheckedAdd + CheckedMul + Euclid + FromPrimitive,
{
    let rem = T::rem_euclid(number, multiple);

    if rem.is_zero() {
        return *number;
    }

    let div = T::checked_add(&T::div_euclid(number, multiple), &T::one())
        .expect("Addition would overflow");

    T::checked_mul(&div, multiple).expect("Multiplication would overflow")
}

pub(crate) fn div_round_up<T>(numerator: &T, denominator: &T) -> T
where
    T: Copy + Num + CheckedAdd + CheckedMul + CheckedDiv + Euclid + FromPrimitive,
{
    let rounded = round_up(numerator, denominator);

    T::checked_div(&rounded, denominator).expect("Division by zero or would overflow")
}
