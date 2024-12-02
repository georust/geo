#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

use crate::{Coord, CoordNum, Line, Point, Triangle};
use alloc::vec;
use alloc::vec::Vec;
use core::iter::FromIterator;
use core::ops::{Index, IndexMut};

/// An ordered collection of [`Coord`]s, representing a path between locations.
/// To be valid, a `LineString` must be empty, or have two or more coords.
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
/// use geo_types::{coord, LineString};
///
/// let line_string = LineString::new(vec![
///     coord! { x: 0., y: 0. },
///     coord! { x: 10., y: 0. },
/// ]);
/// ```
///
/// Create a [`LineString`] with the [`line_string!`][`crate::line_string!`] macro:
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
/// let line_string: LineString = vec![[0., 0.], [10., 0.]].into();
/// ```
//
/// Or by `collect`ing from a [`Coord`] iterator
///
/// ```
/// use geo_types::{coord, LineString};
///
/// let mut coords_iter =
///     vec![coord! { x: 0., y: 0. }, coord! { x: 10., y: 0. }].into_iter();
///
/// let line_string: LineString<f32> = coords_iter.collect();
/// ```
///
/// ## Iteration
/// [`LineString`] provides five iterators: [`coords`](LineString::coords), [`coords_mut`](LineString::coords_mut), [`points`](LineString::points), [`lines`](LineString::lines), and [`triangles`](LineString::triangles):
///
/// ```
/// use geo_types::{coord, LineString};
///
/// let line_string = LineString::new(vec![
///     coord! { x: 0., y: 0. },
///     coord! { x: 10., y: 0. },
/// ]);
///
/// line_string.coords().for_each(|coord| println!("{:?}", coord));
///
/// for point in line_string.points() {
///     println!("Point x = {}, y = {}", point.x(), point.y());
/// }
/// ```
///
/// Note that its [`IntoIterator`] impl yields [`Coord`]s when looping:
///
/// ```
/// use geo_types::{coord, LineString};
///
/// let line_string = LineString::new(vec![
///     coord! { x: 0., y: 0. },
///     coord! { x: 10., y: 0. },
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
/// You can decompose a [`LineString`] into a [`Vec`] of [`Coord`]s or [`Point`]s:
/// ```
/// use geo_types::{coord, LineString, Point};
///
/// let line_string = LineString::new(vec![
///     coord! { x: 0., y: 0. },
///     coord! { x: 10., y: 0. },
/// ]);
///
/// let coordinate_vec = line_string.clone().into_inner();
/// let point_vec = line_string.clone().into_points();
///
/// ```

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LineString<T: CoordNum = f64>(pub Vec<Coord<T>>);

/// A [`Point`] iterator returned by the `points` method
#[derive(Debug)]
pub struct PointsIter<'a, T: CoordNum + 'a>(::core::slice::Iter<'a, Coord<T>>);

impl<T: CoordNum> Iterator for PointsIter<'_, T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| Point::from(*c))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: CoordNum> ExactSizeIterator for PointsIter<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: CoordNum> DoubleEndedIterator for PointsIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|c| Point::from(*c))
    }
}

/// A [`Coord`] iterator used by the `into_iter` method on a [`LineString`]
#[derive(Debug)]
pub struct CoordinatesIter<'a, T: CoordNum + 'a>(::core::slice::Iter<'a, Coord<T>>);

impl<'a, T: CoordNum> Iterator for CoordinatesIter<'a, T> {
    type Item = &'a Coord<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: CoordNum> ExactSizeIterator for CoordinatesIter<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: CoordNum> DoubleEndedIterator for CoordinatesIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T: CoordNum> LineString<T> {
    /// Instantiate Self from the raw content value
    pub fn new(value: Vec<Coord<T>>) -> Self {
        Self(value)
    }

    /// Return an iterator yielding the coordinates of a [`LineString`] as [`Point`]s
    #[deprecated(note = "Use points() instead")]
    pub fn points_iter(&self) -> PointsIter<T> {
        PointsIter(self.0.iter())
    }

    /// Return an iterator yielding the coordinates of a [`LineString`] as [`Point`]s
    pub fn points(&self) -> PointsIter<T> {
        PointsIter(self.0.iter())
    }

    /// Return an iterator yielding the members of a [`LineString`] as [`Coord`]s
    pub fn coords(&self) -> impl DoubleEndedIterator<Item = &Coord<T>> {
        self.0.iter()
    }

    /// Return an iterator yielding the coordinates of a [`LineString`] as mutable [`Coord`]s
    pub fn coords_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Coord<T>> {
        self.0.iter_mut()
    }

    /// Return the coordinates of a [`LineString`] as a [`Vec`] of [`Point`]s
    pub fn into_points(self) -> Vec<Point<T>> {
        self.0.into_iter().map(Point::from).collect()
    }

    /// Return the coordinates of a [`LineString`] as a [`Vec`] of [`Coord`]s
    pub fn into_inner(self) -> Vec<Coord<T>> {
        self.0
    }

    /// Return an iterator yielding one [Line] for each line segment
    /// in the [`LineString`].
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{coord, Line, LineString};
    ///
    /// let mut coords = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let line_string: LineString<f32> = coords.into_iter().collect();
    ///
    /// let mut lines = line_string.lines();
    /// assert_eq!(
    ///     Some(Line::new(
    ///         coord! { x: 0., y: 0. },
    ///         coord! { x: 5., y: 0. }
    ///     )),
    ///     lines.next()
    /// );
    /// assert_eq!(
    ///     Some(Line::new(
    ///         coord! { x: 5., y: 0. },
    ///         coord! { x: 7., y: 9. }
    ///     )),
    ///     lines.next()
    /// );
    /// assert!(lines.next().is_none());
    /// ```
    pub fn lines(&'_ self) -> impl ExactSizeIterator<Item = Line<T>> + '_ {
        self.0.windows(2).map(|w| {
            // slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe { Line::new(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }

    /// An iterator which yields the coordinates of a [`LineString`] as [Triangle]s
    pub fn triangles(&'_ self) -> impl ExactSizeIterator<Item = Triangle<T>> + '_ {
        self.0.windows(3).map(|w| {
            // slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe {
                Triangle::new(
                    *w.get_unchecked(0),
                    *w.get_unchecked(1),
                    *w.get_unchecked(2),
                )
            }
        })
    }

    /// Close the [`LineString`]. Specifically, if the [`LineString`] has at least one [`Coord`], and
    /// the value of the first [`Coord`] **does not** equal the value of the last [`Coord`], then a
    /// new [`Coord`] is added to the end with the value of the first [`Coord`].
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
    ///
    /// # #[allow(deprecated)]
    /// # {
    /// assert_eq!(3, line_string.num_coords());
    /// # }
    /// ```
    #[deprecated(note = "Use geo::CoordsIter::coords_count instead")]
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
impl<T: CoordNum, IC: Into<Coord<T>>> From<Vec<IC>> for LineString<T> {
    fn from(v: Vec<IC>) -> Self {
        Self(v.into_iter().map(|c| c.into()).collect())
    }
}

impl<T: CoordNum> From<Line<T>> for LineString<T> {
    fn from(line: Line<T>) -> Self {
        LineString::from(&line)
    }
}

impl<T: CoordNum> From<&Line<T>> for LineString<T> {
    fn from(line: &Line<T>) -> Self {
        Self(vec![line.start, line.end])
    }
}

/// Turn an iterator of [`Point`]-like objects into a [`LineString`].
impl<T: CoordNum, IC: Into<Coord<T>>> FromIterator<IC> for LineString<T> {
    fn from_iter<I: IntoIterator<Item = IC>>(iter: I) -> Self {
        Self(iter.into_iter().map(|c| c.into()).collect())
    }
}

/// Iterate over all the [`Coord`]s in this [`LineString`].
impl<T: CoordNum> IntoIterator for LineString<T> {
    type Item = Coord<T>;
    type IntoIter = ::alloc::vec::IntoIter<Coord<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a LineString<T> {
    type Item = &'a Coord<T>;
    type IntoIter = CoordinatesIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        CoordinatesIter(self.0.iter())
    }
}

/// Mutably iterate over all the [`Coord`]s in this [`LineString`]
impl<'a, T: CoordNum> IntoIterator for &'a mut LineString<T> {
    type Item = &'a mut Coord<T>;
    type IntoIter = ::core::slice::IterMut<'a, Coord<T>>;

    fn into_iter(self) -> ::core::slice::IterMut<'a, Coord<T>> {
        self.0.iter_mut()
    }
}

impl<T: CoordNum> Index<usize> for LineString<T> {
    type Output = Coord<T>;

    fn index(&self, index: usize) -> &Coord<T> {
        self.0.index(index)
    }
}

impl<T: CoordNum> IndexMut<usize> for LineString<T> {
    fn index_mut(&mut self, index: usize) -> &mut Coord<T> {
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

#[cfg(any(
    feature = "rstar_0_8",
    feature = "rstar_0_9",
    feature = "rstar_0_10",
    feature = "rstar_0_11",
    feature = "rstar_0_12"
))]
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

#[cfg(feature = "rstar_0_8")]
impl_rstar_line_string!(rstar_0_8);

#[cfg(feature = "rstar_0_9")]
impl_rstar_line_string!(rstar_0_9);

#[cfg(feature = "rstar_0_10")]
impl_rstar_line_string!(rstar_0_10);

#[cfg(feature = "rstar_0_11")]
impl_rstar_line_string!(rstar_0_11);

#[cfg(feature = "rstar_0_12")]
impl_rstar_line_string!(rstar_0_12);

#[cfg(test)]
mod test {
    use super::*;
    use crate::coord;
    use approx::AbsDiffEq;

    #[test]
    fn test_exact_size() {
        // see https://github.com/georust/geo/issues/762
        let first = coord! { x: 0., y: 0. };
        let ls = LineString::new(vec![first, coord! { x: 10., y: 0. }]);

        // reference to force the `impl IntoIterator for &LineString` impl, giving a `CoordinatesIter`
        for c in (&ls).into_iter().rev().skip(1).rev() {
            assert_eq!(&first, c);
        }
        for p in ls.points().rev().skip(1).rev() {
            assert_eq!(Point::from(first), p);
        }
    }

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
        let start = coord! { x: 0, y: 0 };
        let end = coord! { x: 10, y: 10 };
        let line = Line::new(start, end);
        let expected = LineString::new(vec![start, end]);

        assert_eq!(expected, LineString::from(line));

        let start = coord! { x: 10., y: 0.5 };
        let end = coord! { x: 10000., y: 10.4 };
        let line = Line::new(start, end);
        let expected = LineString::new(vec![start, end]);

        assert_eq!(expected, LineString::from(line));
    }
}
