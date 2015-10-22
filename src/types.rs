#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Point(pub Coordinate);

impl Point {
    /// Returns the x/horizontal component of the point.
    ///
    /// ```
    /// use geo::Coordinate;
    /// use geo::Point;
    ///
    /// let p = Point(Coordinate{
    ///     x: 1.234,
    ///     y: 2.345,
    /// });
    ///
    /// assert_eq!(p.x(), 1.234);
    /// ```
    pub fn x(&self) -> f64 {
        self.0.x
    }

    /// Returns the y/vertical component of the point.
    ///
    /// ```
    /// use geo::Coordinate;
    /// use geo::Point;
    ///
    /// let p = Point(Coordinate{
    ///     x: 1.234,
    ///     y: 2.345,
    /// });
    ///
    /// assert_eq!(p.y(), 2.345);
    /// ```
    pub fn y(&self) -> f64 {
        self.0.y
    }

    /// Returns the longitude/horizontal component of the point.
    ///
    /// ```
    /// use geo::Coordinate;
    /// use geo::Point;
    ///
    /// let p = Point(Coordinate{
    ///     x: 1.234,
    ///     y: 2.345,
    /// });
    ///
    /// assert_eq!(p.lng(), 1.234);
    /// ```
    pub fn lng(&self) -> f64 {
        self.x()
    }

    /// Returns the latitude/vertical component of the point.
    ///
    /// ```
    /// use geo::Coordinate;
    /// use geo::Point;
    ///
    /// let p = Point(Coordinate{
    ///     x: 1.234,
    ///     y: 2.345,
    /// });
    ///
    /// assert_eq!(p.lat(), 2.345);
    /// ```
    pub fn lat(&self) -> f64 {
        self.y()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct MultiPoint(pub Vec<Point>);

#[derive(PartialEq, Clone, Debug)]
pub struct LineString(pub Vec<Point>);

#[derive(PartialEq, Clone, Debug)]
pub struct MultiLineString(pub Vec<LineString>);

#[derive(PartialEq, Clone, Debug)]
pub struct Polygon(pub LineString, pub Vec<LineString>);

#[derive(PartialEq, Clone, Debug)]
pub struct MultiPolygon(pub Vec<Polygon>);

#[derive(PartialEq, Clone, Debug)]
pub struct GeometryCollection(pub Vec<Geometry>);

#[derive(PartialEq, Clone, Debug)]
pub enum Geometry {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),
    GeometryCollection(GeometryCollection)
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
}
