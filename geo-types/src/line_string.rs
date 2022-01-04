#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

use crate::{CoordNum, Coordinate, Line, Point, Triangle};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

/// An ordered collection of two or more [`Coordinate`]s, representing a
/// path between locations.
///
/// # Semantics
///
/// 1. A [`LineString`] is _closed_ if it is empty, **or** if the first and last coordinates are the same.
/// 2. The _boundary_ of a [`LineString`] is either:
///     - **empty** if it is _closed_ (see **1**) **or**
///     - contains the **start** and **end** coordinates.
/// 3. The _interior_ is the (infinite) set of all coordinates along the [`LineString`], _not including_ the boundary.
/// 4. A [`LineString`] is _simple_ if it does not intersect except **optionally** at the first and last coordinates (in which case it is also _closed_, see **1**).
/// 5. A _simple_ **and** _closed_ [`LineString`] is a `LinearRing` as defined in the OGC-SFA (but is not defined as a separate type in this crate).
///
/// # Validity
///
/// A [`LineString`] is valid if it is either empty or
/// contains 2 or more coordinates.
///
/// Further, a closed [`LineString`] **must not** self-intersect. Note that its
/// validity is **not** enforced, and operations and
/// predicates are **undefined** on invalid `LineString`s.
///
/// # Examples
/// ## Creation
///
/// Create a [`LineString`] by calling it directly:
///
/// ```
/// use geo_types::{Coordinate, LineString};
///
/// let line_string = LineString(vec![
///     Coordinate { x: 0., y: 0. },
///     Coordinate { x: 10., y: 0. },
/// ]);
/// ```
///
/// Create a [`LineString`] with the [`line_string!`] macro:
///
/// ```
/// use geo_types::line_string;
///
/// let line_string = line_string![
///     (x: 0., y: 0.),
///     (x: 10., y: 0.),
/// ];
/// ```
///
/// By converting from a [`Vec`] of coordinate-like things:
///
/// ```
/// use geo_types::LineString;
///
/// let line_string: LineString<f32> = vec![(0., 0.), (10., 0.)].into();
/// ```
///
/// ```
/// use geo_types::LineString;
///
/// let line_string: LineString<f64> = vec![[0., 0.], [10., 0.]].into();
/// ```
//
/// Or by `collect`ing from a [`Coordinate`] iterator
///
/// ```
/// use geo_types::{Coordinate, LineString};
///
/// let mut coords_iter =
///     vec![Coordinate { x: 0., y: 0. }, Coordinate { x: 10., y: 0. }].into_iter();
///
/// let line_string: LineString<f32> = coords_iter.collect();
/// ```
///
/// ## Iteration
/// [`LineString`] provides five iterators: [`coords`](LineString::coords), [`coords_mut`](LineString::coords_mut), [`points`](LineString::points), [`lines`](LineString::lines), and [`triangles`](LineString::triangles):
///
/// ```
/// use geo_types::{Coordinate, LineString};
///
/// let line_string = LineString(vec![
///     Coordinate { x: 0., y: 0. },
///     Coordinate { x: 10., y: 0. },
/// ]);
///
/// line_string.coords().for_each(|coord| println!("{:?}", coord));
///
/// for point in line_string.points() {
///     println!("Point x = {}, y = {}", point.x(), point.y());
/// }
/// ```
///
/// Note that its [`IntoIterator`] impl yields [`Coordinate`]s when looping:
///
/// ```
/// use geo_types::{Coordinate, LineString};
///
/// let line_string = LineString(vec![
///     Coordinate { x: 0., y: 0. },
///     Coordinate { x: 10., y: 0. },
/// ]);
///
/// for coord in &line_string {
///     println!("Coordinate x = {}, y = {}", coord.x, coord.y);
/// }
///
/// for coord in line_string {
///     println!("Coordinate x = {}, y = {}", coord.x, coord.y);
/// }
///
/// ```
/// ## Decomposition
///
/// You can decompose a [`LineString`] into a [`Vec`] of [`Coordinate`]s or [`Point`]s:
/// ```
/// use geo_types::{Coordinate, LineString, Point};
///
/// let line_string = LineString(vec![
///     Coordinate { x: 0., y: 0. },
///     Coordinate { x: 10., y: 0. },
/// ]);
///
/// let coordinate_vec = line_string.clone().into_inner();
/// let point_vec = line_string.clone().into_points();
///
/// ```

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LineString<T>(pub Vec<Coordinate<T>>)
where
    T: CoordNum;

/// A [`Point`] iterator returned by the `points` method
#[derive(Debug)]
pub struct PointsIter<'a, T: CoordNum + 'a>(::std::slice::Iter<'a, Coordinate<T>>);

impl<'a, T: CoordNum> Iterator for PointsIter<'a, T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| Point(*c))
    }
}

impl<'a, T: CoordNum> DoubleEndedIterator for PointsIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|c| Point(*c))
    }
}

/// A [`Coordinate`] iterator used by the `into_iter` method on a [`LineString`]
#[derive(Debug)]
pub struct CoordinatesIter<'a, T: CoordNum + 'a>(::std::slice::Iter<'a, Coordinate<T>>);

impl<'a, T: CoordNum> Iterator for CoordinatesIter<'a, T> {
    type Item = &'a Coordinate<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, T: CoordNum> ExactSizeIterator for CoordinatesIter<'a, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T: CoordNum> DoubleEndedIterator for CoordinatesIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T: CoordNum> LineString<T> {
    /// Return an iterator yielding the coordinates of a [`LineString`] as [`Point`]s
    #[deprecated(note = "Use points() instead")]
    pub fn points_iter(&self) -> PointsIter<T> {
        PointsIter(self.0.iter())
    }

    /// Return an iterator yielding the coordinates of a [`LineString`] as [`Point`]s
    pub fn points(&self) -> PointsIter<T> {
        PointsIter(self.0.iter())
    }

    /// Return an iterator yielding the members of a [`LineString`] as [`Coordinate`]s
    pub fn coords(&self) -> impl Iterator<Item = &Coordinate<T>> {
        self.0.iter()
    }

    /// Return an iterator yielding the coordinates of a [`LineString`] as mutable [`Coordinate`]s
    pub fn coords_mut(&mut self) -> impl Iterator<Item = &mut Coordinate<T>> {
        self.0.iter_mut()
    }

    /// Return the coordinates of a [`LineString`] as a [`Vec`] of [`Point`]s
    pub fn into_points(self) -> Vec<Point<T>> {
        self.0.into_iter().map(Point).collect()
    }

    /// Return the coordinates of a [`LineString`] as a [`Vec`] of [`Coordinate`]s
    pub fn into_inner(self) -> Vec<Coordinate<T>> {
        self.0
    }

    /// Return an iterator yielding one [Line] for each line segment
    /// in the [`LineString`].
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, Line, LineString};
    ///
    /// let mut coords = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let line_string: LineString<f32> = coords.into_iter().collect();
    ///
    /// let mut lines = line_string.lines();
    /// assert_eq!(
    ///     Some(Line::new(
    ///         Coordinate { x: 0., y: 0. },
    ///         Coordinate { x: 5., y: 0. }
    ///     )),
    ///     lines.next()
    /// );
    /// assert_eq!(
    ///     Some(Line::new(
    ///         Coordinate { x: 5., y: 0. },
    ///         Coordinate { x: 7., y: 9. }
    ///     )),
    ///     lines.next()
    /// );
    /// assert!(lines.next().is_none());
    /// ```
    pub fn lines(&'_ self) -> impl ExactSizeIterator + Iterator<Item = Line<T>> + '_ {
        self.0.windows(2).map(|w| {
            // slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe { Line::new(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }

    /// An iterator which yields the coordinates of a [`LineString`] as [Triangle]s
    pub fn triangles(&'_ self) -> impl ExactSizeIterator + Iterator<Item = Triangle<T>> + '_ {
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

    /// Close the [`LineString`]. Specifically, if the [`LineString`] has at least one [`Coordinate`], and
    /// the value of the first [`Coordinate`] **does not** equal the value of the last [`Coordinate`], then a
    /// new [`Coordinate`] is added to the end with the value of the first [`Coordinate`].
    pub fn close(&mut self) {
        if !self.is_closed() {
            // by definition, we treat empty LineString's as closed.
            debug_assert!(!self.0.is_empty());
            self.0.push(self.0[0]);
        }
    }

    /// Return the number of coordinates in the [`LineString`].
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::LineString;
    ///
    /// let mut coords = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let line_string: LineString<f32> = coords.into_iter().collect();
    /// assert_eq!(3, line_string.num_coords());
    /// ```
    #[deprecated(note = "Use geo::algorithm::coords_iter::CoordsIter::coords_count instead")]
    pub fn num_coords(&self) -> usize {
        self.0.len()
    }

    /// Checks if the linestring is closed; i.e. it is
    /// either empty or, the first and last points are the
    /// same.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::LineString;
    ///
    /// let mut coords = vec![(0., 0.), (5., 0.), (0., 0.)];
    /// let line_string: LineString<f32> = coords.into_iter().collect();
    /// assert!(line_string.is_closed());
    /// ```
    ///
    /// Note that we diverge from some libraries ([JTS](https://locationtech.github.io/jts/javadoc/org/locationtech/jts/geom/LinearRing.html) et al), which have a `LinearRing` type,
    /// separate from [`LineString`]. Those libraries treat an empty `LinearRing` as **closed** by
    /// definition, while treating an empty `LineString` as **open**. Since we don't have a separate
    /// `LinearRing` type, and use a [`LineString`] in its place, we adopt the JTS `LinearRing` `is_closed`
    /// behavior in all places: that is, **we consider an empty [`LineString`] as closed**.
    ///
    /// This is expected when used in the context of a [`Polygon.exterior`](crate::Polygon::exterior) and elsewhere; And there
    /// seems to be no reason to maintain the separate behavior for [`LineString`]s used in
    /// non-`LinearRing` contexts.
    pub fn is_closed(&self) -> bool {
        self.0.first() == self.0.last()
    }
}

/// Turn a [`Vec`] of [`Point`]-like objects into a [`LineString`].
impl<T: CoordNum, IC: Into<Coordinate<T>>> From<Vec<IC>> for LineString<T> {
    fn from(v: Vec<IC>) -> Self {
        LineString(v.into_iter().map(|c| c.into()).collect())
    }
}

impl<T: CoordNum> From<Line<T>> for LineString<T> {
    fn from(line: Line<T>) -> Self {
        LineString(vec![line.start, line.end])
    }
}

/// Turn an iterator of [`Point`]-like objects into a [`LineString`].
impl<T: CoordNum, IC: Into<Coordinate<T>>> FromIterator<IC> for LineString<T> {
    fn from_iter<I: IntoIterator<Item = IC>>(iter: I) -> Self {
        LineString(iter.into_iter().map(|c| c.into()).collect())
    }
}

/// Iterate over all the [`Coordinate`]s in this [`LineString`].
impl<T: CoordNum> IntoIterator for LineString<T> {
    type Item = Coordinate<T>;
    type IntoIter = ::std::vec::IntoIter<Coordinate<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a LineString<T> {
    type Item = &'a Coordinate<T>;
    type IntoIter = CoordinatesIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        CoordinatesIter(self.0.iter())
    }
}

/// Mutably iterate over all the [`Coordinate`]s in this [`LineString`]
impl<'a, T: CoordNum> IntoIterator for &'a mut LineString<T> {
    type Item = &'a mut Coordinate<T>;
    type IntoIter = ::std::slice::IterMut<'a, Coordinate<T>>;

    fn into_iter(self) -> ::std::slice::IterMut<'a, Coordinate<T>> {
        self.0.iter_mut()
    }
}

impl<T: CoordNum> Index<usize> for LineString<T> {
    type Output = Coordinate<T>;

    fn index(&self, index: usize) -> &Coordinate<T> {
        self.0.index(index)
    }
}

impl<T: CoordNum> IndexMut<usize> for LineString<T> {
    fn index_mut(&mut self, index: usize) -> &mut Coordinate<T> {
        self.0.index_mut(index)
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for LineString<T>
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
    /// use geo_types::LineString;
    ///
    /// let mut coords_a = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let a: LineString<f32> = coords_a.into_iter().collect();
    ///
    /// let mut coords_b = vec![(0., 0.), (5., 0.), (7.001, 9.)];
    /// let b: LineString<f32> = coords_b.into_iter().collect();
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1)
    /// ```
    ///
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        let points_zipper = self.points().zip(other.points());
        for (lhs, rhs) in points_zipper {
            if lhs.relative_ne(&rhs, epsilon, max_relative) {
                return false;
            }
        }

        true
    }
}

#[cfg(any(feature = "approx", test))]
impl<T: AbsDiffEq<Epsilon = T> + CoordNum> AbsDiffEq for LineString<T> {
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
    /// use geo_types::LineString;
    ///
    /// let mut coords_a = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let a: LineString<f32> = coords_a.into_iter().collect();
    ///
    /// let mut coords_b = vec![(0., 0.), (5., 0.), (7.001, 9.)];
    /// let b: LineString<f32> = coords_b.into_iter().collect();
    ///
    /// approx::assert_relative_eq!(a, b, epsilon=0.1)
    /// ```
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        let mut points_zipper = self.points().zip(other.points());
        points_zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(&rhs, epsilon))
    }
}

macro_rules! impl_rstar_line_string {
    ($rstar:ident) => {
        impl<T> ::$rstar::RTreeObject for LineString<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            type Envelope = ::$rstar::AABB<Point<T>>;

            fn envelope(&self) -> Self::Envelope {
                use num_traits::Bounded;
                let bounding_rect = crate::private_utils::line_string_bounding_rect(self);
                match bounding_rect {
                    None => ::$rstar::AABB::from_corners(
                        Point::new(Bounded::min_value(), Bounded::min_value()),
                        Point::new(Bounded::max_value(), Bounded::max_value()),
                    ),
                    Some(b) => ::$rstar::AABB::from_corners(
                        Point::new(b.min().x, b.min().y),
                        Point::new(b.max().x, b.max().y),
                    ),
                }
            }
        }

        impl<T> ::$rstar::PointDistance for LineString<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            fn distance_2(&self, point: &Point<T>) -> T {
                let d = crate::private_utils::point_line_string_euclidean_distance(*point, self);
                if d == T::zero() {
                    d
                } else {
                    d.powi(2)
                }
            }
        }
    };
}

#[cfg(feature = "rstar")]
impl_rstar_line_string!(rstar);

#[cfg(feature = "rstar_0_9")]
impl_rstar_line_string!(rstar_0_9);

#[cfg(test)]
mod test {
    use super::*;

    use approx::AbsDiffEq;

    #[test]
    fn test_abs_diff_eq() {
        let delta = 1e-6;

        let coords = vec![(0., 0.), (5., 0.), (10., 10.)];
        let ls: LineString<f32> = coords.into_iter().collect();

        let coords_x = vec![(0., 0.), (5. + delta, 0.), (10., 10.)];
        let ls_x: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.abs_diff_eq(&ls_x, 1e-2));
        assert!(ls.abs_diff_ne(&ls_x, 1e-12));

        let coords_y = vec![(0., 0.), (5., 0. + delta), (10., 10.)];
        let ls_y: LineString<f32> = coords_y.into_iter().collect();
        assert!(ls.abs_diff_eq(&ls_y, 1e-2));
        assert!(ls.abs_diff_ne(&ls_y, 1e-12));

        // Undersized, but otherwise equal.
        let coords_x = vec![(0., 0.), (5., 0.)];
        let ls_under: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.abs_diff_ne(&ls_under, 1.));

        // Oversized, but otherwise equal.
        let coords_x = vec![(0., 0.), (5., 0.), (10., 10.), (10., 100.)];
        let ls_oversized: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.abs_diff_ne(&ls_oversized, 1.));
    }

    #[test]
    fn test_relative_eq() {
        let delta = 1e-6;

        let coords = vec![(0., 0.), (5., 0.), (10., 10.)];
        let ls: LineString<f32> = coords.into_iter().collect();

        let coords_x = vec![(0., 0.), (5. + delta, 0.), (10., 10.)];
        let ls_x: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.relative_eq(&ls_x, 1e-2, 1e-2));
        assert!(ls.relative_ne(&ls_x, 1e-12, 1e-12));

        let coords_y = vec![(0., 0.), (5., 0. + delta), (10., 10.)];
        let ls_y: LineString<f32> = coords_y.into_iter().collect();
        assert!(ls.relative_eq(&ls_y, 1e-2, 1e-2));
        assert!(ls.relative_ne(&ls_y, 1e-12, 1e-12));

        // Undersized, but otherwise equal.
        let coords_x = vec![(0., 0.), (5., 0.)];
        let ls_under: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.relative_ne(&ls_under, 1., 1.));

        // Oversized, but otherwise equal.
        let coords_x = vec![(0., 0.), (5., 0.), (10., 10.), (10., 100.)];
        let ls_oversized: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.relative_ne(&ls_oversized, 1., 1.));
    }

    #[test]
    fn should_be_built_from_line() {
        let start = Coordinate { x: 0, y: 0 };
        let end = Coordinate { x: 10, y: 10 };
        let line = Line::new(start, end);
        let expected = LineString(vec![start, end]);

        assert_eq!(expected, LineString::from(line));

        let start = Coordinate { x: 10., y: 0.5 };
        let end = Coordinate { x: 10000., y: 10.4 };
        let line = Line::new(start, end);
        let expected = LineString(vec![start, end]);

        assert_eq!(expected, LineString::from(line));
    }
}
