use delegate::delegate;

use geo_traits::*;
use geo_traits_ext::{
    forward_point_trait_ext_funcs, GeoTraitExtWithTypeTag, PointTag, PointTraitExt,
};
use geo_types::CoordNum;

use super::coord::SimpleCoord;

pub struct SimplePoint<T: CoordNum> {
    inner: Option<SimpleCoord<T>>,
}

impl<T: CoordNum> SimplePoint<T> {
    pub fn new(x: T, y: T) -> Self {
        Self {
            inner: Some(SimpleCoord::new(x, y)),
        }
    }
}

impl<'a, T: CoordNum> PointTrait for &'a SimplePoint<T> {
    type CoordType<'b>
        = &'a SimpleCoord<T>
    where
        Self: 'b;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        self.inner.as_ref()
    }
}

impl<T: CoordNum> PointTrait for SimplePoint<T> {
    type CoordType<'a>
        = &'a SimpleCoord<T>
    where
        Self: 'a;

    delegate! {
        to(&self) {
            fn coord(&self) -> Option<Self::CoordType<'_>>;
        }
    }
}

impl<T: CoordNum> PointTraitExt for SimplePoint<T> {
    forward_point_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for SimplePoint<T> {
    type Tag = PointTag;
}

impl<T: CoordNum> PointTraitExt for &SimplePoint<T> {
    forward_point_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &SimplePoint<T> {
    type Tag = PointTag;
}

#[cfg(test)]
mod tests {
    use geo_traits::CoordTrait;

    use super::*;

    #[test]
    fn test_point_trait() {
        let point = SimplePoint::new(1.0, 2.0);
        assert_eq!(point.dim(), geo_traits::Dimensions::Xy);
        let coord = point.coord().unwrap();
        assert_eq!(coord.nth_or_panic(0), 1.0);
        assert_eq!(coord.nth_or_panic(1), 2.0);
    }

    #[test]
    fn test_point_trait_ref() {
        let point = SimplePoint::new(1.0, 2.0);
        let point_ref = &point;
        assert_eq!(point_ref.dim(), geo_traits::Dimensions::Xy);
        let coord = point_ref.coord().unwrap();
        assert_eq!(coord.nth_or_panic(0), 1.0);
        assert_eq!(coord.nth_or_panic(1), 2.0);
    }
}
