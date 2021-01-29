use crate::{CoordFloat, CoordNum, Coordinate};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

use std::ops::{Add, Div, Mul, Neg, Sub};

/// A single point in 2D space.
///
/// Points can be created using the [`Point::new`] constructor,
/// the [`point!`] macro, or from a `Coordinate`, two-element
/// tuples, or arrays – see the `From` impl section for a
/// complete list.
///
/// # Semantics
///
/// The _interior_ of the point is itself (a singleton set),
/// and its _boundary_ is empty. A point is _valid_ if and
/// only if the `Coordinate` is valid.
///
/// # Examples
///
/// ```
/// use geo_types::{Coordinate, Point};
/// let p1: Point<f64> = (0., 1.).into();
/// let c = Coordinate { x: 10., y: 20. };
/// let p2: Point<f64> = c.into();
/// ```
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Point<T>(pub Coordinate<T>)
where
    T: CoordNum;

impl<T: CoordNum> From<Coordinate<T>> for Point<T> {
    fn from(x: Coordinate<T>) -> Point<T> {
        Point(x)
    }
}

impl<T: CoordNum> From<(T, T)> for Point<T> {
    fn from(coords: (T, T)) -> Point<T> {
        Point::new(coords.0, coords.1)
    }
}

impl<T: CoordNum> From<[T; 2]> for Point<T> {
    fn from(coords: [T; 2]) -> Point<T> {
        Point::new(coords[0], coords[1])
    }
}

impl<T> Point<T>
where
    T: CoordNum,
{
    /// Creates a new point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.x(), 1.234);
    /// assert_eq!(p.y(), 2.345);
    /// ```
    pub fn new(x: T, y: T) -> Point<T> {
        Point(Coordinate { x, y })
    }

    /// Returns the x/horizontal component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.x(), 1.234);
    /// ```
    pub fn x(self) -> T {
        self.0.x
    }

    /// Sets the x/horizontal component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// p.set_x(9.876);
    ///
    /// assert_eq!(p.x(), 9.876);
    /// ```
    pub fn set_x(&mut self, x: T) -> &mut Point<T> {
        self.0.x = x;
        self
    }

    /// Returns the y/vertical component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.y(), 2.345);
    /// ```
    pub fn y(self) -> T {
        self.0.y
    }

    /// Sets the y/vertical component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// p.set_y(9.876);
    ///
    /// assert_eq!(p.y(), 9.876);
    /// ```
    pub fn set_y(&mut self, y: T) -> &mut Point<T> {
        self.0.y = y;
        self
    }

    /// Returns a tuple that contains the x/horizontal & y/vertical component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// let (x, y) = p.x_y();
    ///
    /// assert_eq!(y, 2.345);
    /// assert_eq!(x, 1.234);
    /// ```
    pub fn x_y(self) -> (T, T) {
        (self.0.x, self.0.y)
    }
    /// Returns the longitude/horizontal component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.lng(), 1.234);
    /// ```
    pub fn lng(self) -> T {
        self.x()
    }

    /// Sets the longitude/horizontal component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// p.set_lng(9.876);
    ///
    /// assert_eq!(p.lng(), 9.876);
    /// ```
    pub fn set_lng(&mut self, lng: T) -> &mut Point<T> {
        self.set_x(lng)
    }

    /// Returns the latitude/vertical component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.lat(), 2.345);
    /// ```
    pub fn lat(self) -> T {
        self.y()
    }
    /// Sets the latitude/vertical component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// p.set_lat(9.876);
    ///
    /// assert_eq!(p.lat(), 9.876);
    /// ```
    pub fn set_lat(&mut self, lat: T) -> &mut Point<T> {
        self.set_y(lat)
    }
}

impl<T> Point<T>
where
    T: CoordNum,
{
    /// Returns the dot product of the two points:
    /// `dot = x1 * x2 + y1 * y2`
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, Point};
    ///
    /// let point = Point(Coordinate { x: 1.5, y: 0.5 });
    /// let dot = point.dot(Point(Coordinate { x: 2.0, y: 4.5 }));
    ///
    /// assert_eq!(dot, 5.25);
    /// ```
    pub fn dot(self, other: Point<T>) -> T {
        self.x() * other.x() + self.y() * other.y()
    }

    /// Returns the cross product of 3 points. A positive value implies
    /// `self` → `point_b` → `point_c` is counter-clockwise, negative implies
    /// clockwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, Point};
    ///
    /// let point_a = Point(Coordinate { x: 1., y: 2. });
    /// let point_b = Point(Coordinate { x: 3., y: 5. });
    /// let point_c = Point(Coordinate { x: 7., y: 12. });
    ///
    /// let cross = point_a.cross_prod(point_b, point_c);
    ///
    /// assert_eq!(cross, 2.0)
    /// ```
    pub fn cross_prod(self, point_b: Point<T>, point_c: Point<T>) -> T {
        (point_b.x() - self.x()) * (point_c.y() - self.y())
            - (point_b.y() - self.y()) * (point_c.x() - self.x())
    }
}

impl<T> Point<T>
where
    T: CoordFloat,
{
    /// Converts the (x,y) components of Point to degrees
    ///
    /// # Example
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    /// let (x, y): (f32, f32) = p.to_degrees().x_y();
    /// assert_eq!(x.round(), 71.0);
    /// assert_eq!(y.round(), 134.0);
    /// ```
    pub fn to_degrees(self) -> Point<T> {
        let (x, y) = self.x_y();
        let x = x.to_degrees();
        let y = y.to_degrees();
        Point::new(x, y)
    }

    /// Converts the (x,y) components of Point to radians
    ///
    /// # Example
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(180.0, 341.5);
    /// let (x, y): (f32, f32) = p.to_radians().x_y();
    /// assert_eq!(x.round(), 3.0);
    /// assert_eq!(y.round(), 6.0);
    /// ```
    pub fn to_radians(self) -> Point<T> {
        let (x, y) = self.x_y();
        let x = x.to_radians();
        let y = y.to_radians();
        Point::new(x, y)
    }
}

impl<T> Neg for Point<T>
where
    T: CoordNum + Neg<Output = T>,
{
    type Output = Point<T>;

    /// Returns a point with the x and y components negated.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = -Point::new(-1.25, 2.5);
    ///
    /// assert_eq!(p.x(), 1.25);
    /// assert_eq!(p.y(), -2.5);
    /// ```
    fn neg(self) -> Point<T> {
        Point(-self.0)
    }
}

impl<T> Add for Point<T>
where
    T: CoordNum,
{
    type Output = Point<T>;

    /// Add a point to the given point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.25, 2.5) + Point::new(1.5, 2.5);
    ///
    /// assert_eq!(p.x(), 2.75);
    /// assert_eq!(p.y(), 5.0);
    /// ```
    fn add(self, rhs: Point<T>) -> Point<T> {
        Point(self.0 + rhs.0)
    }
}

impl<T> Sub for Point<T>
where
    T: CoordNum,
{
    type Output = Point<T>;

    /// Subtract a point from the given point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.25, 3.0) - Point::new(1.5, 2.5);
    ///
    /// assert_eq!(p.x(), -0.25);
    /// assert_eq!(p.y(), 0.5);
    /// ```
    fn sub(self, rhs: Point<T>) -> Point<T> {
        Point(self.0 - rhs.0)
    }
}

impl<T> Mul<T> for Point<T>
where
    T: CoordNum,
{
    type Output = Point<T>;

    /// Scaler multiplication of a point
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(2.0, 3.0) * 2.0;
    ///
    /// assert_eq!(p.x(), 4.0);
    /// assert_eq!(p.y(), 6.0);
    /// ```
    fn mul(self, rhs: T) -> Point<T> {
        Point(self.0 * rhs)
    }
}

impl<T> Div<T> for Point<T>
where
    T: CoordNum,
{
    type Output = Point<T>;

    /// Scaler division of a point
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(2.0, 3.0) / 2.0;
    ///
    /// assert_eq!(p.x(), 1.0);
    /// assert_eq!(p.y(), 1.5);
    /// ```
    fn div(self, rhs: T) -> Point<T> {
        Point(self.0 / rhs)
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for Point<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    /// Equality assertion within a relative limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let a = Point::new(2.0, 3.0);
    /// let b = Point::new(2.0, 3.01);
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1)
    /// ```
    #[inline]
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.0.relative_eq(&other.0, epsilon, max_relative)
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> AbsDiffEq for Point<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    T::Epsilon: Copy,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    /// Equality assertion with a absolute limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let a = Point::new(2.0, 3.0);
    /// let b = Point::new(2.0, 3.0000001);
    ///
    /// approx::assert_relative_eq!(a, b, epsilon=0.1)
    /// ```
    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.0.abs_diff_eq(&other.0, epsilon)
    }
}

#[cfg(feature = "rstar")]
// These are required for rstar RTree
impl<T> ::rstar::Point for Point<T>
where
    T: ::num_traits::Float + ::rstar::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
        Point::new(generator(0), generator(1))
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.0.x,
            1 => self.0.y,
            _ => unreachable!(),
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.0.x,
            1 => &mut self.0.y,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use approx::AbsDiffEq;

    #[test]
    fn test_abs_diff_eq() {
        let delta = 1e-6;
        let p = Point::new(1.0, 1.0);

        let p_x = Point::new(1.0 - delta, 1.0);
        assert!(p.abs_diff_eq(&p_x, 1e-2));
        assert!(p.abs_diff_ne(&p_x, 1e-12));

        let p_y = Point::new(1.0, 1.0 + delta);
        assert!(p.abs_diff_eq(&p_y, 1e-2));
        assert!(p.abs_diff_ne(&p_y, 1e-12));

        let p_xy = Point::new(1.0 + delta, 1.0 - delta);
        assert!(p.abs_diff_eq(&p_xy, 1e-2));
        assert!(p.abs_diff_ne(&p_xy, 1e-12));

        let p_inf = Point::new(f64::INFINITY, 1.);
        assert!(p.abs_diff_ne(&p_inf, 1e-2));
    }

    #[test]
    fn test_relative_eq() {
        let delta = 1e-6;
        let p = Point::new(1.0, 1.0);

        let p_x = Point::new(1.0 - delta, 1.0);
        assert!(p.relative_eq(&p_x, 1e-2, 1e-2));
        assert!(p.relative_ne(&p_x, 1e-12, 1e-12));

        let p_y = Point::new(1.0, 1.0 + delta);
        assert!(p.relative_eq(&p_y, 1e-2, 1e-2));
        assert!(p.relative_ne(&p_y, 1e-12, 1e-12));

        let p_xy = Point::new(1.0 + delta, 1.0 - delta);
        assert!(p.relative_eq(&p_xy, 1e-2, 1e-2));
        assert!(p.relative_ne(&p_xy, 1e-12, 1e-12));

        let p_inf = Point::new(f64::INFINITY, 1.);
        assert!(p.relative_ne(&p_inf, 1e-2, 1e-2));
    }
}
