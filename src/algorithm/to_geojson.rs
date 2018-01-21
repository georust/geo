use types::{MultiPoint, Point, Line, LineString, MultiLineString, MultiPolygon, Polygon, Geometry, GeometryCollection};
use geojson;

/// Converts geometry to a GeoJSON type.
pub trait ToGeoJson<T> {
    fn to_geo_json(&self) -> T;
}
impl ToGeoJson<geojson::PointType> for Point<f64> {
    fn to_geo_json(&self) -> geojson::PointType {
        vec![self.x(), self.y()]
    }
}
impl ToGeoJson<geojson::LineStringType> for Line<f64> {
    fn to_geo_json(&self) -> geojson::LineStringType {
        vec![self.start.to_geo_json(), self.end.to_geo_json()]
    }
}
macro_rules! to_geojson_impl {
    ($from:ident, $to:path) => {
        impl ToGeoJson<$to> for $from<f64> {
            fn to_geo_json(&self) -> $to {
                self.0.iter()
                    .map(|x| x.to_geo_json())
                    .collect()
            }
        }
    };
}
to_geojson_impl!(LineString, geojson::LineStringType);
impl ToGeoJson<geojson::PolygonType> for Polygon<f64> {
    fn to_geo_json(&self) -> geojson::PolygonType {
        ::std::iter::once(&self.exterior)
            .chain(self.interiors.iter())
            .map(|x| x.to_geo_json())
            .collect()
    }
}
to_geojson_impl!(MultiPoint, Vec<geojson::PointType>);
to_geojson_impl!(MultiLineString, Vec<geojson::LineStringType>);
to_geojson_impl!(MultiPolygon, Vec<geojson::PolygonType>);
to_geojson_impl!(GeometryCollection, Vec<geojson::Geometry>);
impl ToGeoJson<geojson::Geometry> for Geometry<f64> {
    fn to_geo_json(&self) -> geojson::Geometry {
        let val = match *self {
            Geometry::Point(ref p) => geojson::Value::Point(p.to_geo_json()),
            Geometry::MultiPoint(ref p) => geojson::Value::MultiPoint(p.to_geo_json()),
            Geometry::LineString(ref p) => geojson::Value::LineString(p.to_geo_json()),
            Geometry::Line(ref p) => geojson::Value::LineString(p.to_geo_json()),
            Geometry::MultiLineString(ref p) => geojson::Value::MultiLineString(p.to_geo_json()),
            Geometry::Polygon(ref p) => geojson::Value::Polygon(p.to_geo_json()),
            Geometry::MultiPolygon(ref p) => geojson::Value::MultiPolygon(p.to_geo_json()),
            Geometry::GeometryCollection(ref p) => geojson::Value::GeometryCollection(p.to_geo_json()),
        };
        geojson::Geometry {
            value: val,
            bbox: None,
            foreign_members: None
        }
    }
}
