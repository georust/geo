use crate::{
    structs::{
        GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
    },
    Dimensions, GeometryTrait, UnimplementedLine, UnimplementedRect, UnimplementedTriangle,
};

#[derive(Clone, Debug, PartialEq)]
/// All supported WKT geometry [`types`]
pub enum Geometry<T: Copy> {
    /// A point.
    Point(Point<T>),
    /// A linestring.
    LineString(LineString<T>),
    /// A polygon.
    Polygon(Polygon<T>),
    /// A multipoint.
    MultiPoint(MultiPoint<T>),
    /// A multilinestring.
    MultiLineString(MultiLineString<T>),
    /// A multipolygon.
    MultiPolygon(MultiPolygon<T>),
    /// A geometry collection.
    GeometryCollection(GeometryCollection<T>),
}

impl<T> Geometry<T>
where
    T: Copy,
{
    /// Return the [Dimension] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        match self {
            Self::Point(g) => g.dimension(),
            Self::LineString(g) => g.dimension(),
            Self::Polygon(g) => g.dimension(),
            Self::MultiPoint(g) => g.dimension(),
            Self::MultiLineString(g) => g.dimension(),
            Self::MultiPolygon(g) => g.dimension(),
            Self::GeometryCollection(g) => g.dimension(),
        }
    }
}

impl<T: Copy> GeometryTrait for Geometry<T> {
    type T = T;
    type PointType<'b>
        = Point<T>
    where
        Self: 'b;
    type LineStringType<'b>
        = LineString<T>
    where
        Self: 'b;
    type PolygonType<'b>
        = Polygon<T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint<T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = MultiLineString<T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = MultiPolygon<T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = GeometryCollection<T>
    where
        Self: 'b;
    type RectType<'b>
        = UnimplementedRect<T>
    where
        Self: 'b;
    type LineType<'b>
        = UnimplementedLine<T>
    where
        Self: 'b;
    type TriangleType<'b>
        = UnimplementedTriangle<T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        match self {
            Geometry::Point(geom) => geom.dim(),
            Geometry::LineString(geom) => geom.dim(),
            Geometry::Polygon(geom) => geom.dim(),
            Geometry::MultiPoint(geom) => geom.dim(),
            Geometry::MultiLineString(geom) => geom.dim(),
            Geometry::MultiPolygon(geom) => geom.dim(),
            Geometry::GeometryCollection(geom) => geom.dim(),
        }
    }

    fn as_type(
        &self,
    ) -> crate::GeometryType<
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
        match self {
            Geometry::Point(geom) => crate::GeometryType::Point(geom),
            Geometry::LineString(geom) => crate::GeometryType::LineString(geom),
            Geometry::Polygon(geom) => crate::GeometryType::Polygon(geom),
            Geometry::MultiPoint(geom) => crate::GeometryType::MultiPoint(geom),
            Geometry::MultiLineString(geom) => crate::GeometryType::MultiLineString(geom),
            Geometry::MultiPolygon(geom) => crate::GeometryType::MultiPolygon(geom),
            Geometry::GeometryCollection(geom) => crate::GeometryType::GeometryCollection(geom),
        }
    }
}

impl<T: Copy> GeometryTrait for &Geometry<T> {
    type T = T;
    type PointType<'b>
        = Point<T>
    where
        Self: 'b;
    type LineStringType<'b>
        = LineString<T>
    where
        Self: 'b;
    type PolygonType<'b>
        = Polygon<T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint<T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = MultiLineString<T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = MultiPolygon<T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = GeometryCollection<T>
    where
        Self: 'b;
    type RectType<'b>
        = UnimplementedRect<T>
    where
        Self: 'b;
    type LineType<'b>
        = UnimplementedLine<T>
    where
        Self: 'b;
    type TriangleType<'b>
        = UnimplementedTriangle<T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        match self {
            Geometry::Point(geom) => geom.dim(),
            Geometry::LineString(geom) => geom.dim(),
            Geometry::Polygon(geom) => geom.dim(),
            Geometry::MultiPoint(geom) => geom.dim(),
            Geometry::MultiLineString(geom) => geom.dim(),
            Geometry::MultiPolygon(geom) => geom.dim(),
            Geometry::GeometryCollection(geom) => geom.dim(),
        }
    }

    fn as_type(
        &self,
    ) -> crate::GeometryType<
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
        match self {
            Geometry::Point(geom) => crate::GeometryType::Point(geom),
            Geometry::LineString(geom) => crate::GeometryType::LineString(geom),
            Geometry::Polygon(geom) => crate::GeometryType::Polygon(geom),
            Geometry::MultiPoint(geom) => crate::GeometryType::MultiPoint(geom),
            Geometry::MultiLineString(geom) => crate::GeometryType::MultiLineString(geom),
            Geometry::MultiPolygon(geom) => crate::GeometryType::MultiPolygon(geom),
            Geometry::GeometryCollection(geom) => crate::GeometryType::GeometryCollection(geom),
        }
    }
}

// Specialized implementations on each Geometry concrete type.

macro_rules! impl_specialization {
    ($geometry_type:ident) => {
        impl<T: Copy> GeometryTrait for $geometry_type<T> {
            type T = T;
            type PointType<'b>
                = Point<T>
            where
                Self: 'b;
            type LineStringType<'b>
                = LineString<T>
            where
                Self: 'b;
            type PolygonType<'b>
                = Polygon<T>
            where
                Self: 'b;
            type MultiPointType<'b>
                = MultiPoint<T>
            where
                Self: 'b;
            type MultiLineStringType<'b>
                = MultiLineString<T>
            where
                Self: 'b;
            type MultiPolygonType<'b>
                = MultiPolygon<T>
            where
                Self: 'b;
            type GeometryCollectionType<'b>
                = GeometryCollection<T>
            where
                Self: 'b;
            type RectType<'b>
                = UnimplementedRect<T>
            where
                Self: 'b;
            type LineType<'b>
                = UnimplementedLine<T>
            where
                Self: 'b;
            type TriangleType<'b>
                = UnimplementedTriangle<T>
            where
                Self: 'b;

            fn dim(&self) -> Dimensions {
                self.dim.into()
            }

            fn as_type(
                &self,
            ) -> crate::GeometryType<
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
                crate::GeometryType::$geometry_type(self)
            }
        }

        impl<'a, T: Copy + 'a> GeometryTrait for &'a $geometry_type<T> {
            type T = T;
            type PointType<'b>
                = Point<T>
            where
                Self: 'b;
            type LineStringType<'b>
                = LineString<T>
            where
                Self: 'b;
            type PolygonType<'b>
                = Polygon<T>
            where
                Self: 'b;
            type MultiPointType<'b>
                = MultiPoint<T>
            where
                Self: 'b;
            type MultiLineStringType<'b>
                = MultiLineString<T>
            where
                Self: 'b;
            type MultiPolygonType<'b>
                = MultiPolygon<T>
            where
                Self: 'b;
            type GeometryCollectionType<'b>
                = GeometryCollection<T>
            where
                Self: 'b;
            type RectType<'b>
                = UnimplementedRect<T>
            where
                Self: 'b;
            type LineType<'b>
                = UnimplementedLine<T>
            where
                Self: 'b;
            type TriangleType<'b>
                = UnimplementedTriangle<T>
            where
                Self: 'b;

            fn dim(&self) -> Dimensions {
                self.dim.into()
            }

            fn as_type(
                &self,
            ) -> crate::GeometryType<
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
                crate::GeometryType::$geometry_type(self)
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
