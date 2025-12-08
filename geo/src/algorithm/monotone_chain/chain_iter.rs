//! Iterators which return [`MonotoneChain`]s

use super::{
    MonotoneChain, MonotoneChainLineString, MonotoneChainMultiLineString,
    MonotoneChainMultiPolygon, MonotoneChainPolygon,
};
use crate::GeoNum;
use crate::geometry::*;
use std::iter;

/// An iterator over a compatible Geometry type which yields [`MonotoneChain`]s.
///
/// Similar to [`MonotoneChains`], but returns owned values instead of references
/// and works on non-[`MonotoneChain`] backed geometries too
pub trait MonotoneChainIter<'a, T: GeoNum + 'a> {
    fn chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.exterior_chains_iter()
            .chain(self.interior_chains_iter())
    }
    fn exterior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>>;
    fn interior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>>;
}

/// An iterator over a compatible Geometry type which yields &[`MonotoneChain`]s.
///
/// Similar to [`MonotoneChainIter`], but returns references
/// and only works on [`MonotoneChain`] backed geometries
pub trait MonotoneChains<'a: 'caller, 'caller, T: GeoNum + 'a> {
    fn chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        self.exterior_chains().chain(self.interior_chains())
    }
    fn exterior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>>;
    fn interior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>>;
}

// ============================================================================
// Implementations

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for LineString<T> {
    fn exterior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        iter::once(self).map(Into::<MonotoneChain<'a, T>>::into)
    }
    fn interior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        iter::empty()
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for MonotoneChainLineString<'a, T> {
    fn exterior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        iter::once(self.chain()).cloned()
    }
    fn interior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        iter::empty()
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for MultiLineString<T> {
    fn exterior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.iter()
            .flat_map(MonotoneChainIter::exterior_chains_iter)
    }
    fn interior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.iter()
            .flat_map(MonotoneChainIter::interior_chains_iter)
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for MonotoneChainMultiLineString<'a, T> {
    fn exterior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.components()
            .iter()
            .flat_map(MonotoneChainIter::exterior_chains_iter)
    }
    fn interior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.components()
            .iter()
            .flat_map(MonotoneChainIter::interior_chains_iter)
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for Polygon<T> {
    fn exterior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.exterior().chains_iter()
    }
    fn interior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.interiors()
            .iter()
            .flat_map(MonotoneChainIter::chains_iter)
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for MonotoneChainPolygon<'a, T> {
    fn exterior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.exterior().exterior_chains_iter()
    }
    fn interior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.interiors()
            .iter()
            .flat_map(MonotoneChainIter::chains_iter)
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for MultiPolygon<T> {
    fn exterior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.iter()
            .flat_map(MonotoneChainIter::exterior_chains_iter)
    }
    fn interior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.iter()
            .flat_map(MonotoneChainIter::interior_chains_iter)
    }
}

impl<'a, T: GeoNum + 'a> MonotoneChainIter<'a, T> for MonotoneChainMultiPolygon<'a, T> {
    fn exterior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.components()
            .iter()
            .flat_map(MonotoneChainIter::exterior_chains_iter)
    }
    fn interior_chains_iter(&'a self) -> impl Iterator<Item = MonotoneChain<'a, T>> {
        self.components()
            .iter()
            .flat_map(MonotoneChainIter::interior_chains_iter)
    }
}

// ============================================================================

impl<'a: 'caller, 'caller, T: GeoNum> MonotoneChains<'a, 'caller, T>
    for MonotoneChainLineString<'a, T>
{
    fn exterior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        iter::once(self.chain())
    }
    fn interior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        iter::empty()
    }
}

impl<'a: 'caller, 'caller, T: GeoNum> MonotoneChains<'a, 'caller, T>
    for MonotoneChainMultiLineString<'a, T>
{
    fn exterior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        self.components().iter().flat_map(MonotoneChains::chains)
    }
    fn interior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        self.components().iter().flat_map(MonotoneChains::chains)
    }
}

impl<'a: 'caller, 'caller, T: GeoNum> MonotoneChains<'a, 'caller, T>
    for MonotoneChainPolygon<'a, T>
{
    fn exterior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        self.exterior().chains()
    }
    fn interior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        self.interiors().iter().flat_map(MonotoneChains::chains)
    }
}

impl<'a: 'caller, 'caller, T: GeoNum> MonotoneChains<'a, 'caller, T>
    for MonotoneChainMultiPolygon<'a, T>
{
    fn exterior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        self.components()
            .iter()
            .flat_map(MonotoneChains::exterior_chains)
    }
    fn interior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        self.components()
            .iter()
            .flat_map(MonotoneChains::interior_chains)
    }
}
