//! Iterators which return [`MonotoneChain`]s

use super::{
    MonotoneChain, MonotoneChainLineString, MonotoneChainMultiLineString,
    MonotoneChainMultiPolygon, MonotoneChainPolygon,
};
use crate::GeoNum;
use std::iter;

/// An iterator over a compatible Geometry type which yields &[`MonotoneChain`]s.
pub trait MonotoneChains<'a: 'caller, 'caller, T: GeoNum + 'a> {
    fn chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>> {
        self.exterior_chains().chain(self.interior_chains())
    }
    fn exterior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>>;
    fn interior_chains(&'caller self) -> impl Iterator<Item = &'caller MonotoneChain<'a, T>>;
}

// ============================================================================
// Implementations

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
