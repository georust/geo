extern crate geo_types;
extern crate num_traits;

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
pub trait ToWkt<T>
where
    T: num_traits::Float,
{
    /// Converts the value of `self` to an instance of WKT
    fn to_wkt(&self) -> Wkt<T>;
}

fn g_point_to_w_coord<T>(g_point: &geo_types::Coordinate<T>) -> Coord<T>
where
    T: num_traits::Float,
{
    Coord {
        x: g_point.x,
        y: g_point.y,
        z: None,
        m: None,
    }
}

fn g_point_to_w_point<T>(g_point: &geo_types::Point<T>) -> Point<T>
where
    T: num_traits::Float,
{
    let coord = g_point_to_w_coord(&g_point.0);
    Point(Some(coord))
}

fn g_points_to_w_coords<T>(g_points: &[geo_types::Coordinate<T>]) -> Vec<Coord<T>>
where
    T: num_traits::Float,
{
    g_points.iter().map(g_point_to_w_coord).collect()
}

fn g_points_to_w_points<T>(g_points: &[geo_types::Point<T>]) -> Vec<Point<T>>
where
    T: num_traits::Float,
{
    g_points
        .iter()
        .map(|p| &p.0)
        .map(g_point_to_w_coord)
        .map(|c| Point(Some(c)))
        .collect()
}

fn g_line_to_w_linestring<T>(g_line: &geo_types::Line<T>) -> LineString<T>
where
    T: num_traits::Float,
{
    g_points_to_w_linestring(&vec![g_line.start, g_line.end])
}

fn g_linestring_to_w_linestring<T>(g_linestring: &geo_types::LineString<T>) -> LineString<T>
where
    T: num_traits::Float,
{
    let &geo_types::LineString(ref g_points) = g_linestring;
    g_points_to_w_linestring(g_points)
}

fn g_points_to_w_linestring<T>(g_coords: &[geo_types::Coordinate<T>]) -> LineString<T>
where
    T: num_traits::Float,
{
    let w_coords = g_points_to_w_coords(g_coords);
    LineString(w_coords)
}

fn g_lines_to_w_lines<T>(g_lines: &[geo_types::LineString<T>]) -> Vec<LineString<T>>
where
    T: num_traits::Float,
{
    let mut w_lines = vec![];
    for g_line in g_lines {
        let &geo_types::LineString(ref g_points) = g_line;
        w_lines.push(g_points_to_w_linestring(g_points));
    }
    w_lines
}

fn g_triangle_to_w_polygon<T>(g_triangle: &geo_types::Triangle<T>) -> Polygon<T>
where
    T: num_traits::Float,
{
    let polygon = g_triangle.to_polygon();
    g_polygon_to_w_polygon(&polygon)
}

fn g_rect_to_w_polygon<T>(g_rect: &geo_types::Rect<T>) -> Polygon<T>
where
    T: num_traits::Float,
{
    let polygon = g_rect.to_polygon();
    g_polygon_to_w_polygon(&polygon)
}

fn g_polygon_to_w_polygon<T>(g_polygon: &geo_types::Polygon<T>) -> Polygon<T>
where
    T: num_traits::Float,
{
    let outer_line = g_polygon.exterior();
    let inner_lines = g_polygon.interiors();
    let mut poly_lines = vec![];

    // Outer
    let &geo_types::LineString(ref outer_points) = outer_line;
    if !outer_points.is_empty() {
        poly_lines.push(g_points_to_w_linestring(outer_points));
    }

    // Inner
    let inner = g_lines_to_w_lines(inner_lines);
    poly_lines.extend(inner.into_iter());

    Polygon(poly_lines)
}

fn g_mpoint_to_w_mpoint<T>(g_mpoint: &geo_types::MultiPoint<T>) -> MultiPoint<T>
where
    T: num_traits::Float,
{
    let &geo_types::MultiPoint(ref g_points) = g_mpoint;
    let w_points = g_points_to_w_points(g_points);
    MultiPoint(w_points)
}

fn g_mline_to_w_mline<T>(g_mline: &geo_types::MultiLineString<T>) -> MultiLineString<T>
where
    T: num_traits::Float,
{
    let &geo_types::MultiLineString(ref g_lines) = g_mline;
    let w_lines = g_lines_to_w_lines(g_lines);
    MultiLineString(w_lines)
}

fn g_polygons_to_w_polygons<T>(g_polygons: &[geo_types::Polygon<T>]) -> Vec<Polygon<T>>
where
    T: num_traits::Float,
{
    let mut w_polygons = vec![];
    for g_polygon in g_polygons {
        w_polygons.push(g_polygon_to_w_polygon(g_polygon));
    }
    w_polygons
}

fn g_mpolygon_to_w_mpolygon<T>(g_mpolygon: &geo_types::MultiPolygon<T>) -> MultiPolygon<T>
where
    T: num_traits::Float,
{
    let &geo_types::MultiPolygon(ref g_polygons) = g_mpolygon;
    let w_polygons = g_polygons_to_w_polygons(g_polygons);
    MultiPolygon(w_polygons)
}

fn g_geocol_to_w_geocol<T>(g_geocol: &geo_types::GeometryCollection<T>) -> GeometryCollection<T>
where
    T: num_traits::Float,
{
    let &geo_types::GeometryCollection(ref g_geoms) = g_geocol;
    let mut w_geoms = vec![];
    for g_geom in g_geoms {
        let w_geom = g_geom_to_w_geom(g_geom);
        w_geoms.push(w_geom);
    }
    GeometryCollection(w_geoms)
}

fn g_geom_to_w_geom<T>(g_geom: &geo_types::Geometry<T>) -> Geometry<T>
where
    T: num_traits::Float,
{
    match g_geom {
        &geo_types::Geometry::Point(ref g_point) => g_point_to_w_point(g_point).as_item(),

        &geo_types::Geometry::Line(ref g_line) => g_line_to_w_linestring(g_line).as_item(),

        &geo_types::Geometry::LineString(ref g_line) => {
            g_linestring_to_w_linestring(g_line).as_item()
        }

        &geo_types::Geometry::Triangle(ref g_triangle) => {
            g_triangle_to_w_polygon(g_triangle).as_item()
        }

        &geo_types::Geometry::Rect(ref g_rect) => g_rect_to_w_polygon(g_rect).as_item(),

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

impl<T> ToWkt<T> for geo_types::Geometry<T>
where
    T: num_traits::Float,
{
    fn to_wkt(&self) -> Wkt<T> {
        let w_geom = g_geom_to_w_geom(&self);
        Wkt {
            items: vec![w_geom],
        }
    }
}
