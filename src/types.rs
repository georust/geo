use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Neg;
use std::ops::Sub;

use num_traits::{Float, ToPrimitive};

use algorithm::shoelace_formula::twice_area;

pub static COORD_PRECISION: f32 = 1e-1; // 0.1m

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Coordinate<T>
    where T: Float
{
    pub x: T,
    pub y: T,
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

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Point<T> (pub Coordinate<T>) where T: Float;

impl<T: Float> From<Coordinate<T>> for Point<T> { fn from(x: Coordinate<T>) -> Point<T> { Point(x) } }

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


#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MultiPoint<T>(pub Vec<Point<T>>) where T: Float;

impl<T: Float> From<Point<T>> for MultiPoint<T> { fn from(x: Point<T>) -> MultiPoint<T> { MultiPoint(vec![x]) } }

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
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

#[derive(PartialEq, Clone, Debug)]
pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct LineString<T>(pub Vec<Point<T>>) where T: Float;

impl<T: Float> LineString<T> {

    /// Returns the winding order of this line
    pub fn winding_order(&self) -> WindingOrder {
        let shoelace = twice_area(self);
        if shoelace < T::zero() {
            WindingOrder::Clockwise
        } else if shoelace > T::zero() {
            WindingOrder::CounterClockwise
        } else if shoelace == T::zero() {
            // what should be done here?
            panic!();
        } else {
            // make compiler stop complaining
            unreachable!()
        }
    }

    /// True iff this line is wound clockwise
    pub fn is_cw(&self) -> bool {
        self.winding_order() == WindingOrder::Clockwise
    }

    /// True iff this line is wound counterclockwise
    pub fn is_ccw(&self) -> bool {
        self.winding_order() == WindingOrder::CounterClockwise
    }

    /// Iterate over the points in a clockwise order
    ///
    /// The Linestring isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear clockwise
    pub fn points_clockwise<'a>(&'a self) -> Box<Iterator<Item=&'a Point<T>> + 'a> {
        match self.winding_order() {
            WindingOrder::Clockwise => Box::new(self.0.iter()),
            WindingOrder::CounterClockwise => Box::new(self.0.iter().rev()),
        }
    }

    /// Iterate over the points in a counter-clockwise order
    ///
    /// The Linestring isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear counter-clockwise
    pub fn points_counterclockwise<'a>(&'a self) -> Box<Iterator<Item=&'a Point<T>> + 'a> {
        match self.winding_order() {
            WindingOrder::Clockwise => Box::new(self.0.iter().rev()),
            WindingOrder::CounterClockwise => Box::new(self.0.iter()),
        }
    }

    /// Change this line's points so they are in clockwise winding order
    pub fn make_clockwise_winding(&mut self) {
        match self.winding_order() {
            WindingOrder::Clockwise => {},
            WindingOrder::CounterClockwise => {
                self.0.reverse();
            }
        }
    }

    /// Change this line's points so they are in counterclockwise winding order
    pub fn make_counterclockwise_winding(&mut self) {
        match self.winding_order() {
            WindingOrder::Clockwise => {
                self.0.reverse();
            },
            WindingOrder::CounterClockwise => {}
        }
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MultiLineString<T>(pub Vec<LineString<T>>) where T: Float;

impl<T: Float> From<LineString<T>> for MultiLineString<T> { fn from(x: LineString<T>) -> MultiLineString<T> { MultiLineString(vec![x]) } }

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

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MultiPolygon<T>(pub Vec<Polygon<T>>) where T: Float;

impl<T: Float> From<Polygon<T>> for MultiPolygon<T> { fn from(x: Polygon<T>) -> MultiPolygon<T> { MultiPolygon(vec![x]) } }

#[derive(PartialEq, Clone, Debug)]
pub struct GeometryCollection<T>(pub Vec<Geometry<T>>) where T: Float;

impl<T: Float> From<Geometry<T>> for GeometryCollection<T> { fn from(x: Geometry<T>) -> GeometryCollection<T> { GeometryCollection(vec![x]) } }

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
impl<T: Float> From<GeometryCollection<T>> for Geometry<T> { fn from(x: GeometryCollection<T>) -> Geometry<T> { Geometry::GeometryCollection(x) } }

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

        assert_eq!(cw_line.winding_order(), WindingOrder::Clockwise);
        assert_eq!(cw_line.is_cw(), true);
        assert_eq!(cw_line.is_ccw(), false);
        assert_eq!(ccw_line.winding_order(), WindingOrder::CounterClockwise);
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
        assert_eq!(new_line1.winding_order(), WindingOrder::Clockwise);
        assert_eq!(new_line1, cw_line);
        assert_ne!(new_line1, ccw_line);

        // test make_counterclockwise_winding
        let mut new_line2 = cw_line.clone();
        new_line2.make_counterclockwise_winding();
        assert_eq!(new_line2.winding_order(), WindingOrder::CounterClockwise);
        assert_ne!(new_line2, cw_line);
        assert_eq!(new_line2, ccw_line);

    }
}
