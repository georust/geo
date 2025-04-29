use std::marker::PhantomData;

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use geo_types::{Coord as GeoCoord, Line};

// Coordinate iterator with compile-time endianness
pub struct CoordIter<'a, B: ByteOrder> {
    buf: &'a [u8],
    base_offset: usize,
    remaining: usize,
    dim_size: usize,
    _marker: PhantomData<B>,
}

impl<'a, B: ByteOrder> CoordIter<'a, B> {
    pub fn new(buf: &'a [u8], base_offset: usize, num_points: usize, dim_size: usize) -> Self {
        Self {
            buf,
            base_offset,
            remaining: num_points,
            dim_size,
            _marker: PhantomData,
        }
    }
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

impl<'a, B: ByteOrder> LineIter<'a, B> {
    pub fn new(buf: &'a [u8], base_offset: usize, num_points: usize, dim_size: usize) -> Self {
        Self {
            coord_iter: CoordIter::new(buf, base_offset, num_points, dim_size),
            prev_coord: None,
        }
    }
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
