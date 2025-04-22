use std::io::Cursor;

use crate::wkb::common::WKBDimension;
use crate::wkb::reader::util::ReadBytesExt;
use crate::wkb::Endianness;
use geo_traits::{CoordTrait, Dimensions};
use geo_traits_ext::{CoordTag, CoordTraitExt, GeoTraitExtWithTypeTag};

const F64_WIDTH: u64 = 8;

/// A coordinate in a WKB buffer.
///
/// Note that according to the WKB specification this is called `"Point"`, which is **not** the
/// same as a WKB "framed" `Point`. In particular, a "framed" `Point` has framing that includes the
/// byte order and geometry type of the WKB buffer. In contrast, this `Coord` is the building block
/// of two to four f64 numbers that can occur within any geometry type.
///
/// See page 65 of <https://portal.ogc.org/files/?artifact_id=25355>.
#[derive(Debug, Clone, Copy)]
pub struct Coord<'a> {
    /// The underlying WKB buffer
    buf: &'a [u8],

    /// The byte order of this WKB buffer
    byte_order: Endianness,

    /// The offset into the buffer where this coordinate is located
    ///
    /// Note that this does not have to be immediately after the WKB header! For a `Point`, the
    /// `Point` is immediately after the header, but the `Point` also appears in other geometry
    /// types. I.e. the `LineString` has a header, then the number of points, then a sequence of
    /// `Point` objects.
    offset: u64,

    dim: WKBDimension,
}

impl<'a> Coord<'a> {
    pub(crate) fn new(
        buf: &'a [u8],
        byte_order: Endianness,
        offset: u64,
        dim: WKBDimension,
    ) -> Self {
        Self {
            buf,
            byte_order,
            offset,
            dim,
        }
    }

    #[inline]
    fn get_x(&self) -> f64 {
        let mut reader = Cursor::new(self.buf);
        reader.set_position(self.offset);
        reader.read_f64(self.byte_order).unwrap()
    }

    #[inline]
    fn get_y(&self) -> f64 {
        let mut reader = Cursor::new(self.buf);
        reader.set_position(self.offset + F64_WIDTH);
        reader.read_f64(self.byte_order).unwrap()
    }

    #[inline]
    fn get_nth_unchecked(&self, n: usize) -> f64 {
        debug_assert!(n < self.dim.size());
        let mut reader = Cursor::new(self.buf);
        reader.set_position(self.offset + (n as u64 * F64_WIDTH));
        reader.read_f64(self.byte_order).unwrap()
    }

    /// The number of bytes in this object
    ///
    /// Note that this is not the same as the length of the underlying buffer
    pub fn size(&self) -> u64 {
        // A 2D Coord is just two f64s
        self.dim.size() as u64 * 8
    }

    pub fn slice(&self) -> &'a [u8] {
        &self.buf[self.offset as usize..self.offset as usize + self.size() as usize]
    }
}

impl CoordTrait for Coord<'_> {
    type T = f64;

    fn dim(&self) -> Dimensions {
        self.dim.into()
    }

    #[inline]
    fn nth_or_panic(&self, n: usize) -> Self::T {
        self.get_nth_unchecked(n)
    }

    #[inline]
    fn x(&self) -> Self::T {
        self.get_x()
    }

    #[inline]
    fn y(&self) -> Self::T {
        self.get_y()
    }
}

impl CoordTraitExt for Coord<'_> {}

impl GeoTraitExtWithTypeTag for Coord<'_> {
    type Tag = CoordTag;
}
