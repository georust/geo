use crate::{CoordinateType, Point};
#[cfg(test)]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

/// A lightweight struct used to store coordinates on the 2-dimensional
/// Cartesian plane.
///
/// Unlike `Point` (which in the future may contain additional information such
/// as an envelope, a precision model, and spatial reference system
/// information), a `Coordinate` only contains ordinate values and accessor
/// methods.
///
/// This type obeys the typical [vector space] structure:
/// implements the [`Add`], [`Sub`], [`Neg`], [`Zero`]
/// traits and allows [`scaling`][`Coordinate::scale_by`] by
/// a scalar.
///
/// [vector space]: //en.wikipedia.org/wiki/Vector_space
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coordinate<T>
where
    T: CoordinateType,
{
    pub x: T,
    pub y: T,
}

impl<T: CoordinateType> From<(T, T)> for Coordinate<T> {
    fn from(coords: (T, T)) -> Self {
        Coordinate {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl<T: CoordinateType> From<[T; 2]> for Coordinate<T> {
    fn from(coords: [T; 2]) -> Self {
        Coordinate {
            x: coords[0],
            y: coords[1],
        }
    }
}

impl<T: CoordinateType> From<Point<T>> for Coordinate<T> {
    fn from(point: Point<T>) -> Self {
        Coordinate {
            x: point.x(),
            y: point.y(),
        }
    }
}

impl<T> Coordinate<T>
where
    T: CoordinateType,
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

use std::ops::{Add, Neg, Sub, Mul, Div};
impl<T> Neg for Coordinate<T>
where
    T: CoordinateType + Neg<Output = T>,
{
    type Output = Coordinate<T>;

    /// Returns a coordinate with the x and y components negated.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Coordinate;
    ///
    /// let p: Coordinate<_> = (-1.25, 2.5).into();
    /// let p = -p;
    ///
    /// assert_eq!(p.x, 1.25);
    /// assert_eq!(p.y, -2.5);
    /// ```
    fn neg(self) -> Coordinate<T> {
        (-self.x, -self.y).into()
    }
}

impl<T> Add for Coordinate<T>
where
    T: CoordinateType,
{
    type Output = Coordinate<T>;

    /// Add a point to the given point.
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
    fn add(self, rhs: Coordinate<T>) -> Coordinate<T> {
        (self.x + rhs.x, self.y + rhs.y).into()
    }
}

impl<T> Sub for Coordinate<T>
where
    T: CoordinateType,
{
    type Output = Coordinate<T>;

    /// Subtract a point from the given point.
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
    fn sub(self, rhs: Coordinate<T>) -> Coordinate<T> {
        (self.x - rhs.x, self.y - rhs.y).into()
    }
}

impl<T> Mul<T> for Coordinate<T>
where
    T: CoordinateType,
{
    type Output = Coordinate<T>;

    /// Add a point to the given point.
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
    fn mul(self, rhs: T) -> Coordinate<T> {
        (self.x * rhs, self.y * rhs).into()
    }
}

impl<T> Div<T> for Coordinate<T>
where
    T: CoordinateType,
{
    type Output = Coordinate<T>;

    /// Add a point to the given point.
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
    fn div(self, rhs: T) -> Coordinate<T> {
        (self.x / rhs, self.y / rhs).into()
    }
}

use num_traits::Zero;
impl<T: CoordinateType> Zero for Coordinate<T> {
    fn zero() -> Self {
        Coordinate {
            x: T::zero(),
            y: T::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }
}

#[cfg(test)]
impl<T: CoordinateType + AbsDiffEq> AbsDiffEq for Coordinate<T>
where
    T::Epsilon: Copy,
{
    type Epsilon = T::Epsilon;

    fn default_epsilon() -> T::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: T::Epsilon) -> bool {
        T::abs_diff_eq(&self.x, &other.x, epsilon) && T::abs_diff_eq(&self.y, &other.y, epsilon)
    }
}

#[cfg(test)]
impl<T: CoordinateType + RelativeEq> RelativeEq for Coordinate<T>
where
    T::Epsilon: Copy,
{
    fn default_max_relative() -> T::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(&self, other: &Self, epsilon: T::Epsilon, max_relative: T::Epsilon) -> bool {
        T::relative_eq(&self.x, &other.x, epsilon, max_relative)
            && T::relative_eq(&self.y, &other.y, epsilon, max_relative)
    }
}

#[cfg(test)]
impl<T: CoordinateType + UlpsEq> UlpsEq for Coordinate<T>
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
