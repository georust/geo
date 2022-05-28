use geo_types::MultiPolygon;

use crate::{CoordsIter, GeoFloat, GeoNum, Polygon};

pub trait BooleanOps: Sized {
    type Scalar: GeoNum;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon<Self::Scalar>;
    fn intersection(&self, other: &Self) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Intersection)
    }
    fn union(&self, other: &Self) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Union)
    }
    fn xor(&self, other: &Self) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Xor)
    }
    fn difference(&self, other: &Self) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Difference)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpType {
    Intersection,
    Union,
    Difference,
    Xor,
}

impl<T: GeoFloat> BooleanOps for Polygon<T> {
    type Scalar = T;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon<Self::Scalar> {
        let mut bop = Op::new(op, self.coords_count() + other.coords_count());
        bop.add_polygon(self, true);
        bop.add_polygon(other, false);
        let rings = bop.sweep();
        assemble(rings).into()
    }
}
impl<T: GeoFloat> BooleanOps for MultiPolygon<T> {
    type Scalar = T;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon<Self::Scalar> {
        let mut bop = Op::new(op, self.coords_count() + other.coords_count());
        bop.add_multi_polygon(self, true);
        bop.add_multi_polygon(other, false);
        let rings = bop.sweep();
        assemble(rings).into()
    }
}

mod op;
use op::*;

mod rings;
use rings::{Ring, Rings};

mod laminar;
use laminar::*;

#[cfg(test)]
mod tests;
