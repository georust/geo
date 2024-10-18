/// The logical dimension of the geometry.
///
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimensions {
    /// A two-dimensional geometry with X and Y values
    Xy,

    /// A three-dimensional geometry with X, Y, and Z values
    Xyz,

    /// A three-dimensional geometry with X, Y, and M values
    Xym,

    /// A four-dimensional geometry with X, Y, Z, and M values
    Xyzm,

    /// A geometry with unknown logical type. The contained `usize` value represents the number of
    /// physical dimensions.
    Unknown(usize),
}

impl Dimensions {
    /// The physical number of dimensions in this geometry.
    pub fn size(&self) -> usize {
        match self {
            Self::Xy => 2,
            Self::Xyz | Self::Xym => 3,
            Self::Xyzm => 4,
            Self::Unknown(val) => *val,
        }
    }
}
