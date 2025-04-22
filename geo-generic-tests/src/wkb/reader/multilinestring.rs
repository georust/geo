use std::io::Cursor;

use crate::wkb::common::WKBDimension;
use crate::wkb::reader::linestring::LineString;
use crate::wkb::reader::util::{has_srid, ReadBytesExt};
use crate::wkb::Endianness;
use geo_traits::MultiLineStringTrait;
use geo_traits_ext::{
    forward_multi_line_string_trait_ext_funcs, GeoTraitExtWithTypeTag, MultiLineStringTag,
    MultiLineStringTraitExt,
};

const HEADER_BYTES: u64 = 5;

/// A WKB MultiLineString
///
/// This has been preprocessed, so access to any internal coordinate is `O(1)`.
#[derive(Debug, Clone)]
pub struct MultiLineString<'a> {
    /// A LineString object for each of the internal line strings
    wkb_line_strings: Vec<LineString<'a>>,
    dim: WKBDimension,
    has_srid: bool,
}

impl<'a> MultiLineString<'a> {
    pub(crate) fn new(buf: &'a [u8], byte_order: Endianness, dim: WKBDimension) -> Self {
        let mut offset = 0;
        let has_srid = has_srid(buf, byte_order, offset);
        if has_srid {
            offset += 4;
        }

        let mut reader = Cursor::new(buf);
        reader.set_position(HEADER_BYTES + offset);
        let num_line_strings = reader.read_u32(byte_order).unwrap().try_into().unwrap();

        // - 1: byteOrder
        // - 4: wkbType
        // - 4: numLineStrings
        let mut line_string_offset = 1 + 4 + 4;
        if has_srid {
            line_string_offset += 4;
        }

        let mut wkb_line_strings = Vec::with_capacity(num_line_strings);
        for _ in 0..num_line_strings {
            let ls = LineString::new(buf, byte_order, line_string_offset, dim);
            wkb_line_strings.push(ls);
            line_string_offset += ls.size();
        }

        Self {
            wkb_line_strings,
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
        self.wkb_line_strings
            .iter()
            .fold(header, |acc, ls| acc + ls.size())
    }

    pub fn dimension(&self) -> WKBDimension {
        self.dim
    }
}

impl<'a> MultiLineStringTrait for MultiLineString<'a> {
    type InnerLineStringType<'b>
        = LineString<'a>
    where
        Self: 'b;

    fn num_line_strings(&self) -> usize {
        self.wkb_line_strings.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        *self.wkb_line_strings.get_unchecked(i)
    }
}

impl<'a, 'b> MultiLineStringTrait for &'b MultiLineString<'a>
where
    'a: 'b,
{
    type InnerLineStringType<'c>
        = LineString<'a>
    where
        Self: 'c;

    fn num_line_strings(&self) -> usize {
        self.wkb_line_strings.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        *self.wkb_line_strings.get_unchecked(i)
    }
}

impl MultiLineStringTraitExt for MultiLineString<'_> {
    forward_multi_line_string_trait_ext_funcs!();
}

impl GeoTraitExtWithTypeTag for MultiLineString<'_> {
    type Tag = MultiLineStringTag;
}

impl<'a, 'b> MultiLineStringTraitExt for &'b MultiLineString<'a>
where
    'a: 'b,
{
    forward_multi_line_string_trait_ext_funcs!();
}

impl<'a, 'b> GeoTraitExtWithTypeTag for &'b MultiLineString<'a>
where
    'a: 'b,
{
    type Tag = MultiLineStringTag;
}
