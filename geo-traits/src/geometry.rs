use geo_types::{
    CoordNum, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon, Rect,
};

use crate::{
    Dimension, GeometryCollectionTrait, LineStringTrait, MultiLineStringTrait, MultiPointTrait,
    MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait,
};

/// A trait for accessing data from a generic Geometry.
#[allow(clippy::type_complexity)]
pub trait GeometryTrait {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying Point, which implements [PointTrait]
    type Point<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying LineString, which implements [LineStringTrait]
    type LineString<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying Polygon, which implements [PolygonTrait]
    type Polygon<'a>: 'a + PolygonTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying MultiPoint, which implements [MultiPointTrait]
    type MultiPoint<'a>: 'a + MultiPointTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying MultiLineString, which implements [MultiLineStringTrait]
    type MultiLineString<'a>: 'a + MultiLineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying MultiPolygon, which implements [MultiPolygonTrait]
    type MultiPolygon<'a>: 'a + MultiPolygonTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying GeometryCollection, which implements [GeometryCollectionTrait]
    type GeometryCollection<'a>: 'a + GeometryCollectionTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying Rect, which implements [RectTrait]
    type Rect<'a>: 'a + RectTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimension;

    /// Cast this geometry to a [`GeometryType`] enum, which allows for downcasting to a specific
    /// type
    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Self::Point<'_>,
        Self::LineString<'_>,
        Self::Polygon<'_>,
        Self::MultiPoint<'_>,
        Self::MultiLineString<'_>,
        Self::MultiPolygon<'_>,
        Self::GeometryCollection<'_>,
        Self::Rect<'_>,
    >;
}

/// An enumeration of all geometry types that can be contained inside a [GeometryTrait]. This is
/// used for extracting concrete geometry types out of a [GeometryTrait].
#[derive(Debug)]
pub enum GeometryType<'a, P, L, Y, MP, ML, MY, GC, R>
where
    P: PointTrait,
    L: LineStringTrait,
    Y: PolygonTrait,
    MP: MultiPointTrait,
    ML: MultiLineStringTrait,
    MY: MultiPolygonTrait,
    GC: GeometryCollectionTrait,
    R: RectTrait,
{
    /// A Point, which implements [PointTrait]
    Point(&'a P),
    /// A LineString, which implements [LineStringTrait]
    LineString(&'a L),
    /// A Polygon, which implements [PolygonTrait]
    Polygon(&'a Y),
    /// A MultiPoint, which implements [MultiPointTrait]
    MultiPoint(&'a MP),
    /// A MultiLineString, which implements [MultiLineStringTrait]
    MultiLineString(&'a ML),
    /// A MultiPolygon, which implements [MultiPolygonTrait]
    MultiPolygon(&'a MY),
    /// A GeometryCollection, which implements [GeometryCollectionTrait]
    GeometryCollection(&'a GC),
    /// A Rect, which implements [RectTrait]
    Rect(&'a R),
}

impl<'a, T: CoordNum + 'a> GeometryTrait for Geometry<T> {
    type T = T;
    type Point<'b> = Point<Self::T> where Self: 'b;
    type LineString<'b> = LineString<Self::T> where Self: 'b;
    type Polygon<'b> = Polygon<Self::T> where Self: 'b;
    type MultiPoint<'b> = MultiPoint<Self::T> where Self: 'b;
    type MultiLineString<'b> = MultiLineString<Self::T> where Self: 'b;
    type MultiPolygon<'b> = MultiPolygon<Self::T> where Self: 'b;
    type GeometryCollection<'b> = GeometryCollection<Self::T> where Self: 'b;
    type Rect<'b> = Rect<Self::T> where Self: 'b;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Point<T>,
        LineString<T>,
        Polygon<T>,
        MultiPoint<T>,
        MultiLineString<T>,
        MultiPolygon<T>,
        GeometryCollection<T>,
        Rect<T>,
    > {
        match self {
            Geometry::Point(p) => GeometryType::Point(p),
            Geometry::LineString(p) => GeometryType::LineString(p),
            Geometry::Polygon(p) => GeometryType::Polygon(p),
            Geometry::MultiPoint(p) => GeometryType::MultiPoint(p),
            Geometry::MultiLineString(p) => GeometryType::MultiLineString(p),
            Geometry::MultiPolygon(p) => GeometryType::MultiPolygon(p),
            Geometry::GeometryCollection(p) => GeometryType::GeometryCollection(p),
            Geometry::Rect(p) => GeometryType::Rect(p),
            _ => todo!(),
        }
    }
}

impl<'a, T: CoordNum + 'a> GeometryTrait for &'a Geometry<T> {
    type T = T;
    type Point<'b> = Point<Self::T> where Self: 'b;
    type LineString<'b> = LineString<Self::T> where Self: 'b;
    type Polygon<'b> = Polygon<Self::T> where Self: 'b;
    type MultiPoint<'b> = MultiPoint<Self::T> where Self: 'b;
    type MultiLineString<'b> = MultiLineString<Self::T> where Self: 'b;
    type MultiPolygon<'b> = MultiPolygon<Self::T> where Self: 'b;
    type GeometryCollection<'b> = GeometryCollection<Self::T> where Self: 'b;
    type Rect<'b> = Rect<Self::T> where Self: 'b;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Point<T>,
        LineString<T>,
        Polygon<T>,
        MultiPoint<T>,
        MultiLineString<T>,
        MultiPolygon<T>,
        GeometryCollection<T>,
        Rect<T>,
    > {
        match self {
            Geometry::Point(p) => GeometryType::Point(p),
            Geometry::LineString(p) => GeometryType::LineString(p),
            Geometry::Polygon(p) => GeometryType::Polygon(p),
            Geometry::MultiPoint(p) => GeometryType::MultiPoint(p),
            Geometry::MultiLineString(p) => GeometryType::MultiLineString(p),
            Geometry::MultiPolygon(p) => GeometryType::MultiPolygon(p),
            Geometry::GeometryCollection(p) => GeometryType::GeometryCollection(p),
            Geometry::Rect(p) => GeometryType::Rect(p),
            _ => todo!(),
        }
    }
}
