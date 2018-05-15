use {CoordinateType, Point};

#[cfg(feature = "spade")]
use algorithms::{EuclideanDistance, BoundingBox};

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
            start: start,
            end: end,
        }
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
