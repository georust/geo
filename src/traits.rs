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
    fn opt_z(&self) -> Option<f64> {
        None
    }
    fn opt_m(&self) -> Option<f64> {
        None
    }
}

pub trait LineStringTrait<'a, T: Float> {
    type ItemType: 'a + PointTrait<T>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn points(&'a self) -> Self::Iter;
}

pub trait PolygonTrait<'a, T: Float> {
    type ItemType: 'a + LineStringTrait<'a, T>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn rings(&'a self) -> Self::Iter;
}

pub trait MultiPointTrait<'a, T: Float> {
    type ItemType: 'a + PointTrait<T>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn points(&'a self) -> Self::Iter;
}

pub trait MultiLineStringTrait<'a, T: Float> {
    type ItemType: 'a + LineStringTrait<'a, T>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn lines(&'a self) -> Self::Iter;
}

pub trait MultiPolygonTrait<'a, T: Float> {
    type ItemType: 'a + PolygonTrait<'a, T>;
    type Iter: Iterator<Item=&'a Self::ItemType>;

    fn polygons(&'a self) -> Self::Iter;
}
