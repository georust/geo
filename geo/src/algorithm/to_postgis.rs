use ::{MultiPoint, Line, Point, LineString, MultiLineString, MultiPolygon, Polygon, Geometry, GeometryCollection};
use postgis::ewkb;

/// Converts geometry to a PostGIS type.
///
/// Note that PostGIS databases can include a SRID (spatial reference
/// system identifier) for geometry stored in them. You should specify
/// the SRID of your geometry when converting, using `to_postgis_with_srid()`,
/// or use `to_postgis_wgs84()` if your data is standard WGS84.
pub trait ToPostgis<T> {
    /// Converts this geometry to a PostGIS type, using the supplied SRID.
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> T;
    /// Converts this WGS84 geometry to a PostGIS type.
    fn to_postgis_wgs84(&self) -> T {
        self.to_postgis_with_srid(Some(4326))
    }
}

impl ToPostgis<ewkb::Point> for Point<f64> {
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> ewkb::Point {
        ewkb::Point::new(self.x(), self.y(), srid)
    }
}
impl ToPostgis<ewkb::LineString> for Line<f64> {
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> ewkb::LineString {
        let points = vec![
            self.start.to_postgis_with_srid(srid),
            self.end.to_postgis_with_srid(srid)
        ];
        ewkb::LineString { points, srid }
    }
}
impl ToPostgis<ewkb::Polygon> for Polygon<f64> {
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> ewkb::Polygon {
        let rings = ::std::iter::once(&self.exterior)
            .chain(self.interiors.iter())
            .map(|x| x.to_postgis_with_srid(srid))
            .collect();
        ewkb::Polygon { rings, srid }
    }
}
macro_rules! to_postgis_impl {
    ($from:ident, $to:path, $name:ident) => {
        impl ToPostgis<$to> for $from<f64> {
            fn to_postgis_with_srid(&self, srid: Option<i32>) -> $to {
                let $name = self.0.iter()
                    .map(|x| x.to_postgis_with_srid(srid))
                    .collect();
                $to { $name, srid }
            }
        }
    };
}
to_postgis_impl!(GeometryCollection, ewkb::GeometryCollection, geometries);
to_postgis_impl!(MultiPolygon, ewkb::MultiPolygon, polygons);
to_postgis_impl!(MultiLineString, ewkb::MultiLineString, lines);
to_postgis_impl!(MultiPoint, ewkb::MultiPoint, points);
to_postgis_impl!(LineString, ewkb::LineString, points);
impl ToPostgis<ewkb::Geometry> for Geometry<f64> {
    fn to_postgis_with_srid(&self, srid: Option<i32>) -> ewkb::Geometry {
        match *self {
            Geometry::Point(ref p) => ewkb::GeometryT::Point(p.to_postgis_with_srid(srid)),
            Geometry::Line(ref p) => ewkb::GeometryT::LineString(p.to_postgis_with_srid(srid)),
            Geometry::LineString(ref p) => ewkb::GeometryT::LineString(p.to_postgis_with_srid(srid)),
            Geometry::Polygon(ref p) => ewkb::GeometryT::Polygon(p.to_postgis_with_srid(srid)),
            Geometry::MultiPoint(ref p) => ewkb::GeometryT::MultiPoint(p.to_postgis_with_srid(srid)),
            Geometry::MultiLineString(ref p) => ewkb::GeometryT::MultiLineString(p.to_postgis_with_srid(srid)),
            Geometry::MultiPolygon(ref p) => ewkb::GeometryT::MultiPolygon(p.to_postgis_with_srid(srid)),
            Geometry::GeometryCollection(ref p) => ewkb::GeometryT::GeometryCollection(p.to_postgis_with_srid(srid)),
        }
    }
}
