use super::{MonotoneChain, MonotoneChainSegment};
use crate::geometry::*;
use crate::lines_iter::{LineStringIter, LinesIter};
use crate::{CoordNum, GeoNum};
use std::iter;

pub trait MonotoneChainIter<'a, T: GeoNum + 'a> {
    fn chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.exterior_chains().chain(self.interior_chains())
    }
    fn exterior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>>;
    fn interior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>>;
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for LineString<T> {
    fn exterior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        iter::once(self).map(Into::<MonotoneChain<'a, T>>::into)
    }
    fn interior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        iter::empty()
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for MultiLineString<T> {
    fn exterior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.iter().flat_map(MonotoneChainIter::exterior_chains)
    }
    fn interior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.iter().flat_map(MonotoneChainIter::interior_chains)
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for Polygon<T> {
    fn exterior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.exterior().chains()
    }
    fn interior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.interiors().iter().flat_map(MonotoneChainIter::chains)
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for MultiPolygon<T> {
    fn exterior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.iter().flat_map(MonotoneChainIter::exterior_chains)
    }
    fn interior_chains(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.iter().flat_map(MonotoneChainIter::interior_chains)
    }
}

// ============================================================================

impl<'a, T: CoordNum + 'a> LinesIter<'a> for MonotoneChainSegment<'a, T> {
    type Scalar = T;
    type Iter = LineStringIter<'a, Self::Scalar>;

    fn lines_iter(&'a self) -> Self::Iter {
        LineStringIter::new_from_coords(self.ls)
    }
}
