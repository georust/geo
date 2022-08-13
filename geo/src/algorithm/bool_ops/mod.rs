use geo_types::{MultiLineString, MultiPolygon};

use crate::{CoordsIter, GeoFloat, GeoNum, Polygon};

/// Boolean Operations on geometry.
///
/// Boolean operations are set operations on geometries considered as a subset
/// of the 2-D plane. The operations supported are: intersection, union, xor or
/// symmetric difference, and set-difference on pairs of 2-D geometries and
/// clipping a 1-D geometry with self.
///
/// These operations are implemented on [`Polygon`] and the [`MultiPolygon`]
/// geometries.
///
/// # Validity
///
/// Note that the operations are strictly well-defined only on *valid*
/// geometries. However, the implementation generally works well as long as the
/// interiors of polygons are contained within their corresponding exteriors.
///
/// Degenerate 2-d geoms with 0 area are handled, and ignored by the algorithm.
/// In particular, taking `union` with an empty geom should remove degeneracies
/// and fix invalid polygons as long the interior-exterior requirement above is
/// satisfied.
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

    /// Clip a 1-D geometry with self.
    ///
    /// Returns the set-theoeretic intersection of `self` and `ls` if `invert`
    /// is false, and the difference (`ls - self`) otherwise.
    fn clip(
        &self,
        ls: &MultiLineString<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString<Self::Scalar>;
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
        let spec = BoolOp::from(op);
        let mut bop = Proc::new(spec, self.coords_count() + other.coords_count());
        bop.add_polygon(self, 0);
        bop.add_polygon(other, 1);
        bop.sweep()
    }

    fn clip(
        &self,
        ls: &MultiLineString<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString<Self::Scalar> {
        let spec = ClipOp::new(invert);
        let mut bop = Proc::new(spec, self.coords_count() + ls.coords_count());
        bop.add_polygon(self, 0);
        ls.0.iter().enumerate().for_each(|(idx, l)| {
            bop.add_line_string(l, idx + 1);
        });
        bop.sweep()
    }
}
impl<T: GeoFloat> BooleanOps for MultiPolygon<T> {
    type Scalar = T;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon<Self::Scalar> {
        let spec = BoolOp::from(op);
        let mut bop = Proc::new(spec, self.coords_count() + other.coords_count());
        bop.add_multi_polygon(self, 0);
        bop.add_multi_polygon(other, 1);
        bop.sweep()
    }

    fn clip(
        &self,
        ls: &MultiLineString<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString<Self::Scalar> {
        let spec = ClipOp::new(invert);
        let mut bop = Proc::new(spec, self.coords_count() + ls.coords_count());
        bop.add_multi_polygon(self, 0);
        ls.0.iter().enumerate().for_each(|(idx, l)| {
            bop.add_line_string(l, idx + 1);
        });
        bop.sweep()
    }
}

mod op;
use op::*;
mod assembly;
use assembly::*;
mod spec;
use spec::*;

#[cfg(test)]
mod tests;
