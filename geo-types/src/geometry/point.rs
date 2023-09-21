use crate::{point, Coord, CoordFloat, CoordNum};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A single point in 2D space.
///
/// Points can be created using the [`Point::new`] constructor,
/// the [`point!`] macro, or from a `Coord`, two-element
/// tuples, or arrays – see the `From` impl section for a
/// complete list.
///
/// # Semantics
///
/// The _interior_ of the point is itself (a singleton set),
/// and its _boundary_ is empty. A point is _valid_ if and
/// only if the `Coord` is valid.
///
/// # Examples
///
/// ```
/// use geo_types::{coord, Point};
/// let p1: Point = (0., 1.).into();
/// let c = coord! { x: 10., y: 20. };
/// let p2: Point = c.into();
/// ```
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Point<T: CoordNum = f64>(pub Coord<T>);

impl<T: CoordNum> From<Coord<T>> for Point<T> {
    fn from(x: Coord<T>) -> Self {
        Point(x)
    }
}

impl<T: CoordNum> From<(T, T)> for Point<T> {
    fn from(coords: (T, T)) -> Self {
        Point::new(coords.0, coords.1)
    }
}

impl<T: CoordNum> From<[T; 2]> for Point<T> {
    fn from(coords: [T; 2]) -> Self {
        Point::new(coords[0], coords[1])
    }
}

impl<T: CoordNum> From<Point<T>> for (T, T) {
    fn from(point: Point<T>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum> From<Point<T>> for [T; 2] {
    fn from(point: Point<T>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum> Point<T> {
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
    pub fn new(x: T, y: T) -> Self {
        point! { x: x, y: y }
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
    pub fn set_x(&mut self, x: T) -> &mut Self {
        self.0.x = x;
        self
    }

    /// Returns a mutable reference to the x/horizontal component of the point
    ///
    /// # Examples
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo_types::Point;
    /// let mut p = Point::new(1.234, 2.345);
    /// let mut p_x = p.x_mut();
    /// *p_x += 1.0;
    /// assert_relative_eq!(p.x(), 2.234);
    /// ```
    pub fn x_mut(&mut self) -> &mut T {
        &mut self.0.x
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
    pub fn set_y(&mut self, y: T) -> &mut Self {
        self.0.y = y;
        self
    }

    /// Returns a mutable reference to the x/horizontal component of the point
    ///
    /// # Examples
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo_types::Point;
    /// let mut p = Point::new(1.234, 2.345);
    /// let mut p_y = p.y_mut();
    /// *p_y += 1.0;
    /// assert_relative_eq!(p.y(), 3.345);
    /// ```
    pub fn y_mut(&mut self) -> &mut T {
        &mut self.0.y
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
    /// assert_eq!(p.x(), 1.234);
    /// ```
    #[deprecated = "use `Point::x` instead, it's less ambiguous"]
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
    /// #[allow(deprecated)]
    /// p.set_lng(9.876);
    ///
    /// assert_eq!(p.x(), 9.876);
    /// ```
    #[deprecated = "use `Point::set_x` instead, it's less ambiguous"]
    pub fn set_lng(&mut self, lng: T) -> &mut Self {
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
    /// assert_eq!(p.y(), 2.345);
    /// ```
    #[deprecated = "use `Point::y` instead, it's less ambiguous"]
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
    /// #[allow(deprecated)]
    /// p.set_lat(9.876);
    ///
    /// assert_eq!(p.y(), 9.876);
    /// ```
    #[deprecated = "use `Point::set_y` instead, it's less ambiguous"]
    pub fn set_lat(&mut self, lat: T) -> &mut Self {
        self.set_y(lat)
    }
}

impl<T: CoordNum> Point<T> {
    /// Returns the dot product of the two points:
    /// `dot = x1 * x2 + y1 * y2`
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{point, Point};
    ///
    /// let point = point! { x: 1.5, y: 0.5 };
    /// let dot = point.dot(point! { x: 2.0, y: 4.5 });
    ///
    /// assert_eq!(dot, 5.25);
    /// ```
    pub fn dot(self, other: Self) -> T {
        self.x() * other.x() + self.y() * other.y()
    }

    /// Returns the cross product of 3 points. A positive value implies
    /// `self` → `point_b` → `point_c` is counter-clockwise, negative implies
    /// clockwise.
    ///
    /// # Note on Robustness
    ///
    /// This function is **not** robust against floating-point errors.
    /// The [`geo`](https://docs.rs/geo) crate
    /// offers robust predicates for standard numeric types using the
    /// [`Kernel`](https://docs.rs/geo/algorithm/kernels/trait.Kernel.html)
    /// trait, and these should be preferred if possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::point;
    ///
    /// let point_a = point! { x: 1., y: 2. };
    /// let point_b = point! { x: 3., y: 5. };
    /// let point_c = point! { x: 7., y: 12. };
    ///
    /// let cross = point_a.cross_prod(point_b, point_c);
    ///
    /// assert_eq!(cross, 2.0)
    /// ```
    pub fn cross_prod(self, point_b: Self, point_c: Self) -> T {
        (point_b.x() - self.x()) * (point_c.y() - self.y())
            - (point_b.y() - self.y()) * (point_c.x() - self.x())
    }
}

impl<T: CoordFloat> Point<T> {
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
    pub fn to_degrees(self) -> Self {
        let (x, y) = self.x_y();
        let x = x.to_degrees();
        let y = y.to_degrees();
        Point::new(x, y)
    }
}

impl<T> Neg for Point<T>
where
    T: CoordNum + Neg<Output = T>,
{
    type Output = Self;

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
    fn neg(self) -> Self::Output {
        Point::from(-self.0)
    }
}

impl<T: CoordNum> Add for Point<T> {
    type Output = Self;

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
    fn add(self, rhs: Self) -> Self::Output {
        Point::from(self.0 + rhs.0)
    }
}

impl<T: CoordNum> AddAssign for Point<T> {
    /// Add a point to the given point and assign it to the original point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.25, 2.5);
    /// p += Point::new(1.5, 2.5);
    ///
    /// assert_eq!(p.x(), 2.75);
    /// assert_eq!(p.y(), 5.0);
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl<T: CoordNum> Sub for Point<T> {
    type Output = Self;

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
    fn sub(self, rhs: Self) -> Self::Output {
        Point::from(self.0 - rhs.0)
    }
}

impl<T: CoordNum> SubAssign for Point<T> {
    /// Subtract a point from the given point and assign it to the original point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.25, 2.5);
    /// p -= Point::new(1.5, 2.5);
    ///
    /// assert_eq!(p.x(), -0.25);
    /// assert_eq!(p.y(), 0.0);
    /// ```
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0 - rhs.0;
    }
}

impl<T: CoordNum> Mul<T> for Point<T> {
    type Output = Self;

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
    fn mul(self, rhs: T) -> Self::Output {
        Point::from(self.0 * rhs)
    }
}

impl<T: CoordNum> MulAssign<T> for Point<T> {
    /// Scaler multiplication of a point in place
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(2.0, 3.0);
    /// p *= 2.0;
    ///
    /// assert_eq!(p.x(), 4.0);
    /// assert_eq!(p.y(), 6.0);
    /// ```
    fn mul_assign(&mut self, rhs: T) {
        self.0 = self.0 * rhs
    }
}

impl<T: CoordNum> Div<T> for Point<T> {
    type Output = Self;

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
    fn div(self, rhs: T) -> Self::Output {
        Point::from(self.0 / rhs)
    }
}

impl<T: CoordNum> DivAssign<T> for Point<T> {
    /// Scaler division of a point in place
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(2.0, 3.0);
    /// p /= 2.0;
    ///
    /// assert_eq!(p.x(), 1.0);
    /// assert_eq!(p.y(), 1.5);
    /// ```
    fn div_assign(&mut self, rhs: T) {
        self.0 = self.0 / rhs
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

    /// Equality assertion with an absolute limit.
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

#[cfg(feature = "rstar_0_8")]
// These are required for rstar RTree
impl<T> ::rstar_0_8::Point for Point<T>
where
    T: ::num_traits::Float + ::rstar_0_8::RTreeNum,
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

#[cfg(feature = "rstar_0_9")]
impl<T> ::rstar_0_9::Point for Point<T>
where
    T: ::num_traits::Float + ::rstar_0_9::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
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

#[cfg(feature = "rstar_0_10")]
impl<T> ::rstar_0_10::Point for Point<T>
where
    T: ::num_traits::Float + ::rstar_0_10::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
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

#[cfg(feature = "rstar_0_11")]
impl<T> ::rstar_0_11::Point for Point<T>
where
    T: ::num_traits::Float + ::rstar_0_11::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
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
