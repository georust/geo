extern crate geo;

use Wkt;
use Geometry;
use types::Coord;
use types::GeometryCollection;
use types::LineString;
use types::Point;
use types::Polygon;
use types::MultiPoint;
use types::MultiLineString;
use types::MultiPolygon;


/// A trait for converting values to WKT
pub trait ToWkt {
    /// Converts the value of `self` to an instance of WKT
    fn to_wkt(&self) -> Wkt;
}


fn g_point_to_w_coord(g_point: &geo::Point) -> Coord {
    let geo::Point(coord) = *g_point;
    let geo::Coordinate{x, y} = coord;
    Coord{x: x, y: y, z: None, m: None}
}


fn g_point_to_w_point(g_point: &geo::Point) -> Point {
    let coord = g_point_to_w_coord(g_point);
    Point(Some(coord))
}


fn g_points_to_w_coords(g_points: &Vec<geo::Point>) -> Vec<Coord> {
    let mut w_points = vec![];
    for g_point in g_points {
        w_points.push(g_point_to_w_coord(g_point));
    }
    w_points
}


fn g_line_to_w_line(g_line: &geo::LineString) -> LineString {
    let &geo::LineString(ref g_points) = g_line;
    g_points_to_w_line(g_points)
}


fn g_points_to_w_line(g_points: &Vec<geo::Point>) -> LineString {
    let w_points = g_points_to_w_coords(g_points);
    LineString(w_points)
}


fn g_lines_to_w_lines(g_lines: &Vec<geo::LineString>) -> Vec<LineString> {
    let mut w_lines = vec![];
    for g_line in g_lines {
        let &geo::LineString(ref g_points) = g_line;
        w_lines.push(g_points_to_w_line(g_points));
    }
    w_lines
}


fn g_polygon_to_w_polygon(g_polygon: &geo::Polygon) -> Polygon {
    let &geo::Polygon(ref outer_line, ref inner_lines) = g_polygon;
    let mut poly_lines = vec![];

    // Outer
    let &geo::LineString(ref outer_points) = outer_line;
    poly_lines.push(g_points_to_w_line(outer_points));

    // Inner
    let inner = g_lines_to_w_lines(inner_lines);
    poly_lines.extend(inner.into_iter());

    Polygon(poly_lines)
}


fn g_mpoint_to_w_mpoint(g_mpoint: &geo::MultiPoint) -> MultiPoint {
    let &geo::MultiPoint(ref g_points) = g_mpoint;
    let w_coords = g_points_to_w_coords(g_points);
    let w_points = w_coords.map_in_place(|c| Point(Some(c)));
    MultiPoint(w_points)
}


fn g_mline_to_w_mline(g_mline: &geo::MultiLineString) -> MultiLineString {
    let &geo::MultiLineString(ref g_lines) = g_mline;
    let w_lines = g_lines_to_w_lines(g_lines);
    MultiLineString(w_lines)
}


fn g_polygons_to_w_polygons(g_polygons: &Vec<geo::Polygon>) -> Vec<Polygon> {
    let mut w_polygons = vec![];
    for g_polygon in g_polygons {
        w_polygons.push(g_polygon_to_w_polygon(g_polygon));
    }
    w_polygons
}


fn g_mpolygon_to_w_mpolygon(g_mpolygon: &geo::MultiPolygon) -> MultiPolygon {
    let &geo::MultiPolygon(ref g_polygons) = g_mpolygon;
    let w_polygons = g_polygons_to_w_polygons(g_polygons);
    MultiPolygon(w_polygons)
}


fn g_geocol_to_w_geocol(g_geocol: &geo::GeometryCollection) -> GeometryCollection {
    let &geo::GeometryCollection(ref g_geoms) = g_geocol;
    let mut w_geoms = vec![];
    for g_geom in g_geoms {
        let w_geom = g_geom_to_w_geom(g_geom);
        w_geoms.push(w_geom);
    }
    GeometryCollection(w_geoms)
}


fn g_geom_to_w_geom(g_geom: &geo::Geometry) -> Geometry {
    match g_geom {
        &geo::Geometry::Point(ref g_point) =>
            g_point_to_w_point(g_point).as_item(),

        &geo::Geometry::LineString(ref g_line) =>
            g_line_to_w_line(g_line).as_item(),

        &geo::Geometry::Polygon(ref g_polygon) =>
            g_polygon_to_w_polygon(g_polygon).as_item(),

        &geo::Geometry::MultiPoint(ref g_mpoint) =>
            g_mpoint_to_w_mpoint(g_mpoint).as_item(),

        &geo::Geometry::MultiLineString(ref g_mline) =>
            g_mline_to_w_mline(g_mline).as_item(),

        &geo::Geometry::MultiPolygon(ref g_mpolygon) =>
            g_mpolygon_to_w_mpolygon(g_mpolygon).as_item(),

        &geo::Geometry::GeometryCollection(ref g_geocol) =>
            g_geocol_to_w_geocol(g_geocol).as_item(),
    }
}


impl ToWkt for geo::Geometry {
    fn to_wkt(&self) -> Wkt {
        let w_geom = g_geom_to_w_geom(&self);
        Wkt{items: vec![w_geom]}
    }
}
