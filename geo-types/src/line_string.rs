use std::iter::FromIterator;
use {Coordinate, CoordinateType, Line, Point, Triangle};

#[cfg(feature = "spade")]
use algorithms::{BoundingRect, EuclideanDistance};

/// An ordered collection of two or more [`Coordinate`s](struct.Coordinate.html), representing a
/// path between locations.
///
/// # Examples
///
/// Create a `LineString` by calling it directly:
///
/// ```
/// use geo_types::{LineString, Coordinate};
///
/// let line_string = LineString(vec![
///     Coordinate { x: 0., y: 0. },
///     Coordinate { x: 10., y: 0. },
/// ]);
/// ```
///
/// Converting a `Vec` of `Coordinate`-like things:
///
/// ```
/// use geo_types::LineString;
///
/// let line_string: LineString<f32> = vec![
///     (0., 0.),
///     (10., 0.),
/// ].into();
/// ```
///
/// ```
/// use geo_types::LineString;
///
/// let line_string: LineString<f64> = vec![
///     [0., 0.],
///     [10., 0.],
/// ].into();
/// ```
//
/// Or `collect`ing from a `Coordinate` iterator
///
/// ```
/// use geo_types::{LineString, Coordinate};
///
/// let mut coords_iter = vec![
///     Coordinate { x: 0., y: 0. },
///     Coordinate { x: 10., y: 0. }
/// ].into_iter();
///
/// let line_string: LineString<f32> = coords_iter.collect();
/// ```
///
/// You can iterate over the coordinates in the `LineString`:
///
/// ```
/// use geo_types::{LineString, Coordinate};
///
/// let line_string = LineString(vec![
///     Coordinate { x: 0., y: 0. },
///     Coordinate { x: 10., y: 0. },
/// ]);
///
/// for coord in line_string {
///     println!("Coordinate x = {}, y = {}", coord.x, coord.y);
/// }
/// ```
///
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LineString<T>(pub Vec<Coordinate<T>>)
where
    T: CoordinateType;

pub struct PointsIter<'a, T: CoordinateType + 'a>(::std::slice::Iter<'a, Coordinate<T>>);

impl<'a, T: CoordinateType> Iterator for PointsIter<'a, T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| Point(*c))
    }
}

impl<'a, T: CoordinateType> DoubleEndedIterator for PointsIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|c| Point(*c))
    }
}

impl<T: CoordinateType> LineString<T> {
    pub fn points_iter(&self) -> PointsIter<T> {
        PointsIter(self.0.iter())
    }

    pub fn into_points(self) -> Vec<Point<T>> {
        self.0.into_iter().map(Point).collect()
    }

    /// Return an `Line` iterator that yields one `Line` for each line segment
    /// in the `LineString`.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Line, LineString, Coordinate};
    ///
    /// let mut coords = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let line_string: LineString<f32> = coords.into_iter().collect();
    ///
    /// let mut lines = line_string.lines();
    /// assert_eq!(
    ///     Some(Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 5., y: 0. })),
    ///     lines.next()
    /// );
    /// assert_eq!(
    ///     Some(Line::new(Coordinate { x: 5., y: 0. }, Coordinate { x: 7., y: 9. })),
    ///     lines.next()
    /// );
    /// assert!(lines.next().is_none());
    /// ```
    pub fn lines<'a>(&'a self) -> impl ExactSizeIterator + Iterator<Item = Line<T>> + 'a {
        self.0.windows(2).map(|w| {
            // slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe { Line::new(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }

    pub fn triangles<'a>(&'a self) -> impl ExactSizeIterator + Iterator<Item = Triangle<T>> + 'a {
        self.0.windows(3).map(|w| {
            // slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe {
                Triangle(
                    *w.get_unchecked(0),
                    *w.get_unchecked(1),
                    *w.get_unchecked(2),
                )
            }
        })
    }
}

/// Turn a `Vec` of `Point`-ish objects into a `LineString`.
impl<T: CoordinateType, IC: Into<Coordinate<T>>> From<Vec<IC>> for LineString<T> {
    fn from(v: Vec<IC>) -> Self {
        LineString(v.into_iter().map(|c| c.into()).collect())
    }
}

/// Turn a `Point`-ish iterator into a `LineString`.
impl<T: CoordinateType, IC: Into<Coordinate<T>>> FromIterator<IC> for LineString<T> {
    fn from_iter<I: IntoIterator<Item = IC>>(iter: I) -> Self {
        LineString(iter.into_iter().map(|c| c.into()).collect())
    }
}

/// Iterate over all the [Coordinate](struct.Coordinates.html)s in this `LineString`.
impl<T: CoordinateType> IntoIterator for LineString<T> {
    type Item = Coordinate<T>;
    type IntoIter = ::std::vec::IntoIter<Coordinate<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(feature = "spade")]
impl<T> ::spade::SpatialObject for LineString<T>
where
    T: ::num_traits::Float + ::spade::SpadeNum + ::std::fmt::Debug,
{
    type Point = Point<T>;

    fn mbr(&self) -> ::spade::BoundingRect<Self::Point> {
        let bounding_rect = self.bounding_rect();
        match bounding_rect {
            None => ::spade::BoundingRect::from_corners(
                &Point::new(T::min_value(), T::min_value()),
                &Point::new(T::max_value(), T::max_value()),
            ),
            Some(b) => ::spade::BoundingRect::from_corners(
                &Point::new(b.min.x, b.min.y),
                &Point::new(b.max.x, b.max.y),
            ),
        }
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
