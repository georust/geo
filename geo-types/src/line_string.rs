use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use {Coordinate, CoordinateType, Line, Point, Triangle};

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

    /// Close the `LineString`. Specifically, if the `LineString` has is at least one coordinate,
    /// and the value of the first coordinate does not equal the value of the last coordinate, then
    /// a new coordinate is added to the end with the value of the first coordinate.
    pub(crate) fn close(&mut self) {
        if let (Some(first), Some(last)) = (self.0.first().map(|n| *n), self.0.last().map(|n| *n)) {
            if first != last {
                self.0.push(first);
            }
        }
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

impl<T: CoordinateType> Index<usize> for LineString<T> {
    type Output = Coordinate<T>;

    fn index(&self, index: usize) -> &Coordinate<T> {
        self.0.index(index)
    }
}

impl<T: CoordinateType> IndexMut<usize> for LineString<T> {
    fn index_mut(&mut self, index: usize) -> &mut Coordinate<T> {
        self.0.index_mut(index)
    }
}


#[cfg(feature = "rstar")]
impl<T> ::rstar::RTreeObject for LineString<T>
where
    T: ::num_traits::Float + ::rstar::RTreeNum,
{
    type Envelope = ::rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        use num_traits::Bounded;
        let bounding_rect = ::private_utils::line_string_bounding_rect(self);
        match bounding_rect {
            None => ::rstar::AABB::from_corners(
                Point::new(Bounded::min_value(), Bounded::min_value()),
                Point::new(Bounded::max_value(), Bounded::max_value()),
            ),
            Some(b) => ::rstar::AABB::from_corners(
                Point::new(b.min.x, b.min.y),
                Point::new(b.max.x, b.max.y),
            ),
        }
    }
}

#[cfg(feature = "rstar")]
impl<T> ::rstar::PointDistance for LineString<T>
where
    T: ::num_traits::Float + ::rstar::RTreeNum,
{
    fn distance_2(&self, point: &Point<T>) -> T {
        let d = ::private_utils::point_line_string_euclidean_distance(*point, self);
        if d == T::zero() {
            d
        } else {
            d.powi(2)
        }
    }
}
