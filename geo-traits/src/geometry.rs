use std::marker::PhantomData;

#[cfg(feature = "geo-types")]
use geo_types::{
    CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

use crate::{
    Dimensions, GeometryCollectionTrait, LineStringTrait, LineTrait, MultiLineStringTrait,
    MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait, TriangleTrait,
    UnimplementedGeometryCollection, UnimplementedLine, UnimplementedLineString,
    UnimplementedMultiLineString, UnimplementedMultiPoint, UnimplementedMultiPolygon,
    UnimplementedPoint, UnimplementedPolygon, UnimplementedRect, UnimplementedTriangle,
};

/// A trait for accessing data from a generic Geometry.
#[allow(clippy::type_complexity)]
pub trait GeometryTrait {
    /// The coordinate type of this geometry
    type T;

    /// The type of each underlying Point, which implements [PointTrait]
    type PointType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying LineString, which implements [LineStringTrait]
    type LineStringType<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying Polygon, which implements [PolygonTrait]
    type PolygonType<'a>: 'a + PolygonTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying MultiPoint, which implements [MultiPointTrait]
    type MultiPointType<'a>: 'a + MultiPointTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying MultiLineString, which implements [MultiLineStringTrait]
    type MultiLineStringType<'a>: 'a + MultiLineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying MultiPolygon, which implements [MultiPolygonTrait]
    type MultiPolygonType<'a>: 'a + MultiPolygonTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying GeometryCollection, which implements [GeometryCollectionTrait]
    type GeometryCollectionType<'a>: 'a + GeometryCollectionTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying Rect, which implements [RectTrait]
    type RectType<'a>: 'a + RectTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying Triangle, which implements [TriangleTrait]
    type TriangleType<'a>: 'a + TriangleTrait<T = Self::T>
    where
        Self: 'a;

    /// The type of each underlying Line, which implements [LineTrait]
    type LineType<'a>: 'a + LineTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// Cast this geometry to a [`GeometryType`] enum, which allows for downcasting to a specific
    /// type
    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Self::PointType<'_>,
        Self::LineStringType<'_>,
        Self::PolygonType<'_>,
        Self::MultiPointType<'_>,
        Self::MultiLineStringType<'_>,
        Self::MultiPolygonType<'_>,
        Self::GeometryCollectionType<'_>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    >;
}

/// An enumeration of all geometry types that can be contained inside a [GeometryTrait]. This is
/// used for extracting concrete geometry types out of a [GeometryTrait].
#[derive(Debug)]
pub enum GeometryType<'a, P, LS, Y, MP, ML, MY, GC, R, T, L>
where
    P: PointTrait,
    LS: LineStringTrait,
    Y: PolygonTrait,
    MP: MultiPointTrait,
    ML: MultiLineStringTrait,
    MY: MultiPolygonTrait,
    GC: GeometryCollectionTrait,
    R: RectTrait,
    T: TriangleTrait,
    L: LineTrait,
{
    /// A Point, which implements [PointTrait]
    Point(&'a P),
    /// A LineString, which implements [LineStringTrait]
    LineString(&'a LS),
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
    /// A Triangle, which implements [TriangleTrait]
    Triangle(&'a T),
    /// A Line, which implements [LineTrait]
    Line(&'a L),
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> GeometryTrait for Geometry<T> {
    type T = T;
    type PointType<'b>
        = Point<Self::T>
    where
        Self: 'b;
    type LineStringType<'b>
        = LineString<Self::T>
    where
        Self: 'b;
    type PolygonType<'b>
        = Polygon<Self::T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint<Self::T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = MultiLineString<Self::T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = MultiPolygon<Self::T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = GeometryCollection<Self::T>
    where
        Self: 'b;
    type RectType<'b>
        = Rect<Self::T>
    where
        Self: 'b;
    type TriangleType<'b>
        = Triangle<Self::T>
    where
        Self: 'b;
    type LineType<'b>
        = Line<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
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
        Triangle<T>,
        Line<T>,
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
            Geometry::Triangle(p) => GeometryType::Triangle(p),
            Geometry::Line(p) => GeometryType::Line(p),
        }
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum + 'a> GeometryTrait for &'a Geometry<T> {
    type T = T;
    type PointType<'b>
        = Point<Self::T>
    where
        Self: 'b;
    type LineStringType<'b>
        = LineString<Self::T>
    where
        Self: 'b;
    type PolygonType<'b>
        = Polygon<Self::T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint<Self::T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = MultiLineString<Self::T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = MultiPolygon<Self::T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = GeometryCollection<Self::T>
    where
        Self: 'b;
    type RectType<'b>
        = Rect<Self::T>
    where
        Self: 'b;
    type TriangleType<'b>
        = Triangle<Self::T>
    where
        Self: 'b;
    type LineType<'b>
        = Line<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
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
        Triangle<T>,
        Line<T>,
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
            Geometry::Triangle(p) => GeometryType::Triangle(p),
            Geometry::Line(p) => GeometryType::Line(p),
        }
    }
}

// Specialized implementations on each geo-types concrete type.

macro_rules! impl_specialization {
    ($geometry_type:ident) => {
        #[cfg(feature = "geo-types")]
        impl<T: CoordNum> GeometryTrait for $geometry_type<T> {
            type T = T;
            type PointType<'b>
                = Point<Self::T>
            where
                Self: 'b;
            type LineStringType<'b>
                = LineString<Self::T>
            where
                Self: 'b;
            type PolygonType<'b>
                = Polygon<Self::T>
            where
                Self: 'b;
            type MultiPointType<'b>
                = MultiPoint<Self::T>
            where
                Self: 'b;
            type MultiLineStringType<'b>
                = MultiLineString<Self::T>
            where
                Self: 'b;
            type MultiPolygonType<'b>
                = MultiPolygon<Self::T>
            where
                Self: 'b;
            type GeometryCollectionType<'b>
                = GeometryCollection<Self::T>
            where
                Self: 'b;
            type RectType<'b>
                = Rect<Self::T>
            where
                Self: 'b;
            type TriangleType<'b>
                = Triangle<Self::T>
            where
                Self: 'b;
            type LineType<'b>
                = Line<Self::T>
            where
                Self: 'b;

            fn dim(&self) -> Dimensions {
                Dimensions::Xy
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
                Triangle<T>,
                Line<T>,
            > {
                GeometryType::$geometry_type(self)
            }
        }

        #[cfg(feature = "geo-types")]
        impl<'a, T: CoordNum + 'a> GeometryTrait for &'a $geometry_type<T> {
            type T = T;
            type PointType<'b>
                = Point<Self::T>
            where
                Self: 'b;
            type LineStringType<'b>
                = LineString<Self::T>
            where
                Self: 'b;
            type PolygonType<'b>
                = Polygon<Self::T>
            where
                Self: 'b;
            type MultiPointType<'b>
                = MultiPoint<Self::T>
            where
                Self: 'b;
            type MultiLineStringType<'b>
                = MultiLineString<Self::T>
            where
                Self: 'b;
            type MultiPolygonType<'b>
                = MultiPolygon<Self::T>
            where
                Self: 'b;
            type GeometryCollectionType<'b>
                = GeometryCollection<Self::T>
            where
                Self: 'b;
            type RectType<'b>
                = Rect<Self::T>
            where
                Self: 'b;
            type TriangleType<'b>
                = Triangle<Self::T>
            where
                Self: 'b;
            type LineType<'b>
                = Line<Self::T>
            where
                Self: 'b;

            fn dim(&self) -> Dimensions {
                Dimensions::Xy
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
                Triangle<T>,
                Line<T>,
            > {
                GeometryType::$geometry_type(self)
            }
        }
    };
}

impl_specialization!(Point);
impl_specialization!(LineString);
impl_specialization!(Polygon);
impl_specialization!(MultiPoint);
impl_specialization!(MultiLineString);
impl_specialization!(MultiPolygon);
impl_specialization!(GeometryCollection);
impl_specialization!(Rect);
impl_specialization!(Triangle);
impl_specialization!(Line);

/// An empty struct that implements [GeometryTrait].
///
/// This is used internally for [`UnimplementedGeometryCollection`], so that
/// `UnimplementedGeometryCollection` can be used as the `GeometryCollectionType` of the
/// `GeometryTrait` by implementations that don't have a GeometryCollection concept
pub struct UnimplementedGeometry<T>(PhantomData<T>);

impl<T> GeometryTrait for UnimplementedGeometry<T> {
    type T = T;
    type PointType<'b>
        = UnimplementedPoint<T>
    where
        Self: 'b;
    type LineStringType<'b>
        = UnimplementedLineString<Self::T>
    where
        Self: 'b;
    type PolygonType<'b>
        = UnimplementedPolygon<Self::T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = UnimplementedMultiPoint<Self::T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = UnimplementedMultiLineString<Self::T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = UnimplementedMultiPolygon<Self::T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = UnimplementedGeometryCollection<Self::T>
    where
        Self: 'b;
    type RectType<'b>
        = UnimplementedRect<Self::T>
    where
        Self: 'b;
    type TriangleType<'b>
        = UnimplementedTriangle<Self::T>
    where
        Self: 'b;
    type LineType<'b>
        = UnimplementedLine<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Self::PointType<'_>,
        Self::LineStringType<'_>,
        Self::PolygonType<'_>,
        Self::MultiPointType<'_>,
        Self::MultiLineStringType<'_>,
        Self::MultiPolygonType<'_>,
        Self::GeometryCollectionType<'_>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    > {
        unimplemented!()
    }
}
