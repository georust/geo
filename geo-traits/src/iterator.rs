use super::{
    CoordTrait, GeometryCollectionTrait, GeometryTrait, LineStringTrait, MultiLineStringTrait,
    MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait,
};

macro_rules! impl_iterator {
    ($struct_name:ident, $self_trait:ident, $item_trait:ident, $access_method:ident, $item_type:ident) => {
        /// An iterator over the parts of this geometry.
        pub(crate) struct $struct_name<
            'a,
            $item_type: 'a + $item_trait,
            G: $self_trait<$item_type<'a> = $item_type>,
        > {
            geom: &'a G,
            index: usize,
            end: usize,
        }

        impl<'a, $item_type: 'a + $item_trait, G: $self_trait<$item_type<'a> = $item_type>>
            $struct_name<'a, $item_type, G>
        {
            /// Create a new iterator
            pub fn new(geom: &'a G, index: usize, end: usize) -> Self {
                Self { geom, index, end }
            }
        }

        impl<'a, $item_type: 'a + $item_trait, G: $self_trait<$item_type<'a> = $item_type>> Iterator
            for $struct_name<'a, $item_type, G>
        {
            type Item = $item_type;

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

        impl<'a, $item_type: 'a + $item_trait, G: $self_trait<$item_type<'a> = $item_type>>
            ExactSizeIterator for $struct_name<'a, $item_type, G>
        {
        }

        impl<'a, $item_type: 'a + $item_trait, G: $self_trait<$item_type<'a> = $item_type>>
            DoubleEndedIterator for $struct_name<'a, $item_type, G>
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
    coord_unchecked,
    CoordType
);
impl_iterator!(
    PolygonInteriorIterator,
    PolygonTrait,
    LineStringTrait,
    interior_unchecked,
    RingType
);
impl_iterator!(
    MultiPointIterator,
    MultiPointTrait,
    PointTrait,
    point_unchecked,
    PointType
);
impl_iterator!(
    MultiLineStringIterator,
    MultiLineStringTrait,
    LineStringTrait,
    line_string_unchecked,
    LineStringType
);
impl_iterator!(
    MultiPolygonIterator,
    MultiPolygonTrait,
    PolygonTrait,
    polygon_unchecked,
    PolygonType
);
impl_iterator!(
    GeometryCollectionIterator,
    GeometryCollectionTrait,
    GeometryTrait,
    geometry_unchecked,
    GeometryType
);
