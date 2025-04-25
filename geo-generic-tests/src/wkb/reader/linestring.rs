use std::io::Cursor;

use crate::wkb::common::WKBDimension;
use crate::wkb::reader::coord::Coord;
use crate::wkb::reader::util::{has_srid, ReadBytesExt};
use crate::wkb::Endianness;
use geo_traits::LineStringTrait;
use geo_traits_ext::{
    forward_line_string_trait_ext_funcs, GeoTraitExtWithTypeTag, LineStringTag, LineStringTraitExt,
};
use geo_types::Line;

const HEADER_BYTES: u64 = 5;

/// A WKB LineString
///
/// This has been preprocessed, so access to any internal coordinate is `O(1)`.
#[derive(Debug, Clone, Copy)]
pub struct LineString<'a> {
    buf: &'a [u8],
    byte_order: Endianness,

    /// The number of points in this LineString WKB
    num_points: usize,

    /// This offset will be 0 for a single LineString but it will be non zero for a
    /// LineString contained within a MultiLineString
    offset: u64,
    dim: WKBDimension,
    has_srid: bool,
}

impl<'a> LineString<'a> {
    pub fn new(buf: &'a [u8], byte_order: Endianness, mut offset: u64, dim: WKBDimension) -> Self {
        let has_srid = has_srid(buf, byte_order, offset);
        if has_srid {
            offset += 4;
        }

        let mut reader = Cursor::new(buf);
        reader.set_position(HEADER_BYTES + offset);
        let num_points = reader.read_u32(byte_order).unwrap().try_into().unwrap();

        Self {
            buf,
            byte_order,
            num_points,
            offset,
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
        // - 2 * 8 * self.num_points: two f64s for each coordinate
        let mut header = 1 + 4 + 4;
        if self.has_srid {
            header += 4;
        }
        header + (self.dim.size() as u64 * 8 * self.num_points as u64)
    }

    /// The offset into this buffer of any given coordinate
    pub fn coord_offset(&self, i: u64) -> u64 {
        self.offset + 1 + 4 + 4 + (self.dim.size() as u64 * 8 * i)
    }

    pub fn dimension(&self) -> WKBDimension {
        self.dim
    }
}

impl<'a> LineStringTrait for LineString<'a> {
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

impl<'a, 'b> LineStringTrait for &'b LineString<'a>
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

impl LineStringTraitExt for LineString<'_> {
    forward_line_string_trait_ext_funcs!();

    // Delegate to the `geo-types` implementation for less performance overhead
    #[inline(always)]
    fn lines(&'_ self) -> impl ExactSizeIterator<Item = Line<f64>> + '_ {
        // Initialize variables for direct memory access
        let num_coords = self.num_points;
        let base_offset = self.coord_offset(0) as usize;
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

        (0..num_coords.saturating_sub(1)).map(move |i| {
            let coord_pos = base_offset + ((i + 1) * dim_size * 8);

            // SAFETY: Same safety assumptions as in coord_iter
            let current_coord = unsafe {
                let x_bytes = std::slice::from_raw_parts(buf.as_ptr().add(coord_pos), 8);
                let y_bytes = std::slice::from_raw_parts(buf.as_ptr().add(coord_pos + 8), 8);

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
            line
        })
    }

    #[inline(always)]
    fn coord_iter(&self) -> impl Iterator<Item = geo_types::Coord<f64>> {
        let num_coords = self.num_points;
        let base_offset = self.coord_offset(0) as usize;
        let _byte_order = self.byte_order;
        let buf = self.buf;

        (0..num_coords).map(move |i| {
            let coord_pos = base_offset + (i * self.dim.size() * 8);

            // SAFETY: We're reading raw memory from the buffer at calculated offsets.
            // This assumes that:
            // 1. The buffer contains valid f64 data at these positions
            // 2. The offsets are correctly calculated and within bounds
            // 3. The dimension size ensures we don't read past the end of the buffer
            unsafe {
                let x_bytes = std::slice::from_raw_parts(buf.as_ptr().add(coord_pos), 8);
                let y_bytes = std::slice::from_raw_parts(buf.as_ptr().add(coord_pos + 8), 8);

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

impl GeoTraitExtWithTypeTag for LineString<'_> {
    type Tag = LineStringTag;
}

impl<'a, 'b> LineStringTraitExt for &'b LineString<'a>
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

impl<'a, 'b> GeoTraitExtWithTypeTag for &'b LineString<'a>
where
    'a: 'b,
{
    type Tag = LineStringTag;
}
