use crate::{polygon, CoordNum, Coordinate, Line, Polygon};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

/// A bounded 2D area whose three vertices are defined by
/// `Coordinate`s. The semantics and validity are that of
/// the equivalent [`Polygon`]; in addition, the three
/// vertices must not be collinear and they must be distinct.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Triangle<T: CoordNum>(
    #[deprecated(
        since = "0.7.5",
        note = "Direct field access is deprecated - use `triangle.vertex_0()` or `triangle.vertex_0_mut()` for field access and `Triangle::new(v0, v1, v2) for construction"
    )]
    pub Coordinate<T>,
    #[deprecated(
        since = "0.7.5",
        note = "Direct field access is deprecated - use `triangle.vertex_1()` or `triangle.vertex_1_mut()` for field access and `Triangle::new(v0, v1, v2) for construction"
    )]
    pub Coordinate<T>,
    #[deprecated(
        since = "0.7.5",
        note = "Direct field access is deprecated - use `triangle.vertex_2()` or `triangle.vertex_2_mut()` for field access and `Triangle::new(v0, v1, v2) for construction"
    )]
    pub Coordinate<T>,
);

impl<T: CoordNum> Triangle<T> {
    /// Create a new [`Triangle`] with the given [`Coordinate`]s as vertices.
    #[inline]
    pub fn new(v0: Coordinate<T>, v1: Coordinate<T>, v2: Coordinate<T>) -> Self {
        Self(v0, v1, v2)
    }

    /// Get the first of the triangle's three corners
    #[inline]
    pub fn vertex_0(&self) -> Coordinate<T> {
        #[allow(deprecated)]
        self.0
    }

    /// Mutably borrow the first of the triangle's three corners
    #[inline]
    pub fn vertex_0_mut(&mut self) -> &mut Coordinate<T> {
        #[allow(deprecated)]
        &mut self.0
    }

    /// Get the second of the triangle's three corners
    #[inline]
    pub fn vertex_1(&self) -> Coordinate<T> {
        #[allow(deprecated)]
        self.1
    }

    /// Mutably borrow the second of the triangle's three corners
    #[inline]
    pub fn vertex_1_mut(&mut self) -> &mut Coordinate<T> {
        #[allow(deprecated)]
        &mut self.1
    }

    /// Get the third of the triangle's three corners
    #[inline]
    pub fn vertex_2(&self) -> Coordinate<T> {
        #[allow(deprecated)]
        self.2
    }

    /// Mutably borrow the third of the triangle's three corners
    #[inline]
    pub fn vertex_2_mut(&mut self) -> &mut Coordinate<T> {
        #[allow(deprecated)]
        &mut self.2
    }

    pub fn to_array(&self) -> [Coordinate<T>; 3] {
        [self.vertex_0(), self.vertex_1(), self.vertex_2()]
    }

    pub fn to_lines(&self) -> [Line<T>; 3] {
        [
            Line::new(self.vertex_0(), self.vertex_1()),
            Line::new(self.vertex_1(), self.vertex_2()),
            Line::new(self.vertex_2(), self.vertex_0()),
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
        polygon![
            self.vertex_0(),
            self.vertex_1(),
            self.vertex_2(),
            self.vertex_0()
        ]
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
        if !self
            .vertex_0()
            .relative_eq(&other.vertex_0(), epsilon, max_relative)
        {
            return false;
        }
        if !self
            .vertex_1()
            .relative_eq(&other.vertex_1(), epsilon, max_relative)
        {
            return false;
        }
        if !self
            .vertex_2()
            .relative_eq(&other.vertex_2(), epsilon, max_relative)
        {
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
        if !self.vertex_0().abs_diff_eq(&other.vertex_0(), epsilon) {
            return false;
        }
        if !self.vertex_1().abs_diff_eq(&other.vertex_1(), epsilon) {
            return false;
        }
        if !self.vertex_2().abs_diff_eq(&other.vertex_2(), epsilon) {
            return false;
        }

        true
    }
}
