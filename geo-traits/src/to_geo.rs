//! Convert structs that implement geo-traits to [geo-types] objects.

use geo_types::{
    Coord, CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

use crate::{
    CoordTrait, GeometryCollectionTrait, GeometryTrait, GeometryType, LineStringTrait, LineTrait,
    MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait,
    TriangleTrait,
};

/// Convert any coordinate to a [`Coord`].
///
/// Only the first two dimensions will be kept.
pub fn coord_to_geo<T: CoordNum>(coord: &impl CoordTrait<T = T>) -> Coord<T> {
    Coord {
        x: coord.x(),
        y: coord.y(),
    }
}

/// Convert any Point to a [`Point`].
///
/// Only the first two dimensions will be kept.
pub fn point_to_geo<T: CoordNum>(point: &impl PointTrait<T = T>) -> Point<T> {
    if let Some(coord) = point.coord() {
        Point(coord_to_geo(&coord))
    } else {
        panic!("converting empty point to geo not implemented")
    }
}

/// Convert any LineString to a [`LineString`].
///
/// Only the first two dimensions will be kept.
pub fn line_string_to_geo<T: CoordNum>(line_string: &impl LineStringTrait<T = T>) -> LineString<T> {
    LineString::new(
        line_string
            .coords()
            .map(|coord| coord_to_geo(&coord))
            .collect(),
    )
}

/// Convert any Polygon to a [`Polygon`].
///
/// Only the first two dimensions will be kept.
pub fn polygon_to_geo<T: CoordNum, P: PolygonTrait<T = T>>(polygon: &P) -> Polygon<T> {
    let exterior = if let Some(exterior) = polygon.exterior() {
        line_string_to_geo(&exterior)
    } else {
        LineString::new(vec![])
    };
    let interiors = polygon
        .interiors()
        .map(|interior| line_string_to_geo(&interior))
        .collect();
    Polygon::new(exterior, interiors)
}

/// Convert any MultiPoint to a [`MultiPoint`].
///
/// Only the first two dimensions will be kept.
pub fn multi_point_to_geo<T: CoordNum>(multi_point: &impl MultiPointTrait<T = T>) -> MultiPoint<T> {
    MultiPoint::new(
        multi_point
            .points()
            .map(|point| point_to_geo(&point))
            .collect(),
    )
}

/// Convert any MultiLineString to a [`MultiLineString`].
///
/// Only the first two dimensions will be kept.
pub fn multi_line_string_to_geo<T: CoordNum>(
    multi_line_string: &impl MultiLineStringTrait<T = T>,
) -> MultiLineString<T> {
    MultiLineString::new(
        multi_line_string
            .line_strings()
            .map(|line| line_string_to_geo(&line))
            .collect(),
    )
}

/// Convert any MultiPolygon to a [`MultiPolygon`].
///
/// Only the first two dimensions will be kept.
pub fn multi_polygon_to_geo<T: CoordNum>(
    multi_polygon: &impl MultiPolygonTrait<T = T>,
) -> MultiPolygon<T> {
    MultiPolygon::new(
        multi_polygon
            .polygons()
            .map(|polygon| polygon_to_geo(&polygon))
            .collect(),
    )
}

/// Convert any Rect to a [`Rect`].
///
/// Only the first two dimensions will be kept.
pub fn rect_to_geo<T: CoordNum>(rect: &impl RectTrait<T = T>) -> Rect<T> {
    let c1 = coord_to_geo(&rect.min());
    let c2 = coord_to_geo(&rect.max());
    Rect::new(c1, c2)
}

/// Convert any Line to a [`Line`].
///
/// Only the first two dimensions will be kept.
pub fn line_to_geo<T: CoordNum>(line: &impl LineTrait<T = T>) -> Line<T> {
    let start = coord_to_geo(&line.start());
    let end = coord_to_geo(&line.end());
    Line::new(start, end)
}

/// Convert any Triangle to a [`Triangle`].
///
/// Only the first two dimensions will be kept.
pub fn triangle_to_geo<T: CoordNum>(triangle: &impl TriangleTrait<T = T>) -> Triangle<T> {
    let v1 = coord_to_geo(&triangle.first());
    let v2 = coord_to_geo(&triangle.second());
    let v3 = coord_to_geo(&triangle.third());
    Triangle::new(v1, v2, v3)
}

/// Convert any Geometry to a [`Geometry`].
///
/// Only the first two dimensions will be kept.
pub fn geometry_to_geo<T: CoordNum>(geometry: &impl GeometryTrait<T = T>) -> Geometry<T> {
    use GeometryType::*;

    match geometry.as_type() {
        Point(geom) => Geometry::Point(point_to_geo(geom)),
        LineString(geom) => Geometry::LineString(line_string_to_geo(geom)),
        Polygon(geom) => Geometry::Polygon(polygon_to_geo(geom)),
        MultiPoint(geom) => Geometry::MultiPoint(multi_point_to_geo(geom)),
        MultiLineString(geom) => Geometry::MultiLineString(multi_line_string_to_geo(geom)),
        MultiPolygon(geom) => Geometry::MultiPolygon(multi_polygon_to_geo(geom)),
        GeometryCollection(geom) => Geometry::GeometryCollection(geometry_collection_to_geo(geom)),
        Rect(geom) => Geometry::Rect(rect_to_geo(geom)),
        Line(geom) => Geometry::Line(line_to_geo(geom)),
        Triangle(geom) => Geometry::Triangle(triangle_to_geo(geom)),
    }
}

/// Convert any GeometryCollection to a [`GeometryCollection`].
///
/// Only the first two dimensions will be kept.
pub fn geometry_collection_to_geo<T: CoordNum>(
    geometry_collection: &impl GeometryCollectionTrait<T = T>,
) -> GeometryCollection<T> {
    GeometryCollection::new_from(
        geometry_collection
            .geometries()
            .map(|geometry| geometry_to_geo(&geometry))
            .collect(),
    )
}
