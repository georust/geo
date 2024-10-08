use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::f64::overlay::F64Overlay;
use i_overlay::i_float::f64_point::F64Point;

use geo_types::{MultiLineString, MultiPolygon};

use crate::{CoordsIter, GeoFloat, GeoNum, Polygon};

// use i_overlay

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
    /// Returns the portion of `ls` that lies within `self` (known as the set-theoeretic
    /// intersection) if `invert` is false, and the difference (`ls - self`) otherwise.
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

// TODO: make generic - make part of GeoNum conformance to specify the various F64Overlay, F64Point etc.
impl BooleanOps for Polygon<f64> {
    type Scalar = f64;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon<Self::Scalar> {
        // let spec = BoolOp::from(op);
        // let mut bop = Proc::new(spec, self.coords_count() + other.coords_count());
        // bop.add_polygon(self, 0);
        // bop.add_polygon(other, 1);
        // bop.sweep()
        todo!()
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
        // let spec = BoolOp::from(op);
        // let mut bop = Proc::new(spec, self.coords_count() + other.coords_count());
        // bop.add_multi_polygon(self, 0);
        // bop.add_multi_polygon(other, 1);
        // bop.sweep()
        // TODO get subj from self
        let subj = [
            // Define the subject polygon (a square)
            F64Point::new(-10.0, -10.0),
            F64Point::new(-10.0, 10.0),
            F64Point::new(10.0, 10.0),
            F64Point::new(10.0, -10.0),
        ]
        .to_vec();

        // TODO get clip from parm
        let clip = [
            // Define the clip polygon (a slightly shifted square)
            F64Point::new(-5.0, -5.0),
            F64Point::new(-5.0, 15.0),
            F64Point::new(15.0, 15.0),
            F64Point::new(15.0, -5.0),
        ]
        .to_vec();

        // get overlay from GeoNum
        let mut overlay = F64Overlay::new();
        overlay.add_path(subj, ShapeType::Subject);
        overlay.add_path(clip, ShapeType::Clip);

        // REVIEW: fill rule?
        let graph = overlay.into_graph(FillRule::NonZero);
        // TODO get op from param
        let shapes = graph.extract_shapes(OverlayRule::Union);
        // TODO get MultiPolygon from shapes
        MultiPolygon(vec![])
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
