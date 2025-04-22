use delegate::delegate;

use geo_traits::CoordTrait;
use geo_traits_ext::{CoordTag, CoordTraitExt, GeoTraitExtWithTypeTag};
use geo_types::CoordNum;

pub struct SimpleCoord<T: CoordNum> {
    x: T,
    y: T,
}

impl<T: CoordNum> SimpleCoord<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: CoordNum> CoordTrait for &SimpleCoord<T> {
    type T = T;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x,
            1 => self.y,
            _ => panic!("Invalid dimension: {}", n),
        }
    }
}

impl<T: CoordNum> CoordTrait for SimpleCoord<T> {
    type T = T;

    delegate! {
        to(&self) {
            fn dim(&self) -> geo_traits::Dimensions;
            fn x(&self) -> Self::T;
            fn y(&self) -> Self::T;
            fn nth_or_panic(&self, n: usize) -> Self::T;
        }
    }
}

impl<T: CoordNum> CoordTraitExt for SimpleCoord<T> {}

impl<T: CoordNum> GeoTraitExtWithTypeTag for SimpleCoord<T> {
    type Tag = CoordTag;
}

impl<T: CoordNum> CoordTraitExt for &SimpleCoord<T> {}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &SimpleCoord<T> {
    type Tag = CoordTag;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coord_trait() {
        let coord = SimpleCoord::new(1.0, 2.0);
        assert_eq!(coord.dim(), geo_traits::Dimensions::Xy);
        assert_eq!(coord.x(), 1.0);
        assert_eq!(coord.y(), 2.0);
    }

    #[test]
    fn test_coord_trait_ref() {
        let coord = SimpleCoord::new(1.0, 2.0);
        let coord_ref = &coord;
        assert_eq!(coord_ref.dim(), geo_traits::Dimensions::Xy);
        assert_eq!(coord_ref.x(), 1.0);
        assert_eq!(coord_ref.y(), 2.0);
    }
}
