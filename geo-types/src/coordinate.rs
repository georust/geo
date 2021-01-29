use crate::{CoordNum, Point};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

/// A lightweight struct used to store coordinates on the 2-dimensional
/// Cartesian plane.
///
/// Unlike `Point` (which in the future may contain additional information such
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
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coordinate<T>
where
    T: CoordNum,
{
    pub x: T,
    pub y: T,
}

impl<T: Default + CoordNum> Default for Coordinate<T> {
    fn default() -> Coordinate<T> {
        Coordinate {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T: CoordNum> From<(T, T)> for Coordinate<T> {
    fn from(coords: (T, T)) -> Self {
        Coordinate {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl<T: CoordNum> From<[T; 2]> for Coordinate<T> {
    fn from(coords: [T; 2]) -> Self {
        Coordinate {
            x: coords[0],
            y: coords[1],
        }
    }
}

impl<T: CoordNum> From<Point<T>> for Coordinate<T> {
    fn from(point: Point<T>) -> Self {
        Coordinate {
            x: point.x(),
            y: point.y(),
        }
    }
}

impl<T> Coordinate<T>
where
    T: CoordNum,
{
    /// Returns a tuple that contains the x/horizontal & y/vertical component of the coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Coordinate;
    ///
    /// let c = Coordinate {
    ///     x: 40.02f64,
    ///     y: 116.34,
    /// };
    /// let (x, y) = c.x_y();
    ///
    /// assert_eq!(y, 116.34);
    /// assert_eq!(x, 40.02f64);
    /// ```
    pub fn x_y(&self) -> (T, T) {
        (self.x, self.y)
    }
}

use std::ops::{Add, Div, Mul, Neg, Sub};

/// Negate a coordinate.
///
/// # Examples
///
/// ```
/// use geo_types::Coordinate;
///
/// let p: Coordinate<_> = (1.25, 2.5).into();
/// let q = -p;
///
/// assert_eq!(q.x, -p.x);
/// assert_eq!(q.y, -p.y);
/// ```
impl<T> Neg for Coordinate<T>
where
    T: CoordNum + Neg<Output = T>,
{
    type Output = Coordinate<T>;

    fn neg(self) -> Coordinate<T> {
        (-self.x, -self.y).into()
    }
}

/// Add two coordinates.
///
/// # Examples
///
/// ```
/// use geo_types::Coordinate;
///
/// let p: Coordinate<_> = (1.25, 2.5).into();
/// let q: Coordinate<_> = (1.5, 2.5).into();
/// let sum = p + q;
///
/// assert_eq!(sum.x, 2.75);
/// assert_eq!(sum.y, 5.0);
/// ```
impl<T> Add for Coordinate<T>
where
    T: CoordNum,
{
    type Output = Coordinate<T>;

    fn add(self, rhs: Coordinate<T>) -> Coordinate<T> {
        (self.x + rhs.x, self.y + rhs.y).into()
    }
}

/// Subtract a coordinate from another.
///
/// # Examples
///
/// ```
/// use geo_types::Coordinate;
///
/// let p: Coordinate<_> = (1.5, 2.5).into();
/// let q: Coordinate<_> = (1.25, 2.5).into();
/// let diff = p - q;
///
/// assert_eq!(diff.x, 0.25);
/// assert_eq!(diff.y, 0.);
/// ```
impl<T> Sub for Coordinate<T>
where
    T: CoordNum,
{
    type Output = Coordinate<T>;

    fn sub(self, rhs: Coordinate<T>) -> Coordinate<T> {
        (self.x - rhs.x, self.y - rhs.y).into()
    }
}

/// Multiply coordinate wise by a scalar.
///
/// # Examples
///
/// ```
/// use geo_types::Coordinate;
///
/// let p: Coordinate<_> = (1.25, 2.5).into();
/// let q: Coordinate<_> = p * 4.;
///
/// assert_eq!(q.x, 5.0);
/// assert_eq!(q.y, 10.0);
/// ```
impl<T> Mul<T> for Coordinate<T>
where
    T: CoordNum,
{
    type Output = Coordinate<T>;

    fn mul(self, rhs: T) -> Coordinate<T> {
        (self.x * rhs, self.y * rhs).into()
    }
}

/// Divide coordinate wise by a scalar.
///
/// # Examples
///
/// ```
/// use geo_types::Coordinate;
///
/// let p: Coordinate<_> = (5., 10.).into();
/// let q: Coordinate<_> = p / 4.;
///
/// assert_eq!(q.x, 1.25);
/// assert_eq!(q.y, 2.5);
/// ```
impl<T> Div<T> for Coordinate<T>
where
    T: CoordNum,
{
    type Output = Coordinate<T>;

    fn div(self, rhs: T) -> Coordinate<T> {
        (self.x / rhs, self.y / rhs).into()
    }
}

use num_traits::Zero;
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
impl<T: CoordNum> Coordinate<T> {
    pub fn zero() -> Self {
        Coordinate {
            x: T::zero(),
            y: T::zero(),
        }
    }
}

impl<T: CoordNum> Zero for Coordinate<T> {
    fn zero() -> Self {
        Coordinate::zero()
    }
    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
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
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: T::Epsilon, max_ulps: u32) -> bool {
        T::ulps_eq(&self.x, &other.x, epsilon, max_ulps)
            && T::ulps_eq(&self.y, &other.y, epsilon, max_ulps)
    }
}

#[cfg(feature = "rstar")]
impl<T> ::rstar::Point for Coordinate<T>
where
    T: ::num_traits::Float + ::rstar::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
        Coordinate {
            x: generator(0),
            y: generator(1),
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}
