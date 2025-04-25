use std::io::Cursor;

use crate::wkb::common::WKBDimension;
use crate::wkb::reader::coord::Coord;
use crate::wkb::reader::util::ReadBytesExt;
use crate::wkb::Endianness;
use geo_traits::LineStringTrait;
use geo_traits_ext::forward_line_string_trait_ext_funcs;
use geo_traits_ext::LineStringTraitExt;
use geo_traits_ext::{GeoTraitExtWithTypeTag, LineStringTag};
use geo_types::Line;

/// A linear ring in a WKB buffer.
///
/// This has been preprocessed, so access to any internal coordinate is `O(1)`.
///
/// See page 65 of <https://portal.ogc.org/files/?artifact_id=25355>.
#[derive(Debug, Clone, Copy)]
pub struct WKBLinearRing<'a> {
    /// The underlying WKB buffer
    buf: &'a [u8],

    /// The byte order of this WKB buffer
    byte_order: Endianness,

    /// The offset into the buffer where this linear ring is located
    ///
    /// Note that this does not have to be immediately after the WKB header! For a `Point`, the
    /// `Point` is immediately after the header, but the `Point` also appears in other geometry
    /// types. I.e. the `LineString` has a header, then the number of points, then a sequence of
    /// `Point` objects.
    offset: u64,

    /// The number of points in this linear ring
    num_points: usize,

    dim: WKBDimension,
}

impl<'a> WKBLinearRing<'a> {
    pub fn new(buf: &'a [u8], byte_order: Endianness, offset: u64, dim: WKBDimension) -> Self {
        let mut reader = Cursor::new(buf);
        reader.set_position(offset);
        let num_points = reader.read_u32(byte_order).unwrap().try_into().unwrap();

        Self {
            buf,
            byte_order,
            offset,
            num_points,
            dim,
        }
    }

    /// The number of bytes in this object, including any header
    ///
    /// Note that this is not the same as the length of the underlying buffer
    pub fn size(&self) -> u64 {
        // - 4: numPoints
        // - 2 * 8 * self.num_points: two f64s for each coordinate
        4 + (self.dim.size() as u64 * 8 * self.num_points as u64)
    }

    /// The offset into this buffer of any given coordinate
    #[inline]
    pub fn coord_offset(&self, i: u64) -> u64 {
        self.offset + 4 + (self.dim.size() as u64 * 8 * i)
    }

    pub fn dimension(&self) -> WKBDimension {
        self.dim
    }
}

impl<'a> LineStringTrait for WKBLinearRing<'a> {
    type CoordType<'b>
        = Coord<'a>
    where
        Self: 'b;

    #[inline]
    fn num_coords(&self) -> usize {
        self.num_points
    }

    #[inline]
    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        Coord::new(
            self.buf,
            self.byte_order,
            self.coord_offset(i.try_into().unwrap()),
            self.dim,
        )
    }
}

impl LineStringTraitExt for WKBLinearRing<'_> {
    forward_line_string_trait_ext_funcs!();

    // Delegate to the `geo-types` implementation for less performance overhead
    #[inline(always)]
    fn lines(&'_ self) -> impl ExactSizeIterator<Item = Line<f64>> + '_ {
        // Initialize variables for direct memory access
        let num_coords = self.num_points;
        let mut base_offset = self.coord_offset(0) as usize;
        let _byte_order = self.byte_order;
        let buf = self.buf;
        let dim_size = self.dim.size();

        // Read the first coordinate using unsafe code
        let mut prev_coord = if num_coords > 0 {
            // SAFETY: Same safety assumptions as in coord_iter
            unsafe {
                let x_bytes = std::slice::from_raw_parts(buf.as_ptr().add(base_offset), 8);
                let y_bytes = std::slice::from_raw_parts(buf.as_ptr().add(base_offset + 8), 8);

                // let x = match byte_order {
                //     Endianness::LittleEndian => f64::from_le_bytes(x_bytes.try_into().unwrap()),
                //     Endianness::BigEndian => f64::from_be_bytes(x_bytes.try_into().unwrap()),
                // };
                let x = f64::from_le_bytes(x_bytes.try_into().unwrap());

                // let y = match byte_order {
                //     Endianness::LittleEndian => f64::from_le_bytes(y_bytes.try_into().unwrap()),
                //     Endianness::BigEndian => f64::from_be_bytes(y_bytes.try_into().unwrap()),
                // };
                let y = f64::from_le_bytes(y_bytes.try_into().unwrap());

                geo_types::Coord { x, y }
            }
        } else {
            geo_types::Coord::default()
        };

        base_offset += dim_size * 8;
        (0..num_coords.saturating_sub(1)).map(move |_i| {
            // SAFETY: Same safety assumptions as in coord_iter
            let current_coord = unsafe {
                let x_bytes = std::slice::from_raw_parts(buf.as_ptr().add(base_offset), 8);
                let y_bytes = std::slice::from_raw_parts(buf.as_ptr().add(base_offset + 8), 8);

                // let x = match byte_order {
                //     Endianness::LittleEndian => f64::from_le_bytes(x_bytes.try_into().unwrap()),
                //     Endianness::BigEndian => f64::from_be_bytes(x_bytes.try_into().unwrap()),
                // };
                let x = f64::from_le_bytes(x_bytes.try_into().unwrap());

                // let y = match byte_order {
                //     Endianness::LittleEndian => f64::from_le_bytes(y_bytes.try_into().unwrap()),
                //     Endianness::BigEndian => f64::from_be_bytes(y_bytes.try_into().unwrap()),
                // };
                let y = f64::from_le_bytes(y_bytes.try_into().unwrap());

                geo_types::Coord { x, y }
            };

            let line = Line::new(prev_coord, current_coord);
            prev_coord = current_coord;
            base_offset += dim_size * 8;
            line
        })
    }

    #[inline(always)]
    fn coord_iter(&self) -> impl Iterator<Item = geo_types::Coord<f64>> {
        let num_coords = self.num_points;
        let mut base_offset = self.coord_offset(0) as usize;
        let _byte_order = self.byte_order;
        let buf = self.buf;
        let dim_size = self.dim.size();

        (0..num_coords).map(move |_i| {
            // SAFETY: We're reading raw memory from the buffer at calculated offsets.
            // This assumes that:
            // 1. The buffer contains valid f64 data at these positions
            // 2. The offsets are correctly calculated and within bounds
            // 3. The dimension size ensures we don't read past the end of the buffer
            unsafe {
                let x_bytes = std::slice::from_raw_parts(buf.as_ptr().add(base_offset), 8);
                let y_bytes = std::slice::from_raw_parts(buf.as_ptr().add(base_offset + 8), 8);
                base_offset += dim_size * 8;

                // let x = match byte_order {
                //     Endianness::LittleEndian => f64::from_le_bytes(x_bytes.try_into().unwrap()),
                //     Endianness::BigEndian => f64::from_be_bytes(x_bytes.try_into().unwrap()),
                // };
                let x = f64::from_le_bytes(x_bytes.try_into().unwrap());

                // let y = match byte_order {
                //     Endianness::LittleEndian => f64::from_le_bytes(y_bytes.try_into().unwrap()),
                //     Endianness::BigEndian => f64::from_be_bytes(y_bytes.try_into().unwrap()),
                // };
                let y = f64::from_le_bytes(y_bytes.try_into().unwrap());

                geo_types::Coord { x, y }
            }
        })
    }
}

impl GeoTraitExtWithTypeTag for WKBLinearRing<'_> {
    type Tag = LineStringTag;
}

impl<'a, 'b> LineStringTrait for &'b WKBLinearRing<'a>
where
    'a: 'b,
{
    type CoordType<'c>
        = Coord<'a>
    where
        Self: 'c;

    #[inline]
    fn num_coords(&self) -> usize {
        self.num_points
    }

    #[inline]
    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        Coord::new(
            self.buf,
            self.byte_order,
            self.coord_offset(i.try_into().unwrap()),
            self.dim,
        )
    }
}

impl<'a, 'b> LineStringTraitExt for &'b WKBLinearRing<'a>
where
    'a: 'b,
{
    forward_line_string_trait_ext_funcs!();

    #[inline(always)]
    fn lines(&'_ self) -> impl ExactSizeIterator<Item = Line<f64>> + '_ {
        (*self).lines()
    }

    #[inline(always)]
    fn coord_iter(&self) -> impl Iterator<Item = geo_types::Coord<f64>> {
        (*self).coord_iter()
    }
}

impl<'a, 'b> GeoTraitExtWithTypeTag for &'b WKBLinearRing<'a>
where
    'a: 'b,
{
    type Tag = LineStringTag;
}
