use crate::{polygon, CoordNum, Coordinate, Line, Polygon};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

/// A bounded 2D area whose three vertices are defined by
/// `Coordinate`s. The semantics and validity are that of
/// the equivalent [`Polygon`]; in addition, the three
/// vertices must not be collinear and they must be distinct.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Triangle<T: CoordNum = f64>(pub Coordinate<T>, pub Coordinate<T>, pub Coordinate<T>);

impl<T: CoordNum> Triangle<T> {
    /// Instantiate Self from the raw content value
    pub fn new(v1: Coordinate<T>, v2: Coordinate<T>, v3: Coordinate<T>) -> Self {
        Self(v1, v2, v3)
    }

    pub fn to_array(&self) -> [Coordinate<T>; 3] {
        [self.0, self.1, self.2]
    }

    pub fn to_lines(&self) -> [Line<T>; 3] {
        [
            Line::new(self.0, self.1),
            Line::new(self.1, self.2),
            Line::new(self.2, self.0),
        ]
    }

    /// Create a `Polygon` from the `Triangle`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{coord, Triangle, polygon};
    ///
    /// let triangle = Triangle::new(
    ///     coord! { x: 0., y: 0. },
    ///     coord! { x: 10., y: 20. },
    ///     coord! { x: 20., y: -10. },
    /// );
    ///
    /// assert_eq!(
    ///     triangle.to_polygon(),
    ///     polygon![
    ///         (x: 0., y: 0.),
    ///         (x: 10., y: 20.),
    ///         (x: 20., y: -10.),
    ///         (x: 0., y: 0.),
    ///     ],
    /// );
    /// ```
    pub fn to_polygon(self) -> Polygon<T> {
        polygon![self.0, self.1, self.2, self.0]
    }
}

impl<IC: Into<Coordinate<T>> + Copy, T: CoordNum> From<[IC; 3]> for Triangle<T> {
    fn from(array: [IC; 3]) -> Self {
        Self(array[0].into(), array[1].into(), array[2].into())
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for Triangle<T>
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
    /// use geo_types::{point, Triangle};
    ///
    /// let a = Triangle::new((0.0, 0.0).into(), (10.0, 10.0).into(), (0.0, 5.0).into());
    /// let b = Triangle::new((0.0, 0.0).into(), (10.01, 10.0).into(), (0.0, 5.0).into());
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1);
    /// approx::assert_relative_ne!(a, b, max_relative=0.0001);
    /// ```
    #[inline]
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if !self.0.relative_eq(&other.0, epsilon, max_relative) {
            return false;
        }
        if !self.1.relative_eq(&other.1, epsilon, max_relative) {
            return false;
        }
        if !self.2.relative_eq(&other.2, epsilon, max_relative) {
            return false;
        }

        true
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> AbsDiffEq for Triangle<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    T::Epsilon: Copy,
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    /// Equality assertion with an absolute limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{point, Triangle};
    ///
    /// let a = Triangle::new((0.0, 0.0).into(), (10.0, 10.0).into(), (0.0, 5.0).into());
    /// let b = Triangle::new((0.0, 0.0).into(), (10.01, 10.0).into(), (0.0, 5.0).into());
    ///
    /// approx::abs_diff_eq!(a, b, epsilon=0.1);
    /// approx::abs_diff_ne!(a, b, epsilon=0.001);
    /// ```
    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if !self.0.abs_diff_eq(&other.0, epsilon) {
            return false;
        }
        if !self.1.abs_diff_eq(&other.1, epsilon) {
            return false;
        }
        if !self.2.abs_diff_eq(&other.2, epsilon) {
            return false;
        }

        true
    }
}
