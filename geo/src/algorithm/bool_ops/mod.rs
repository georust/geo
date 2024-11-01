mod i_overlay_integration;
#[cfg(test)]
mod tests;

pub use i_overlay_integration::BoolOpsNum;

use crate::geometry::{LineString, MultiLineString, MultiPolygon, Polygon};

/// Boolean Operations on geometry.
///
/// Requires the `"i_overlay"` feature, which is enabled by default.
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
pub trait BooleanOps {
    type Scalar: BoolOpsNum;

    /// The exterior and interior rings of the geometry.
    ///
    /// It doesn't particularly matter which order they are in, as the topology algorithm counts crossings
    /// to determine the interior and exterior of the polygon.
    ///
    /// It is required that the rings are from valid geometries, that the rings not overlap.
    /// In the case of a MultiPolygon, this requires that none of its polygon's interiors may overlap.
    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>>;

    fn boolean_op(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
        op: OpType,
    ) -> MultiPolygon<Self::Scalar> {
        use i_overlay::core::fill_rule::FillRule;
        use i_overlay::core::overlay::ShapeType;
        use i_overlay_integration::{convert, BoolOpsOverlay, BoolOpsOverlayGraph};
        let mut overlay = <Self::Scalar as BoolOpsNum>::OverlayType::new();

        for ring in self.rings() {
            overlay.add_path(convert::ring_to_shape_path(ring), ShapeType::Subject);
        }
        for ring in other.rings() {
            overlay.add_path(convert::ring_to_shape_path(ring), ShapeType::Clip);
        }

        let graph = overlay.into_graph(FillRule::EvenOdd);
        let shapes = graph.extract_shapes(op.into());

        convert::multi_polygon_from_shapes(shapes)
    }

    fn intersection(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Intersection)
    }
    fn union(&self, other: &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Union)
    }
    fn xor(&self, other: &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Xor)
    }
    fn difference(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Difference)
    }

    /// Clip a 1-D geometry with self.
    ///
    /// Returns the portion of `ls` that lies within `self` (known as the set-theoeretic
    /// intersection) if `invert` is false, and the difference (`ls - self`) otherwise.
    fn clip(
        &self,
        multi_line_string: &MultiLineString<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString<Self::Scalar> {
        use i_overlay::core::fill_rule::FillRule;
        use i_overlay::string::clip::ClipRule;
        use i_overlay_integration::{convert, BoolOpsStringGraph, BoolOpsStringOverlay};

        let mut overlay = <Self::Scalar as BoolOpsNum>::StringOverlayType::new();

        for ring in self.rings() {
            overlay.add_shape_path(convert::ring_to_shape_path(ring));
        }
        for line_string in multi_line_string {
            for line in line_string.lines() {
                let line = [
                    Self::Scalar::to_bops_coord(line.start),
                    Self::Scalar::to_bops_coord(line.end),
                ];
                overlay.add_string_line(line)
            }
        }

        let graph = overlay.into_graph(FillRule::EvenOdd);
        let paths = graph.clip_string_lines(ClipRule {
            invert,
            boundary_included: true,
        });
        convert::multi_line_string_from_paths(paths)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpType {
    Intersection,
    Union,
    Difference,
    Xor,
}

impl<T: BoolOpsNum> BooleanOps for Polygon<T> {
    type Scalar = T;

    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>> {
        std::iter::once(self.exterior()).chain(self.interiors().iter())
    }
}

impl<T: BoolOpsNum> BooleanOps for MultiPolygon<T> {
    type Scalar = T;

    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>> {
        self.0.iter().flat_map(|p| p.rings())
    }
}
