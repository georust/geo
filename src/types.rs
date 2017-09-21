use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::cmp::Ordering;

use algorithm::orient::{signed_ring_area, WindingOrder};

use std::fmt::Debug;

use std::iter::{self, Iterator, FromIterator};

use num_traits::{Float, ToPrimitive};
use spade::SpadeNum;
use spade::PointN;

pub static COORD_PRECISION: f32 = 1e-1; // 0.1m

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Coordinate<T>
    where T: Float
{
    pub x: T,
    pub y: T,
}

impl<T: Float> From<(T, T)> for Coordinate<T> {
    fn from(coords: (T, T)) -> Self {
        Coordinate{ x: coords.0, y: coords.1 }
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Bbox<T>
    where T: Float
{
    pub xmin: T,
    pub xmax: T,
    pub ymin: T,
    pub ymax: T,
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Extremes {
    pub ymin: usize,
    pub xmax: usize,
    pub ymax: usize,
    pub xmin: usize,
}

impl From<Vec<usize>> for Extremes {
    fn from(original: Vec<usize>) -> Extremes {
        Extremes {
            ymin: original[0],
            xmax: original[1],
            ymax: original[2],
            xmin: original[3],
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ExtremePoint<T>
    where T: Float
 {
    pub ymin: Point<T>,
    pub xmax: Point<T>,
    pub ymax: Point<T>,
    pub xmin: Point<T>,
}

/// A single Point in 2D space.
///
/// Points can be created using the `new(x, y)` constructor, or from a `Coordinate` or pair of points.
///
/// ```
/// use geo::{Point, Coordinate};
/// let p1: Point<f64> = (0., 1.).into();
/// let c = Coordinate{ x: 10., y: 20.};
/// let p2: Point<f64> = c.into();
/// ```
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Point<T> (pub Coordinate<T>) where T: Float;

impl<T: Float> From<Coordinate<T>> for Point<T> {
    fn from(x: Coordinate<T>) -> Point<T> {
        Point(x)
    }
}

impl<T: Float> From<(T, T)> for Point<T> {
    fn from(coords: (T, T)) -> Point<T> {
        Point::new(coords.0, coords.1)
    }
}

impl<T> Point<T>
    where T: Float + ToPrimitive
{
    /// Creates a new point.
    ///
    /// ```
    /// use geo::Point;
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
    /// use geo::Point;
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
    /// use geo::Point;
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
    /// use geo::Point;
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
    /// use geo::Point;
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
    /// use geo::Point;
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
    /// use geo::Point;
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
    /// use geo::Point;
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
    /// use geo::Point;
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
    /// use geo::Point;
    ///
    /// let p = Point::new(1.5, 0.5);
    /// let dot = p.dot(&Point::new(2.0, 4.5));
    ///
    /// assert_eq!(dot, 5.25);
    /// ```
    pub fn dot(&self, point: &Point<T>) -> T {
        self.x() * point.x() + self.y() * point.y()
    }
}

impl<T> Neg for Point<T>
    where T: Float + Neg<Output = T> + ToPrimitive
{
    type Output = Point<T>;

    /// Returns a point with the x and y components negated.
    ///
    /// ```
    /// use geo::Point;
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
    where T: Float + ToPrimitive
{
    type Output = Point<T>;

    /// Add a point to the given point.
    ///
    /// ```
    /// use geo::Point;
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
    where T: Float + ToPrimitive
{
    type Output = Point<T>;

    /// Subtract a point from the given point.
    ///
    /// ```
    /// use geo::Point;
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

// These are required for Spade RTree
impl<T> PointN for Point<T>
    where T: Float + SpadeNum + Debug
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
            _ => unreachable!()
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.0.x,
            1 => &mut self.0.y,
            _ => unreachable!()
        }
    }
}

impl<T> Add for Bbox<T>
    where T: Float + ToPrimitive
{
    type Output = Bbox<T>;

    /// Add a BoundingBox to the given BoundingBox.
    ///
    /// ```
    /// use geo::Bbox;
    ///
    /// let bbox0 = Bbox{xmin: 0.,  xmax: 10000., ymin: 10., ymax: 100.};
    /// let bbox1 = Bbox{xmin: 100., xmax: 1000.,  ymin: 100.,  ymax: 1000.};
    /// let bbox = bbox0 + bbox1;
    ///
    /// assert_eq!(0., bbox.xmin);
    /// assert_eq!(10000., bbox.xmax);
    /// assert_eq!(10., bbox.ymin);
    /// assert_eq!(1000., bbox.ymax);
    /// ```
    fn add(self, rhs: Bbox<T>) -> Bbox<T> {
        Bbox{
            xmin: if self.xmin <= rhs.xmin {self.xmin} else {rhs.xmin},
            xmax: if self.xmax >= rhs.xmax {self.xmax} else {rhs.xmax},
            ymin: if self.ymin <= rhs.ymin {self.ymin} else {rhs.ymin},
            ymax: if self.ymax >= rhs.ymax {self.ymax} else {rhs.ymax},
        }
    }
}

impl<T> AddAssign for Bbox<T>
    where T: Float + ToPrimitive
{
    /// Add a BoundingBox to the given BoundingBox.
    ///
    /// ```
    /// use geo::Bbox;
    ///
    /// let mut bbox0 = Bbox{xmin: 0.,  xmax: 10000., ymin: 10., ymax: 100.};
    /// let bbox1 = Bbox{xmin: 100., xmax: 1000.,  ymin: 100.,  ymax: 1000.};
    /// bbox0 += bbox1;
    ///
    /// assert_eq!(0., bbox0.xmin);
    /// assert_eq!(10000., bbox0.xmax);
    /// assert_eq!(10., bbox0.ymin);
    /// assert_eq!(1000., bbox0.ymax);
    /// ```
    fn add_assign(&mut self, rhs: Bbox<T>){
        self.xmin = if self.xmin <= rhs.xmin {self.xmin} else {rhs.xmin};
        self.xmax = if self.xmax >= rhs.xmax {self.xmax} else {rhs.xmax};
        self.ymin = if self.ymin <= rhs.ymin {self.ymin} else {rhs.ymin};
        self.ymax = if self.ymax >= rhs.ymax {self.ymax} else {rhs.ymax};
    }
}


/// A collection of [`Point`s](struct.Point.html).
///
/// Iterating over a `MultiPoint` yields the `Point`s inside.
///
/// ```
/// use geo::{MultiPoint, Point};
/// let points: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
/// for point in points {
///     println!("Point x = {}, y = {}", point.x(), point.y());
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MultiPoint<T>(pub Vec<Point<T>>) where T: Float;

impl<T: Float, IP: Into<Point<T>>> From<IP> for MultiPoint<T> {
    /// Convert a single `Point` (or something which can be converted to a `Point`) into a
    /// one-member `MultiPoint`
    fn from(x: IP) -> MultiPoint<T> {
        MultiPoint(vec![x.into()])
    }
}

impl<T: Float, IP: Into<Point<T>>> From<Vec<IP>> for MultiPoint<T> {
    /// Convert a `Vec` of `Points` (or `Vec` of things which can be converted to a `Point`) into a
    /// `MultiPoint`.
    fn from(v: Vec<IP>) -> MultiPoint<T> {
        MultiPoint(v.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: Float, IP: Into<Point<T>>> FromIterator<IP> for MultiPoint<T> {
    /// Collect the results of a `Point` iterator into a `MultiPoint`
    fn from_iter<I: IntoIterator<Item=IP>>(iter: I) -> Self {
        MultiPoint(iter.into_iter().map(|p| p.into()).collect())
    }
}

/// Iterate over the `Point`s in this `MultiPoint`.
impl<T: Float> IntoIterator for MultiPoint<T> {
    type Item = Point<T>;
    type IntoIter = ::std::vec::IntoIter<Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Line<T>
    where T: Float
{
    pub start: Point<T>,
    pub end: Point<T>
}

impl<T> Line<T>
    where T: Float
{
    /// Creates a new line segment.
    ///
    /// ```
    /// use geo::{Point, Line};
    ///
    /// let line = Line::new(Point::new(0., 0.), Point::new(1., 2.));
    ///
    /// assert_eq!(line.start, Point::new(0., 0.));
    /// assert_eq!(line.end, Point::new(1., 2.));
    /// ```
    pub fn new(start: Point<T>, end: Point<T>) -> Line<T> {
        Line {start: start, end: end}
    }
}

/// A LineString, which is an ordered collection of [`Point`s](struct.Point.html).
///
/// Create a LineString by calling it directly:
///
/// ```
/// use geo::{LineString, Point};
/// let line = LineString(vec![Point::new(0., 0.), Point::new(10., 0.)]);
/// ```
///
/// Converting a `Vec` of `Point`-like things:
///
/// ```
/// # use geo::{LineString, Point};
/// let line: LineString<f32> = vec![(0., 0.), (10., 0.)].into();
/// ```
///
/// Or `collect`ing from a Point iterator
///
/// ```
/// # use geo::{LineString, Point};
/// let mut points = vec![Point::new(0., 0.), Point::new(10., 0.)];
/// let line: LineString<f32> = points.into_iter().collect();
/// ```
///
/// You can iterate over the points in the `LineString`
///
/// ```
/// use geo::{LineString, Point};
/// let line = LineString(vec![Point::new(0., 0.), Point::new(10., 0.)]);
/// for point in line {
///     println!("Point x = {}, y = {}", point.x(), point.y());
/// }
/// ```
///
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct LineString<T>(pub Vec<Point<T>>) where T: Float;

impl<T: Float> LineString<T> {
    /// Return an `Line` iterator that yields one `Line` for each line segment
    /// in the `LineString`.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Line, LineString, Point};
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
    pub fn lines<'a>(&'a self) -> Box<Iterator<Item = Line<T>> + 'a> {
        if self.0.len() < 2 {
            return Box::new(iter::empty());
        }
        Box::new(self.0.windows(2).map(|w| unsafe {
            // As long as the LineString has at least two points, we shouldn't
            // need to do bounds checking here.
            Line::new(*w.get_unchecked(0), *w.get_unchecked(1))
        }))
    }
}

/// Turn a `Vec` of `Point`-ish objects into a `LineString`.
impl<T: Float, IP: Into<Point<T>>> From<Vec<IP>> for LineString<T> {
    fn from(v: Vec<IP>) -> Self {
        LineString(v.into_iter().map(|p| p.into()).collect())
    }
}

/// Turn a `Point`-ish iterator into a `LineString`.
impl<T: Float, IP: Into<Point<T>>> FromIterator<IP> for LineString<T> {
    fn from_iter<I: IntoIterator<Item=IP>>(iter: I) -> Self {
        LineString(iter.into_iter().map(|p| p.into()).collect())
    }
}

/// Iterate over all the [Point](struct.Point.html)s in this linestring
impl<T: Float> IntoIterator for LineString<T> {
    type Item = Point<T>;
    type IntoIter = ::std::vec::IntoIter<Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Float> LineString<T> {

    /// Returns the winding order of this line. None if there is no valid winding order
    pub fn winding_order(&self) -> Option<WindingOrder> {
        match signed_ring_area(self).partial_cmp(&T::zero()) {
            None => None,
            Some(Ordering::Equal) => None,
            Some(Ordering::Less) => Some(WindingOrder::Clockwise),
            Some(Ordering::Greater) => Some(WindingOrder::CounterClockwise),
        }
    }

    /// True iff this line is wound clockwise, false if it's counter-clockwise or there's no valid
    /// winding order
    pub fn is_cw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::Clockwise)
    }

    /// True iff this line is wound counter-clockwise, false if it's clockwise or there's no valid
    /// winding order
    pub fn is_ccw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::CounterClockwise)
    }

    /// Iterate over the points in a clockwise order
    ///
    /// The Linestring isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear clockwise.
    ///
    /// If there is no valid winding order, what points there are, are returned in the order they
    /// are stored.
    pub fn points_clockwise<'a>(&'a self) -> Box<Iterator<Item=&'a Point<T>> + 'a> {
        match self.winding_order() {
            Some(WindingOrder::Clockwise) => Box::new(self.0.iter()),
            Some(WindingOrder::CounterClockwise) => Box::new(self.0.iter().rev()),
            None => Box::new(self.0.iter()),
        }
    }

    /// Iterate over the points in a counter-clockwise order
    ///
    /// The Linestring isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear counter-clockwise
    ///
    /// If there is no valid winding order, what points there are, are returned in the order they
    /// are stored.
    pub fn points_counterclockwise<'a>(&'a self) -> Box<Iterator<Item=&'a Point<T>> + 'a> {
        match self.winding_order() {
            Some(WindingOrder::Clockwise) => Box::new(self.0.iter().rev()),
            Some(WindingOrder::CounterClockwise) => Box::new(self.0.iter()),
            None => Box::new(self.0.iter()),
        }
    }

    /// Return a clone of this linestring, but in the specified winding order
    pub fn clone_to_winding_order(&self, winding_order: WindingOrder) -> Self {
        let mut new = self.clone();
        new.make_winding_order(winding_order);
        new
    }

    /// Change the winding order so that it is in this winding order
    pub fn make_winding_order(&mut self, winding_order: WindingOrder) {
        match winding_order {
            WindingOrder::Clockwise => self.make_clockwise_winding(),
            WindingOrder::CounterClockwise => self.make_counterclockwise_winding(),
        }
    }

    /// Change this line's points so they are in clockwise winding order
    ///
    /// If there is no valid winding order, the line isn't changed.
    pub fn make_clockwise_winding(&mut self) {
        match self.winding_order() {
            Some(WindingOrder::Clockwise) => {},
            Some(WindingOrder::CounterClockwise) => {
                self.0.reverse();
            },
            None => {},
        }
    }

    /// Change this line's points so they are in counterclockwise winding order
    ///
    /// If there is no valid winding order, the line isn't changed.
    pub fn make_counterclockwise_winding(&mut self) {
        match self.winding_order() {
            Some(WindingOrder::Clockwise) => {
                self.0.reverse();
            },
            Some(WindingOrder::CounterClockwise) => {},
            None => {},
        }
    }
}

/// A collection of [`LineString`s](struct.LineString.html).
///
/// Can be created from a `Vec` of `LineString`s, or from an Iterator which yields LineStrings.
///
/// Iterating over this objects, yields the component LineStrings.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MultiLineString<T>(pub Vec<LineString<T>>) where T: Float;

impl<T: Float, ILS: Into<LineString<T>>> From<ILS> for MultiLineString<T> {
    fn from(ls: ILS) -> Self {
        MultiLineString(vec![ls.into()])
    }
}

impl<T: Float, ILS: Into<LineString<T>>> FromIterator<ILS> for MultiLineString<T> {
    fn from_iter<I: IntoIterator<Item=ILS>>(iter: I) -> Self {
        MultiLineString(iter.into_iter().map(|ls| ls.into()).collect())
    }
}

impl<T: Float> IntoIterator for MultiLineString<T> {
    type Item = LineString<T>;
    type IntoIter = ::std::vec::IntoIter<LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A 2D polygon area.
///
/// It has one exterior ring, and zero or more interior rings.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Polygon<T>
    where T: Float
{
    pub exterior: LineString<T>,
    pub interiors: Vec<LineString<T>>
}

impl<T> Polygon<T>
    where T: Float
{
    /// Creates a new polygon.
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
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
        Polygon { exterior: exterior, interiors: interiors }
    }
}

/// A collection of [`Polygon`s](struct.Polygon.html).
///
/// Can be created from a `Vec` of `Polygon`s, or `collect`ed from an Iterator which yields `Polygon`s.
///
/// Iterating over this objects, yields the component Polygons.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MultiPolygon<T>(pub Vec<Polygon<T>>) where T: Float;

impl<T: Float, IP: Into<Polygon<T>>> From<IP> for MultiPolygon<T> {
    fn from(x: IP) -> Self {
        MultiPolygon(vec![x.into()])
    }
}

impl<T: Float, IP: Into<Polygon<T>>> FromIterator<IP> for MultiPolygon<T> {
    fn from_iter<I: IntoIterator<Item=IP>>(iter: I) -> Self {
        MultiPolygon(iter.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: Float> IntoIterator for MultiPolygon<T> {
    type Item = Polygon<T>;
    type IntoIter = ::std::vec::IntoIter<Polygon<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A collection of [`Geometry`s](enum.Geometry.html).
///
/// Can be created from a `Vec` of Geometries, or from an Iterator which yields Geometries.
///
/// Iterating over this objects, yields the component Geometries.
#[derive(PartialEq, Clone, Debug)]
pub struct GeometryCollection<T>(pub Vec<Geometry<T>>) where T: Float;

impl<T: Float, IG: Into<Geometry<T>>> From<IG> for GeometryCollection<T> {
    fn from(x: IG) -> Self {
        GeometryCollection(vec![x.into()])
    }
}

impl<T: Float, IG: Into<Geometry<T>>> FromIterator<IG> for GeometryCollection<T> {
    fn from_iter<I: IntoIterator<Item=IG>>(iter: I) -> Self {
        GeometryCollection(iter.into_iter().map(|g| g.into()).collect())
    }
}

impl<T: Float> IntoIterator for GeometryCollection<T> {
    type Item = Geometry<T>;
    type IntoIter = ::std::vec::IntoIter<Geometry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// An enum representing any possible geomtry type.
///
/// All types can be converted to a `Geometry` using the `.into()` (as part of the
/// `std::convert::Into` pattern).
#[derive(PartialEq, Clone, Debug)]
pub enum Geometry<T>
    where T: Float
{
    Point(Point<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>)
}

impl<T: Float> From<Point<T>> for Geometry<T> { fn from(x: Point<T>) -> Geometry<T> { Geometry::Point(x) } }
impl<T: Float> From<LineString<T>> for Geometry<T> { fn from(x: LineString<T>) -> Geometry<T> { Geometry::LineString(x) } }
impl<T: Float> From<Polygon<T>> for Geometry<T> { fn from(x: Polygon<T>) -> Geometry<T> { Geometry::Polygon(x) } }
impl<T: Float> From<MultiPoint<T>> for Geometry<T> { fn from(x: MultiPoint<T>) -> Geometry<T> { Geometry::MultiPoint(x) } }
impl<T: Float> From<MultiLineString<T>> for Geometry<T> { fn from(x: MultiLineString<T>) -> Geometry<T> { Geometry::MultiLineString(x) } }
impl<T: Float> From<MultiPolygon<T>> for Geometry<T> { fn from(x: MultiPolygon<T>) -> Geometry<T> { Geometry::MultiPolygon(x) } }

#[cfg(test)]
mod test {
    use ::types::*;

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
    fn polygon_new_test() {
        let exterior = LineString(vec![Point::new(0., 0.), Point::new(1., 1.),
                                       Point::new(1., 0.), Point::new(0., 0.)]);
        let interiors = vec![LineString(vec![Point::new(0.1, 0.1), Point::new(0.9, 0.9),
                                             Point::new(0.9, 0.1), Point::new(0.1, 0.1)])];
        let p = Polygon::new(exterior.clone(), interiors.clone());

        assert_eq!(p.exterior, exterior);
        assert_eq!(p.interiors, interiors);
        }

    #[test]
    fn winding_order() {
        // 3 points forming a triangle
        let a = Point::new(0., 0.);
        let b = Point::new(2., 0.);
        let c = Point::new(1., 2.);

        // That triangle, but in clockwise ordering
        let cw_line = LineString(vec![a, c, b, a].clone());
        // That triangle, but in counterclockwise ordering
        let ccw_line = LineString(vec![a, b, c, a].clone());

        assert_eq!(cw_line.winding_order(), Some(WindingOrder::Clockwise));
        assert_eq!(cw_line.is_cw(), true);
        assert_eq!(cw_line.is_ccw(), false);
        assert_eq!(ccw_line.winding_order(), Some(WindingOrder::CounterClockwise));
        assert_eq!(ccw_line.is_cw(), false);
        assert_eq!(ccw_line.is_ccw(), true);

        let cw_points1: Vec<_> = cw_line.points_clockwise().cloned().collect();
        assert_eq!(cw_points1.len(), 4);
        assert_eq!(cw_points1[0], a);
        assert_eq!(cw_points1[1], c);
        assert_eq!(cw_points1[2], b);
        assert_eq!(cw_points1[3], a);

        let ccw_points1: Vec<_> = cw_line.points_counterclockwise().cloned().collect();
        assert_eq!(ccw_points1.len(), 4);
        assert_eq!(ccw_points1[0], a);
        assert_eq!(ccw_points1[1], b);
        assert_eq!(ccw_points1[2], c);
        assert_eq!(ccw_points1[3], a);

        assert_ne!(cw_points1, ccw_points1);

        let cw_points2: Vec<_> = ccw_line.points_clockwise().cloned().collect();
        let ccw_points2: Vec<_> = ccw_line.points_counterclockwise().cloned().collect();

        // cw_line and ccw_line are wound differently, but the ordered winding iterator should have
        // make them similar
        assert_eq!(cw_points2, cw_points2);
        assert_eq!(ccw_points2, ccw_points2);

        // test make_clockwise_winding
        let mut new_line1 = ccw_line.clone();
        new_line1.make_clockwise_winding();
        assert_eq!(new_line1.winding_order(), Some(WindingOrder::Clockwise));
        assert_eq!(new_line1, cw_line);
        assert_ne!(new_line1, ccw_line);

        // test make_counterclockwise_winding
        let mut new_line2 = cw_line.clone();
        new_line2.make_counterclockwise_winding();
        assert_eq!(new_line2.winding_order(), Some(WindingOrder::CounterClockwise));
        assert_ne!(new_line2, cw_line);
        assert_eq!(new_line2, ccw_line);

        // There isn't always a valid winding order, here are cases where it's no valid
        assert_eq!(LineString::<f32>(vec![]).winding_order(), None);
        assert_eq!(LineString(vec![Point::new(0., 0.)]).winding_order(), None);
        assert_eq!(LineString(vec![Point::new(0., 0.), Point::new(1., 0.)]).winding_order(), None);

    }

    #[test]
    fn iters() {
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();

        let _: LineString<_> = vec![(0., 0.), (1., 2.)].into();
        let _: LineString<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();
    }
}
