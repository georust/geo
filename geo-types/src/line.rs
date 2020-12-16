use crate::{Coordinate, CoordinateType, Point};
#[cfg(test)]
use approx::AbsDiffEq;
#[cfg(test)]
use approx::RelativeEq;
#[cfg(test)]
use num_traits::Float;
/// A line segment made up of exactly two
/// [`Coordinate`s](struct.Coordinate.html).
///
/// # Semantics
///
/// The _interior_ and _boundary_ are defined as with a
/// `LineString` with the two end points.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Line<T>
where
    T: CoordinateType,
{
    pub start: Coordinate<T>,
    pub end: Coordinate<T>,
}

impl<T> Line<T>
where
    T: CoordinateType,
{
    /// Creates a new line segment.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, Line};
    ///
    /// let line = Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 1., y: 2. });
    ///
    /// assert_eq!(line.start, Coordinate { x: 0., y: 0. });
    /// assert_eq!(line.end, Coordinate { x: 1., y: 2. });
    /// ```
    pub fn new<C>(start: C, end: C) -> Line<T>
    where
        C: Into<Coordinate<T>>,
    {
        Line {
            start: start.into(),
            end: end.into(),
        }
    }

    /// Calculate the difference in coordinates (Δx, Δy).
    pub fn delta(&self) -> Coordinate<T> {
        self.end - self.start
    }

    /// Calculate the difference in ‘x’ components (Δx).
    ///
    /// Equivalent to:
    ///
    /// ```rust
    /// # use geo_types::{Line, Coordinate, Point};
    /// # let line = Line::new(
    /// #     Point(Coordinate { x: 4., y: -12. }),
    /// #     Point(Coordinate { x: 0., y: 9. }),
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
    /// # use geo_types::{Line, Coordinate, Point};
    /// # let line = Line::new(
    /// #     Point(Coordinate { x: 4., y: -12. }),
    /// #     Point(Coordinate { x: 0., y: 9. }),
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
    /// # use geo_types::{Line, Coordinate, Point};
    /// # let line = Line::new(
    /// #     Point(Coordinate { x: 4., y: -12. }),
    /// #     Point(Coordinate { x: 0., y: 9. }),
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
    /// # use geo_types::{Line, Coordinate, Point};
    /// # let a = Point(Coordinate { x: 4., y: -12. });
    /// # let b = Point(Coordinate { x: 0., y: 9. });
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
    /// # use geo_types::{Line, Coordinate, Point};
    /// # let line = Line::new(
    /// #     Point(Coordinate { x: 4., y: -12. }),
    /// #     Point(Coordinate { x: 0., y: 9. }),
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
    /// # use geo_types::{Line, Coordinate, Point};
    /// # let a = Point(Coordinate { x: 4., y: -12. });
    /// # let b = Point(Coordinate { x: 0., y: 9. });
    /// # assert!(
    /// Line::new(a, b).determinant() == -Line::new(b, a).determinant()
    /// # );
    /// ```
    pub fn determinant(&self) -> T {
        self.start.x * self.end.y - self.start.y * self.end.x
    }

    pub fn start_point(&self) -> Point<T> {
        Point(self.start)
    }

    pub fn end_point(&self) -> Point<T> {
        Point(self.end)
    }

    pub fn points(&self) -> (Point<T>, Point<T>) {
        (self.start_point(), self.end_point())
    }
}

impl<T: CoordinateType> From<[(T, T); 2]> for Line<T> {
    fn from(coord: [(T, T); 2]) -> Line<T> {
        Line::new(coord[0], coord[1])
    }
}

#[cfg(test)]
impl<T> RelativeEq for Line<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordinateType + Float + RelativeEq
{

    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::epsilon()
    }

    #[inline]
    fn relative_eq(
        &self,
        other: &Line<T>,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.start
            .relative_eq(&other.start, epsilon, max_relative)
            && self
                .end
                .relative_eq(&other.end, epsilon, max_relative)
    }
}

#[cfg(test)]
impl<T: AbsDiffEq + CoordinateType + Float> AbsDiffEq for Line<T> {
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Line<T>, epsilon: Self::Epsilon) -> bool {
        self.start_point()
            .abs_diff_eq(&other.start_point(), epsilon)
            && self.end_point().abs_diff_eq(&other.end_point(), epsilon)
    }
}

#[cfg(feature = "rstar")]
impl<T> ::rstar::RTreeObject for Line<T>
where
    T: ::num_traits::Float + ::rstar::RTreeNum,
{
    type Envelope = ::rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        let bounding_rect = crate::private_utils::line_bounding_rect(*self);
        ::rstar::AABB::from_corners(bounding_rect.min().into(), bounding_rect.max().into())
    }
}

#[cfg(feature = "rstar")]
impl<T> ::rstar::PointDistance for Line<T>
where
    T: ::num_traits::Float + ::rstar::RTreeNum,
{
    fn distance_2(&self, point: &Point<T>) -> T {
        let d = crate::private_utils::point_line_euclidean_distance(*point, *self);
        d.powi(2)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use super::{Coordinate, Line, Point};
    use approx::AbsDiffEq;
    #[test]
    fn test_abs_diff_eq() {
        let delta = 1e-6;
        let line = Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 1., y: 1. });
        let line_start_x = Line::new(
            Point(Coordinate {
                x: 0. + delta,
                y: 0.,
            }),
            Point(Coordinate { x: 1., y: 1. }),
        );
        assert!(line.abs_diff_eq(&line_start_x, 1e-2));
        assert!(line.abs_diff_ne(&line_start_x, 1e-12));

        let line_start_y = Line::new(
            Coordinate {
                x: 0.,
                y: 0. + delta,
            },
            Coordinate { x: 1., y: 1. },
        );
        assert!(line.abs_diff_eq(&line_start_y, 1e-2));
        assert!(line.abs_diff_ne(&line_start_y, 1e-12));

        let line_end_x = Line::new(
            Coordinate { x: 0., y: 0. },
            Coordinate {
                x: 1. + delta,
                y: 1.,
            },
        );

        assert!(line.abs_diff_eq(&line_end_x, 1e-2));
        assert!(line.abs_diff_ne(&line_end_x, 1e-12));

        let line_end_y = Line::new(
            Coordinate { x: 0., y: 0. },
            Coordinate {
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

        let line = Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 1., y: 1. });
        let line_start_x = Line::new(
            Point(Coordinate {
                x: 0. + delta,
                y: 0.,
            }),
            Point(Coordinate { x: 1., y: 1. }),
        );
        let line_start_y = Line::new(
            Coordinate {
                x: 0.,
                y: 0. + delta,
            },
            Coordinate { x: 1., y: 1. },
        );

        assert!(line.relative_eq(&line_start_x, 1e-2, 1e-2));
        assert!(line.relative_ne(&line_start_x, 1e-12, 1e-12));

        assert!(line.relative_eq(&line_start_y, 1e-2, 1e-2));
        assert!(line.relative_ne(&line_start_y, 1e-12, 1e-12));
    }
}
