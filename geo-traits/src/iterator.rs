use super::{
    CoordTrait, GeometryCollectionTrait, GeometryTrait, LineStringTrait, MultiLineStringTrait,
    MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait,
};
use geo_types::CoordNum;

macro_rules! impl_iterator {
    ($struct_name:ident, $self_trait:ident, $item_trait:ident, $access_method:ident) => {
        /// An iterator over the parts of this geometry.
        pub struct $struct_name<
            'a,
            T: CoordNum,
            ItemType: 'a + $item_trait<T = T>,
            G: $self_trait<T = T, ItemType<'a> = ItemType>,
        > {
            geom: &'a G,
            index: usize,
            end: usize,
        }

        impl<
                'a,
                T: CoordNum,
                ItemType: 'a + $item_trait<T = T>,
                G: $self_trait<T = T, ItemType<'a> = ItemType>,
            > $struct_name<'a, T, ItemType, G>
        {
            pub fn new(geom: &'a G, index: usize, end: usize) -> Self {
                Self { geom, index, end }
            }
        }

        impl<
                'a,
                T: CoordNum,
                ItemType: 'a + $item_trait<T = T>,
                G: $self_trait<T = T, ItemType<'a> = ItemType>,
            > Iterator for $struct_name<'a, T, ItemType, G>
        {
            type Item = ItemType;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.index == self.end {
                    return None;
                }
                let old = self.index;
                self.index += 1;
                unsafe { Some(self.geom.$access_method(old)) }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.end - self.index, Some(self.end - self.index))
            }
        }

        impl<
                'a,
                T: CoordNum,
                ItemType: 'a + $item_trait<T = T>,
                G: $self_trait<T = T, ItemType<'a> = ItemType>,
            > DoubleEndedIterator for $struct_name<'a, T, ItemType, G>
        {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.index == self.end {
                    None
                } else {
                    self.end -= 1;
                    unsafe { Some(self.geom.$access_method(self.end)) }
                }
            }
        }
    };
}

impl_iterator!(
    LineStringIterator,
    LineStringTrait,
    CoordTrait,
    coord_unchecked
);
impl_iterator!(
    PolygonInteriorIterator,
    PolygonTrait,
    LineStringTrait,
    interior_unchecked
);
impl_iterator!(
    MultiPointIterator,
    MultiPointTrait,
    PointTrait,
    point_unchecked
);
impl_iterator!(
    MultiLineStringIterator,
    MultiLineStringTrait,
    LineStringTrait,
    line_unchecked
);
impl_iterator!(
    MultiPolygonIterator,
    MultiPolygonTrait,
    PolygonTrait,
    polygon_unchecked
);
impl_iterator!(
    GeometryCollectionIterator,
    GeometryCollectionTrait,
    GeometryTrait,
    geometry_unchecked
);
