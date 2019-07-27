use crate::{Coordinate, CoordinateType};
use num_traits::Float;
use std::ops::Add;
use std::ops::Neg;
use std::ops::Sub;

/// A single point in 2D space.
///
/// Points can be created using the `new(x, y)` constructor, or from a `Coordinate` or pair of points.
///
/// # Examples
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

impl<T> Point<T>
where
    T: CoordinateType,
{
    /// Creates a new point.
    ///
    /// # Examples
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
        Point(Coordinate { x, y })
    }

    /// Returns the x/horizontal component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.x(), 1.234);
    /// ```
    pub fn x(self) -> T {
        self.0.x
    }

    /// Sets the x/horizontal component of the point.
    ///
    /// # Examples
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
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.y(), 2.345);
    /// ```
    pub fn y(self) -> T {
        self.0.y
    }

    /// Sets the y/vertical component of the point.
    ///
    /// # Examples
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

    /// Returns a tuple that contains the x/horizontal & y/vertical component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let mut p = Point::new(1.234, 2.345);
    /// let (x, y) = p.x_y();
    ///
    /// assert_eq!(y, 2.345);
    /// assert_eq!(x, 1.234);
    /// ```
    pub fn x_y(self) -> (T, T) {
        (self.0.x, self.0.y)
    }
    /// Returns the longitude/horizontal component of the point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.lng(), 1.234);
    /// ```
    pub fn lng(self) -> T {
        self.x()
    }

    /// Sets the longitude/horizontal component of the point.
    ///
    /// # Examples
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
    /// # Examples
    ///
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.lat(), 2.345);
    /// ```
    pub fn lat(self) -> T {
        self.y()
    }
    /// Sets the latitude/vertical component of the point.
    ///
    /// # Examples
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
}

impl<T> Point<T>
where
    T: CoordinateType,
{
    /// Returns the dot product of the two points:
    /// `dot = x1 * x2 + y1 * y2`
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, Point};
    ///
    /// let point = Point(Coordinate { x: 1.5, y: 0.5 });
    /// let dot = point.dot(Point(Coordinate { x: 2.0, y: 4.5 }));
    ///
    /// assert_eq!(dot, 5.25);
    /// ```
    pub fn dot(self, other: Point<T>) -> T {
        self.x() * other.x() + self.y() * other.y()
    }

    /// Returns the cross product of 3 points. A positive value implies
    /// `self` → `point_b` → `point_c` is counter-clockwise, negative implies
    /// clockwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, Point};
    ///
    /// let point_a = Point(Coordinate { x: 1., y: 2. });
    /// let point_b = Point(Coordinate { x: 3., y: 5. });
    /// let point_c = Point(Coordinate { x: 7., y: 12. });
    ///
    /// let cross = point_a.cross_prod(point_b, point_c);
    ///
    /// assert_eq!(cross, 2.0)
    /// ```
    pub fn cross_prod(self, point_b: Point<T>, point_c: Point<T>) -> T {
        (point_b.x() - self.x()) * (point_c.y() - self.y())
            - (point_b.y() - self.y()) * (point_c.x() - self.x())
    }
}

impl<T> Point<T>
where
    T: CoordinateType + Float,
{
    /// Converts the (x,y) components of Point to degrees
    ///
    /// # Example
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    /// let (x,y): (f32, f32) = p.to_degrees().x_y();
    /// assert_eq!(x.round(), 71.0);
    /// assert_eq!(y.round(), 134.0);
    /// ```
    pub fn to_degrees(self) -> Point<T> {
        let (x, y) = self.x_y();
        let x = x.to_degrees();
        let y = y.to_degrees();
        Point::new(x, y)
    }

    /// Converts the (x,y) components of Point to radians
    ///
    /// # Example
    /// ```
    /// use geo_types::Point;
    ///
    /// let p = Point::new(180.0, 341.5);
    /// let (x,y): (f32, f32) = p.to_radians().x_y();
    /// assert_eq!(x.round(), 3.0);
    /// assert_eq!(y.round(), 6.0);
    /// ```
    pub fn to_radians(self) -> Point<T> {
        let (x, y) = self.x_y();
        let x = x.to_radians();
        let y = y.to_radians();
        Point::new(x, y)
    }
}

impl<T> Neg for Point<T>
where
    T: CoordinateType + Neg<Output = T>,
{
    type Output = Point<T>;

    /// Returns a point with the x and y components negated.
    ///
    /// # Examples
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
    T: CoordinateType,
{
    type Output = Point<T>;

    /// Add a point to the given point.
    ///
    /// # Examples
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
    T: CoordinateType,
{
    type Output = Point<T>;

    /// Subtract a point from the given point.
    ///
    /// # Examples
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

#[cfg(feature = "rstar")]
// These are required for rstar RTree
impl<T> ::rstar::Point for Point<T>
where
    T: ::num_traits::Float + ::rstar::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
        Point::new(generator(0), generator(1))
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.0.x,
            1 => self.0.y,
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

impl<T: CoordinateType> From<[T; 2]> for Point<T> {
    fn from(coords: [T; 2]) -> Point<T> {
        Point::new(coords[0], coords[1])
    }
}
