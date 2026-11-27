//! Codes which combine a geometry dimension and a location on the
//! geometry.
//!
//! Port of JTS `DimensionLocation` (an enum instead of int codes).

use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DimensionLocation {
    Exterior,
    PointInterior,
    LineInterior,
    LineBoundary,
    AreaInterior,
    AreaBoundary,
}

impl DimensionLocation {
    pub fn from_location_point(loc: CoordPos) -> Self {
        match loc {
            CoordPos::Inside => Self::PointInterior,
            _ => Self::Exterior,
        }
    }

    pub fn from_location_line(loc: CoordPos) -> Self {
        match loc {
            CoordPos::Inside => Self::LineInterior,
            CoordPos::OnBoundary => Self::LineBoundary,
            CoordPos::Outside => Self::Exterior,
        }
    }

    pub fn from_location_area(loc: CoordPos) -> Self {
        match loc {
            CoordPos::Inside => Self::AreaInterior,
            CoordPos::OnBoundary => Self::AreaBoundary,
            CoordPos::Outside => Self::Exterior,
        }
    }

    pub fn location(self) -> CoordPos {
        match self {
            Self::PointInterior | Self::LineInterior | Self::AreaInterior => CoordPos::Inside,
            Self::LineBoundary | Self::AreaBoundary => CoordPos::OnBoundary,
            Self::Exterior => CoordPos::Outside,
        }
    }

    pub fn dimension(self) -> Dimensions {
        match self {
            Self::PointInterior => Dimensions::ZeroDimensional,
            Self::LineInterior | Self::LineBoundary => Dimensions::OneDimensional,
            Self::AreaInterior | Self::AreaBoundary => Dimensions::TwoDimensional,
            Self::Exterior => Dimensions::Empty,
        }
    }

    /// The dimension of the intersection, with an exterior location taking
    /// the given exterior dimension.
    pub fn dimension_with_exterior(self, exterior_dim: Dimensions) -> Dimensions {
        match self {
            Self::Exterior => exterior_dim,
            _ => self.dimension(),
        }
    }
}
