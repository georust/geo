use std::iter::FromIterator;
use {CoordinateType, Line, Point};

/// An ordered collection of two or more [`Point`s](struct.Point.html), representing a path between locations
///
/// Create a `LineString` by calling it directly:
///
/// ```
/// use geo_types::{LineString, Point};
/// let line = LineString(vec![Point::new(0., 0.), Point::new(10., 0.)]);
/// ```
///
/// Converting a `Vec` of `Point`-like things:
///
/// ```
/// # use geo_types::{LineString, Point};
/// let line: LineString<f32> = vec![(0., 0.), (10., 0.)].into();
/// ```
///
/// Or `collect`ing from a Point iterator
///
/// ```
/// # use geo_types::{LineString, Point};
/// let mut points = vec![Point::new(0., 0.), Point::new(10., 0.)];
/// let line: LineString<f32> = points.into_iter().collect();
/// ```
///
/// You can iterate over the points in the `LineString`
///
/// ```
/// use geo_types::{LineString, Point};
/// let line = LineString(vec![Point::new(0., 0.), Point::new(10., 0.)]);
/// for point in line {
///     println!("Point x = {}, y = {}", point.x(), point.y());
/// }
/// ```
///
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LineString<T>(pub Vec<Point<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType> LineString<T> {
    /// Return an `Line` iterator that yields one `Line` for each line segment
    /// in the `LineString`.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Line, LineString, Point};
    ///
    /// let mut points = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let linestring: LineString<f32> = points.into_iter().collect();
    ///
    /// let mut lines = linestring.lines();
    /// assert_eq!(
    ///     Some(Line::new(Point::new(0., 0.), Point::new(5., 0.))),
    ///     lines.next()
    /// );
    /// assert_eq!(
    ///     Some(Line::new(Point::new(5., 0.), Point::new(7., 9.))),
    ///     lines.next()
    /// );
    /// assert!(lines.next().is_none());
    /// ```
    pub fn lines<'a>(&'a self) -> impl Iterator<Item = Line<T>> + 'a {
        self.0.windows(2).map(|w| unsafe {
            // As long as the LineString has at least two points, we shouldn't
            // need to do bounds checking here.
            Line::new(*w.get_unchecked(0), *w.get_unchecked(1))
        })
    }

    pub fn points(&self) -> ::std::slice::Iter<Point<T>> {
        self.0.iter()
    }

    pub fn points_mut(&mut self) -> ::std::slice::IterMut<Point<T>> {
        self.0.iter_mut()
    }
}

/// Turn a `Vec` of `Point`-ish objects into a `LineString`.
impl<T: CoordinateType, IP: Into<Point<T>>> From<Vec<IP>> for LineString<T> {
    fn from(v: Vec<IP>) -> Self {
        LineString(v.into_iter().map(|p| p.into()).collect())
    }
}

/// Turn a `Point`-ish iterator into a `LineString`.
impl<T: CoordinateType, IP: Into<Point<T>>> FromIterator<IP> for LineString<T> {
    fn from_iter<I: IntoIterator<Item = IP>>(iter: I) -> Self {
        LineString(iter.into_iter().map(|p| p.into()).collect())
    }
}

/// Iterate over all the [Point](struct.Point.html)s in this linestring
impl<T: CoordinateType> IntoIterator for LineString<T> {
    type Item = Point<T>;
    type IntoIter = ::std::vec::IntoIter<Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
