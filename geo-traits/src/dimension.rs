/// The logical dimension of the geometry.
///
///
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    /// A two-dimensional geometry with X and Y values
    XY,

    /// A three-dimensional geometry with X, Y, and Z values
    XYZ,

    /// A three-dimensional geometry with X, Y, and M values
    XYM,

    /// A four-dimensional geometry with X, Y, Z, and M values
    XYZM,

    /// A geometry with unknown logical type. The contained `usize` value represents the number of
    /// physical dimensions.
    Unknown(usize),
}

impl Dimension {
    /// The physical number of dimensions in this geometry.
    pub fn size(&self) -> usize {
        match self {
            Self::XY => 2,
            Self::XYZ | Self::XYM => 3,
            Self::XYZM => 4,
            Self::Unknown(val) => *val,
        }
    }
}
