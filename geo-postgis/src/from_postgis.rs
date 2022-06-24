use geo_types::{
    Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};

use postgis::ewkb::{GeometryCollectionT, GeometryT};

#[cfg_attr(docsrs, doc(cfg(feature = "postgis")))]
/// Creates geometry from a PostGIS type.
///
/// Note that PostGIS databases can store data under any spatial
/// reference system - not just WGS84. No attempt is made to convert
/// data between reference systems.
pub trait FromPostgis<T> {
    fn from_postgis(_: T) -> Self;
}

impl<'a, T> FromPostgis<&'a T> for Point
where
    T: postgis::Point,
{
    fn from_postgis(pt: &'a T) -> Self {
        Point::new(pt.x(), pt.y())
    }
}
impl<'a, T> FromPostgis<&'a T> for LineString
where
    T: postgis::LineString<'a>,
{
    fn from_postgis(ls: &'a T) -> Self {
        let ret: Vec<Point> = ls.points().map(Point::from_postgis).collect();
        LineString::from(ret)
    }
}
impl<'a, T> FromPostgis<&'a T> for Option<Polygon<f64>>
where
    T: postgis::Polygon<'a>,
{
    /// This returns an `Option`, because it's possible for a PostGIS `Polygon`
    /// to contain zero rings, which makes for an invalid `geo::Polygon`.
    fn from_postgis(poly: &'a T) -> Self {
        let mut rings = poly
            .rings()
            .map(LineString::from_postgis)
            .collect::<Vec<_>>();
        if rings.is_empty() {
            return None;
        }
        let exterior = rings.remove(0);
        Some(Polygon::new(exterior, rings))
    }
}
impl<'a, T> FromPostgis<&'a T> for MultiPoint
where
    T: postgis::MultiPoint<'a>,
{
    fn from_postgis(mp: &'a T) -> Self {
        let ret = mp.points().map(Point::from_postgis).collect();
        MultiPoint::new(ret)
    }
}
impl<'a, T> FromPostgis<&'a T> for MultiLineString
where
    T: postgis::MultiLineString<'a>,
{
    fn from_postgis(mp: &'a T) -> Self {
        let ret = mp.lines().map(LineString::from_postgis).collect();
        MultiLineString::new(ret)
    }
}
impl<'a, T> FromPostgis<&'a T> for MultiPolygon
where
    T: postgis::MultiPolygon<'a>,
{
    /// This implementation discards PostGIS polygons that don't convert
    /// (return `None` when `from_postgis()` is called on them).
    fn from_postgis(mp: &'a T) -> Self {
        let ret = mp.polygons().filter_map(Option::from_postgis).collect();
        MultiPolygon::new(ret)
    }
}
impl<'a, T> FromPostgis<&'a GeometryCollectionT<T>> for GeometryCollection
where
    T: postgis::Point + postgis::ewkb::EwkbRead,
{
    /// This implementation discards geometries that don't convert
    /// (return `None` when `from_postgis()` is called on them).
    fn from_postgis(gc: &'a GeometryCollectionT<T>) -> Self {
        let geoms = gc
            .geometries
            .iter()
            .filter_map(Option::from_postgis)
            .collect();
        GeometryCollection::new_from(geoms)
    }
}
impl<'a, T> FromPostgis<&'a GeometryT<T>> for Option<Geometry>
where
    T: postgis::Point + postgis::ewkb::EwkbRead,
{
    /// This returns an `Option`, because the supplied geometry
    /// could be an invalid `Polygon`.
    fn from_postgis(geo: &'a GeometryT<T>) -> Self {
        Some(match *geo {
            GeometryT::Point(ref p) => Geometry::Point(Point::from_postgis(p)),
            GeometryT::LineString(ref ls) => Geometry::LineString(LineString::from_postgis(ls)),
            GeometryT::Polygon(ref p) => match Option::from_postgis(p) {
                Some(p) => Geometry::Polygon(p),
                None => return None,
            },
            GeometryT::MultiPoint(ref p) => Geometry::MultiPoint(MultiPoint::from_postgis(p)),
            GeometryT::MultiLineString(ref p) => {
                Geometry::MultiLineString(MultiLineString::from_postgis(p))
            }
            GeometryT::MultiPolygon(ref p) => Geometry::MultiPolygon(MultiPolygon::from_postgis(p)),
            GeometryT::GeometryCollection(ref p) => {
                Geometry::GeometryCollection(GeometryCollection::from_postgis(p))
            }
        })
    }
}
