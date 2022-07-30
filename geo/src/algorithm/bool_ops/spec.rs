use std::fmt::Debug;

use geo_types::MultiPolygon;

use crate::{sweep::LineOrPoint, GeoFloat, OpType};

use super::assembly::Assembly;

pub trait Spec<T: GeoFloat> {
    type Region: Copy + Debug;
    type Output;

    fn infinity(&self) -> Self::Region;
    fn cross(&self, prev_region: Self::Region, is_first: bool) -> Self::Region;
    fn output(&mut self, regions: [Self::Region; 2], geom: LineOrPoint<T>);
    fn finish(self) -> Self::Output;
}

pub struct BoolOp<T: GeoFloat> {
    ty: OpType,
    assembly: Assembly<T>,
}

impl<T: GeoFloat> From<OpType> for  BoolOp<T> {
    fn from(ty: OpType) -> Self {
        Self {
            ty,
            assembly: Assembly::default(),
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

    fn cross(&self, mut prev_region: Self::Region, is_first: bool) -> Self::Region {
        prev_region.cross(is_first);
        prev_region
    }

    fn output(&mut self, regions: [Self::Region; 2], geom: LineOrPoint<T>) {
        if regions[0].is_ty(self.ty) ^ regions[1].is_ty(self.ty) {
            self.assembly.add_edge(geom)
        }
    }

    fn finish(self) -> Self::Output {
        self.assembly.finish()
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
