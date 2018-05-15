extern crate num_traits;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg(feature = "spade")]
extern crate spade;

#[cfg(feature = "spade")]
mod algorithms;
#[cfg(feature = "spade")]
use algorithms::*;

use std::ops::Add;
use std::ops::Neg;
use std::ops::Sub;

use num_traits::{Float, Num, NumCast, Signed, ToPrimitive};
use std::iter::{FromIterator, Iterator};

#[cfg(feature = "spade")]
use spade::{BoundingRect, PointN, SpadeNum, SpatialObject, TwoDimensional};

/// The type of an x or y value of a point/coordinate.
///
/// Floats (`f32` and `f64`) and Integers (`u8`, `i32` etc.) implement this. Many algorithms only
/// make sense for Float types (like area, or length calculations).
pub trait CoordinateType: Num + Copy + NumCast + PartialOrd {}
// Little bit of a hack to make to make this work
impl<T: Num + Copy + NumCast + PartialOrd> CoordinateType for T {}

/// A primitive type which holds `x` and `y` position information
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coordinate<T>
where
    T: CoordinateType,
{
    pub x: T,
    pub y: T,
}

impl<T: CoordinateType> From<(T, T)> for Coordinate<T> {
    fn from(coords: (T, T)) -> Self {
        Coordinate {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl<T: CoordinateType> From<[T; 2]> for Coordinate<T> {
    fn from(coords: [T; 2]) -> Self {
        Coordinate {
            x: coords[0],
            y: coords[1],
        }
    }
}

/// A single Point in 2D space.
///
/// Points can be created using the `new(x, y)` constructor, or from a `Coordinate` or pair of points.
///
/// ```
/// use geo_types::{Point, Coordinate};
/// let p1: Point<f64> = (0., 1.).into();
/// let c = Coordinate{ x: 10., y: 20.};
/// let p2: Point<f64> = c.into();
/// ```
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Point<T>(pub Coordinate<T>)
where
    T: CoordinateType;

impl<T: CoordinateType> From<Coordinate<T>> for Point<T> {
    fn from(x: Coordinate<T>) -> Point<T> {
        Point(x)
    }
}

impl<T: CoordinateType> From<(T, T)> for Point<T> {
    fn from(coords: (T, T)) -> Point<T> {
        Point::new(coords.0, coords.1)
    }
}

impl<T: CoordinateType> From<[T; 2]> for Point<T> {
    fn from(coords: [T; 2]) -> Point<T> {
        Point::new(coords[0], coords[1])
    }
}

impl<T> Point<T>
where
    T: CoordinateType + ToPrimitive,
{
    /// Creates a new point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.x(), 1.234);
    /// assert_eq!(p.y(), 2.345);
    /// ```
    pub fn new(x: T, y: T) -> Point<T> {
        Point(Coordinate { x: x, y: y })
    }

    /// Returns the x/horizontal component of the point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.x(), 1.234);
    /// ```
    pub fn x(&self) -> T {
        self.0.x
    }

    /// Sets the x/horizontal component of the point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// p.set_x(9.876);
    ///
    /// assert_eq!(p.x(), 9.876);
    /// ```
    pub fn set_x(&mut self, x: T) -> &mut Point<T> {
        self.0.x = x;
        self
    }

    /// Returns the y/vertical component of the point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.y(), 2.345);
    /// ```
    pub fn y(&self) -> T {
        self.0.y
    }

    /// Sets the y/vertical component of the point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// p.set_y(9.876);
    ///
    /// assert_eq!(p.y(), 9.876);
    /// ```
    pub fn set_y(&mut self, y: T) -> &mut Point<T> {
        self.0.y = y;
        self
    }

    /// Returns the longitude/horizontal component of the point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.lng(), 1.234);
    /// ```
    pub fn lng(&self) -> T {
        self.x()
    }

    /// Sets the longitude/horizontal component of the point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// p.set_lng(9.876);
    ///
    /// assert_eq!(p.lng(), 9.876);
    /// ```
    pub fn set_lng(&mut self, lng: T) -> &mut Point<T> {
        self.set_x(lng)
    }

    /// Returns the latitude/vertical component of the point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.lat(), 2.345);
    /// ```
    pub fn lat(&self) -> T {
        self.y()
    }

    /// Sets the latitude/vertical component of the point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// p.set_lat(9.876);
    ///
    /// assert_eq!(p.lat(), 9.876);
    /// ```
    pub fn set_lat(&mut self, lat: T) -> &mut Point<T> {
        self.set_y(lat)
    }

    /// Returns the dot product of the two points:
    /// `dot = x1 * x2 + y1 * y2`
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.5, 0.5);
    /// let dot = p.dot(&Point::new(2.0, 4.5));
    ///
    /// assert_eq!(dot, 5.25);
    /// ```
    pub fn dot(&self, point: &Point<T>) -> T {
        self.x() * point.x() + self.y() * point.y()
    }

    /// Returns the cross product of 3 points. A positive value implies
    /// `self` → `point_b` → `point_c` is counter-clockwise, negative implies
    /// clockwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p_a = Point::new(1.0, 2.0);
    /// let p_b = Point::new(3.0,5.0);
    /// let p_c = Point::new(7.0,12.0);
    ///
    /// let cross = p_a.cross_prod(&p_b, &p_c);
    ///
    /// assert_eq!(cross, 2.0)
    /// ```
    pub fn cross_prod(&self, point_b: &Point<T>, point_c: &Point<T>) -> T
    where
        T: Float,
    {
        (point_b.x() - self.x()) * (point_c.y() - self.y())
            - (point_b.y() - self.y()) * (point_c.x() - self.x())
    }
}

impl<T> Neg for Point<T>
where
    T: CoordinateType + Neg<Output = T> + ToPrimitive,
{
    type Output = Point<T>;

    /// Returns a point with the x and y components negated.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = -Point::new(-1.25, 2.5);
    ///
    /// assert_eq!(p.x(), 1.25);
    /// assert_eq!(p.y(), -2.5);
    /// ```
    fn neg(self) -> Point<T> {
        Point::new(-self.x(), -self.y())
    }
}

impl<T> Add for Point<T>
where
    T: CoordinateType + ToPrimitive,
{
    type Output = Point<T>;

    /// Add a point to the given point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.25, 2.5) + Point::new(1.5, 2.5);
    ///
    /// assert_eq!(p.x(), 2.75);
    /// assert_eq!(p.y(), 5.0);
    /// ```
    fn add(self, rhs: Point<T>) -> Point<T> {
        Point::new(self.x() + rhs.x(), self.y() + rhs.y())
    }
}

impl<T> Sub for Point<T>
where
    T: CoordinateType + ToPrimitive,
{
    type Output = Point<T>;

    /// Subtract a point from the given point.
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.25, 3.0) - Point::new(1.5, 2.5);
    ///
    /// assert_eq!(p.x(), -0.25);
    /// assert_eq!(p.y(), 0.5);
    /// ```
    fn sub(self, rhs: Point<T>) -> Point<T> {
        Point::new(self.x() - rhs.x(), self.y() - rhs.y())
    }
}

#[cfg(feature = "spade")]
// These are required for Spade RTree
impl<T> PointN for Point<T>
where
    T: Float + SpadeNum + std::fmt::Debug,
{
    type Scalar = T;

    fn dimensions() -> usize {
        2
    }
    fn from_value(value: Self::Scalar) -> Self {
        Point::new(value, value)
    }
    fn nth(&self, index: usize) -> &Self::Scalar {
        match index {
            0 => &self.0.x,
            1 => &self.0.y,
            _ => unreachable!(),
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.0.x,
            1 => &mut self.0.y,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "spade")]
impl<T> TwoDimensional for Point<T>
where
    T: Float + SpadeNum + std::fmt::Debug,
{
}

/// A collection of [`Point`s](struct.Point.html)
///
/// Iterating over a `MultiPoint` yields the `Point`s inside.
///
/// ```
/// use geo_types::{MultiPoint, Point};
/// let points: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
/// for point in points {
///     println!("Point x = {}, y = {}", point.x(), point.y());
/// }
/// ```
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiPoint<T>(pub Vec<Point<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType, IP: Into<Point<T>>> From<IP> for MultiPoint<T> {
    /// Convert a single `Point` (or something which can be converted to a `Point`) into a
    /// one-member `MultiPoint`
    fn from(x: IP) -> MultiPoint<T> {
        MultiPoint(vec![x.into()])
    }
}

impl<T: CoordinateType, IP: Into<Point<T>>> From<Vec<IP>> for MultiPoint<T> {
    /// Convert a `Vec` of `Points` (or `Vec` of things which can be converted to a `Point`) into a
    /// `MultiPoint`.
    fn from(v: Vec<IP>) -> MultiPoint<T> {
        MultiPoint(v.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordinateType, IP: Into<Point<T>>> FromIterator<IP> for MultiPoint<T> {
    /// Collect the results of a `Point` iterator into a `MultiPoint`
    fn from_iter<I: IntoIterator<Item = IP>>(iter: I) -> Self {
        MultiPoint(iter.into_iter().map(|p| p.into()).collect())
    }
}

/// Iterate over the `Point`s in this `MultiPoint`.
impl<T: CoordinateType> IntoIterator for MultiPoint<T> {
    type Item = Point<T>;
    type IntoIter = ::std::vec::IntoIter<Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

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
impl<T> SpatialObject for Line<T>
where
    T: Float + SpadeNum + std::fmt::Debug,
{
    type Point = Point<T>;

    fn mbr(&self) -> BoundingRect<Self::Point> {
        let bbox = self.bbox();
        BoundingRect::from_corners(
            &Point::new(bbox.xmin, bbox.ymin),
            &Point::new(bbox.xmax, bbox.ymax),
        )
    }

    fn distance2(&self, point: &Self::Point) -> <Self::Point as PointN>::Scalar {
        let d = self.euclidean_distance(point);
        if d == T::zero() {
            d
        } else {
            d.powi(2)
        }
    }
}

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
/// ```
/// # use geo_types::{LineString, Point};
/// let line: LineString<f64> = vec![[0., 0.], [10., 0.]].into();
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

/// A collection of [`LineString`s](struct.LineString.html)
///
/// Can be created from a `Vec` of `LineString`s, or from an Iterator which yields `LineString`s.
///
/// Iterating over this objects, yields the component `LineString`s.
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiLineString<T>(pub Vec<LineString<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType, ILS: Into<LineString<T>>> From<ILS> for MultiLineString<T> {
    fn from(ls: ILS) -> Self {
        MultiLineString(vec![ls.into()])
    }
}

impl<T: CoordinateType, ILS: Into<LineString<T>>> FromIterator<ILS> for MultiLineString<T> {
    fn from_iter<I: IntoIterator<Item = ILS>>(iter: I) -> Self {
        MultiLineString(iter.into_iter().map(|ls| ls.into()).collect())
    }
}

impl<T: CoordinateType> IntoIterator for MultiLineString<T> {
    type Item = LineString<T>;
    type IntoIter = ::std::vec::IntoIter<LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A representation of an area. Its outer boundary is represented by a [`LineString`](struct.LineString.html) that is both closed and simple
///
/// It has one exterior *ring* or *shell*, and zero or more interior rings, representing holes.
///
/// Polygons can be created from collections of `Point`-like objects, such as arrays or tuples:
///
/// ```
/// use geo_types::{Point, LineString, Polygon};
/// let poly1 = Polygon::new(vec![[0., 0.], [10., 0.]].into(), vec![]);
/// let poly2 = Polygon::new(vec![(0., 0.), (10., 0.)].into(), vec![]);
/// ```
///
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Polygon<T>
where
    T: CoordinateType,
{
    pub exterior: LineString<T>,
    pub interiors: Vec<LineString<T>>,
}

impl<T> Polygon<T>
where
    T: CoordinateType,
{
    /// Creates a new polygon.
    ///
    /// ```
    /// use geo_types::{Point, LineString, Polygon};
    ///
    /// let exterior = LineString(vec![Point::new(0., 0.), Point::new(1., 1.),
    ///                                Point::new(1., 0.), Point::new(0., 0.)]);
    /// let interiors = vec![LineString(vec![Point::new(0.1, 0.1), Point::new(0.9, 0.9),
    ///                                      Point::new(0.9, 0.1), Point::new(0.1, 0.1)])];
    /// let p = Polygon::new(exterior.clone(), interiors.clone());
    /// assert_eq!(p.exterior, exterior);
    /// assert_eq!(p.interiors, interiors);
    /// ```
    pub fn new(exterior: LineString<T>, interiors: Vec<LineString<T>>) -> Polygon<T> {
        Polygon {
            exterior: exterior,
            interiors: interiors,
        }
    }
    /// Wrap-around previous-vertex
    fn previous_vertex(&self, current_vertex: &usize) -> usize
    where
        T: Float,
    {
        (current_vertex + (self.exterior.0.len() - 1) - 1) % (self.exterior.0.len() - 1)
    }
}

// used to check the sign of a vec of floats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ListSign {
    Empty,
    Positive,
    Negative,
    Mixed,
}

impl<T> Polygon<T>
where
    T: Float + Signed,
{
    /// Determine whether a Polygon is convex
    // For each consecutive pair of edges of the polygon (each triplet of points),
    // compute the z-component of the cross product of the vectors defined by the
    // edges pointing towards the points in increasing order.
    // Take the cross product of these vectors
    // The polygon is convex if the z-components of the cross products are either
    // all positive or all negative. Otherwise, the polygon is non-convex.
    // see: http://stackoverflow.com/a/1881201/416626
    pub fn is_convex(&self) -> bool {
        let convex = self
            .exterior
            .0
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                let prev_1 = self.previous_vertex(&idx);
                let prev_2 = self.previous_vertex(&prev_1);
                self.exterior.0[prev_2].cross_prod(
                    &self.exterior.0[prev_1],
                    &self.exterior.0[idx]
                )
            })
            // accumulate and check cross-product result signs in a single pass
            // positive implies ccw convexity, negative implies cw convexity
            // anything else implies non-convexity
            .fold(ListSign::Empty, |acc, n| {
                match (acc, n.is_positive()) {
                    (ListSign::Empty, true) | (ListSign::Positive, true) => ListSign::Positive,
                    (ListSign::Empty, false) | (ListSign::Negative, false) => ListSign::Negative,
                    _ => ListSign::Mixed
                }
            });
        convex != ListSign::Mixed
    }
}

/// A collection of [`Polygon`s](struct.Polygon.html)
///
/// Can be created from a `Vec` of `Polygon`s, or `collect`ed from an Iterator which yields `Polygon`s.
///
/// Iterating over this object yields the component Polygons.
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiPolygon<T>(pub Vec<Polygon<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType, IP: Into<Polygon<T>>> From<IP> for MultiPolygon<T> {
    fn from(x: IP) -> Self {
        MultiPolygon(vec![x.into()])
    }
}

impl<T: CoordinateType, IP: Into<Polygon<T>>> FromIterator<IP> for MultiPolygon<T> {
    fn from_iter<I: IntoIterator<Item = IP>>(iter: I) -> Self {
        MultiPolygon(iter.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordinateType> IntoIterator for MultiPolygon<T> {
    type Item = Polygon<T>;
    type IntoIter = ::std::vec::IntoIter<Polygon<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A collection of [`Geometry`](enum.Geometry.html) types
///
/// Can be created from a `Vec` of Geometries, or from an Iterator which yields Geometries.
///
/// Iterating over this objects, yields the component Geometries.
#[derive(PartialEq, Clone, Debug)]
pub struct GeometryCollection<T>(pub Vec<Geometry<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType, IG: Into<Geometry<T>>> From<IG> for GeometryCollection<T> {
    fn from(x: IG) -> Self {
        GeometryCollection(vec![x.into()])
    }
}

impl<T: CoordinateType, IG: Into<Geometry<T>>> FromIterator<IG> for GeometryCollection<T> {
    fn from_iter<I: IntoIterator<Item = IG>>(iter: I) -> Self {
        GeometryCollection(iter.into_iter().map(|g| g.into()).collect())
    }
}

impl<T: CoordinateType> IntoIterator for GeometryCollection<T> {
    type Item = Geometry<T>;
    type IntoIter = ::std::vec::IntoIter<Geometry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// An enum representing any possible geometry type.
///
/// All `Geo` types can be converted to a `Geometry` member using `.into()` (as part of the
/// `std::convert::Into` pattern).
#[derive(PartialEq, Clone, Debug)]
pub enum Geometry<T>
where
    T: CoordinateType,
{
    Point(Point<T>),
    Line(Line<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
}

impl<T: CoordinateType> From<Point<T>> for Geometry<T> {
    fn from(x: Point<T>) -> Geometry<T> {
        Geometry::Point(x)
    }
}
impl<T: CoordinateType> From<LineString<T>> for Geometry<T> {
    fn from(x: LineString<T>) -> Geometry<T> {
        Geometry::LineString(x)
    }
}
impl<T: CoordinateType> From<Polygon<T>> for Geometry<T> {
    fn from(x: Polygon<T>) -> Geometry<T> {
        Geometry::Polygon(x)
    }
}
impl<T: CoordinateType> From<MultiPoint<T>> for Geometry<T> {
    fn from(x: MultiPoint<T>) -> Geometry<T> {
        Geometry::MultiPoint(x)
    }
}
impl<T: CoordinateType> From<MultiLineString<T>> for Geometry<T> {
    fn from(x: MultiLineString<T>) -> Geometry<T> {
        Geometry::MultiLineString(x)
    }
}
impl<T: CoordinateType> From<MultiPolygon<T>> for Geometry<T> {
    fn from(x: MultiPolygon<T>) -> Geometry<T> {
        Geometry::MultiPolygon(x)
    }
}

impl<T: CoordinateType> Geometry<T> {
    /// If this Geometry is a Point, then return that, else None.
    ///
    /// ```
    /// use geo_types::*;
    /// let g = Geometry::Point(Point::new(0., 0.));
    /// let p2: Point<f32> = g.as_point().unwrap();
    /// assert_eq!(p2, Point::new(0., 0.,));
    /// ```
    pub fn as_point(self) -> Option<Point<T>> {
        if let Geometry::Point(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a LineString, then return that LineString, else None.
    pub fn as_linestring(self) -> Option<LineString<T>> {
        if let Geometry::LineString(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a Line, then return that Line, else None.
    pub fn as_line(self) -> Option<Line<T>> {
        if let Geometry::Line(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a Polygon, then return that, else None.
    pub fn as_polygon(self) -> Option<Polygon<T>> {
        if let Geometry::Polygon(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a MultiPoint, then return that, else None.
    pub fn as_multipoint(self) -> Option<MultiPoint<T>> {
        if let Geometry::MultiPoint(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a MultiLineString, then return that, else None.
    pub fn as_multilinestring(self) -> Option<MultiLineString<T>> {
        if let Geometry::MultiLineString(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a MultiPolygon, then return that, else None.
    pub fn as_multipolygon(self) -> Option<MultiPolygon<T>> {
        if let Geometry::MultiPolygon(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn type_test() {
        let c = Coordinate {
            x: 40.02f64,
            y: 116.34,
        };

        let p = Point(c);

        let Point(c2) = p;
        assert_eq!(c, c2);
        assert_eq!(c.x, c2.x);
        assert_eq!(c.y, c2.y);

        let p: Point<f32> = (0f32, 1f32).into();
        assert_eq!(p.x(), 0.);
        assert_eq!(p.y(), 1.);
    }

    #[test]
    fn convert_types() {
        let p: Point<f32> = Point::new(0., 0.);
        let p1 = p.clone();
        let g: Geometry<f32> = p.into();
        let p2 = g.as_point().unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn polygon_new_test() {
        let exterior = LineString(vec![
            Point::new(0., 0.),
            Point::new(1., 1.),
            Point::new(1., 0.),
            Point::new(0., 0.),
        ]);
        let interiors = vec![LineString(vec![
            Point::new(0.1, 0.1),
            Point::new(0.9, 0.9),
            Point::new(0.9, 0.1),
            Point::new(0.1, 0.1),
        ])];
        let p = Polygon::new(exterior.clone(), interiors.clone());

        assert_eq!(p.exterior, exterior);
        assert_eq!(p.interiors, interiors);
    }

    #[test]
    fn iters() {
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();

        let _: LineString<_> = vec![(0., 0.), (1., 2.)].into();
        let _: LineString<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();
    }

    #[test]
    fn test_coordinate_types() {
        let p: Point<u8> = Point::new(0, 0);
        assert_eq!(p.x(), 0u8);

        let p: Point<i64> = Point::new(1_000_000, 0);
        assert_eq!(p.x(), 1_000_000i64);
    }

    #[cfg(feature = "spade")]
    #[test]
    /// ensure Line's SpatialObject impl is correct
    fn line_test() {
        use spade::primitives::SimpleEdge;
        let se = SimpleEdge::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        assert_eq!(se.mbr(), l.mbr());
        // difference in 15th decimal place
        assert_eq!(26.0, se.distance2(&Point::new(4.0, 10.0)));
        assert_eq!(25.999999999999996, l.distance2(&Point::new(4.0, 10.0)));
    }
}
