#[cfg(any(feature = "approx", test))]
use approx::AbsDiffEq;
use num_traits::{One, Zero};
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

#[derive(Eq, PartialEq, PartialOrd, Clone, Copy, Debug, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NoValue;

impl Add for NoValue {
    type Output = Self;

    fn add(self, _: Self) -> Self::Output {
        NoValue::default()
    }
}

impl<T> Div<T> for NoValue {
    type Output = Self;

    fn div(self, _: T) -> Self::Output {
        NoValue::default()
    }
}

impl<T> Mul<T> for NoValue {
    type Output = Self;

    fn mul(self, _: T) -> Self::Output {
        NoValue::default()
    }
}

impl Neg for NoValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        NoValue::default()
    }
}

impl<T> Rem<T> for NoValue {
    type Output = Self;

    fn rem(self, _: T) -> Self::Output {
        NoValue::default()
    }
}

impl Sub for NoValue {
    type Output = Self;

    fn sub(self, _: Self) -> Self::Output {
        NoValue::default()
    }
}

/// This hack allows mathematical operations that result in noop due to above ops
impl Zero for NoValue {
    fn zero() -> Self {
        NoValue::default()
    }

    fn is_zero(&self) -> bool {
        true
    }
}

/// This hack allows mathematical operations that result in noop due to above ops
impl One for NoValue {
    fn one() -> Self {
        NoValue::default()
    }
}

#[cfg(any(feature = "approx", test))]
impl AbsDiffEq for NoValue {
    type Epsilon = Self;

    fn default_epsilon() -> Self::Epsilon {
        NoValue::default()
    }

    fn abs_diff_eq(&self, _: &Self, _: Self::Epsilon) -> bool {
        true
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for NoValue {
    fn arbitrary(_: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(NoValue::default())
    }
}
