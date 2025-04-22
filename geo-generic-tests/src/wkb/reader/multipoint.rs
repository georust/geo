use std::io::Cursor;

use crate::wkb::common::WKBDimension;
use crate::wkb::reader::point::Point;
use crate::wkb::reader::util::{has_srid, ReadBytesExt};
use crate::wkb::Endianness;
use geo_traits::MultiPointTrait;
use geo_traits_ext::{
    forward_multi_point_trait_ext_funcs, GeoTraitExtWithTypeTag, MultiPointTag, MultiPointTraitExt,
};

/// A WKB MultiPoint
///
/// This has been preprocessed, so access to any internal coordinate is `O(1)`.
#[derive(Debug, Clone, Copy)]
pub struct MultiPoint<'a> {
    buf: &'a [u8],
    byte_order: Endianness,

    /// The number of points in this multi point
    num_points: usize,
    dim: WKBDimension,
    has_srid: bool,
}

impl<'a> MultiPoint<'a> {
    pub(crate) fn new(buf: &'a [u8], byte_order: Endianness, dim: WKBDimension) -> Self {
        let mut offset = 0;
        let has_srid = has_srid(buf, byte_order, offset);
        if has_srid {
            offset += 4;
        }

        let mut reader = Cursor::new(buf);
        // Set reader to after 1-byte byteOrder and 4-byte wkbType
        reader.set_position(1 + 4 + offset);
        let num_points = reader.read_u32(byte_order).unwrap().try_into().unwrap();

        Self {
            buf,
            byte_order,
            num_points,
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
        // - Point::size() * self.num_points: the size of each Point for each point
        let mut header = 1 + 4 + 4;
        if self.has_srid {
            header += 4;
        }
        header + ((1 + 4 + (self.dim.size() as u64 * 8)) * self.num_points as u64)
    }

    /// The offset into this buffer of any given Point
    pub fn point_offset(&self, i: u64) -> u64 {
        // - 1: byteOrder
        // - 4: wkbType
        // - 4: numPoints
        let mut header = 1 + 4 + 4;
        if self.has_srid {
            header += 4;
        }
        header + ((1 + 4 + (self.dim.size() as u64 * 8)) * i)
    }

    pub fn dimension(&self) -> WKBDimension {
        self.dim
    }
}

impl<'a> MultiPointTrait for MultiPoint<'a> {
    type InnerPointType<'b>
        = Point<'a>
    where
        Self: 'b;

    fn num_points(&self) -> usize {
        self.num_points
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        Point::new(
            self.buf,
            self.byte_order,
            self.point_offset(i.try_into().unwrap()),
            self.dim,
        )
    }
}

impl<'a, 'b> MultiPointTrait for &'b MultiPoint<'a>
where
    'a: 'b,
{
    type InnerPointType<'c>
        = Point<'a>
    where
        Self: 'c;

    fn num_points(&self) -> usize {
        self.num_points
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        Point::new(
            self.buf,
            self.byte_order,
            self.point_offset(i.try_into().unwrap()),
            self.dim,
        )
    }
}

impl MultiPointTraitExt for MultiPoint<'_> {
    forward_multi_point_trait_ext_funcs!();
}

impl GeoTraitExtWithTypeTag for MultiPoint<'_> {
    type Tag = MultiPointTag;
}

impl<'a, 'b> MultiPointTraitExt for &'b MultiPoint<'a>
where
    'a: 'b,
{
    forward_multi_point_trait_ext_funcs!();
}

impl<'a, 'b> GeoTraitExtWithTypeTag for &'b MultiPoint<'a>
where
    'a: 'b,
{
    type Tag = MultiPointTag;
}
