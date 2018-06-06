use {CoordinateType, Point};

#[cfg(feature = "spade")]
use algorithms::{BoundingBox, EuclideanDistance};

/// A line segment made up of exactly two [`Point`s](struct.Point.html)
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Line<T>
where
    T: CoordinateType,
{
    pub start: Point<T>,
    pub end: Point<T>,
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
    /// use geo_types::{Point, Line};
    ///
    /// let line = Line::new(Point::new(0., 0.), Point::new(1., 2.));
    ///
    /// assert_eq!(line.start, Point::new(0., 0.));
    /// assert_eq!(line.end, Point::new(1., 2.));
    /// ```
    pub fn new(start: Point<T>, end: Point<T>) -> Line<T> {
        Line {
            start,
            end
        }
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
    /// line.end.x() - line.start.x()
    /// # );
    /// ```
    pub fn dx(&self) -> T {
        self.end.x() - self.start.x()
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
    /// line.end.y() - line.start.y()
    /// # );
    /// ```
    pub fn dy(&self) -> T {
        self.end.y() - self.start.y()
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
    /// Line::new(a, b).slope() ==
    ///     Line::new(b, a).slope()
    /// # );
    /// ```
    pub fn slope(&self) -> T {
        self.dy() / self.dx()
    }

    /// Calculate the [determinant] of the line.
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
    /// line.start.x() * line.end.y() -
    ///     line.start.y() * line.end.x()
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
    /// Line::new(a, b).determinant() ==
    ///     -Line::new(b, a).determinant()
    /// # );
    /// ```
    ///
    /// [determinant]: https://en.wikipedia.org/wiki/Determinant
    pub fn determinant(&self) -> T {
        self.start.x() * self.end.y() - self.start.y() * self.end.x()
    }
}

#[cfg(feature = "spade")]
impl<T> ::spade::SpatialObject for Line<T>
where
    T: ::num_traits::Float + ::spade::SpadeNum + ::std::fmt::Debug,
{
    type Point = Point<T>;

    fn mbr(&self) -> ::spade::BoundingRect<Self::Point> {
        let bbox = self.bbox();
        ::spade::BoundingRect::from_corners(
            &Point::new(bbox.xmin, bbox.ymin),
            &Point::new(bbox.xmax, bbox.ymax),
        )
    }

    fn distance2(&self, point: &Self::Point) -> <Self::Point as ::spade::PointN>::Scalar {
        let d = self.euclidean_distance(point);
        if d == T::zero() {
            d
        } else {
            d.powi(2)
        }
    }
}
