use std::io::Cursor;

use byteorder::ReadBytesExt;
use geo_traits_ext::{
    forward_geometry_trait_ext_funcs, GeoTraitExtWithTypeTag, GeometryTag, GeometryTraitExt,
    GeometryTypeExt,
};

use crate::wkb::common::{WKBDimension, WKBType};
use crate::wkb::error::WKBResult;
use crate::wkb::reader::{
    GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use crate::wkb::Endianness;
use geo_traits::{
    Dimensions, GeometryTrait, GeometryType, UnimplementedLine, UnimplementedRect,
    UnimplementedTriangle,
};

use super::linearring::WKBLinearRing;

/// Parse a WKB byte slice into a geometry.
///
/// An opaque object that implements [`GeometryTrait`]. Use methods provided by [`geo_traits`] to
/// access the underlying data.
///
/// The contained [dimension][geo_traits::Dimensions] will never be `Unknown`.
#[derive(Debug, Clone)]
pub struct Wkb<'a>(WkbInner<'a>);

impl<'a> Wkb<'a> {
    /// Parse a WKB byte slice into a geometry.
    ///
    /// ### Performance
    ///
    /// WKB is not a zero-copy format because coordinates are not 8-byte aligned and because an
    /// initial scan needs to take place to know internal buffer offsets.
    ///
    /// This function does an initial pass over the WKB buffer to validate the contents and record
    /// the byte offsets for relevant coordinate slices but does not copy the underlying data to an
    /// alternate representation. This means that coordinates will **always be constant-time to
    /// access** but **not zero-copy**. This is because the raw WKB buffer is not 8-byte aligned,
    /// so when accessing a coordinate the underlying bytes need to be copied into a
    /// newly-allocated `f64`.
    pub fn try_new(buf: &'a [u8]) -> WKBResult<Self> {
        let inner = WkbInner::try_new(buf)?;
        Ok(Self(inner))
    }

    pub(crate) fn dimension(&self) -> WKBDimension {
        use WkbInner::*;
        match &self.0 {
            Point(g) => g.dimension(),
            LineString(g) => g.dimension(),
            Polygon(g) => g.dimension(),
            MultiPoint(g) => g.dimension(),
            MultiLineString(g) => g.dimension(),
            MultiPolygon(g) => g.dimension(),
            GeometryCollection(g) => g.dimension(),
        }
    }

    pub(crate) fn size(&self) -> u64 {
        use WkbInner::*;
        match &self.0 {
            Point(g) => g.size(),
            LineString(g) => g.size(),
            Polygon(g) => g.size(),
            MultiPoint(g) => g.size(),
            MultiLineString(g) => g.size(),
            MultiPolygon(g) => g.size(),
            GeometryCollection(g) => g.size(),
        }
    }
}

/// This is **not** exported publicly because we don't want to expose the enum variants publicly.
#[derive(Debug, Clone)]
pub(crate) enum WkbInner<'a> {
    Point(Point<'a>),
    LineString(LineString<'a>),
    Polygon(Polygon<'a>),
    MultiPoint(MultiPoint<'a>),
    MultiLineString(MultiLineString<'a>),
    MultiPolygon(MultiPolygon<'a>),
    GeometryCollection(GeometryCollection<'a>),
}

impl<'a> WkbInner<'a> {
    fn try_new(buf: &'a [u8]) -> WKBResult<Self> {
        let mut reader = Cursor::new(buf);
        let byte_order = Endianness::try_from(reader.read_u8()?).unwrap();
        let wkb_type = WKBType::from_buffer(buf)?;

        let out = match wkb_type {
            WKBType::Point(dim) => Self::Point(Point::new(buf, byte_order, 0, dim)),
            WKBType::LineString(dim) => Self::LineString(LineString::new(buf, byte_order, 0, dim)),
            WKBType::Polygon(dim) => Self::Polygon(Polygon::new(buf, byte_order, 0, dim)),
            WKBType::MultiPoint(dim) => Self::MultiPoint(MultiPoint::new(buf, byte_order, dim)),
            WKBType::MultiLineString(dim) => {
                Self::MultiLineString(MultiLineString::new(buf, byte_order, dim))
            }
            WKBType::MultiPolygon(dim) => {
                Self::MultiPolygon(MultiPolygon::new(buf, byte_order, dim))
            }
            WKBType::GeometryCollection(dim) => {
                Self::GeometryCollection(GeometryCollection::try_new(buf, byte_order, dim)?)
            }
        };
        Ok(out)
    }
}

impl<'a> GeometryTrait for Wkb<'a> {
    type T = f64;
    type PointType<'b>
        = Point<'a>
    where
        Self: 'b;
    type LineStringType<'b>
        = LineString<'a>
    where
        Self: 'b;
    type PolygonType<'b>
        = Polygon<'a>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint<'a>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = MultiLineString<'a>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = MultiPolygon<'a>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = GeometryCollection<'a>
    where
        Self: 'b;
    type RectType<'b>
        = UnimplementedRect<f64>
    where
        Self: 'b;
    type TriangleType<'b>
        = UnimplementedTriangle<f64>
    where
        Self: 'b;
    type LineType<'b>
        = UnimplementedLine<f64>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        self.dimension().into()
    }

    fn as_type(
        &self,
    ) -> geo_traits::GeometryType<
        '_,
        Point<'a>,
        LineString<'a>,
        Polygon<'a>,
        MultiPoint<'a>,
        MultiLineString<'a>,
        MultiPolygon<'a>,
        GeometryCollection<'a>,
        UnimplementedRect<f64>,
        UnimplementedTriangle<f64>,
        UnimplementedLine<f64>,
    > {
        use geo_traits::GeometryType as B;
        use WkbInner as A;
        match &self.0 {
            A::Point(p) => B::Point(p),
            A::LineString(ls) => B::LineString(ls),
            A::Polygon(ls) => B::Polygon(ls),
            A::MultiPoint(ls) => B::MultiPoint(ls),
            A::MultiLineString(ls) => B::MultiLineString(ls),
            A::MultiPolygon(ls) => B::MultiPolygon(ls),
            A::GeometryCollection(gc) => B::GeometryCollection(gc),
        }
    }
}

impl<'a, 'b> GeometryTrait for &'b Wkb<'a>
where
    'a: 'b,
{
    type T = f64;
    type PointType<'c>
        = Point<'a>
    where
        Self: 'c;
    type LineStringType<'c>
        = LineString<'a>
    where
        Self: 'c;
    type PolygonType<'c>
        = Polygon<'a>
    where
        Self: 'c;
    type MultiPointType<'c>
        = MultiPoint<'a>
    where
        Self: 'c;
    type MultiLineStringType<'c>
        = MultiLineString<'a>
    where
        Self: 'c;
    type MultiPolygonType<'c>
        = MultiPolygon<'a>
    where
        Self: 'c;
    type GeometryCollectionType<'c>
        = GeometryCollection<'a>
    where
        Self: 'c;
    type RectType<'c>
        = UnimplementedRect<f64>
    where
        Self: 'c;
    type TriangleType<'c>
        = UnimplementedTriangle<f64>
    where
        Self: 'c;
    type LineType<'c>
        = UnimplementedLine<f64>
    where
        Self: 'c;

    fn dim(&self) -> Dimensions {
        self.dimension().into()
    }

    fn as_type(
        &self,
    ) -> geo_traits::GeometryType<
        '_,
        Point<'a>,
        LineString<'a>,
        Polygon<'a>,
        MultiPoint<'a>,
        MultiLineString<'a>,
        MultiPolygon<'a>,
        GeometryCollection<'a>,
        UnimplementedRect<f64>,
        UnimplementedTriangle<f64>,
        UnimplementedLine<f64>,
    > {
        use geo_traits::GeometryType as B;
        use WkbInner as A;
        match &self.0 {
            A::Point(p) => B::Point(p),
            A::LineString(ls) => B::LineString(ls),
            A::Polygon(ls) => B::Polygon(ls),
            A::MultiPoint(ls) => B::MultiPoint(ls),
            A::MultiLineString(ls) => B::MultiLineString(ls),
            A::MultiPolygon(ls) => B::MultiPolygon(ls),
            A::GeometryCollection(gc) => B::GeometryCollection(gc),
        }
    }
}

impl GeometryTraitExt for Wkb<'_> {
    forward_geometry_trait_ext_funcs!(f64);
}

impl<'a, 'b> GeometryTraitExt for &'b Wkb<'a>
where
    'a: 'b,
{
    forward_geometry_trait_ext_funcs!(f64);
}

impl GeoTraitExtWithTypeTag for Wkb<'_> {
    type Tag = GeometryTag;
}

impl<'a, 'b> GeoTraitExtWithTypeTag for &'b Wkb<'a>
where
    'a: 'b,
{
    type Tag = GeometryTag;
}

macro_rules! impl_geometry_trait_for_wkb_type {
    ($geometry_type:ident, $geometry_type_tag:ident) => {
        impl<'a, 'b> GeometryTrait for &'b $geometry_type<'a>
        where
            'a: 'b,
        {
            type T = f64;

            type PointType<'c>
                = Point<'a>
            where
                Self: 'c;
            type LineStringType<'c>
                = LineString<'a>
            where
                Self: 'c;
            type PolygonType<'c>
                = Polygon<'a>
            where
                Self: 'c;
            type MultiPointType<'c>
                = MultiPoint<'a>
            where
                Self: 'c;
            type MultiLineStringType<'c>
                = MultiLineString<'a>
            where
                Self: 'c;
            type MultiPolygonType<'c>
                = MultiPolygon<'a>
            where
                Self: 'c;
            type GeometryCollectionType<'c>
                = GeometryCollection<'a>
            where
                Self: 'c;
            type RectType<'c>
                = UnimplementedRect<f64>
            where
                Self: 'c;
            type TriangleType<'c>
                = UnimplementedTriangle<f64>
            where
                Self: 'c;
            type LineType<'c>
                = UnimplementedLine<f64>
            where
                Self: 'c;

            fn dim(&self) -> Dimensions {
                self.dimension().into()
            }

            fn as_type(
                &self,
            ) -> geo_traits::GeometryType<
                '_,
                Point<'a>,
                LineString<'a>,
                Polygon<'a>,
                MultiPoint<'a>,
                MultiLineString<'a>,
                MultiPolygon<'a>,
                GeometryCollection<'a>,
                UnimplementedRect<f64>,
                UnimplementedTriangle<f64>,
                UnimplementedLine<f64>,
            > {
                geo_traits::GeometryType::$geometry_type_tag(self)
            }
        }

        impl<'a> GeometryTrait for $geometry_type<'a> {
            type T = f64;

            type PointType<'c>
                = Point<'a>
            where
                Self: 'c;
            type LineStringType<'c>
                = LineString<'a>
            where
                Self: 'c;
            type PolygonType<'c>
                = Polygon<'a>
            where
                Self: 'c;
            type MultiPointType<'c>
                = MultiPoint<'a>
            where
                Self: 'c;
            type MultiLineStringType<'c>
                = MultiLineString<'a>
            where
                Self: 'c;
            type MultiPolygonType<'c>
                = MultiPolygon<'a>
            where
                Self: 'c;
            type GeometryCollectionType<'c>
                = GeometryCollection<'a>
            where
                Self: 'c;
            type RectType<'c>
                = UnimplementedRect<f64>
            where
                Self: 'c;
            type TriangleType<'c>
                = UnimplementedTriangle<f64>
            where
                Self: 'c;
            type LineType<'c>
                = UnimplementedLine<f64>
            where
                Self: 'c;

            fn dim(&self) -> Dimensions {
                self.dimension().into()
            }

            fn as_type(
                &self,
            ) -> geo_traits::GeometryType<
                '_,
                Point<'a>,
                LineString<'a>,
                Polygon<'a>,
                MultiPoint<'a>,
                MultiLineString<'a>,
                MultiPolygon<'a>,
                GeometryCollection<'a>,
                UnimplementedRect<f64>,
                UnimplementedTriangle<f64>,
                UnimplementedLine<f64>,
            > {
                geo_traits::GeometryType::$geometry_type_tag(self)
            }
        }
    };
}

impl_geometry_trait_for_wkb_type!(Point, Point);
impl_geometry_trait_for_wkb_type!(LineString, LineString);
impl_geometry_trait_for_wkb_type!(Polygon, Polygon);
impl_geometry_trait_for_wkb_type!(MultiPoint, MultiPoint);
impl_geometry_trait_for_wkb_type!(MultiLineString, MultiLineString);
impl_geometry_trait_for_wkb_type!(MultiPolygon, MultiPolygon);
impl_geometry_trait_for_wkb_type!(GeometryCollection, GeometryCollection);

impl<'a> GeometryTrait for WKBLinearRing<'a> {
    type T = f64;

    type PointType<'c>
        = Point<'a>
    where
        Self: 'c;
    type LineStringType<'c>
        = WKBLinearRing<'a>
    where
        Self: 'c;
    type PolygonType<'c>
        = Polygon<'a>
    where
        Self: 'c;
    type MultiPointType<'c>
        = MultiPoint<'a>
    where
        Self: 'c;
    type MultiLineStringType<'c>
        = MultiLineString<'a>
    where
        Self: 'c;
    type MultiPolygonType<'c>
        = MultiPolygon<'a>
    where
        Self: 'c;
    type GeometryCollectionType<'c>
        = GeometryCollection<'a>
    where
        Self: 'c;
    type RectType<'c>
        = UnimplementedRect<f64>
    where
        Self: 'c;
    type TriangleType<'c>
        = UnimplementedTriangle<f64>
    where
        Self: 'c;
    type LineType<'c>
        = UnimplementedLine<f64>
    where
        Self: 'c;

    fn dim(&self) -> Dimensions {
        self.dimension().into()
    }

    fn as_type(
        &self,
    ) -> geo_traits::GeometryType<
        '_,
        Point<'a>,
        WKBLinearRing<'a>,
        Polygon<'a>,
        MultiPoint<'a>,
        MultiLineString<'a>,
        MultiPolygon<'a>,
        GeometryCollection<'a>,
        UnimplementedRect<f64>,
        UnimplementedTriangle<f64>,
        UnimplementedLine<f64>,
    > {
        geo_traits::GeometryType::LineString(self)
    }
}
