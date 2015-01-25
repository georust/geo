
#[derive(PartialEq, Clone)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

#[derive(PartialEq, Clone)]
pub struct Point {
    pub coordinate: Coordinate
}

#[derive(PartialEq, Clone)]
pub struct MultiPoint {
    pub points: Vec<Point>
}

#[derive(PartialEq, Clone)]
pub struct LineString {
    pub points: Vec<Point>
}

#[derive(PartialEq, Clone)]
pub struct MultiLineString {
    pub line_strings: Vec<Polygon>
}

#[derive(PartialEq, Clone)]
pub struct Polygon {
    pub exteror_ring: LineString,
    pub interior_ring: Vec<LineString>
}

#[derive(PartialEq, Clone)]
pub struct MultiPolygon {
    pub polygons: Vec<Polygon>
}

#[derive(PartialEq, Clone)]
pub struct GeometryCollection {
    pub geometries: Vec<Geometry>
}

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
