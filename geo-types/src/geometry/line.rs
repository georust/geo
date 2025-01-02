use crate::{Coord, CoordNum, Point};
#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

/// A line segment made up of exactly two
/// [`Coord`]s.
///
/// # Semantics
///
/// The _interior_ and _boundary_ are defined as with a
/// `LineString` with the two end points.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Line<T: CoordNum = f64> {
    pub start: Coord<T>,
    pub end: Coord<T>,
}

impl<T: CoordNum> Line<T> {
    /// Creates a new line segment.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{coord, Line};
    ///
    /// let line = Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 2. });
    ///
    /// assert_eq!(line.start, coord! { x: 0., y: 0. });
    /// assert_eq!(line.end, coord! { x: 1., y: 2. });
    /// ```
    pub fn new<C>(start: C, end: C) -> Self
    where
        C: Into<Coord<T>>,
    {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }

    /// Calculate the difference in coordinates (Δx, Δy).
    pub fn delta(&self) -> Coord<T> {
        self.end - self.start
    }

    /// Calculate the difference in ‘x’ components (Δx).
    ///
    /// Equivalent to:
    ///
    /// ```rust
    /// # use geo_types::{Line, point};
    /// # let line = Line::new(
    /// #     point! { x: 4., y: -12. },
    /// #     point! { x: 0., y: 9. },
    /// # );
    /// # assert_eq!(
    /// #     line.dx(),
    /// line.end.x - line.start.x
    /// # );
    /// ```
    pub fn dx(&self) -> T {
        self.delta().x
    }

    /// Calculate the difference in ‘y’ components (Δy).
    ///
    /// Equivalent to:
    ///
    /// ```rust
    /// # use geo_types::{Line, point};
    /// # let line = Line::new(
    /// #     point! { x: 4., y: -12. },
    /// #     point! { x: 0., y: 9. },
    /// # );
    /// # assert_eq!(
    /// #     line.dy(),
    /// line.end.y - line.start.y
    /// # );
    /// ```
    pub fn dy(&self) -> T {
        self.delta().y
    }

    /// Calculate the slope (Δy/Δx).
    ///
    /// Equivalent to:
    ///
    /// ```rust
    /// # use geo_types::{Line, point};
    /// # let line = Line::new(
    /// #     point! { x: 4., y: -12. },
    /// #     point! { x: 0., y: 9. },
    /// # );
    /// # assert_eq!(
    /// #     line.slope(),
    /// line.dy() / line.dx()
    /// # );
    /// ```
    ///
    /// Note that:
    ///
    /// ```rust
    /// # use geo_types::{Line, point};
    /// # let a = point! { x: 4., y: -12. };
    /// # let b = point! { x: 0., y: 9. };
    /// # assert!(
    /// Line::new(a, b).slope() == Line::new(b, a).slope()
    /// # );
    /// ```
    pub fn slope(&self) -> T {
        self.dy() / self.dx()
    }

    /// Calculate the [determinant](https://en.wikipedia.org/wiki/Determinant) of the line.
    ///
    /// Equivalent to:
    ///
    /// ```rust
    /// # use geo_types::{Line, point};
    /// # let line = Line::new(
    /// #     point! { x: 4., y: -12. },
    /// #     point! { x: 0., y: 9. },
    /// # );
    /// # assert_eq!(
    /// #     line.determinant(),
    /// line.start.x * line.end.y - line.start.y * line.end.x
    /// # );
    /// ```
    ///
    /// Note that:
    ///
    /// ```rust
    /// # use geo_types::{Line, point};
    /// # let a = point! { x: 4., y: -12. };
    /// # let b = point! { x: 0., y: 9. };
    /// # assert!(
    /// Line::new(a, b).determinant() == -Line::new(b, a).determinant()
    /// # );
    /// ```
    pub fn determinant(&self) -> T {
        self.start.x * self.end.y - self.start.y * self.end.x
    }

    pub fn start_point(&self) -> Point<T> {
        Point::from(self.start)
    }

    pub fn end_point(&self) -> Point<T> {
        Point::from(self.end)
    }

    pub fn points(&self) -> (Point<T>, Point<T>) {
        (self.start_point(), self.end_point())
    }
}

impl<T: CoordNum> From<[(T, T); 2]> for Line<T> {
    fn from(coord: [(T, T); 2]) -> Self {
        Line::new(coord[0], coord[1])
    }
}
#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for Line<T>
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
    /// use geo_types::{coord, Line};
    ///
    /// let a = Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. });
    /// let b = Line::new(coord! { x: 0., y: 0. }, coord! { x: 1.001, y: 1. });
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1);
    /// ```
    #[inline]
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.start.relative_eq(&other.start, epsilon, max_relative)
            && self.end.relative_eq(&other.end, epsilon, max_relative)
    }
}

#[cfg(any(feature = "approx", test))]
impl<T: AbsDiffEq<Epsilon = T> + CoordNum> AbsDiffEq for Line<T> {
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
    /// use geo_types::{coord, Line};
    ///
    /// let a = Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. });
    /// let b = Line::new(coord! { x: 0., y: 0. }, coord! { x: 1.001, y: 1. });
    ///
    /// approx::assert_abs_diff_eq!(a, b, epsilon=0.1);
    /// ```
    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.start.abs_diff_eq(&other.start, epsilon) && self.end.abs_diff_eq(&other.end, epsilon)
    }
}

#[cfg(any(
    feature = "rstar_0_8",
    feature = "rstar_0_9",
    feature = "rstar_0_10",
    feature = "rstar_0_11",
    feature = "rstar_0_12"
))]
macro_rules! impl_rstar_line {
    ($rstar:ident) => {
        impl<T> ::$rstar::RTreeObject for Line<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            type Envelope = ::$rstar::AABB<Point<T>>;

            fn envelope(&self) -> Self::Envelope {
                ::$rstar::AABB::from_corners(self.start_point(), self.end_point())
            }
        }

        impl<T> ::$rstar::PointDistance for Line<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            fn distance_2(&self, point: &Point<T>) -> T {
                let d = crate::private_utils::point_line_euclidean_distance(*point, *self);
                d.powi(2)
            }
        }
    };
}

#[cfg(feature = "rstar_0_8")]
impl_rstar_line!(rstar_0_8);

#[cfg(feature = "rstar_0_9")]
impl_rstar_line!(rstar_0_9);

#[cfg(feature = "rstar_0_10")]
impl_rstar_line!(rstar_0_10);

#[cfg(feature = "rstar_0_11")]
impl_rstar_line!(rstar_0_11);

#[cfg(feature = "rstar_0_12")]
impl_rstar_line!(rstar_0_12);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, point};

    #[test]
    fn test_abs_diff_eq() {
        let delta = 1e-6;
        let line = Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. });
        let line_start_x = Line::new(
            point! {
                x: 0. + delta,
                y: 0.,
            },
            point! { x: 1., y: 1. },
        );
        assert!(line.abs_diff_eq(&line_start_x, 1e-2));
        assert!(line.abs_diff_ne(&line_start_x, 1e-12));

        let line_start_y = Line::new(
            coord! {
                x: 0.,
                y: 0. + delta,
            },
            coord! { x: 1., y: 1. },
        );
        assert!(line.abs_diff_eq(&line_start_y, 1e-2));
        assert!(line.abs_diff_ne(&line_start_y, 1e-12));

        let line_end_x = Line::new(
            coord! { x: 0., y: 0. },
            coord! {
                x: 1. + delta,
                y: 1.,
            },
        );

        assert!(line.abs_diff_eq(&line_end_x, 1e-2));
        assert!(line.abs_diff_ne(&line_end_x, 1e-12));

        let line_end_y = Line::new(
            coord! { x: 0., y: 0. },
            coord! {
                x: 1.,
                y: 1. + delta,
            },
        );

        assert!(line.abs_diff_eq(&line_end_y, 1e-2));
        assert!(line.abs_diff_ne(&line_end_y, 1e-12));
    }

    #[test]
    fn test_relative_eq() {
        let delta = 1e-6;

        let line = Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. });
        let line_start_x = Line::new(
            point! {
                x: 0. + delta,
                y: 0.,
            },
            point! { x: 1., y: 1. },
        );
        let line_start_y = Line::new(
            coord! {
                x: 0.,
                y: 0. + delta,
            },
            coord! { x: 1., y: 1. },
        );

        assert!(line.relative_eq(&line_start_x, 1e-2, 1e-2));
        assert!(line.relative_ne(&line_start_x, 1e-12, 1e-12));

        assert!(line.relative_eq(&line_start_y, 1e-2, 1e-2));
        assert!(line.relative_ne(&line_start_y, 1e-12, 1e-12));
    }
}
