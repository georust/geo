use geo_types::{
    CoordNum, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon,
};

use super::{
    GeometryCollectionTrait, LineStringTrait, MultiLineStringTrait, MultiPointTrait,
    MultiPolygonTrait, PointTrait, PolygonTrait,
};

#[allow(clippy::type_complexity)]
pub trait GeometryTrait<'a> {
    type Point: 'a + PointTrait;
    type LineString: 'a + LineStringTrait<'a>;
    type Polygon: 'a + PolygonTrait<'a>;
    type MultiPoint: 'a + MultiPointTrait<'a>;
    type MultiLineString: 'a + MultiLineStringTrait<'a>;
    type MultiPolygon: 'a + MultiPolygonTrait<'a>;
    type GeometryCollection: 'a + GeometryCollectionTrait<'a>;

    fn as_type(
        &'a self,
    ) -> GeometryType<
        'a,
        Self::Point,
        Self::LineString,
        Self::Polygon,
        Self::MultiPoint,
        Self::MultiLineString,
        Self::MultiPolygon,
        Self::GeometryCollection,
    >;
}

#[derive(Debug)]
pub enum GeometryType<'a, P, L, Y, MP, ML, MY, GC>
where
    P: 'a + PointTrait,
    L: 'a + LineStringTrait<'a>,
    Y: 'a + PolygonTrait<'a>,
    MP: 'a + MultiPointTrait<'a>,
    ML: 'a + MultiLineStringTrait<'a>,
    MY: 'a + MultiPolygonTrait<'a>,
    GC: 'a + GeometryCollectionTrait<'a>,
{
    Point(&'a P),
    LineString(&'a L),
    Polygon(&'a Y),
    MultiPoint(&'a MP),
    MultiLineString(&'a ML),
    MultiPolygon(&'a MY),
    GeometryCollection(&'a GC),
}

impl<'a, T: CoordNum + 'a> GeometryTrait<'a> for Geometry<T> {
    type Point = Point<T>;
    type LineString = LineString<T>;
    type Polygon = Polygon<T>;
    type MultiPoint = MultiPoint<T>;
    type MultiLineString = MultiLineString<T>;
    type MultiPolygon = MultiPolygon<T>;
    type GeometryCollection = GeometryCollection<T>;

    fn as_type(
        &'a self,
    ) -> GeometryType<
        'a,
        Point<T>,
        LineString<T>,
        Polygon<T>,
        MultiPoint<T>,
        MultiLineString<T>,
        MultiPolygon<T>,
        GeometryCollection<T>,
    > {
        match self {
            Geometry::Point(p) => GeometryType::Point(p),
            Geometry::LineString(p) => GeometryType::LineString(p),
            Geometry::Polygon(p) => GeometryType::Polygon(p),
            Geometry::MultiPoint(p) => GeometryType::MultiPoint(p),
            Geometry::MultiLineString(p) => GeometryType::MultiLineString(p),
            Geometry::MultiPolygon(p) => GeometryType::MultiPolygon(p),
            Geometry::GeometryCollection(p) => GeometryType::GeometryCollection(p),
            _ => todo!(),
        }
    }
}
