use crate::{
    CoordNum, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};

use super::{
    LineStringTrait, MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait, PointTrait,
    PolygonTrait,
};

pub trait GeometryTrait<'a>: Send + Sync {
    type Point: 'a + PointTrait;
    type LineString: 'a + LineStringTrait<'a>;
    type Polygon: 'a + PolygonTrait<'a>;
    type MultiPoint: 'a + MultiPointTrait<'a>;
    type MultiLineString: 'a + MultiLineStringTrait<'a>;
    type MultiPolygon: 'a + MultiPolygonTrait<'a>;
    // type GeometryCollection: 'a + GeometryCollection<'a>;
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
        // Self::GeometryCollection,
    >;
}

pub enum GeometryType<'a, P, L, Y, MP, ML, MY>
where
    P: 'a + PointTrait,
    L: 'a + LineStringTrait<'a>,
    Y: 'a + PolygonTrait<'a>,
    MP: 'a + MultiPointTrait<'a>,
    ML: 'a + MultiLineStringTrait<'a>,
    MY: 'a + MultiPolygonTrait<'a>,
    // GC: 'a + GeometryCollection<'a>,
{
    Point(&'a P),
    LineString(&'a L),
    Polygon(&'a Y),
    MultiPoint(&'a MP),
    MultiLineString(&'a ML),
    MultiPolygon(&'a MY),
    // GeometryCollection(&'a GC),
}

impl<'a, T: CoordNum + Send + Sync + 'a> GeometryTrait<'a> for Geometry<T> {
    type Point = Point<T>;
    type LineString = LineString<T>;
    type Polygon = Polygon<T>;
    type MultiPoint = MultiPoint<T>;
    type MultiLineString = MultiLineString<T>;
    type MultiPolygon = MultiPolygon<T>;

    fn as_type(
        &'a self,
    ) -> GeometryType<
        'a,
        // Self::Point,
        <Geometry<T> as GeometryTrait>::Point,
        <Geometry<T> as GeometryTrait>::LineString,
        <Geometry<T> as GeometryTrait>::Polygon,
        <Geometry<T> as GeometryTrait>::MultiPoint,
        <Geometry<T> as GeometryTrait>::MultiLineString,
        <Geometry<T> as GeometryTrait>::MultiPolygon,
        // Self::GeometryCollection,
    > {
        match self {
            Geometry::Point(p) => GeometryType::Point(p),
            Geometry::LineString(p) => GeometryType::LineString(p),
            Geometry::Polygon(p) => GeometryType::Polygon(p),
            Geometry::MultiPoint(p) => GeometryType::MultiPoint(p),
            Geometry::MultiLineString(p) => GeometryType::MultiLineString(p),
            Geometry::MultiPolygon(p) => GeometryType::MultiPolygon(p),
            _ => todo!(),
        }
    }
}
