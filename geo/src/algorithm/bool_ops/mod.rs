mod i_overlay_integration;
#[cfg(test)]
mod tests;

use crate::bool_ops::i_overlay_integration::convert::{
    multi_polygon_from_shapes, ring_to_shape_path,
};
use crate::bool_ops::i_overlay_integration::BoolOpsCoord;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::float::clip::FloatClip;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::string::clip::ClipRule;
#[cfg(feature = "multithreading")]
use rayon::prelude::*;

pub use i_overlay_integration::BoolOpsNum;

use crate::geometry::{LineString, MultiLineString, MultiPolygon, Polygon};
use rstar::{
    primitives::CachedEnvelope, primitives::ObjectRef, ParentNode, RTree, RTreeNode, RTreeObject,
};

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
///
/// # Performance
///
/// For union operations on a collection of overlapping and / or adjacent [`Polygon`]s
/// (e.g. contained in a `Vec` or a [`MultiPolygon`]), using [`UnaryUnion`] will
/// yield far better performance.
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
        let subject = self.rings().map(ring_to_shape_path).collect::<Vec<_>>();
        let clip = other.rings().map(ring_to_shape_path).collect::<Vec<_>>();
        let shapes = subject.overlay(&clip, op.into(), FillRule::EvenOdd);
        multi_polygon_from_shapes(shapes)
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
        let subject: Vec<Vec<_>> = multi_line_string
            .iter()
            .map(|line_string| line_string.coords().map(|c| BoolOpsCoord(*c)).collect())
            .collect();

        let clip = self.rings().map(ring_to_shape_path).collect::<Vec<_>>();

        let clip_rule = ClipRule {
            invert,
            boundary_included: true,
        };
        let paths = subject.clip_by(&clip, FillRule::EvenOdd, clip_rule);
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

// Recursive algorithms can benefit from grouping those parameters which are constant over
// the whole algorithm to reduce the overhead of the recursive calls, in this case the single-
// and multi-threaded unary union tree traversals
#[derive(Debug)]
struct Ops<I, F, R> {
    init: I,
    fold: F,
    reduce: R,
}

/// Efficient [BooleanOps::union] of adjacent / overlapping geometries
///
/// For geometries with a high degree of overlap or adjacency
/// (for instance, merging a large contiguous area made up of many adjacent polygons)
/// this method will be orders of magnitude faster than a manual iteration and union approach.
pub trait UnaryUnion {
    type Scalar: BoolOpsNum;

    /// Construct a tree of all the input geometries and progressively union them from the "bottom up"
    ///
    /// This is considerably more efficient than using e.g. `fold()` over an iterator of Polygons.
    /// # Examples
    ///
    /// ```
    /// use geo::{BooleanOps, UnaryUnion};
    /// use geo::{MultiPolygon, polygon};
    /// let poly1 = polygon![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 4.0, y: 0.0),
    ///     (x: 4.0, y: 4.0),
    ///     (x: 0.0, y: 4.0),
    ///     (x: 0.0, y: 0.0),
    /// ];
    /// let poly2 = polygon![
    ///     (x: 4.0, y: 0.0),
    ///     (x: 8.0, y: 0.0),
    ///     (x: 8.0, y: 4.0),
    ///     (x: 4.0, y: 4.0),
    ///     (x: 4.0, y: 0.0),
    /// ];
    /// let merged = &poly1.union(&poly2);
    /// let mp = MultiPolygon(vec![poly1, poly2]);
    /// // A larger single rectangle
    /// let combined = mp.unary_union();
    /// assert_eq!(&combined, merged);
    /// ```
    fn unary_union(self) -> MultiPolygon<Self::Scalar>;
}

// This function carries out a full post-order traversal of the tree, building up MultiPolygons from inside to outside.
// Though the operation is carried out via fold() over the tree iterator, there are two actual nested operations:
// "fold" operations on leaf nodes build up output MultiPolygons by adding Polygons to them via union and
// "reduce" operations on parent nodes combine these output MultiPolygons from leaf operations by recursion
fn bottom_up_fold_reduce<T, S, I, F, R>(ops: &Ops<I, F, R>, parent: &ParentNode<T>) -> S
where
    // This operation only requires two types: output (S) and input (T)
    T: RTreeObject,
    // Because this is a fold operation, we need to initialise a "container" to which we'll be adding using union.
    // The output of a union op is a MultiPolygon.
    I: Fn() -> S,
    // The meat of the fold op is unioning input borrowed leaf Polygons into an output MultiPolygon.
    F: Fn(S, &T) -> S,
    // Parent nodes require us to process their child nodes to produce a MultiPolygon. We do this recursively.
    // This is a reduce op so there's no need for an init value and the two inputs must have the same type: MultiPolygon
    R: Fn(S, S) -> S,
{
    parent
        .children()
        .iter()
        .fold((ops.init)(), |accum, child| match child {
            RTreeNode::Leaf(value) => (ops.fold)(accum, value),
            RTreeNode::Parent(parent) => {
                let value = bottom_up_fold_reduce(ops, parent);
                (ops.reduce)(accum, value)
            }
        })
}

fn par_bottom_up_fold_reduce<T, S, I, F, R>(ops: &Ops<I, F, R>, parent: &ParentNode<T>) -> S
where
    T: RTreeObject,
    RTreeNode<T>: Send + Sync,
    S: Send,
    I: Fn() -> S + Send + Sync,
    F: Fn(S, &T) -> S + Send + Sync,
    R: Fn(S, S) -> S + Send + Sync,
{
    parent
        .children()
        .into_par_iter()
        .fold(&ops.init, |accum, child| match child {
            RTreeNode::Leaf(value) => (ops.fold)(accum, value),
            RTreeNode::Parent(parent) => {
                let value = par_bottom_up_fold_reduce(ops, parent);
                (ops.reduce)(accum, value)
            }
        })
        .reduce(&ops.init, &ops.reduce)
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

impl<'a, T, Boppable, BoppableCollection> UnaryUnion for &'a BoppableCollection
where
    T: BoolOpsNum,
    Boppable: BooleanOps<Scalar = T> + RTreeObject + 'a,
    &'a BoppableCollection: IntoIterator<Item = &'a Boppable>,
{
    type Scalar = T;

    fn unary_union(self) -> MultiPolygon<Self::Scalar> {
        // these three functions drive the union operation
        let init = || MultiPolygon::<T>::new(vec![]);
        let fold = |mut accum: MultiPolygon<T>,
                    poly: &CachedEnvelope<ObjectRef<'a, Boppable>>|
         -> MultiPolygon<T> {
            accum = accum.union(&***poly);
            accum
        };
        let reduce = |accum1: MultiPolygon<T>, accum2: MultiPolygon<T>| -> MultiPolygon<T> {
            accum1.union(&accum2)
        };
        let rtree = RTree::bulk_load(
            self.into_iter()
                .map(|p| CachedEnvelope::new(ObjectRef::new(p)))
                .collect(),
        );
        let ops = Ops { init, fold, reduce };
        bottom_up_fold_reduce(&ops, rtree.root())
    }
}

#[cfg(feature = "multithreading")]
/// Wrapper type which signals to algorithms operating on `T` that utilizing parallelism might be viable.
pub struct AllowMultithreading<T>(pub T);

#[cfg(feature = "multithreading")]
impl<'a, T, Boppable, BoppableCollection> UnaryUnion for AllowMultithreading<&'a BoppableCollection>
where
    T: BoolOpsNum + Send,
    Boppable: BooleanOps<Scalar = T> + RTreeObject + 'a + Send + Sync,
    <Boppable as RTreeObject>::Envelope: Send + Sync,
    &'a BoppableCollection: IntoParallelIterator<Item = &'a Boppable>,
{
    type Scalar = T;

    fn unary_union(self) -> MultiPolygon<Self::Scalar> {
        // these three functions drive the union operation
        let init = || MultiPolygon::<T>::new(vec![]);
        let fold = |mut accum: MultiPolygon<T>,
                    poly: &CachedEnvelope<ObjectRef<'a, Boppable>>|
         -> MultiPolygon<T> {
            accum = accum.union(&***poly);
            accum
        };
        let reduce = |accum1: MultiPolygon<T>, accum2: MultiPolygon<T>| -> MultiPolygon<T> {
            accum1.union(&accum2)
        };
        let rtree = RTree::bulk_load(
            self.0
                .into_par_iter()
                .map(|p| CachedEnvelope::new(ObjectRef::new(p)))
                .collect(),
        );

        let ops = Ops { init, fold, reduce };
        par_bottom_up_fold_reduce(&ops, rtree.root())
    }
}
