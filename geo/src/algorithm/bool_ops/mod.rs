use geo_types::{MultiLineString, MultiPolygon};

use crate::{CoordsIter, GeoFloat, GeoNum, Polygon};

/// Enum to represent geometry types that can be used in boolean ops
pub enum BopGeometry<'a, T: GeoNum> {
    Polygon(&'a Polygon<T>),
    MultiPolygon(&'a MultiPolygon<T>),
}

impl<'a, T: GeoNum> From<&'a Polygon<T>> for BopGeometry<'a, T> {
    fn from(polygon: &'a Polygon<T>) -> Self {
        BopGeometry::Polygon(polygon)
    }
}

impl<'a, T: GeoNum> From<&'a MultiPolygon<T>> for BopGeometry<'a, T> {
    fn from(multi_polygon: &'a MultiPolygon<T>) -> Self {
        BopGeometry::MultiPolygon(multi_polygon)
    }
}

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

    fn boolean_op<'a, G>(&self, other: &'a G, op: OpType) -> MultiPolygon<Self::Scalar>
    where
        BopGeometry<'a, Self::Scalar>: From<&'a G>,
        <Self>::Scalar: 'a;

    fn intersection<'a, G>(&self, other: &'a G) -> MultiPolygon<Self::Scalar>
    where
        BopGeometry<'a, Self::Scalar>: From<&'a G>,
        <Self>::Scalar: 'a,
    {
        self.boolean_op(other, OpType::Intersection)
    }

    fn union<'a, G>(&self, other: &'a G) -> MultiPolygon<Self::Scalar>
    where
        BopGeometry<'a, Self::Scalar>: From<&'a G>,
        <Self>::Scalar: 'a,
    {
        self.boolean_op(other, OpType::Union)
    }

    fn xor<'a, G>(&self, other: &'a G) -> MultiPolygon<Self::Scalar>
    where
        BopGeometry<'a, Self::Scalar>: From<&'a G>,
        <Self>::Scalar: 'a,
    {
        self.boolean_op(other, OpType::Xor)
    }

    fn difference<'a, G>(&self, other: &'a G) -> MultiPolygon<Self::Scalar>
    where
        BopGeometry<'a, Self::Scalar>: From<&'a G>,
        <Self>::Scalar: 'a,
    {
        self.boolean_op(other, OpType::Difference)
    }

    /// Clip a 1-D geometry with self.
    ///
    /// Returns the portion of `ls` that lies within `self` (known as the set-theoretic
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

impl<T: GeoFloat> BooleanOps for Polygon<T> {
    type Scalar = T;

    fn boolean_op<'a, G>(&self, other: &'a G, op: OpType) -> MultiPolygon<Self::Scalar>
    where
        BopGeometry<'a, Self::Scalar>: From<&'a G>,
        <Self>::Scalar: 'a,
    {
        let other: BopGeometry<'a, Self::Scalar> = other.into();
        let spec = BoolOp::from(op);
        let mut bop = Proc::new(spec, self.coords_count() + other.coords_count());
        bop.add_polygon(self, 0);
        match other {
            BopGeometry::Polygon(other_polygon) => bop.add_polygon(other_polygon, 1),
            BopGeometry::MultiPolygon(other_multi_polygon) => {
                bop.add_multi_polygon(other_multi_polygon, 1)
            }
        }
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

    fn boolean_op<'a, G>(&self, other: &'a G, op: OpType) -> MultiPolygon<Self::Scalar>
    where
        BopGeometry<'a, Self::Scalar>: From<&'a G>,
        <Self>::Scalar: 'a,
    {
        let other: BopGeometry<'a, Self::Scalar> = other.into();
        let spec = BoolOp::from(op);
        let mut bop = Proc::new(spec, self.coords_count() + other.coords_count());
        bop.add_multi_polygon(self, 0);
        match other {
            BopGeometry::Polygon(other_polygon) => bop.add_polygon(other_polygon, 1),
            BopGeometry::MultiPolygon(other_multi_polygon) => {
                bop.add_multi_polygon(other_multi_polygon, 1)
            }
        }
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
        ls.0.iter().for_each(|l| bop.add_line_string(l, 0));
        bop.sweep()
    }
}

impl<'a, T: GeoNum> BopGeometry<'a, T> {
    fn coords_count(&self) -> usize {
        match self {
            BopGeometry::Polygon(polygon) => polygon.coords_count(),
            BopGeometry::MultiPolygon(multi_polygon) => multi_polygon.coords_count(),
        }
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
