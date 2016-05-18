use std::ops::Add;
use std::ops::Neg;
use std::ops::Sub;

use num::{Num, ToPrimitive};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Coordinate<T>
    where T: Num + Copy
{
    pub x: T,
    pub y: T,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Point<T> (pub Coordinate<T>) where T: Num + Copy;

impl<T> Point<T>
    where T: Num + Copy + ToPrimitive
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
        Point(Coordinate {
            x: x,
            y: y,
        })
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
    /// Returns the distance between two points:
    ///
    /// ```
    /// use geo::Point;
    ///
    /// let p = Point::new(-72.1235, 42.3521);
    /// let dist = p.distance_to(&Point::new(-72.1260, 42.45));
    ///
    /// assert!(dist < 1e-1)
    /// ```
    pub fn distance_to(&self, point: &Point<T>) -> f64 {
        let dx : f64 = self.x().to_f64().unwrap() - point.x().to_f64().unwrap();
        let dy : f64 = self.y().to_f64().unwrap() - point.y().to_f64().unwrap();
        (dx * dx + dy * dy).sqrt()
    }
}

impl<T> Neg for Point<T>
    where T: Num + Neg<Output = T> + Copy + ToPrimitive
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
    where T: Num + Copy + ToPrimitive
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
    where T: Num + Copy + ToPrimitive
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

#[derive(PartialEq, Clone, Debug)]
pub struct MultiPoint<T>(pub Vec<Point<T>>) where T: Num + Copy;

#[derive(PartialEq, Clone, Debug)]
pub struct LineString<T>(pub Vec<Point<T>>) where T: Num + Copy;

#[derive(PartialEq, Clone, Debug)]
pub struct MultiLineString<T>(pub Vec<LineString<T>>) where T: Num + Copy;

#[derive(PartialEq, Clone, Debug)]
pub struct Polygon<T>(pub LineString<T>, pub Vec<LineString<T>>) where T: Num + Copy;

#[derive(PartialEq, Clone, Debug)]
pub struct MultiPolygon<T>(pub Vec<Polygon<T>>) where T: Num + Copy;

#[derive(PartialEq, Clone, Debug)]
pub struct GeometryCollection<T>(pub Vec<Geometry<T>>) where T: Num + Copy;

#[derive(PartialEq, Clone, Debug)]
pub enum Geometry<T>
    where T: Num + Copy
{
    Point(Point<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>)
}

#[cfg(test)]
mod test {
    use ::types::*;

    #[test]
    fn type_test() {
        let c = Coordinate {
            x: 40.02f64,
            y: 116.34
        };

        let p = Point(c);

        let Point(c2) = p;
        assert_eq!(c, c2);
        assert_eq!(c.x, c2.x);
        assert_eq!(c.y, c2.y);
    }
    #[test]
    fn distance_to_test() {
        assert_eq!(Point::new(0., 0.).distance_to(&Point::new(1., 0.)), 1.);
    }
}
