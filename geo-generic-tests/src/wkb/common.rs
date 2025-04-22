use std::io::Cursor;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::wkb::error::{WKBError, WKBResult};

/// Bit flag for EWKB Geometry with a z coordinate
const EWKB_FLAG_Z: u32 = 0x80000000;
/// Bit flag for EWKB Geometry with an m coordinate
const EWKB_FLAG_M: u32 = 0x40000000;
/// Bit flag for EWKB Geometry with an embedded SRID
const EWKB_FLAG_SRID: u32 = 0x20000000;

/// Supported WKB dimensions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WKBDimension {
    Xy,
    Xyz,
    Xym,
    Xyzm,
}

impl WKBDimension {
    fn as_u32_offset(&self) -> u32 {
        match self {
            Self::Xy => 0,
            Self::Xyz => 1000,
            Self::Xym => 2000,
            Self::Xyzm => 3000,
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Self::Xy => 2,
            Self::Xyz | Self::Xym => 3,
            Self::Xyzm => 4,
        }
    }
}

impl TryFrom<geo_traits::Dimensions> for WKBDimension {
    type Error = WKBError;

    fn try_from(value: geo_traits::Dimensions) -> Result<Self, Self::Error> {
        use geo_traits::Dimensions::*;

        let result = match value {
            Xy | Unknown(2) => Self::Xy,
            Xyz | Unknown(3) => Self::Xyz,
            Xym => Self::Xym,
            Xyzm | Unknown(4) => Self::Xyzm,
            Unknown(n_dim) => {
                return Err(WKBError::General(format!(
                    "Unsupported number of dimensions: {}",
                    n_dim
                )))
            }
        };
        Ok(result)
    }
}

impl From<WKBDimension> for geo_traits::Dimensions {
    fn from(value: WKBDimension) -> Self {
        match value {
            WKBDimension::Xy => Self::Xy,
            WKBDimension::Xyz => Self::Xyz,
            WKBDimension::Xym => Self::Xym,
            WKBDimension::Xyzm => Self::Xyzm,
        }
    }
}

/// The geometry "code" of the WKB buffer
///
/// This is the four-byte `u32` directly after the one-byte endianness.
///
/// In ISO WKB this tells the geometry type and dimension of the buffer.
/// In extended WKB this additionally informs whether there's a u32 SRID immediately after this,
/// which we need to know to skip.
#[repr(transparent)]
pub(crate) struct WKBGeometryCode(u32);

impl WKBGeometryCode {
    pub(crate) fn new(code: u32) -> Self {
        Self(code)
    }

    pub(crate) fn has_srid(&self) -> bool {
        self.0 & EWKB_FLAG_SRID == EWKB_FLAG_SRID
    }

    pub(crate) fn get_type(&self) -> WKBResult<WKBType> {
        let code = self.0;
        let mut dim = WKBDimension::Xy;

        // For ISO WKB:
        // Values 1, 2, 3 are 2D,
        // 1001, 1002, 1003 are XYZ,
        // 2001 etc are XYM,
        // 3001 etc are XYZM
        match code / 1000 {
            1 => dim = WKBDimension::Xyz,
            2 => dim = WKBDimension::Xym,
            3 => dim = WKBDimension::Xyzm,
            _ => (),
        };

        // For extended WKB, higher dimensions are provided via bit flags
        let is_ewkb_z = code & EWKB_FLAG_Z == EWKB_FLAG_Z;
        let is_ewkb_m = code & EWKB_FLAG_M == EWKB_FLAG_M;

        match (is_ewkb_z, is_ewkb_m) {
            (true, true) => dim = WKBDimension::Xyzm,
            (true, false) => dim = WKBDimension::Xyz,
            (false, true) => dim = WKBDimension::Xym,
            _ => (),
        }

        let typ = match code & 0x7 {
            1 => WKBType::Point(dim),
            2 => WKBType::LineString(dim),
            3 => WKBType::Polygon(dim),
            4 => WKBType::MultiPoint(dim),
            5 => WKBType::MultiLineString(dim),
            6 => WKBType::MultiPolygon(dim),
            7 => WKBType::GeometryCollection(dim),
            _ => {
                return Err(WKBError::General(format!(
                    "WKB type code out of range. Got: {}",
                    code
                )))
            }
        };
        Ok(typ)
    }
}

/// The various WKB types supported by this crate
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WKBType {
    /// A WKB Point
    Point(WKBDimension),
    /// A WKB LineString
    LineString(WKBDimension),
    /// A WKB Polygon
    Polygon(WKBDimension),
    /// A WKB MultiPoint
    MultiPoint(WKBDimension),
    /// A WKB MultiLineString
    MultiLineString(WKBDimension),
    /// A WKB MultiPolygon
    MultiPolygon(WKBDimension),
    /// A WKB GeometryCollection
    GeometryCollection(WKBDimension),
}

impl WKBType {
    /// Construct from a byte slice representing a WKB geometry
    pub(crate) fn from_buffer(buf: &[u8]) -> WKBResult<Self> {
        let mut reader = Cursor::new(buf);
        let byte_order = reader.read_u8().unwrap();
        let geometry_code = match byte_order {
            0 => reader.read_u32::<BigEndian>().unwrap(),
            1 => reader.read_u32::<LittleEndian>().unwrap(),
            other => {
                return Err(WKBError::General(format!(
                    "Unexpected byte order: {}",
                    other
                )))
            }
        };
        WKBGeometryCode(geometry_code).get_type()
    }

    pub(crate) fn as_geometry_code(&self) -> WKBGeometryCode {
        let code = match self {
            Self::Point(dim) => 1 + dim.as_u32_offset(),
            Self::LineString(dim) => 2 + dim.as_u32_offset(),
            Self::Polygon(dim) => 3 + dim.as_u32_offset(),
            Self::MultiPoint(dim) => 4 + dim.as_u32_offset(),
            Self::MultiLineString(dim) => 5 + dim.as_u32_offset(),
            Self::MultiPolygon(dim) => 6 + dim.as_u32_offset(),
            Self::GeometryCollection(dim) => 7 + dim.as_u32_offset(),
        };
        WKBGeometryCode(code)
    }
}

impl From<WKBType> for u32 {
    fn from(value: WKBType) -> Self {
        value.as_geometry_code().0
    }
}

/// Endianness
#[derive(Debug, Clone, Copy, Default, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Endianness {
    BigEndian = 0,
    #[default]
    LittleEndian = 1,
}
