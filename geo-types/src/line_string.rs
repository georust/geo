use crate::*;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

/// A generic line string with 3D space + Measure value support.
///
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
/// use geo_types::{coord, LineString};
///
/// let line_string = LineString::new(vec![
///     coord! { x: 0., y: 0. },
///     coord! { x: 10., y: 0. },
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
/// Note that its [`IntoIterator`] impl yields [`Coordinate`]s when looping:
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
/// You can decompose a [`LineString`] into a [`Vec`] of [`Coordinate`]s or [`Point`]s:
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
pub struct LineString<T: CoordNum, Z: ZCoord = NoValue, M: Measure = NoValue>(
    pub Vec<Coordinate<T, Z, M>>,
);

/// A line string with a measurement value in 2D space.
///
/// See [LineString]
pub type LineStringM<T> = LineString<T, NoValue, T>;

/// A line string in 3D space.
///
/// See [LineString]
pub type LineString3D<T> = LineString<T, T, NoValue>;

/// A line string with a measurement value in 3D space.
///
/// See [LineString]
pub type LineString3DM<T> = LineString<T, T, T>;

/// A [`Point`] iterator returned by the `points` method
#[derive(Debug)]
pub struct PointsIter<'a, T, Z = NoValue, M = NoValue>(::std::slice::Iter<'a, Coordinate<T, Z, M>>)
where
    T: CoordNum + 'a,
    Z: ZCoord + 'a,
    M: Measure + 'a;

pub type PointsIterM<'a, T> = PointsIter<'a, T, NoValue, T>;
pub type PointsIter3D<'a, T> = PointsIter<'a, T, T, NoValue>;
pub type PointsIter3DM<'a, T> = PointsIter<'a, T, T, T>;

impl<'a, T, Z, M> Iterator for PointsIter<'a, T, Z, M>
where
    T: CoordNum + 'a,
    Z: ZCoord + 'a,
    M: Measure + 'a,
{
    type Item = Point<T, Z, M>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| Point::from(*c))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T: CoordNum, Z: ZCoord, M: Measure> ExactSizeIterator for PointsIter<'a, T, Z, M> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T: CoordNum, Z: ZCoord, M: Measure> DoubleEndedIterator for PointsIter<'a, T, Z, M> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|c| Point::from(*c))
    }
}

/// A [`Coordinate`] iterator used by the `into_iter` method on a [`LineString`]
#[derive(Debug)]
pub struct CoordinatesIter<'a, T: CoordNum + 'a, Z: ZCoord + 'a, M: Measure + 'a>(
    ::std::slice::Iter<'a, Coordinate<T, Z, M>>,
);

impl<'a, T: CoordNum, Z: ZCoord, M: Measure> Iterator for CoordinatesIter<'a, T, Z, M> {
    type Item = &'a Coordinate<T, Z, M>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T: CoordNum, Z: ZCoord, M: Measure> ExactSizeIterator for CoordinatesIter<'a, T, Z, M> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T: CoordNum, Z: ZCoord, M: Measure> DoubleEndedIterator for CoordinatesIter<'a, T, Z, M> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T: CoordNum, Z: ZCoord, M: Measure> LineString<T, Z, M> {
    /// Instantiate Self from the raw content value
    pub fn new(value: Vec<Coordinate<T, Z, M>>) -> Self {
        Self(value)
    }

    /// Return an iterator yielding the coordinates of a [`LineString`] as [`Point`]s
    pub fn points(&self) -> PointsIter<T, Z, M> {
        PointsIter(self.0.iter())
    }

    /// Return an iterator yielding the members of a [`LineString`] as [`Coordinate`]s
    pub fn coords(&self) -> impl Iterator<Item = &Coordinate<T, Z, M>> {
        self.0.iter()
    }

    /// Return an iterator yielding the coordinates of a [`LineString`] as mutable [`Coordinate`]s
    pub fn coords_mut(&mut self) -> impl Iterator<Item = &mut Coordinate<T, Z, M>> {
        self.0.iter_mut()
    }

    /// Return the coordinates of a [`LineString`] as a [`Vec`] of [`Point`]s
    pub fn into_points(self) -> Vec<Point<T, Z, M>> {
        self.0.into_iter().map(Point::from).collect()
    }

    /// Return the coordinates of a [`LineString`] as a [`Vec`] of [`Coordinate`]s
    pub fn into_inner(self) -> Vec<Coordinate<T, Z, M>> {
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
    pub fn lines(&'_ self) -> impl ExactSizeIterator + Iterator<Item = Line<T, Z, M>> + '_ {
        self.0.windows(2).map(|w| {
            // slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe { Line::new(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }

    /// An iterator which yields the coordinates of a [`LineString`] as [Triangle]s
    pub fn triangles(&'_ self) -> impl ExactSizeIterator + Iterator<Item = Triangle<T, Z, M>> + '_ {
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
impl<T: CoordNum, Z: ZCoord, M: Measure, IC: Into<Coordinate<T, Z, M>>> From<Vec<IC>>
    for LineString<T, Z, M>
{
    fn from(v: Vec<IC>) -> Self {
        Self(v.into_iter().map(|c| c.into()).collect())
    }
}

impl<T: CoordNum, Z: ZCoord, M: Measure> From<Line<T, Z, M>> for LineString<T, Z, M> {
    fn from(line: Line<T, Z, M>) -> Self {
        Self(vec![line.start, line.end])
    }
}

/// Turn an iterator of [`Point`]-like objects into a [`LineString`].
impl<T: CoordNum, Z: ZCoord, M: Measure, IC: Into<Coordinate<T, Z, M>>> FromIterator<IC>
    for LineString<T, Z, M>
{
    fn from_iter<I: IntoIterator<Item = IC>>(iter: I) -> Self {
        Self(iter.into_iter().map(|c| c.into()).collect())
    }
}

/// Iterate over all the [`Coordinate`]s in this [`LineString`].
impl<T: CoordNum, Z: ZCoord, M: Measure> IntoIterator for LineString<T, Z, M> {
    type Item = Coordinate<T, Z, M>;
    type IntoIter = ::std::vec::IntoIter<Coordinate<T, Z, M>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum, Z: ZCoord, M: Measure> IntoIterator for &'a LineString<T, Z, M> {
    type Item = &'a Coordinate<T, Z, M>;
    type IntoIter = CoordinatesIter<'a, T, Z, M>;

    fn into_iter(self) -> Self::IntoIter {
        CoordinatesIter(self.0.iter())
    }
}

/// Mutably iterate over all the [`Coordinate`]s in this [`LineString`]
impl<'a, T: CoordNum, Z: ZCoord, M: Measure> IntoIterator for &'a mut LineString<T, Z, M> {
    type Item = &'a mut Coordinate<T, Z, M>;
    type IntoIter = ::std::slice::IterMut<'a, Coordinate<T, Z, M>>;

    fn into_iter(self) -> ::std::slice::IterMut<'a, Coordinate<T, Z, M>> {
        self.0.iter_mut()
    }
}

impl<T: CoordNum, Z: ZCoord, M: Measure> Index<usize> for LineString<T, Z, M> {
    type Output = Coordinate<T, Z, M>;

    fn index(&self, index: usize) -> &Coordinate<T, Z, M> {
        self.0.index(index)
    }
}

impl<T: CoordNum, Z: ZCoord, M: Measure> IndexMut<usize> for LineString<T, Z, M> {
    fn index_mut(&mut self, index: usize) -> &mut Coordinate<T, Z, M> {
        self.0.index_mut(index)
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> approx::RelativeEq for LineString<T>
where
    T: approx::AbsDiffEq<Epsilon = T> + CoordNum + approx::RelativeEq,
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
impl<T: approx::AbsDiffEq<Epsilon = T> + CoordNum> approx::AbsDiffEq for LineString<T> {
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

#[cfg(any(feature = "rstar_0_8", feature = "rstar_0_9"))]
macro_rules! impl_rstar_line_string {
    ($rstar:ident) => {
        impl<T> ::$rstar::RTreeObject for crate::LineString<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            type Envelope = ::$rstar::AABB<crate::Point<T>>;

            fn envelope(&self) -> Self::Envelope {
                use num_traits::Bounded;
                let bounding_rect = crate::private_utils::line_string_bounding_rect(self);
                match bounding_rect {
                    None => ::$rstar::AABB::from_corners(
                        crate::Point::new(Bounded::min_value(), Bounded::min_value()),
                        crate::Point::new(Bounded::max_value(), Bounded::max_value()),
                    ),
                    Some(b) => ::$rstar::AABB::from_corners(
                        crate::Point::new(b.min().x, b.min().y),
                        crate::Point::new(b.max().x, b.max().y),
                    ),
                }
            }
        }

        impl<T> ::$rstar::PointDistance for crate::LineString<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            fn distance_2(&self, point: &crate::Point<T>) -> T {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, Line};
    use approx::{AbsDiffEq, RelativeEq};

    #[test]
    fn test_exact_size() {
        // see https://github.com/georust/geo/issues/762
        let ls = LineString::new(vec![coord! { x: 0., y: 0. }, coord! { x: 10., y: 0. }]);

        // reference to force the `impl IntoIterator for &LineString` impl, giving a `CoordinatesIter`
        for c in (&ls).into_iter().rev().skip(1).rev() {
            println!("{:?}", c);
        }
        for p in (&ls).points().rev().skip(1).rev() {
            println!("{:?}", p);
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
