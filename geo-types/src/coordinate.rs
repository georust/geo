use crate::{coord, CoordNum, NoValue, Point};
#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use num_traits::Zero;
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// A generic struct used to store a single coordinate with optional
/// 3D (Z) and measurement support.
///
/// Unlike [`Point`] (which in the future may contain additional information such
/// as an envelope, a precision model, and spatial reference system
/// information), a `Coordinate` only contains ordinate values and accessor
/// methods.
///
/// This type implements the [vector space] operations:
/// [`Add`], [`Sub`], [`Neg`], [`Zero`],
/// [`Mul<T>`][`Mul`], and [`Div<T>`][`Div`] traits.
///
/// # Semantics
///
/// This type does not represent any geospatial primitive,
/// but is used in their definitions. The only requirement
/// is that the coordinates it contains are valid numbers
/// (for eg. not `f64::NAN`).
///
/// [vector space]: //en.wikipedia.org/wiki/Vector_space
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coordinate<T: CoordNum, Z: CoordNum = NoValue, M: CoordNum = NoValue> {
    pub x: T,
    pub y: T,
    pub z: Z,
    pub m: M,
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> Coordinate<T, Z, M> {
    /// Create a new instance of a coordinate.
    /// **ATTENTION** Use [`coord!`] macro or one of the [`from`] methods instead.
    #[inline]
    #[doc(hidden)]
    pub fn new__(x: T, y: T, z: Z, m: M) -> Self {
        Self { x, y, z, m }
    }
}

/// A lightweight struct used to store coordinates on the 2-dimensional
/// Cartesian plane together with a Measure value of the same type.
///
/// See also [Coordinate]
pub type CoordinateM<T, M = T> = Coordinate<T, NoValue, M>;

/// A lightweight struct used to store coordinates on the 3-dimensional
/// Cartesian plane.
///
/// See also [Coordinate]
pub type Coordinate3D<T> = Coordinate<T, T, NoValue>;

/// A lightweight struct used to store coordinates on the 3-dimensional
/// Cartesian plane together with a Measure value of the same type.
///
/// See also [Coordinate]
pub type Coordinate3DM<T, M = T> = Coordinate<T, T, M>;

impl<T: CoordNum> From<(T, T)> for Coordinate<T> {
    #[inline]
    fn from(coords: (T, T)) -> Self {
        coord! {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl<T: CoordNum> From<[T; 2]> for Coordinate<T> {
    #[inline]
    fn from(coords: [T; 2]) -> Self {
        coord! {
            x: coords[0],
            y: coords[1],
        }
    }
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<Point<T, Z, M>> for Coordinate<T, Z, M> {
    #[inline]
    fn from(point: Point<T, Z, M>) -> Self {
        point.0
    }
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<Coordinate<T, Z, M>> for (T, T) {
    #[inline]
    fn from(coord: Coordinate<T, Z, M>) -> Self {
        (coord.x, coord.y)
    }
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<Coordinate<T, Z, M>> for [T; 2] {
    #[inline]
    fn from(coord: Coordinate<T, Z, M>) -> Self {
        [coord.x, coord.y]
    }
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> Coordinate<T, Z, M> {
    /// Returns a tuple that contains the x/horizontal & y/vertical component of the coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::coord;
    ///
    /// let c = coord! {
    ///     x: 40.02f64,
    ///     y: 116.34,
    /// };
    /// let (x, y) = c.x_y();
    ///
    /// assert_eq!(y, 116.34);
    /// assert_eq!(x, 40.02f64);
    /// ```
    #[inline]
    pub fn x_y(&self) -> (T, T) {
        (self.x, self.y)
    }
}

/// Negate a coordinate.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.25, y: 2.5 };
/// let q = -p;
///
/// assert_eq!(q.x, -p.x);
/// assert_eq!(q.y, -p.y);
/// ```
impl<T, Z, M> Neg for Coordinate<T, Z, M>
where
    T: CoordNum + Neg<Output = T>,
    Z: CoordNum + Neg<Output = Z>,
    M: CoordNum + Neg<Output = M>,
{
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        coord! {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            m: -self.m,
        }
    }
}

/// Add two coordinates.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.25, y: 2.5 };
/// let q = coord! { x: 1.5, y: 2.5 };
/// let sum = p + q;
///
/// assert_eq!(sum.x, 2.75);
/// assert_eq!(sum.y, 5.0);
/// ```
impl<T: CoordNum, Z: CoordNum, M: CoordNum> Add for Coordinate<T, Z, M> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        coord! {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            m: self.m + rhs.m,
        }
    }
}

/// Subtract a coordinate from another.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.5, y: 2.5 };
/// let q = coord! { x: 1.25, y: 2.5 };
/// let diff = p - q;
///
/// assert_eq!(diff.x, 0.25);
/// assert_eq!(diff.y, 0.);
/// ```
impl<T: CoordNum, Z: CoordNum, M: CoordNum> Sub for Coordinate<T, Z, M> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        coord! {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            m: self.m - rhs.m,
        }
    }
}

/// Multiply coordinate wise by a scalar.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.25, y: 2.5 };
/// let q = p * 4.;
///
/// assert_eq!(q.x, 5.0);
/// assert_eq!(q.y, 10.0);
/// ```
impl<T, Z, M> Mul<T> for Coordinate<T, Z, M>
where
    T: CoordNum,
    Z: CoordNum + Mul<T, Output = Z>,
    M: CoordNum + Mul<T, Output = M>,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: T) -> Self {
        coord! {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            m: self.m * rhs,
        }
    }
}

/// Divide coordinate wise by a scalar.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 5., y: 10. };
/// let q = p / 4.;
///
/// assert_eq!(q.x, 1.25);
/// assert_eq!(q.y, 2.5);
/// ```
impl<T, Z, M> Div<T> for Coordinate<T, Z, M>
where
    T: CoordNum,
    Z: CoordNum + Div<T, Output = Z>,
    M: CoordNum + Div<T, Output = M>,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: T) -> Self {
        coord! {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            m: self.m / rhs,
        }
    }
}

/// Create a coordinate at the origin.
///
/// # Examples
///
/// ```
/// use geo_types::Coordinate;
/// use num_traits::Zero;
///
/// let p: Coordinate<f64> = Zero::zero();
///
/// assert_eq!(p.x, 0.);
/// assert_eq!(p.y, 0.);
/// ```
impl<T: CoordNum, Z: CoordNum, M: CoordNum> Coordinate<T, Z, M> {
    #[inline]
    pub fn zero() -> Self {
        coord! {
            x: T::zero(),
            y: T::zero(),
            z: Z::zero(),
            m: M::zero(),
        }
    }
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> Zero for Coordinate<T, Z, M> {
    #[inline]
    fn zero() -> Self {
        Self::zero()
    }
    #[inline]
    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero() && self.z.is_zero() && self.m.is_zero()
    }
}

#[cfg(any(feature = "approx", test))]
impl<T: CoordNum + AbsDiffEq> AbsDiffEq for Coordinate<T>
where
    T::Epsilon: Copy,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> T::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: T::Epsilon) -> bool {
        T::abs_diff_eq(&self.x, &other.x, epsilon) && T::abs_diff_eq(&self.y, &other.y, epsilon)
    }
}

#[cfg(any(feature = "approx", test))]
impl<T: CoordNum + RelativeEq> RelativeEq for Coordinate<T>
where
    T::Epsilon: Copy,
{
    #[inline]
    fn default_max_relative() -> T::Epsilon {
        T::default_max_relative()
    }

    #[inline]
    fn relative_eq(&self, other: &Self, epsilon: T::Epsilon, max_relative: T::Epsilon) -> bool {
        T::relative_eq(&self.x, &other.x, epsilon, max_relative)
            && T::relative_eq(&self.y, &other.y, epsilon, max_relative)
    }
}

#[cfg(any(feature = "approx", test))]
impl<T: CoordNum + UlpsEq> UlpsEq for Coordinate<T>
where
    T::Epsilon: Copy,
{
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    #[inline]
    fn ulps_eq(&self, other: &Self, epsilon: T::Epsilon, max_ulps: u32) -> bool {
        T::ulps_eq(&self.x, &other.x, epsilon, max_ulps)
            && T::ulps_eq(&self.y, &other.y, epsilon, max_ulps)
    }
}

#[cfg(feature = "rstar_0_8")]
impl<T> ::rstar_0_8::Point for Coordinate<T>
where
    T: ::num_traits::Float + ::rstar_0_8::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    #[inline]
    fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
        coord! {
            x: generator(0),
            y: generator(1),
        }
    }

    #[inline]
    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "rstar_0_9")]
impl<T> ::rstar_0_9::Point for Coordinate<T>
where
    T: ::num_traits::Float + ::rstar_0_9::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    #[inline]
    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        coord! {
            x: generator(0),
            y: generator(1),
        }
    }

    #[inline]
    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_coordinates() {
        let p = coord! { x: 1.0, y: 2.0 };
        assert_relative_eq!(p.x, 1.0);
        assert_relative_eq!(p.y, 2.0);
        assert_eq!(p.z, NoValue);
        assert_eq!(p.m, NoValue);

        let p = coord! { x: 1.0, y: 2.0, z: 3.0 };
        assert_relative_eq!(p.x, 1.0);
        assert_relative_eq!(p.y, 2.0);
        assert_relative_eq!(p.z, 3.0);
        assert_eq!(p.m, NoValue);

        let p = coord! { x: 1.0, y: 2.0, m: 4_u8 };
        assert_relative_eq!(p.x, 1.0);
        assert_relative_eq!(p.y, 2.0);
        assert_eq!(p.z, NoValue);
        assert_eq!(p.m, 4_u8);

        let p = coord! { x: 1_i32, y: 2_i32, z: 3_i32, m: 4.0_f64 };
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
        assert_eq!(p.z, 3);
        assert_relative_eq!(p.m, 4.0);
    }
}
