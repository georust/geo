pub(crate) mod i_overlay_integration;
#[cfg(test)]
mod tests;

use i_overlay_integration::convert::{multi_polygon_from_shapes, ring_to_shape_path};
use i_overlay_integration::BoolOpsCoord;
pub use i_overlay_integration::BoolOpsNum;

use crate::geometry::{LineString, MultiLineString, MultiPolygon, Polygon};
use crate::winding_order::{Winding, WindingOrder};

pub use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::clip::FloatClip;
use i_overlay::float::overlay::FloatOverlay;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::string::clip::ClipRule;

/// Boolean Operations on geometry.
///
/// Boolean operations are set operations on geometries considered as a subset
/// of the 2-D plane. The operations supported are: intersection, union,
/// symmetric difference (xor), and set-difference on pairs of 2-D geometries and
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
///
/// # Performance
///
/// For union operations on a large number of [`Polygon`]s or [`MultiPolygon`]s,
/// using [`unary_union`] will yield far better performance.
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

    /// Performs a boolean operation between shapes using the default [`FillRule::EvenOdd`] fill rule.
    ///
    /// The `EvenOdd` rule determines filled regions based on the parity of path crossings.
    /// It correctly handles "holes" in polygons by accounting for path direction and overlapping regions.
    ///
    /// This behavior models that of a real-world laser cutter: each shape is treated as a continuous
    /// cut from solid material. When a new shape overlaps an existing cut, the overlapping area
    /// is removed, potentially splitting the geometry into multiple disjoint parts.
    ///
    /// This fill rule is intuitive for applications like CAD tooling, CNC routing,
    /// and other manufacturing processes where overlapping shapes subtract from the material
    /// rather than accumulate.
    ///
    /// To use a different fill rule, such as [`FillRule::NonZero`], use [`Self::boolean_op_with_fill_rule`] instead.
    fn boolean_op(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
        op: OpType,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op_with_fill_rule(other, op, FillRule::EvenOdd)
    }

    /// Performs a boolean operation with the specified fill rule.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::bool_ops::{BooleanOps, OpType, FillRule};
    /// use geo::wkt;
    ///
    /// let polygon1 = wkt!(POLYGON((0.0 0.0, 10.0 0.0, 10.0 10.0, 0.0 10.0, 0.0 0.0)));
    /// let polygon2 = wkt!(POLYGON((5.0 5.0, 15.0 5.0, 15.0 15.0, 5.0 15.0, 5.0 5.0)));
    /// let result = polygon1.boolean_op_with_fill_rule(&polygon2, OpType::Union, FillRule::NonZero);
    /// assert_eq!(result.0.len(), 1);
    /// ```
    fn boolean_op_with_fill_rule(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
        op: OpType,
        fill_rule: FillRule,
    ) -> MultiPolygon<Self::Scalar> {
        let subject = self.rings().map(ring_to_shape_path).collect::<Vec<_>>();
        let clip = other.rings().map(ring_to_shape_path).collect::<Vec<_>>();
        let shapes = subject.overlay(&clip, op.into(), fill_rule);
        multi_polygon_from_shapes(shapes)
    }

    /// Returns the overlapping regions shared by both `self` and `other`.
    fn intersection(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Intersection)
    }

    /// Returns the overlapping regions shared by both `self` and `other`, using the specified fill rule.
    fn intersection_with_fill_rule(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
        fill_rule: FillRule,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op_with_fill_rule(other, OpType::Intersection, fill_rule)
    }

    /// Combines the regions of both `self` and `other` into a single geometry, removing
    /// overlaps and merging boundaries.
    fn union(&self, other: &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Union)
    }

    /// Combines the regions of both `self` and `other` into a single geometry, removing
    /// overlaps and merging boundaries, using the specified fill rule.
    fn union_with_fill_rule(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
        fill_rule: FillRule,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op_with_fill_rule(other, OpType::Union, fill_rule)
    }

    /// The regions that are in either `self` or `other`, but not in both.
    fn xor(&self, other: &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Xor)
    }

    /// The regions that are in either `self` or `other`, but not in both.
    fn xor_with_fill_rule(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
        fill_rule: FillRule,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op_with_fill_rule(other, OpType::Xor, fill_rule)
    }

    /// The regions of `self` which are not in `other`.
    fn difference(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Difference)
    }

    /// The regions of `self` which are not in `other`, using the specified fill rule.
    fn difference_with_fill_rule(
        &self,
        other: &impl BooleanOps<Scalar = Self::Scalar>,
        fill_rule: FillRule,
    ) -> MultiPolygon<Self::Scalar> {
        self.boolean_op_with_fill_rule(other, OpType::Difference, fill_rule)
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
        self.clip_with_fill_rule(multi_line_string, invert, FillRule::EvenOdd)
    }

    /// Clip a 1-D geometry with self.
    ///
    /// Returns the portion of `ls` that lies within `self` (known as the set-theoeretic
    /// intersection) if `invert` is false, and the difference (`ls - self`) otherwise.
    fn clip_with_fill_rule(
        &self,
        multi_line_string: &MultiLineString<Self::Scalar>,
        invert: bool,
        fill_rule: FillRule,
    ) -> MultiLineString<Self::Scalar> {
        let subject: Vec<Vec<_>> = multi_line_string
            .iter()
            .map(|line_string| line_string.coords().map(|c| BoolOpsCoord(*c)).collect())
            .collect();

        let clip = self.rings().map(ring_to_shape_path).collect::<Vec<_>>();

        let clip_rule = ClipRule {
            invert,
            boundary_included: true,
        };
        let paths = subject.clip_by(&clip, fill_rule, clip_rule);
        i_overlay_integration::convert::multi_line_string_from_paths(paths)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpType {
    Intersection,
    Union,
    Difference,
    Xor,
}

/// Efficient [union](BooleanOps::union) of many adjacent / overlapping geometries
///
/// This is typically much faster than `union`ing a bunch of geometries together one at a time.
///
/// Note: Geometries can be wound in either direction, but the winding order must be consistent,
/// and each polygon's interior rings must be wound opposite to its exterior.
///
/// See [Orient] for more information.
///
/// [Orient]: crate::algorithm::orient::Orient
///
/// # Arguments
///
/// `boppables`: A collection of `Polygon` or `MultiPolygons` to union together.
///
/// returns the union of all the inputs.
///
/// # Examples
///
/// ```
/// use geo::algorithm::unary_union;
/// use geo::wkt;
///
/// let right_piece = wkt!(POLYGON((4. 0.,4. 4.,8. 4.,8. 0.,4. 0.)));
/// let left_piece = wkt!(POLYGON((0. 0.,0. 4.,4. 4.,4. 0.,0. 0.)));
///
/// // touches neither right nor left piece
/// let separate_piece = wkt!(POLYGON((14. 10.,14. 14.,18. 14.,18. 10.,14. 10.)));
///
/// let polygons = vec![left_piece, separate_piece, right_piece];
/// let actual_output = unary_union(&polygons);
///
/// let expected_output = wkt!(MULTIPOLYGON(
///     // left and right piece have been combined
///     ((0.0 0.0, 8.0 0.0, 8.0 4.0, 0.0 4.0, 0.0 0.0)),
///     // separate piece remains separate
///     ((14.0 10.0, 18.0 10.0, 18.0 14.0, 14.0 14.0, 14.0 10.0))
/// ));
/// assert_eq!(actual_output, expected_output);
/// ```
pub fn unary_union<'a, B: BooleanOps + 'a>(
    boppables: impl IntoIterator<Item = &'a B>,
) -> MultiPolygon<B::Scalar> {
    let mut winding_order: Option<WindingOrder> = None;
    let subject = boppables
        .into_iter()
        .flat_map(|boppable| {
            let rings = boppable.rings();
            rings
                .map(|ring| {
                    if winding_order.is_none() {
                        winding_order = ring.winding_order();
                    }
                    ring_to_shape_path(ring)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let fill_rule = if winding_order == Some(WindingOrder::Clockwise) {
        FillRule::Positive
    } else {
        FillRule::Negative
    };

    let shapes = FloatOverlay::with_subj(&subject).overlay(OverlayRule::Subject, fill_rule);
    multi_polygon_from_shapes(shapes)
}

impl<T: BoolOpsNum> BooleanOps for Polygon<T> {
    type Scalar = T;

    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>> {
        std::iter::once(self.exterior()).chain(self.interiors())
    }
}

impl<T: BoolOpsNum> BooleanOps for MultiPolygon<T> {
    type Scalar = T;

    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>> {
        self.iter().flat_map(BooleanOps::rings)
    }
}
