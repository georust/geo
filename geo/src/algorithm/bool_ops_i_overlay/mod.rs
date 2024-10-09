use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::f64::overlay::F64Overlay;
use i_overlay::i_float::f64_point::F64Point;
use i_overlay::i_shape::f64::shape::{F64Path, F64Shape, F64Shapes};

use geo_types::{Coord, LineString, MultiLineString, MultiPolygon};

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

trait Paths {
    type PathType;
    fn paths(&self) -> impl Iterator<Item = Self::PathType>;
}

impl Paths for Polygon<f64> {
    type PathType = F64Path;

    fn paths(&self) -> impl Iterator<Item = Self::PathType> {
        std::iter::once(self.exterior())
            .chain(self.interiors().into_iter())
            .map(|r| {
                r.into_iter()
                    .map(|c| F64Point::new(c.x, c.y))
                    .collect::<Vec<_>>()
            })
    }
}

impl Paths for MultiPolygon<f64> {
    type PathType = F64Path;

    fn paths(&self) -> impl Iterator<Item = Self::PathType> {
        self.0.iter().map(|p| p.paths()).flatten()
    }
}

fn line_string_from_path(path: F64Path) -> LineString<f64> {
    let coords = path
        .into_iter()
        .map(|p| Coord { x: p.x, y: p.y })
        .collect::<Vec<_>>();

    LineString(coords)
}

fn polygon_from_shape(shape: F64Shape) -> Polygon<f64> {
    let rings: Vec<_> = shape
        .into_iter()
        .map(|p| line_string_from_path(p))
        .collect();

    // TODO: avoid OOB panic, avoid clone
    let exterior = rings[0].clone();
    let interiors = rings[1..].to_vec();
    Polygon::new(exterior, interiors)
}

fn multi_polygon_from_shapes(shapes: F64Shapes) -> MultiPolygon<f64> {
    MultiPolygon(shapes.into_iter().map(|s| polygon_from_shape(s)).collect())
}

// TODO: make generic - make part of GeoNum conformance to specify the various F64Overlay, F64Point etc.
impl BooleanOps for Polygon<f64> {
    type Scalar = f64;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon<Self::Scalar> {
        // let spec = BoolOp::from(op);
        // let mut bop = Proc::new(spec, self.coords_count() + other.coords_count());
        // bop.add_multi_polygon(self, 0);
        // bop.add_multi_polygon(other, 1);
        // bop.sweep()

        // get overlay from GeoNum
        let mut overlay = F64Overlay::new();

        for path in self.paths() {
            overlay.add_path(path, ShapeType::Subject);
        }
        for path in other.paths() {
            overlay.add_path(path, ShapeType::Clip);
        }

        let overlay_rule = match op {
            OpType::Intersection => OverlayRule::Intersect,
            OpType::Union => OverlayRule::Union,
            OpType::Difference => OverlayRule::Difference,
            OpType::Xor => OverlayRule::Xor,
        };

        // REVIEW: fill rule?
        let graph = overlay.into_graph(FillRule::NonZero);
        let shapes = graph.extract_shapes(overlay_rule);

        multi_polygon_from_shapes(shapes)
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
impl BooleanOps for MultiPolygon<f64> {
    type Scalar = f64;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon<Self::Scalar> {
        let mut overlay = F64Overlay::new();

        for path in self.paths() {
            overlay.add_path(path, ShapeType::Subject);
        }
        for path in other.paths() {
            overlay.add_path(path, ShapeType::Clip);
        }

        let overlay_rule = match op {
            OpType::Intersection => OverlayRule::Intersect,
            OpType::Union => OverlayRule::Union,
            OpType::Difference => OverlayRule::Difference,
            OpType::Xor => OverlayRule::Xor,
        };

        // REVIEW: fill rule?
        let graph = overlay.into_graph(FillRule::NonZero);
        let shapes = graph.extract_shapes(overlay_rule);

        multi_polygon_from_shapes(shapes)
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
