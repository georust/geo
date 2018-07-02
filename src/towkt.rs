extern crate geo_types;

use types::Coord;
use types::GeometryCollection;
use types::LineString;
use types::MultiLineString;
use types::MultiPoint;
use types::MultiPolygon;
use types::Point;
use types::Polygon;
use Geometry;
use Wkt;

/// A trait for converting values to WKT
pub trait ToWkt {
    /// Converts the value of `self` to an instance of WKT
    fn to_wkt(&self) -> Wkt;
}

fn g_point_to_w_coord(g_point: &geo_types::Point<f64>) -> Coord {
    let geo_types::Point(coord) = *g_point;
    let geo_types::Coordinate { x, y } = coord;
    Coord {
        x: x,
        y: y,
        z: None,
        m: None,
    }
}

fn g_point_to_w_point(g_point: &geo_types::Point<f64>) -> Point {
    let coord = g_point_to_w_coord(g_point);
    Point(Some(coord))
}

fn g_points_to_w_coords(g_points: &[geo_types::Point<f64>]) -> Vec<Coord> {
    let mut w_points = vec![];
    for g_point in g_points {
        w_points.push(g_point_to_w_coord(g_point));
    }
    w_points
}

fn g_line_to_w_linestring(g_line: &geo_types::Line<f64>) -> LineString {
    g_points_to_w_linestring(&vec![g_line.start, g_line.end])
}

fn g_linestring_to_w_linestring(g_linestring: &geo_types::LineString<f64>) -> LineString {
    let &geo_types::LineString(ref g_points) = g_linestring;
    g_points_to_w_linestring(g_points)
}

fn g_points_to_w_linestring(g_points: &[geo_types::Point<f64>]) -> LineString {
    let w_points = g_points_to_w_coords(g_points);
    LineString(w_points)
}

fn g_lines_to_w_lines(g_lines: &[geo_types::LineString<f64>]) -> Vec<LineString> {
    let mut w_lines = vec![];
    for g_line in g_lines {
        let &geo_types::LineString(ref g_points) = g_line;
        w_lines.push(g_points_to_w_linestring(g_points));
    }
    w_lines
}

fn g_polygon_to_w_polygon(g_polygon: &geo_types::Polygon<f64>) -> Polygon {
    let &geo_types::Polygon {
        exterior: ref outer_line,
        interiors: ref inner_lines,
    } = g_polygon;
    let mut poly_lines = vec![];

    // Outer
    let &geo_types::LineString(ref outer_points) = outer_line;
    poly_lines.push(g_points_to_w_linestring(outer_points));

    // Inner
    let inner = g_lines_to_w_lines(inner_lines);
    poly_lines.extend(inner.into_iter());

    Polygon(poly_lines)
}

fn g_mpoint_to_w_mpoint(g_mpoint: &geo_types::MultiPoint<f64>) -> MultiPoint {
    let &geo_types::MultiPoint(ref g_points) = g_mpoint;
    let w_coords = g_points_to_w_coords(g_points);
    let w_points = w_coords.into_iter().map(|c| Point(Some(c))).collect();
    MultiPoint(w_points)
}

fn g_mline_to_w_mline(g_mline: &geo_types::MultiLineString<f64>) -> MultiLineString {
    let &geo_types::MultiLineString(ref g_lines) = g_mline;
    let w_lines = g_lines_to_w_lines(g_lines);
    MultiLineString(w_lines)
}

fn g_polygons_to_w_polygons(g_polygons: &[geo_types::Polygon<f64>]) -> Vec<Polygon> {
    let mut w_polygons = vec![];
    for g_polygon in g_polygons {
        w_polygons.push(g_polygon_to_w_polygon(g_polygon));
    }
    w_polygons
}

fn g_mpolygon_to_w_mpolygon(g_mpolygon: &geo_types::MultiPolygon<f64>) -> MultiPolygon {
    let &geo_types::MultiPolygon(ref g_polygons) = g_mpolygon;
    let w_polygons = g_polygons_to_w_polygons(g_polygons);
    MultiPolygon(w_polygons)
}

fn g_geocol_to_w_geocol(g_geocol: &geo_types::GeometryCollection<f64>) -> GeometryCollection {
    let &geo_types::GeometryCollection(ref g_geoms) = g_geocol;
    let mut w_geoms = vec![];
    for g_geom in g_geoms {
        let w_geom = g_geom_to_w_geom(g_geom);
        w_geoms.push(w_geom);
    }
    GeometryCollection(w_geoms)
}

fn g_geom_to_w_geom(g_geom: &geo_types::Geometry<f64>) -> Geometry {
    match g_geom {
        &geo_types::Geometry::Point(ref g_point) => g_point_to_w_point(g_point).as_item(),

        &geo_types::Geometry::Line(ref g_line) => g_line_to_w_linestring(g_line).as_item(),

        &geo_types::Geometry::LineString(ref g_line) => {
            g_linestring_to_w_linestring(g_line).as_item()
        }

        &geo_types::Geometry::Polygon(ref g_polygon) => g_polygon_to_w_polygon(g_polygon).as_item(),

        &geo_types::Geometry::MultiPoint(ref g_mpoint) => g_mpoint_to_w_mpoint(g_mpoint).as_item(),

        &geo_types::Geometry::MultiLineString(ref g_mline) => g_mline_to_w_mline(g_mline).as_item(),

        &geo_types::Geometry::MultiPolygon(ref g_mpolygon) => {
            g_mpolygon_to_w_mpolygon(g_mpolygon).as_item()
        }

        &geo_types::Geometry::GeometryCollection(ref g_geocol) => {
            g_geocol_to_w_geocol(g_geocol).as_item()
        }
    }
}

impl ToWkt for geo_types::Geometry<f64> {
    fn to_wkt(&self) -> Wkt {
        let w_geom = g_geom_to_w_geom(&self);
        Wkt {
            items: vec![w_geom],
        }
    }
}
