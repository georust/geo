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
pub trait ToGeoCoord<T: CoordNum> {
    /// Convert to a geo_types [`Coord`].
    fn to_coord(&self) -> Coord<T>;
}

impl<T: CoordNum, G: CoordTrait<T = T>> ToGeoCoord<T> for G {
    fn to_coord(&self) -> Coord<T> {
        Coord {
            x: self.x(),
            y: self.y(),
        }
    }
}

/// Convert any Point to a [`Point`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoPoint<T: CoordNum> {
    /// Convert to a geo_types [`Point`].
    ///
    /// # Panics
    ///
    /// This will panic on an empty point.
    fn to_point(&self) -> Point<T> {
        self.try_to_point()
            .expect("geo-types does not support empty points.")
    }

    /// Convert to a geo_types [`Point`].
    ///
    /// Empty points will return `None`.
    fn try_to_point(&self) -> Option<Point<T>>;
}

impl<T: CoordNum, G: PointTrait<T = T>> ToGeoPoint<T> for G {
    fn try_to_point(&self) -> Option<Point<T>> {
        self.coord().map(|coord| Point(coord.to_coord()))
    }
}

/// Convert any LineString to a [`LineString`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoLineString<T: CoordNum> {
    /// Convert to a geo_types [`LineString`].
    fn to_line_string(&self) -> LineString<T>;
}

impl<T: CoordNum, G: LineStringTrait<T = T>> ToGeoLineString<T> for G {
    fn to_line_string(&self) -> LineString<T> {
        LineString::new(self.coords().map(|coord| coord.to_coord()).collect())
    }
}

/// Convert any Polygon to a [`Polygon`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoPolygon<T: CoordNum> {
    /// Convert to a geo_types [`Polygon`].
    fn to_polygon(&self) -> Polygon<T>;
}

impl<T: CoordNum, G: PolygonTrait<T = T>> ToGeoPolygon<T> for G {
    fn to_polygon(&self) -> Polygon<T> {
        let exterior = if let Some(exterior) = self.exterior() {
            exterior.to_line_string()
        } else {
            LineString::empty()
        };
        let interiors = self
            .interiors()
            .map(|interior| interior.to_line_string())
            .collect();
        Polygon::new(exterior, interiors)
    }
}

/// Convert any MultiPoint to a [`MultiPoint`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoMultiPoint<T: CoordNum> {
    /// Convert to a geo_types [`MultiPoint`].
    ///
    /// # Panics
    ///
    /// This will panic if any of the points contained in the MultiPoint are empty.
    fn to_multi_point(&self) -> MultiPoint<T> {
        self.try_to_multi_point()
            .expect("geo-types does not support MultiPoint containing empty points.")
    }

    /// Convert to a geo_types [`MultiPoint`].
    ///
    /// `None` will be returned if any of the points contained in the MultiPoint are empty.
    fn try_to_multi_point(&self) -> Option<MultiPoint<T>>;
}

impl<T: CoordNum, G: MultiPointTrait<T = T>> ToGeoMultiPoint<T> for G {
    fn try_to_multi_point(&self) -> Option<MultiPoint<T>> {
        let mut geo_points = vec![];
        for point in self.points() {
            if let Some(geo_point) = point.try_to_point() {
                geo_points.push(geo_point);
            } else {
                // Return None if any points are empty
                return None;
            }
        }
        Some(MultiPoint::new(geo_points))
    }
}

/// Convert any MultiLineString to a [`MultiLineString`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoMultiLineString<T: CoordNum> {
    /// Convert to a geo_types [`MultiLineString`].
    fn to_multi_line_string(&self) -> MultiLineString<T>;
}

impl<T: CoordNum, G: MultiLineStringTrait<T = T>> ToGeoMultiLineString<T> for G {
    fn to_multi_line_string(&self) -> MultiLineString<T> {
        MultiLineString::new(
            self.line_strings()
                .map(|line_string| line_string.to_line_string())
                .collect(),
        )
    }
}

/// Convert any MultiPolygon to a [`MultiPolygon`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoMultiPolygon<T: CoordNum> {
    /// Convert to a geo_types [`MultiPolygon`].
    fn to_multi_polygon(&self) -> MultiPolygon<T>;
}

impl<T: CoordNum, G: MultiPolygonTrait<T = T>> ToGeoMultiPolygon<T> for G {
    fn to_multi_polygon(&self) -> MultiPolygon<T> {
        MultiPolygon::new(
            self.polygons()
                .map(|polygon| polygon.to_polygon())
                .collect(),
        )
    }
}

/// Convert any Rect to a [`Rect`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoRect<T: CoordNum> {
    /// Convert to a geo_types [`Rect`].
    fn to_rect(&self) -> Rect<T>;
}

impl<T: CoordNum, G: RectTrait<T = T>> ToGeoRect<T> for G {
    fn to_rect(&self) -> Rect<T> {
        let c1 = self.min().to_coord();
        let c2 = self.max().to_coord();
        Rect::new(c1, c2)
    }
}

/// Convert any Line to a [`Line`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoLine<T: CoordNum> {
    /// Convert to a geo_types [`Line`].
    fn to_line(&self) -> Line<T>;
}

impl<T: CoordNum, G: LineTrait<T = T>> ToGeoLine<T> for G {
    fn to_line(&self) -> Line<T> {
        let start = self.start().to_coord();
        let end = self.end().to_coord();
        Line::new(start, end)
    }
}

/// Convert any Triangle to a [`Triangle`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoTriangle<T: CoordNum> {
    /// Convert to a geo_types [`Triangle`].
    fn to_triangle(&self) -> Triangle<T>;
}

impl<T: CoordNum, G: TriangleTrait<T = T>> ToGeoTriangle<T> for G {
    fn to_triangle(&self) -> Triangle<T> {
        let v1 = self.first().to_coord();
        let v2 = self.second().to_coord();
        let v3 = self.third().to_coord();
        Triangle::new(v1, v2, v3)
    }
}

/// Convert any Geometry to a [`Geometry`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoGeometry<T: CoordNum> {
    /// Convert to a geo_types [`Geometry`].
    ///
    /// # Panics
    ///
    /// This will panic on an empty point or a MultiPoint containing empty points.
    fn to_geometry(&self) -> Geometry<T> {
        self.try_to_geometry().expect(
            "geo-types does not support empty point or a MultiPoint containing empty points.",
        )
    }

    /// Convert to a geo_types [`Geometry`].
    ///
    /// Empty Geometrys will return `None`.
    fn try_to_geometry(&self) -> Option<Geometry<T>>;
}

impl<T: CoordNum, G: GeometryTrait<T = T>> ToGeoGeometry<T> for G {
    fn try_to_geometry(&self) -> Option<Geometry<T>> {
        use GeometryType::*;

        match self.as_type() {
            Point(geom) => geom.try_to_point().map(Geometry::Point),
            LineString(geom) => Some(Geometry::LineString(geom.to_line_string())),
            Polygon(geom) => Some(Geometry::Polygon(geom.to_polygon())),
            MultiPoint(geom) => geom.try_to_multi_point().map(Geometry::MultiPoint),
            MultiLineString(geom) => Some(Geometry::MultiLineString(geom.to_multi_line_string())),
            MultiPolygon(geom) => Some(Geometry::MultiPolygon(geom.to_multi_polygon())),
            GeometryCollection(geom) => geom
                .try_to_geometry_collection()
                .map(Geometry::GeometryCollection),
            Rect(geom) => Some(Geometry::Rect(geom.to_rect())),
            Line(geom) => Some(Geometry::Line(geom.to_line())),
            Triangle(geom) => Some(Geometry::Triangle(geom.to_triangle())),
        }
    }
}

/// Convert any GeometryCollection to a [`GeometryCollection`].
///
/// Only the first two dimensions will be kept.
pub trait ToGeoGeometryCollection<T: CoordNum> {
    /// Convert to a geo_types [`GeometryCollection`].
    ///
    /// # Panics
    ///
    /// This will panic on an empty point or a MultiPoint containing empty points.
    fn to_geometry_collection(&self) -> GeometryCollection<T> {
        self.try_to_geometry_collection()
            .expect("geo-types does not support empty GeometryCollections.")
    }

    /// Convert to a geo_types [`GeometryCollection`].
    ///
    /// This will return `None` for an empty point or a MultiPoint containing empty points.
    fn try_to_geometry_collection(&self) -> Option<GeometryCollection<T>>;
}

impl<T: CoordNum, G: GeometryCollectionTrait<T = T>> ToGeoGeometryCollection<T> for G {
    fn try_to_geometry_collection(&self) -> Option<GeometryCollection<T>> {
        let mut geo_geometries = vec![];
        for geom in self.geometries() {
            if let Some(geo_geom) = geom.try_to_geometry() {
                geo_geometries.push(geo_geom);
            } else {
                // Return None if any points are empty
                return None;
            }
        }
        Some(GeometryCollection::new_from(geo_geometries))
    }
}
