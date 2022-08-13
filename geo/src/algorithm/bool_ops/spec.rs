use geo_types::{MultiLineString, MultiPolygon};
use std::fmt::Debug;

use super::*;
use crate::{sweep::LineOrPoint, GeoFloat, OpType};

pub trait Spec<T: GeoFloat> {
    type Region: Copy + Debug;
    type Output;

    fn infinity(&self) -> Self::Region;
    fn cross(&self, prev_region: Self::Region, idx: usize) -> Self::Region;
    fn output(&mut self, regions: [Self::Region; 2], geom: LineOrPoint<T>, idx: usize);
    fn finish(self) -> Self::Output;
}

pub struct BoolOp<T: GeoFloat> {
    ty: OpType,
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
            is_second: matches!(self.ty, OpType::Difference),
        }
    }

    fn cross(&self, mut prev_region: Self::Region, idx: usize) -> Self::Region {
        prev_region.cross(idx == 0);
        prev_region
    }

    fn output(&mut self, regions: [Self::Region; 2], geom: LineOrPoint<T>, _idx: usize) {
        if regions[0].is_ty(self.ty) ^ regions[1].is_ty(self.ty) {
            self.assembly.add_edge(geom)
        }
    }

    fn finish(self) -> Self::Output {
        self.assembly.finish()
    }
}

pub struct ClipOp<T: GeoFloat> {
    invert: bool,
    assembly: LineAssembly<T>,
}

impl<T: GeoFloat> ClipOp<T> {
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

#[derive(Clone, Copy)]
pub struct Region {
    is_first: bool,
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
    fn cross(&mut self, first: bool) {
        if first {
            self.is_first = !self.is_first;
        } else {
            self.is_second = !self.is_second;
        }
    }
    fn is_ty(&self, ty: OpType) -> bool {
        match ty {
            OpType::Intersection | OpType::Difference => self.is_first && self.is_second,
            OpType::Union => self.is_first || self.is_second,
            OpType::Xor => self.is_first ^ self.is_second,
        }
    }
}
