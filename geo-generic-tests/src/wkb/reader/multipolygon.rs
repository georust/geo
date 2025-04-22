use std::io::Cursor;

use crate::wkb::common::WKBDimension;
use crate::wkb::reader::polygon::Polygon;
use crate::wkb::reader::util::{has_srid, ReadBytesExt};
use crate::wkb::Endianness;
use geo_traits::Dimensions;
use geo_traits::MultiPolygonTrait;
use geo_traits_ext::{
    forward_multi_polygon_trait_ext_funcs, GeoTraitExtWithTypeTag, MultiPolygonTag,
    MultiPolygonTraitExt,
};

/// skip endianness and wkb type
const HEADER_BYTES: u64 = 5;

/// A WKB MultiPolygon
#[derive(Debug, Clone)]
pub struct MultiPolygon<'a> {
    /// A Polygon object for each of the internal line strings
    wkb_polygons: Vec<Polygon<'a>>,

    dim: WKBDimension,
    has_srid: bool,
}

impl<'a> MultiPolygon<'a> {
    pub(crate) fn new(buf: &'a [u8], byte_order: Endianness, dim: WKBDimension) -> Self {
        let mut offset = 0;
        let has_srid = has_srid(buf, byte_order, offset);
        if has_srid {
            offset += 4;
        }

        let mut reader = Cursor::new(buf);
        reader.set_position(HEADER_BYTES + offset);
        let num_polygons = reader.read_u32(byte_order).unwrap().try_into().unwrap();

        // - 1: byteOrder
        // - 4: wkbType
        // - 4: numLineStrings
        let mut polygon_offset = 1 + 4 + 4;
        if has_srid {
            polygon_offset += 4;
        }

        let mut wkb_polygons = Vec::with_capacity(num_polygons);
        for _ in 0..num_polygons {
            let polygon = Polygon::new(buf, byte_order, polygon_offset, dim);
            polygon_offset += polygon.size();
            wkb_polygons.push(polygon);
        }

        Self {
            wkb_polygons,
            dim,
            has_srid,
        }
    }

    /// The number of bytes in this object, including any header
    ///
    /// Note that this is not the same as the length of the underlying buffer
    pub fn size(&self) -> u64 {
        // - 1: byteOrder
        // - 4: wkbType
        // - 4: numPolygons
        let mut header = 1 + 4 + 4;
        if self.has_srid {
            header += 4;
        }
        self.wkb_polygons
            .iter()
            .fold(header, |acc, x| acc + x.size())
    }

    pub fn dimension(&self) -> WKBDimension {
        self.dim
    }
}

impl<'a> MultiPolygonTrait for MultiPolygon<'a> {
    type InnerPolygonType<'b>
        = Polygon<'a>
    where
        Self: 'b;

    fn num_polygons(&self) -> usize {
        self.wkb_polygons.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        self.wkb_polygons.get_unchecked(i).clone()
    }
}

impl<'a, 'b> MultiPolygonTrait for &'b MultiPolygon<'a>
where
    'a: 'b,
{
    type InnerPolygonType<'c>
        = Polygon<'a>
    where
        Self: 'c;

    fn num_polygons(&self) -> usize {
        self.wkb_polygons.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        self.wkb_polygons.get_unchecked(i).clone()
    }
}

impl MultiPolygonTraitExt for MultiPolygon<'_> {
    forward_multi_polygon_trait_ext_funcs!();
}

impl GeoTraitExtWithTypeTag for MultiPolygon<'_> {
    type Tag = MultiPolygonTag;
}

impl<'a, 'b> MultiPolygonTraitExt for &'b MultiPolygon<'a>
where
    'a: 'b,
{
    forward_multi_polygon_trait_ext_funcs!();
}

impl<'a, 'b> GeoTraitExtWithTypeTag for &'b MultiPolygon<'a>
where
    'a: 'b,
{
    type Tag = MultiPolygonTag;
}
