#[cfg(any(feature = "approx", test))]
use approx::AbsDiffEq;
use num_traits::{Num, NumCast, One, ToPrimitive, Zero};
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

/// An empty placeholder type that can be used instead of the real
/// numerical value types for 3D (z) and measurement (m) values.
#[derive(Eq, PartialEq, PartialOrd, Clone, Copy, Debug, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NoValue;

impl Add for NoValue {
    type Output = Self;

    #[inline]
    fn add(self, _: Self) -> Self::Output {
        NoValue::default()
    }
}

impl<T> Div<T> for NoValue {
    type Output = Self;

    #[inline]
    fn div(self, _: T) -> Self::Output {
        NoValue::default()
    }
}

impl<T> Mul<T> for NoValue {
    type Output = Self;

    #[inline]
    fn mul(self, _: T) -> Self::Output {
        NoValue::default()
    }
}

impl Neg for NoValue {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        NoValue::default()
    }
}

impl<T> Rem<T> for NoValue {
    type Output = Self;

    #[inline]
    fn rem(self, _: T) -> Self::Output {
        NoValue::default()
    }
}

impl Sub for NoValue {
    type Output = Self;

    #[inline]
    fn sub(self, _: Self) -> Self::Output {
        NoValue::default()
    }
}

/// This hack allows mathematical operations that result in noop due to above ops
impl Zero for NoValue {
    #[inline]
    fn zero() -> Self {
        NoValue::default()
    }

    #[inline]
    fn is_zero(&self) -> bool {
        true
    }
}

/// Thhese hacks allows mathematical operations that result in noop due to above ops
impl One for NoValue {
    #[inline]
    fn one() -> Self {
        NoValue::default()
    }
}

impl ToPrimitive for NoValue {
    fn to_i64(&self) -> Option<i64> {
        Some(0)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(0)
    }
}

impl NumCast for NoValue {
    fn from<T: ToPrimitive>(_: T) -> Option<Self> {
        Some(Self::default())
    }
}

impl Num for NoValue {
    type FromStrRadixErr = ();

    fn from_str_radix(_str: &str, _radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::default())
    }
}

#[cfg(any(feature = "approx", test))]
impl AbsDiffEq for NoValue {
    type Epsilon = Self;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        NoValue::default()
    }

    #[inline]
    fn abs_diff_eq(&self, _: &Self, _: Self::Epsilon) -> bool {
        true
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for NoValue {
    #[inline]
    fn arbitrary(_: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(NoValue::default())
    }
}
