use std::io::Cursor;
use std::marker::PhantomData;

use crate::wkb::common::WKBDimension;
use crate::wkb::reader::coord::Coord;
use crate::wkb::reader::util::{has_srid, ReadBytesExt};
use crate::wkb::Endianness;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use geo_traits::LineStringTrait;
use geo_traits_ext::{
    forward_line_string_trait_ext_funcs, GeoTraitExtWithTypeTag, LineStringTag, LineStringTraitExt,
};
use geo_types::{Coord as GeoCoord, Line};

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

// Coordinate iterator with compile-time endianness
pub struct CoordIter<'a, B: ByteOrder> {
    buf: &'a [u8],
    base_offset: usize,
    remaining: usize,
    dim_size: usize,
    _marker: PhantomData<B>,
}

impl<B: ByteOrder> Iterator for CoordIter<'_, B> {
    type Item = GeoCoord<f64>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        // SAFETY: We're reading raw memory from the buffer at calculated offsets.
        // This assumes the buffer contains valid data and offsets are within bounds.
        let coord = unsafe {
            let x_bytes = std::slice::from_raw_parts(self.buf.as_ptr().add(self.base_offset), 8);
            let y_bytes =
                std::slice::from_raw_parts(self.buf.as_ptr().add(self.base_offset + 8), 8);

            let x = B::read_f64(x_bytes);
            let y = B::read_f64(y_bytes);

            GeoCoord { x, y }
        };

        self.base_offset += self.dim_size * 8;
        self.remaining -= 1;

        Some(coord)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<B: ByteOrder> ExactSizeIterator for CoordIter<'_, B> {}

// Line iterator with compile-time endianness
pub struct LineIter<'a, B: ByteOrder> {
    coord_iter: CoordIter<'a, B>,
    prev_coord: Option<GeoCoord<f64>>,
}

impl<B: ByteOrder> Iterator for LineIter<'_, B> {
    type Item = Line<f64>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current_coord = self.coord_iter.next()?;

        let prev_coord = match self.prev_coord {
            Some(coord) => coord,
            None => {
                // Store the first coordinate and get the next one
                self.prev_coord = Some(current_coord);
                let next_coord = self.coord_iter.next()?;
                let line = Line::new(current_coord, next_coord);
                self.prev_coord = Some(next_coord);
                return Some(line);
            }
        };

        let line = Line::new(prev_coord, current_coord);
        self.prev_coord = Some(current_coord);
        Some(line)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.coord_iter.size_hint();
        (min.saturating_sub(1), max.map(|m| m.saturating_sub(1)))
    }
}

impl<B: ByteOrder> ExactSizeIterator for LineIter<'_, B> {}

// Enum-based wrappers to handle the different endianness types without boxing.
// The dispatch in the iterator methods is static and can be inlined by the compiler.
pub enum EndianCoordIter<'a> {
    LE(CoordIter<'a, LittleEndian>),
    BE(CoordIter<'a, BigEndian>),
}

impl Iterator for EndianCoordIter<'_> {
    type Item = GeoCoord<f64>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // We rely on compiler optimization to hoist the match out of the loop, so that
        // there's no performance overhead of checking the endianness inside the loop.
        match self {
            EndianCoordIter::LE(iter) => iter.next(),
            EndianCoordIter::BE(iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            EndianCoordIter::LE(iter) => iter.size_hint(),
            EndianCoordIter::BE(iter) => iter.size_hint(),
        }
    }
}

pub enum EndianLineIter<'a> {
    LE(LineIter<'a, LittleEndian>),
    BE(LineIter<'a, BigEndian>),
}

impl Iterator for EndianLineIter<'_> {
    type Item = Line<f64>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EndianLineIter::LE(iter) => iter.next(),
            EndianLineIter::BE(iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            EndianLineIter::LE(iter) => iter.size_hint(),
            EndianLineIter::BE(iter) => iter.size_hint(),
        }
    }
}

impl ExactSizeIterator for EndianLineIter<'_> {}

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

    // Create a coordinate iterator with compile-time endianness
    #[inline]
    fn create_coord_iter<B: ByteOrder>(&self) -> CoordIter<'a, B> {
        CoordIter {
            buf: self.buf,
            base_offset: self.coord_offset(0) as usize,
            remaining: self.num_points,
            dim_size: self.dim.size(),
            _marker: PhantomData,
        }
    }

    // Create a line iterator with compile-time endianness
    #[inline]
    fn create_line_iter<B: ByteOrder>(&self) -> LineIter<'a, B> {
        LineIter {
            coord_iter: self.create_coord_iter::<B>(),
            prev_coord: None,
        }
    }

    // Helper methods that return enum-based iterators based on endianness
    #[inline]
    fn get_coord_iter(&self) -> EndianCoordIter<'a> {
        match self.byte_order {
            Endianness::LittleEndian => {
                EndianCoordIter::LE(self.create_coord_iter::<LittleEndian>())
            }
            Endianness::BigEndian => EndianCoordIter::BE(self.create_coord_iter::<BigEndian>()),
        }
    }

    #[inline]
    fn get_line_iter(&self) -> EndianLineIter<'a> {
        match self.byte_order {
            Endianness::LittleEndian => EndianLineIter::LE(self.create_line_iter::<LittleEndian>()),
            Endianness::BigEndian => EndianLineIter::BE(self.create_line_iter::<BigEndian>()),
        }
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

    #[inline(always)]
    fn lines(&'_ self) -> impl ExactSizeIterator<Item = Line<f64>> + '_ {
        self.get_line_iter()
    }

    #[inline(always)]
    fn coord_iter(&self) -> impl Iterator<Item = GeoCoord<f64>> {
        self.get_coord_iter()
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
        (*self).get_line_iter()
    }

    #[inline(always)]
    fn coord_iter(&self) -> impl Iterator<Item = GeoCoord<f64>> {
        (*self).get_coord_iter()
    }
}

impl<'a, 'b> GeoTraitExtWithTypeTag for &'b LineString<'a>
where
    'a: 'b,
{
    type Tag = LineStringTag;
}
