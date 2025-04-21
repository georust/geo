use super::{
    CoordTrait, GeometryCollectionTrait, GeometryTrait, LineStringTrait, MultiLineStringTrait,
    MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait,
};

macro_rules! impl_iterator {
    ($struct_name:ident, $self_trait:ident, $item_trait:ident, $access_method:ident, $item_type:ident, $associated_type:ident) => {
        /// An iterator over the parts of this geometry.
        pub(crate) struct $struct_name<
            'a,
            T,
            $item_type: 'a + $item_trait<T = T>,
            G: $self_trait<T = T, $associated_type<'a> = $item_type>,
        > {
            geom: &'a G,
            index: usize,
            end: usize,
        }

        impl<
                'a,
                T,
                $item_type: 'a + $item_trait<T = T>,
                G: $self_trait<T = T, $associated_type<'a> = $item_type>,
            > $struct_name<'a, T, $item_type, G>
        {
            /// Create a new iterator
            pub fn new(geom: &'a G, index: usize, end: usize) -> Self {
                Self { geom, index, end }
            }
        }

        impl<
                'a,
                T,
                $item_type: 'a + $item_trait<T = T>,
                G: $self_trait<T = T, $associated_type<'a> = $item_type>,
            > Iterator for $struct_name<'a, T, $item_type, G>
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

        impl<
                'a,
                T,
                $item_type: 'a + $item_trait<T = T>,
                G: $self_trait<T = T, $associated_type<'a> = $item_type>,
            > ExactSizeIterator for $struct_name<'a, T, $item_type, G>
        {
        }

        impl<
                'a,
                T,
                $item_type: 'a + $item_trait<T = T>,
                G: $self_trait<T = T, $associated_type<'a> = $item_type>,
            > DoubleEndedIterator for $struct_name<'a, T, $item_type, G>
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
    LineStringIterator, // struct_name
    LineStringTrait,    // self_trait
    CoordTrait,         // item_trait
    coord_unchecked,    // access_method
    CoordType,          // item_type
    CoordType           // associated_type
);
impl_iterator!(
    PolygonInteriorIterator, // struct_name
    PolygonTrait,            // self_trait
    LineStringTrait,         // item_trait
    interior_unchecked,      // access_method
    RingType,                // item_type
    RingType                 // associated_type
);
impl_iterator!(
    MultiPointIterator, // struct_name
    MultiPointTrait,    // self_trait
    PointTrait,         // item_trait
    point_unchecked,    // access_method
    PointType,          // item_type
    InnerPointType      // associated_type
);
impl_iterator!(
    MultiLineStringIterator, // struct_name
    MultiLineStringTrait,    // self_trait
    LineStringTrait,         // item_trait
    line_string_unchecked,   // access_method
    LineStringType,          // item_type
    InnerLineStringType      // associated_type
);
impl_iterator!(
    MultiPolygonIterator, // struct_name
    MultiPolygonTrait,    // self_trait
    PolygonTrait,         // item_trait
    polygon_unchecked,    // access_method
    PolygonType,          // item_type
    InnerPolygonType      // associated_type
);
impl_iterator!(
    GeometryCollectionIterator, // struct_name
    GeometryCollectionTrait,    // self_trait
    GeometryTrait,              // item_trait
    geometry_unchecked,         // access_method
    GeometryType,               // item_type
    GeometryType                // associated_type
);
