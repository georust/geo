use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Neg;
use std::ops::Sub;

use cgmath::Angle;
use num_traits::{Float, ToPrimitive};

pub static COORD_PRECISION: f32 = 1e-1; // 0.1m

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Coordinate<T> {
    pub x: T,
    pub y: T,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Bbox<T> {
    pub xmin: T,
    pub xmax: T,
    pub ymin: T,
    pub ymax: T,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Point<T> (pub Coordinate<T>);

impl<T> Point<T> {
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
}

impl<T> Point<T>
    where T: Copy
{
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
}

impl<T> Point<T>
    where T: Angle
{
    /// Returns the longitude/horizontal component of the point.
    ///
    /// ```
    /// # extern crate geo;
    /// # extern crate cgmath;
    /// #
    /// use geo::Point;
    /// use cgmath::Deg;
    ///
    /// # fn main() {
    /// let p = Point::new(Deg(1.234), Deg(2.345));
    ///
    /// assert_eq!(p.lng(), Deg(1.234));
    /// # }
    /// ```
    pub fn lng(&self) -> T {
        self.x()
    }

    /// Sets the longitude/horizontal component of the point.
    ///
    /// ```
    /// # extern crate geo;
    /// # extern crate cgmath;
    /// #
    /// use geo::Point;
    /// use cgmath::Deg;
    ///
    /// # fn main() {
    /// let mut p = Point::new(Deg(1.234), Deg(2.345));
    /// p.set_lng(Deg(9.876));
    ///
    /// assert_eq!(p.lng(), Deg(9.876));
    /// # }
    /// ```
    pub fn set_lng(&mut self, lng: T) -> &mut Point<T> {
        self.set_x(lng)
    }

    /// Returns the latitude/vertical component of the point.
    ///
    /// ```
    /// # extern crate geo;
    /// # extern crate cgmath;
    /// #
    /// use geo::Point;
    /// use cgmath::Deg;
    ///
    /// # fn main() {
    /// let p = Point::new(Deg(1.234), Deg(2.345));
    ///
    /// assert_eq!(p.lat(), Deg(2.345));
    /// # }
    /// ```
    pub fn lat(&self) -> T {
        self.y()
    }

    /// Sets the latitude/vertical component of the point.
    ///
    /// ```
    /// # extern crate geo;
    /// # extern crate cgmath;
    /// #
    /// use geo::Point;
    /// use cgmath::Deg;
    ///
    /// # fn main() {
    /// let mut p = Point::new(Deg(1.234), Deg(2.345));
    /// p.set_lat(Deg(9.876));
    ///
    /// assert_eq!(p.lat(), Deg(9.876));
    /// # }
    /// ```
    pub fn set_lat(&mut self, lat: T) -> &mut Point<T> {
        self.set_y(lat)
    }
}

impl<T> Point<T>
    where T: Float + ToPrimitive
{
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
    where T: Neg + Copy
{
    type Output = Point<T::Output>;

    /// Returns a point with the x and y components negated:
    ///
    /// ```
    /// use geo::Point;
    ///
    /// let p = -Point::new(-1.25, 2.5);
    ///
    /// assert_eq!(p.x(), 1.25);
    /// assert_eq!(p.y(), -2.5);
    /// ```
    ///
    /// or using angles:
    ///
    /// ```
    /// # extern crate geo;
    /// # extern crate cgmath;
    /// #
    /// use geo::Point;
    /// use cgmath::Deg;
    ///
    /// # fn main() {
    /// let p = -Point::new(Deg(1.234), Deg(-2.345));
    ///
    /// assert_eq!(p.lng(), Deg(-1.234));
    /// assert_eq!(p.lat(), Deg(2.345));
    /// # }
    /// ```
    fn neg(self) -> Self::Output {
        Point::new(-self.x(), -self.y())
    }
}

impl<T> Add for Point<T>
    where T: Add + Copy
{
    type Output = Point<T::Output>;

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
    ///
    /// or using angles:
    ///
    /// ```
    /// # extern crate geo;
    /// # extern crate cgmath;
    /// #
    /// use geo::Point;
    /// use cgmath::Deg;
    ///
    /// # fn main() {
    /// let p = Point::new(Deg(1.25), Deg(2.5)) + Point::new(Deg(1.5), Deg(2.5));
    ///
    /// assert_eq!(p.lng(), Deg(2.75));
    /// assert_eq!(p.lat(), Deg(5.0));
    /// # }
    /// ```
    fn add(self, rhs: Point<T>) -> Self::Output {
        Point::new(self.x() + rhs.x(), self.y() + rhs.y())
    }
}

impl<T> Sub for Point<T>
    where T: Sub + Copy
{
    type Output = Point<T::Output>;

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
    ///
    /// or using angles:
    ///
    /// ```
    /// # extern crate geo;
    /// # extern crate cgmath;
    /// #
    /// use geo::Point;
    /// use cgmath::Deg;
    ///
    /// # fn main() {
    /// let p = Point::new(Deg(1.25), Deg(3.0)) - Point::new(Deg(1.5), Deg(2.5));
    ///
    /// assert_eq!(p.lng(), Deg(-0.25));
    /// assert_eq!(p.lat(), Deg(0.5));
    /// # }
    /// ```
    /// ```
    fn sub(self, rhs: Point<T>) -> Self::Output {
        Point::new(self.x() - rhs.x(), self.y() - rhs.y())
    }
}

impl<T> Add for Bbox<T>
    where T: Float + ToPrimitive
{
    type Output = Bbox<T>;

    /// Add a boundingox to the given boundingbox.
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
    /// Add a boundingox to the given boundingbox.
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


#[derive(PartialEq, Clone, Debug)]
pub struct MultiPoint<T>(pub Vec<Point<T>>);

#[derive(PartialEq, Clone, Debug)]
pub struct LineString<T>(pub Vec<Point<T>>);

#[derive(PartialEq, Clone, Debug)]
pub struct MultiLineString<T>(pub Vec<LineString<T>>);

#[derive(PartialEq, Clone, Debug)]
pub struct Polygon<T> {
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

#[derive(PartialEq, Clone, Debug)]
pub struct MultiPolygon<T>(pub Vec<Polygon<T>>);

#[derive(PartialEq, Clone, Debug)]
pub struct GeometryCollection<T>(pub Vec<Geometry<T>>);

#[derive(PartialEq, Clone, Debug)]
pub enum Geometry<T> {
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
}
