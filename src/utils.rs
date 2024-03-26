use num_traits::{CheckedAdd, CheckedMul, Euclid, FromPrimitive, Num};

pub(crate) fn round_up<T>(number: &T, multiple: &T) -> T
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
