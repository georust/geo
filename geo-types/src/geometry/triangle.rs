use crate::{polygon, Coord, CoordNum, Line, Point, Polygon};
use core::cmp::Ordering;

/// A bounded 2D area whose three vertices are defined by
/// `Coord`s. The semantics and validity are that of
/// the equivalent [`Polygon`]; in addition, the three
/// vertices **must not** be collinear and they *must* be distinct.
///
/// # Notes
/// Irrespective of input order the resulting geometry has ccw order and its vertices are yielded in ccw order by iterators
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Triangle<T: CoordNum = f64>(
    #[deprecated(
        since = "0.7.19",
        note = "Use Triangle::new for construction and triangle.v1() to retrieve the value."
    )]
    pub Coord<T>,
    #[deprecated(
        since = "0.7.19",
        note = "Use Triangle::new for construction and triangle.v2() to retrieve the value."
    )]
    pub Coord<T>,
    #[deprecated(
        since = "0.7.19",
        note = "Use Triangle::new for construction and triangle.v3() to retrieve the value."
    )]
    pub Coord<T>,
);

#[allow(deprecated)]
impl<T: CoordNum> Triangle<T> {
    /// Create a new [`Triangle`], enforcing default (counter clockwise) winding order.
    ///
    /// If the vertices are given in clockwise order, they will be reversed.
    /// Degenerate triangles (collinear or identical vertices) are stored as-is.
    pub fn new(v1: Coord<T>, v2: Coord<T>, v3: Coord<T>) -> Self {
        // determine cross product of input points. NB: non-robust
        let orientation = Point::from(v1).cross_prod(v2.into(), v3.into());
        match orientation.partial_cmp(&T::zero()) {
            Some(Ordering::Greater) => Self(v1, v2, v3),
            Some(Ordering::Less) => Self(v3, v2, v1),
            // degenerate: collinear or identical — store as-is
            _ => Self(v1, v2, v3),
        }
    }

    /// Create a [`Triangle`] without normalising the winding order.
    ///
    /// Use this when the caller has already verified CCW winding, or when the
    /// calling code does not depend on a particular winding order (e.g. inside
    /// tight loops where the orientation check would be wasteful).
    ///
    /// Unlike [`Triangle::new`], vertices are stored in the order given.
    pub fn unchecked_winding(v1: Coord<T>, v2: Coord<T>, v3: Coord<T>) -> Self {
        Self(v1, v2, v3)
    }

    /// Return the first vertex of this triangle.
    pub fn v1(&self) -> Coord<T> {
        self.0
    }

    /// Return the second vertex of this triangle.
    pub fn v2(&self) -> Coord<T> {
        self.1
    }

    /// Return the third vertex of this triangle.
    pub fn v3(&self) -> Coord<T> {
        self.2
    }

    pub fn to_array(&self) -> [Coord<T>; 3] {
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
    /// // Input is CW
    /// let triangle = Triangle::new(
    ///     coord! { x: 0., y: 0. },
    ///     coord! { x: 10., y: 20. },
    ///     coord! { x: 20., y: -10. },
    /// );
    ///
    /// // Output is CCW
    /// assert_eq!(
    ///     triangle.to_polygon(),
    ///     polygon![
    ///         (x: 20., y: -10.),
    ///         (x: 10., y: 20.),
    ///         (x: 0., y: 0.),
    ///         (x: 20., y: -10.),
    ///     ],
    /// );
    /// ```
    pub fn to_polygon(self) -> Polygon<T> {
        polygon![self.0, self.1, self.2, self.0]
    }
}

impl<IC: Into<Coord<T>> + Copy, T: CoordNum> From<[IC; 3]> for Triangle<T> {
    fn from(array: [IC; 3]) -> Self {
        Self::new(array[0].into(), array[1].into(), array[2].into())
    }
}

#[cfg(any(feature = "approx", test))]
mod approx_integration {
    use super::*;
    use approx::{AbsDiffEq, RelativeEq, UlpsEq};

    impl<T> RelativeEq for Triangle<T>
    where
        T: CoordNum + RelativeEq<Epsilon = T>,
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
            if !self.v1().relative_eq(&other.v1(), epsilon, max_relative) {
                return false;
            }
            if !self.v2().relative_eq(&other.v2(), epsilon, max_relative) {
                return false;
            }
            if !self.v3().relative_eq(&other.v3(), epsilon, max_relative) {
                return false;
            }

            true
        }
    }

    impl<T> AbsDiffEq for Triangle<T>
    where
        T: CoordNum + AbsDiffEq<Epsilon = T>,
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
            if !self.v1().abs_diff_eq(&other.v1(), epsilon) {
                return false;
            }
            if !self.v2().abs_diff_eq(&other.v2(), epsilon) {
                return false;
            }
            if !self.v3().abs_diff_eq(&other.v3(), epsilon) {
                return false;
            }

            true
        }
    }

    impl<T> UlpsEq for Triangle<T>
    where
        T: CoordNum + UlpsEq<Epsilon = T>,
    {
        fn default_max_ulps() -> u32 {
            T::default_max_ulps()
        }

        fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
            if !self.v1().ulps_eq(&other.v1(), epsilon, max_ulps) {
                return false;
            }
            if !self.v2().ulps_eq(&other.v2(), epsilon, max_ulps) {
                return false;
            }
            if !self.v3().ulps_eq(&other.v3(), epsilon, max_ulps) {
                return false;
            }
            true
        }
    }
}

#[cfg(any(
    feature = "rstar_0_8",
    feature = "rstar_0_9",
    feature = "rstar_0_10",
    feature = "rstar_0_11",
    feature = "rstar_0_12"
))]
macro_rules! impl_rstar_triangle {
    ($rstar:ident) => {
        impl<T> ::$rstar::RTreeObject for Triangle<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            type Envelope = ::$rstar::AABB<Point<T>>;

            fn envelope(&self) -> Self::Envelope {
                let bounding_rect =
                    crate::private_utils::get_bounding_rect(self.to_array()).unwrap();
                ::$rstar::AABB::from_corners(bounding_rect.min().into(), bounding_rect.max().into())
            }
        }
    };
}

#[cfg(feature = "rstar_0_8")]
impl_rstar_triangle!(rstar_0_8);

#[cfg(feature = "rstar_0_9")]
impl_rstar_triangle!(rstar_0_9);

#[cfg(feature = "rstar_0_10")]
impl_rstar_triangle!(rstar_0_10);

#[cfg(feature = "rstar_0_11")]
impl_rstar_triangle!(rstar_0_11);

#[cfg(feature = "rstar_0_12")]
impl_rstar_triangle!(rstar_0_12);
