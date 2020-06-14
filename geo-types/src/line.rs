use crate::{Coordinate, CoordinateType, Point};

/// A line segment made up of exactly two [`Point`s](struct.Point.html).
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
        self.end.x - self.start.x
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
        self.end.y - self.start.y
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
