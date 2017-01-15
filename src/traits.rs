pub use ::Geometry;

use num_traits::Float;

pub trait ToGeo<T: Float>
{
    fn to_geo(&self) -> Geometry<T>;
}

// FIXME: find good names for these traits, don't use XyzTrait naming scheme

pub trait PointTrait {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn opt_z(&self) -> Option<f64> {
        None
    }
    fn opt_m(&self) -> Option<f64> {
        None
    }
}

pub trait LineStringTrait<'a> {
    type ItemType: 'a + PointTrait;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn points(&'a self) -> Self::Iter;
}

pub trait PolygonTrait<'a> {
    type ItemType: 'a + LineStringTrait<'a>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn rings(&'a self) -> Self::Iter;
}

pub trait MultiPointTrait<'a> {
    type ItemType: 'a + PointTrait;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn points(&'a self) -> Self::Iter;
}

pub trait MultiLineStringTrait<'a> {
    type ItemType: 'a + LineStringTrait<'a>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn lines(&'a self) -> Self::Iter;
}

pub trait MultiPolygonTrait<'a> {
    type ItemType: 'a + PolygonTrait<'a>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn polygons(&'a self) -> Self::Iter;
}
