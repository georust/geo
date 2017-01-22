pub use ::Geometry;

use num_traits::{Float, FromPrimitive};

pub trait ToGeo<T: Float + FromPrimitive>
{
    fn to_geo(&self) -> Geometry<T>;
}

// FIXME: find good names for these traits, don't use XyzTrait naming scheme
// FIXME: remove FromPrimitive trait

pub trait PointTrait<T: Float + FromPrimitive> {
    fn x(&self) -> T;
    fn y(&self) -> T;

    fn distance_to_point(&self, other: &Self) -> T {
        ::algorithm::distance::point_to_point(self, other)
    }
}

pub trait LineStringTrait<'a, T>
    where T: 'a + Float + FromPrimitive
{
    type ItemType: 'a + PointTrait<T>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn points(&'a self) -> Self::Iter;

    // FIXME: decide if this should be called 'len'
    fn length(&'a self) -> T {
        ::algorithm::length::line_string(self)
    }

    /// Centroid on a LineString is the mean of the middle of the segment
    /// weighted by the length of the segments.
    fn centroid(&'a self) -> Option<::Point<T>> {
        ::algorithm::centroid::line_string(self)
    }
}

pub trait PolygonTrait<'a, T>
    where T: 'a + Float + FromPrimitive,
{
    type ItemType: 'a + LineStringTrait<'a, T>;
    type Iter: 'a + Iterator<Item=&'a Self::ItemType>;

    fn rings(&'a self) -> Self::Iter;

    fn area(&'a self) -> T {
        ::algorithm::area::polygon(self)
    }

    /// Centroid on a Polygon.
    /// See: https://en.wikipedia.org/wiki/Centroid
    fn centroid(&'a self) -> Option<::Point<T>> {
        ::algorithm::centroid::polygon(self)
    }
}

pub trait MultiPointTrait<'a, T>
    where T: 'a + Float + FromPrimitive,
{
    type ItemType: 'a + PointTrait<T>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn points(&'a self) -> Self::Iter;
}

pub trait MultiLineStringTrait<'a, T>
    where T: 'a + Float + FromPrimitive,
{
    type ItemType: 'a + LineStringTrait<'a, T>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn lines(&'a self) -> Self::Iter;

    // FIXME: decide if this should be called 'len'
    fn length(&'a self) -> T {
        ::algorithm::length::multi_line_string(self)
    }
}

pub trait MultiPolygonTrait<'a, T>
    where T: 'a + Float + FromPrimitive,
{
    type ItemType: 'a + PolygonTrait<'a, T>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn polygons(&'a self) -> Self::Iter;

    fn area(&'a self) -> T {
        ::algorithm::area::multi_polygon(self)
    }

    fn centroid(&'a self) -> Option<::Point<T>> {
        ::algorithm::centroid::multi_polygon(self)
    }
}
