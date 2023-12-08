use geo_types::{MultiLineString, MultiPolygon};
use std::fmt::Debug;

use super::*;
use crate::{sweep::LineOrPoint, GeoFloat, OpType};

/// A trait to compute the final shape of a collection of non-intersecting edges
/// by tracking the regions formed by those edges.
pub trait Spec<T: GeoFloat> {
    // Region is an associated type rather than the concrete `Region` type to
    // support possible future implementations (e.g., union of a Vec of polygons).
    type Region: Copy + Debug;
    type Output;

    /// Creates the region describing the "infinity point" (in other words, the
    /// "exterior" of both shapes).
    fn infinity(&self) -> Self::Region;
    /// Returns the new region of `prev_region` after crossing an edge from
    /// shape `idx`.
    fn cross(&self, prev_region: Self::Region, idx: usize) -> Self::Region;
    /// Records `geom` in the `output` if required. `regions` are the regions
    /// before and after crossing `geom`. `idx` is the index of the shape that
    /// `geom` belongs to.
    fn output(&mut self, regions: [Self::Region; 2], geom: LineOrPoint<T>, idx: usize);
    /// Finishes the assembly, producing the output value.
    fn finish(self) -> Self::Output;
}

/// State for a boolean operation on two shapes.
pub struct BoolOp<T: GeoFloat> {
    /// The operation to compute.
    ty: OpType,
    /// The assembly of the output shape.
    assembly: RegionAssembly<T>,
}
impl<T: GeoFloat> From<OpType> for BoolOp<T> {
    fn from(ty: OpType) -> Self {
        Self {
            ty,
            assembly: RegionAssembly::default(),
        }
    }
}
impl<T: GeoFloat> Spec<T> for BoolOp<T> {
    type Region = Region;
    type Output = MultiPolygon<T>;

    fn infinity(&self) -> Self::Region {
        Region {
            is_first: false,
            // A difference is just an intersection with the inverse of the second shape. So pretend
            // that the exterior of the second shape includes the infinity point.
            is_second: matches!(self.ty, OpType::Difference),
        }
    }

    fn cross(&self, mut prev_region: Self::Region, idx: usize) -> Self::Region {
        prev_region.cross(idx == 0);
        prev_region
    }

    fn output(&mut self, regions: [Self::Region; 2], geom: LineOrPoint<T>, _idx: usize) {
        // Only add `geom` to the output if the regions don't agree on whether they are
        // in the output.
        if regions[0].is_ty(self.ty) ^ regions[1].is_ty(self.ty) {
            self.assembly.add_edge(geom)
        }
    }

    fn finish(self) -> Self::Output {
        self.assembly.finish()
    }
}

/// State for clipping lines by some shape.
pub struct ClipOp<T: GeoFloat> {
    /// Whether the clipping shape should be inverted (lines outside the
    /// clipping shape should be in the output instead).
    invert: bool,
    /// The assembly of the output lines.
    assembly: LineAssembly<T>,
}

impl<T: GeoFloat> ClipOp<T> {
    /// Creates a ClipOp. If `invert` is true, lines outside the clipping shape
    /// will be in the output rather than lines inside.
    pub fn new(invert: bool) -> Self {
        Self {
            invert,
            assembly: Default::default(),
        }
    }
}

impl<T: GeoFloat> Spec<T> for ClipOp<T> {
    type Region = Region;
    type Output = MultiLineString<T>;

    fn infinity(&self) -> Self::Region {
        Region {
            is_first: false,
            is_second: false,
        }
    }

    fn cross(&self, mut prev_region: Self::Region, idx: usize) -> Self::Region {
        // Only the clipping shape can change the region in/out. All other shapes are
        // lines.
        if idx == 0 {
            prev_region.cross(true);
        }
        prev_region
    }

    fn output(&mut self, regions: [Self::Region; 2], geom: LineOrPoint<T>, idx: usize) {
        if idx > 0 && (regions[0].is_first && regions[1].is_first) != self.invert {
            self.assembly.add_edge(geom, idx);
        }
    }

    fn finish(self) -> Self::Output {
        MultiLineString::new(self.assembly.finish())
    }
}

/// Flags of a region when clipping.
#[derive(Clone, Copy)]
pub struct Region {
    /// Whether this region is inside the first shape.
    is_first: bool,
    /// Whether this region is inside the second shape.
    is_second: bool,
}
impl std::fmt::Debug for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{f}{s}]",
            f = if self.is_first { "A" } else { "" },
            s = if self.is_second { "B" } else { "" },
        )
    }
}

impl Region {
    /// Determines the flags after crossing an edge. `first` describes whether
    /// the edge was from the first shape, or the second shape.
    fn cross(&mut self, first: bool) {
        if first {
            self.is_first = !self.is_first;
        } else {
            self.is_second = !self.is_second;
        }
    }
    /// Determines whether this region is in the resulting shape when applying
    /// `ty`.
    fn is_ty(&self, ty: OpType) -> bool {
        match ty {
            OpType::Intersection | OpType::Difference => self.is_first && self.is_second,
            OpType::Union => self.is_first || self.is_second,
            OpType::Xor => self.is_first ^ self.is_second,
        }
    }
}
