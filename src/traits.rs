pub use ::Geometry;

use num_traits::Float;

pub trait ToGeo<T: Float>
{
    fn to_geo(&self) -> Geometry<T>;
}

// FIXME: find good names for these traits, don't use XyzTrait naming scheme

pub trait PointTrait<T: Float> {
    fn x(&self) -> T;
    fn y(&self) -> T;
}

pub trait LineStringTrait<T: Float> {
    type ItemType: PointTrait<T>;
    type Iter: Iterator<Item=Self::ItemType>;

    fn points(&self) -> Self::Iter;
}

pub trait PolygonTrait<T: Float> {
    type ItemType: LineStringTrait<T>;
    type Iter: Iterator<Item=Self::ItemType>;

    fn rings(&self) -> Self::Iter;
}

pub trait MultiPointTrait<T: Float> {
    type ItemType: PointTrait<T>;
    type Iter: Iterator<Item=Self::ItemType>;

    fn points(&self) -> Self::Iter;
}

pub trait MultiLineStringTrait<T: Float> {
    type ItemType: LineStringTrait<T>;
    type Iter: Iterator<Item=Self::ItemType>;

    fn lines(&self) -> Self::Iter;
}

pub trait MultiPolygonTrait<T: Float> {
    type ItemType: PolygonTrait<T>;
    type Iter: Iterator<Item=Self::ItemType>;

    fn polygons(&self) -> Self::Iter;
}
