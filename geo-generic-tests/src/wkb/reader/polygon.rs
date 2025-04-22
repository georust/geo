use std::io::Cursor;

use crate::wkb::common::WKBDimension;
use crate::wkb::reader::linearring::WKBLinearRing;
use crate::wkb::reader::util::{has_srid, ReadBytesExt};
use crate::wkb::Endianness;
use geo_traits::PolygonTrait;
use geo_traits_ext::{
    forward_polygon_trait_ext_funcs, GeoTraitExtWithTypeTag, PolygonTag, PolygonTraitExt,
};

/// skip endianness and wkb type
const HEADER_BYTES: u64 = 5;

/// A WKB Polygon
///
/// This has been preprocessed, so access to any internal coordinate is `O(1)`.
#[derive(Debug, Clone)]
pub struct Polygon<'a> {
    wkb_linear_rings: Vec<WKBLinearRing<'a>>,
    dim: WKBDimension,
    has_srid: bool,
}

impl<'a> Polygon<'a> {
    pub fn new(buf: &'a [u8], byte_order: Endianness, mut offset: u64, dim: WKBDimension) -> Self {
        let has_srid = has_srid(buf, byte_order, offset);
        if has_srid {
            offset += 4;
        }

        let mut reader = Cursor::new(buf);
        reader.set_position(HEADER_BYTES + offset);

        let num_rings = reader.read_u32(byte_order).unwrap().try_into().unwrap();

        // - existing offset into buffer
        // - 1: byteOrder
        // - 4: wkbType
        // - 4: numLineStrings
        let mut ring_offset = offset + 1 + 4 + 4;
        let mut wkb_linear_rings = Vec::with_capacity(num_rings);
        for _ in 0..num_rings {
            let polygon = WKBLinearRing::new(buf, byte_order, ring_offset, dim);
            wkb_linear_rings.push(polygon);
            ring_offset += polygon.size();
        }

        Self {
            wkb_linear_rings,
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
        // - 4: numPoints
        // - size of each linear ring
        let mut header = 1 + 4 + 4;
        if self.has_srid {
            header += 4;
        }

        self.wkb_linear_rings
            .iter()
            .fold(header, |acc, ring| acc + ring.size())
    }

    pub fn dimension(&self) -> WKBDimension {
        self.dim
    }
}

impl<'a> PolygonTrait for Polygon<'a> {
    type RingType<'b>
        = WKBLinearRing<'a>
    where
        Self: 'b;

    fn num_interiors(&self) -> usize {
        // Support an empty polygon with no rings
        if self.wkb_linear_rings.is_empty() {
            0
        } else {
            self.wkb_linear_rings.len() - 1
        }
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        if self.wkb_linear_rings.is_empty() {
            None
        } else {
            Some(self.wkb_linear_rings[0])
        }
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        *self.wkb_linear_rings.get_unchecked(i + 1)
    }
}

impl<'a, 'b> PolygonTrait for &'b Polygon<'a>
where
    'a: 'b,
{
    type RingType<'c>
        = WKBLinearRing<'a>
    where
        Self: 'c;

    fn num_interiors(&self) -> usize {
        // Support an empty polygon with no rings
        if self.wkb_linear_rings.is_empty() {
            0
        } else {
            self.wkb_linear_rings.len() - 1
        }
    }

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        if self.wkb_linear_rings.is_empty() {
            None
        } else {
            Some(self.wkb_linear_rings[0])
        }
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        *self.wkb_linear_rings.get_unchecked(i + 1)
    }
}

impl PolygonTraitExt for Polygon<'_> {
    forward_polygon_trait_ext_funcs!();
}

impl GeoTraitExtWithTypeTag for Polygon<'_> {
    type Tag = PolygonTag;
}

impl<'a, 'b> PolygonTraitExt for &'b Polygon<'a>
where
    'a: 'b,
{
    forward_polygon_trait_ext_funcs!();
}

impl<'a, 'b> GeoTraitExtWithTypeTag for &'b Polygon<'a>
where
    'a: 'b,
{
    type Tag = PolygonTag;
}
