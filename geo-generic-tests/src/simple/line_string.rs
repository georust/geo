use delegate::delegate;
use geo_traits::*;
use geo_traits_ext::{
    forward_line_string_trait_ext_funcs, GeoTraitExtWithTypeTag, LineStringTag, LineStringTraitExt,
};
use geo_types::CoordNum;

use super::coord::SimpleCoord;

pub struct SimpleLineString<T: CoordNum> {
    inner: Vec<SimpleCoord<T>>,
}

impl<T: CoordNum> SimpleLineString<T> {
    pub fn new(coords: Vec<SimpleCoord<T>>) -> Self {
        Self { inner: coords }
    }
}

impl<'a, T: CoordNum> LineStringTrait for &'a SimpleLineString<T> {
    type CoordType<'b>
        = &'a SimpleCoord<T>
    where
        Self: 'b;

    fn num_coords(&self) -> usize {
        self.inner.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.inner.get_unchecked(i)
    }
}

impl<T: CoordNum> LineStringTrait for SimpleLineString<T> {
    type CoordType<'a>
        = &'a SimpleCoord<T>
    where
        Self: 'a;

    delegate! {
        to(&self) {
            fn num_coords(&self) -> usize;
            unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_>;
        }
    }
}

impl<T: CoordNum> LineStringTraitExt for SimpleLineString<T> {
    forward_line_string_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for SimpleLineString<T> {
    type Tag = LineStringTag;
}

impl<T: CoordNum> LineStringTraitExt for &SimpleLineString<T> {
    forward_line_string_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &SimpleLineString<T> {
    type Tag = LineStringTag;
}

#[cfg(test)]
mod tests {
    use geo_traits::CoordTrait;

    use super::*;

    #[test]
    fn test_line_string_trait() {
        let line_string =
            SimpleLineString::new(vec![SimpleCoord::new(0.0, 0.0), SimpleCoord::new(1.0, 1.0)]);

        assert_eq!(line_string.dim(), geo_traits::Dimensions::Xy);
        assert_eq!(line_string.num_coords(), 2);
        assert_eq!(line_string.coord(0).unwrap().nth_or_panic(0), 0.0);
        assert_eq!(line_string.coord(0).unwrap().nth_or_panic(1), 0.0);
        assert_eq!(line_string.coord(1).unwrap().nth_or_panic(0), 1.0);
        assert_eq!(line_string.coord(1).unwrap().nth_or_panic(1), 1.0);
    }

    #[test]
    fn test_line_string_trait_ref() {
        let line_string =
            SimpleLineString::new(vec![SimpleCoord::new(0.0, 0.0), SimpleCoord::new(1.0, 1.0)]);
        let line_string_ref = &line_string;

        assert_eq!(line_string_ref.dim(), geo_traits::Dimensions::Xy);
        assert_eq!(line_string_ref.num_coords(), 2);
        assert_eq!(line_string_ref.coord(0).unwrap().nth_or_panic(0), 0.0);
        assert_eq!(line_string_ref.coord(0).unwrap().nth_or_panic(1), 0.0);
        assert_eq!(line_string_ref.coord(1).unwrap().nth_or_panic(0), 1.0);
        assert_eq!(line_string_ref.coord(1).unwrap().nth_or_panic(1), 1.0);
    }
}
