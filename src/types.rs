#[derive(PartialEq, Clone, Copy, Show)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

#[derive(PartialEq, Clone, Copy)]
pub struct Point(pub Coordinate);

#[derive(PartialEq, Clone)]
pub struct MultiPoint(pub Vec<Point>);

#[derive(PartialEq, Clone)]
pub struct LineString(pub Vec<Point>);

#[derive(PartialEq, Clone)]
pub struct MultiLineString(pub Vec<LineString>);

#[derive(PartialEq, Clone)]
pub struct Polygon(pub LineString, pub Vec<LineString>);

#[derive(PartialEq, Clone)]
pub struct MultiPolygon(pub Vec<Polygon>);

#[derive(PartialEq, Clone)]
pub struct GeometryCollection(pub Vec<Geometry>);

#[derive(PartialEq, Clone)]
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
