use byteorder::{BigEndian, LittleEndian};

use crate::wkb::common::WKBGeometryCode;
use crate::wkb::Endianness;
use std::io::{Cursor, Error};

pub(crate) trait ReadBytesExt: byteorder::ReadBytesExt {
    fn read_u32(&mut self, byte_order: Endianness) -> Result<u32, Error> {
        match byte_order {
            Endianness::BigEndian => byteorder::ReadBytesExt::read_u32::<BigEndian>(self),
            Endianness::LittleEndian => byteorder::ReadBytesExt::read_u32::<LittleEndian>(self),
        }
    }

    fn read_f64(&mut self, byte_order: Endianness) -> Result<f64, Error> {
        match byte_order {
            Endianness::BigEndian => byteorder::ReadBytesExt::read_f64::<BigEndian>(self),
            Endianness::LittleEndian => byteorder::ReadBytesExt::read_f64::<LittleEndian>(self),
        }
    }
}

/// All types that implement `Read` get methods defined in `ReadBytesExt`
/// for free.
impl<R: std::io::Read + ?Sized> ReadBytesExt for R {}

/// Return `true` if this WKB item is EWKB and has an embedded SRID
pub(crate) fn has_srid(buf: &[u8], byte_order: Endianness, offset: u64) -> bool {
    // Read geometry code to see if an SRID exists.
    let mut reader = Cursor::new(buf);

    // Skip 1-byte byte order that we already know
    reader.set_position(1 + offset);

    let geometry_code = WKBGeometryCode::new(reader.read_u32(byte_order).unwrap());
    geometry_code.has_srid()
}
