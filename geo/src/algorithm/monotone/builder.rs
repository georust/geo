//! Polygon monotone subdivision algorithm
//!
//! This implementation is based on these awesome [lecture notes] by David
//! Mount.  The broad idea is to run a left-right planar sweep the segments of the
//! polygon and try to iteratively extend parallel chains of montone segments
//! along sweep points.  
//!
//! [lecture notes]:
//! //www.cs.umd.edu/class/spring2020/cmsc754/Lects/lect05-triangulate.pdf

use super::{MonoPoly, SimpleSweep};
use crate::{
    sweep::{EventType, LineOrPoint, SweepPoint},
    *,
};
use std::{cell::Cell, mem::replace};

pub fn monotone_subdivision<T: GeoNum>(polygon: Polygon<T>) -> Vec<MonoPoly<T>> {
    Builder::from_polygon(polygon).build()
}

pub(super) struct Builder<T: GeoNum> {
    sweep: SimpleSweep<T, Info>,
    chains: Vec<Option<Chain<T>>>,
    outputs: Vec<MonoPoly<T>>,
}

impl<T: GeoNum> Builder<T> {
    /// Create a new builder from a polygon.
    pub fn from_polygon(polygon: Polygon<T>) -> Self {
        let (ext, ints) = polygon.into_inner();
        let iter = Some(ext)
            .into_iter()
            .chain(ints.into_iter())
            .flat_map(|ls| -> Vec<_> { ls.lines().collect() })
            .filter_map(|line| {
                if line.start == line.end {
                    None
                } else {
                    Some((LineOrPoint::from(line), Default::default()))
                }
            });
        Self {
            sweep: SimpleSweep::new(iter),
            chains: Vec::new(),
            outputs: Vec::new(),
        }
    }
    pub fn build(mut self) -> Vec<MonoPoly<T>> {
        while self.process_next_pt() {}
        self.outputs
    }

    fn process_next_pt(&mut self) -> bool {
        // Step 1. Get all the incoming and outgoing segments at the next point,
        // and sort each of them by sweep ordering.
        let mut incoming = vec![];
        let mut outgoing = vec![];

        let pt = if let Some(pt) = self.sweep.next_point(|seg, ev| match ev {
            EventType::LineRight => {
                incoming.push(seg);
            }
            EventType::LineLeft => {
                outgoing.push(seg);
            }
            _ => unreachable!("unexpected event type"),
        }) {
            pt
        } else {
            return false;
        };
        incoming.sort_by(|a, b| a.partial_cmp(b).unwrap());
        outgoing.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Step 2. Calculate region below the point, and if any previous point
        // registered a help.
        let bot_segment = self.sweep.prev_active_from_geom(pt.into());
        let (bot_region, bot_help) = bot_segment
            .as_ref()
            .map(|seg| (seg.payload().next_is_inside.get(), seg.payload().help.get()))
            .unwrap_or((false, None));

        // Step 3. Reduce incoming segments.  Any two consecutive incoming
        // segment that encloses the input region should not complete a
        // mono-polygon; so we `finish` their chains.  Thus, we should be left
        // with at-most two incoming segments.
        if !incoming.is_empty() {
            let n = incoming.len();

            let start_idx = if bot_region { 1 } else { 0 };
            let ub_idx = n - (n - start_idx) % 2;

            let mut iter = incoming.drain(start_idx..ub_idx);
            while let Some(first) = iter.next() {
                let second = iter.next().unwrap();

                let fc = self.chains[first.payload().chain_idx.get()].take().unwrap();
                let sc = self.chains[second.payload().chain_idx.get()]
                    .take()
                    .unwrap();

                // Any help registered on the first segment should be considered.
                if let Some(help) = first.payload().help.get() {
                    first.payload().help.set(None);
                    let fhc = self.chains[help[0]].take().unwrap();
                    let shc = self.chains[help[1]].take().unwrap();
                    self.outputs.push(fc.finish_with(fhc, *pt));
                    self.outputs.push(shc.finish_with(sc, *pt));
                } else {
                    self.outputs.push(fc.finish_with(sc, *pt));
                }
            }
        }
        debug_assert!(incoming.len() <= 2);
        // Handle help on bot segment and reduce further to at-most two chain
        // indices that need to be extended.
        let in_chains = if let Some(h) = bot_help {
            bot_segment.as_ref().unwrap().payload().help.set(None);
            if !incoming.is_empty() {
                let sc = self.chains[incoming[0].payload().chain_idx.get()]
                    .take()
                    .unwrap();
                let shc = self.chains[h[1]].take().unwrap();
                self.outputs.push(shc.finish_with(sc, *pt));
                if incoming.len() == 1 {
                    (Some(h[0]), None)
                } else {
                    (Some(h[0]), Some(incoming[1].payload().chain_idx.get()))
                }
            } else {
                (Some(h[0]), Some(h[1]))
            }
        } else {
            if incoming.is_empty() {
                (None, None)
            } else if incoming.len() == 1 {
                (Some(incoming[0].payload().chain_idx.get()), None)
            } else {
                (
                    Some(incoming[0].payload().chain_idx.get()),
                    Some(incoming[1].payload().chain_idx.get()),
                )
            }
        };

        // Step 4. Reduce outgoing segments.  Any two consecutive outgoing
        // segment that encloses the input region can be started as a new
        // region.  This again reduces the outgoing list to atmost two segments.
        if !outgoing.is_empty() {
            let n = outgoing.len();
            let start_idx = if bot_region { 1 } else { 0 };
            let ub_idx = n - (n - start_idx) % 2;
            let mut iter = outgoing.drain(start_idx..ub_idx);
            while let Some(first) = iter.next() {
                let second = iter.next().unwrap();

                let bot = first.line().right();
                let top = second.line().right();
                self.chains
                    .extend(Chain::from_segment_pair(*pt, *bot, *top).map(Some));
                first.payload().next_is_inside.set(true);
                second.payload().next_is_inside.set(false);
                first.payload().chain_idx.set(self.chains.len() - 2);
                second.payload().chain_idx.set(self.chains.len() - 1);
            }
        }
        debug_assert!(outgoing.len() <= 2);

        // TODO: This is not needed; the below should handle setting regions.
        let mut curr_region = bot_region;
        outgoing.iter().for_each(|seg| {
            curr_region = !curr_region;
            seg.payload().next_is_inside.set(curr_region);
        });

        // Step 5. Tie up incoming and outgoing as applicable
        match in_chains {
            (None, None) => {
                // No incoming segments left after reduction.  Since we have
                // reduced the outgoing, the only case is the "split vertex" or
                // "<" case.  Here, we will use the helper_chain to extend the
                // chain.
                if !outgoing.is_empty() {
                    assert!(outgoing.len() == 2);
                    let first = &outgoing[0];
                    let second = &outgoing[1];
                    let bot_segment = bot_segment.as_ref().unwrap();

                    let idx = bot_segment
                        .payload()
                        .helper_chain
                        .get()
                        .unwrap_or_else(|| bot_segment.payload().chain_idx.get());
                    let new_chains = self.chains[idx].as_mut().unwrap().swap_at_top(*pt);
                    self.chains.extend(new_chains.map(Some));
                    first.payload().next_is_inside.set(false);
                    second.payload().next_is_inside.set(true);
                    first.payload().chain_idx.set(self.chains.len() - 2);
                    second.payload().chain_idx.set(self.chains.len() - 1);
                }
            }
            (Some(idx), None) => {
                assert!(outgoing.len() == 1);
                let first = &outgoing[0];
                let bot = first.line().right();
                self.chains[idx].as_mut().unwrap().push(*bot);
                first.payload().next_is_inside.set(!bot_region);
                first.payload().chain_idx.set(idx);
            }
            (Some(idx), Some(jdx)) => {
                if !outgoing.is_empty() {
                    assert!(outgoing.len() == 2);
                    let first = &outgoing[0];
                    let second = &outgoing[1];
                    let bot = first.line().right();
                    let top = second.line().right();
                    self.chains[idx].as_mut().unwrap().push(*bot);
                    self.chains[jdx].as_mut().unwrap().push(*top);
                    first.payload().next_is_inside.set(true);
                    second.payload().next_is_inside.set(false);
                } else {
                    bot_segment.unwrap().payload().help.set(Some([idx, jdx]));
                }
            }
            _ => unreachable!(),
        }

        true
    }
}

pub(super) struct Chain<T: GeoNum>(LineString<T>);

impl<T: GeoNum> Chain<T> {
    pub fn from_segment_pair(start: Coord<T>, first: Coord<T>, second: Coord<T>) -> [Self; 2] {
        [
            Chain(line_string![start, first]),
            Chain(line_string![start, second]),
        ]
    }

    pub fn swap_at_top(&mut self, pt: Coord<T>) -> [Self; 2] {
        let top = self.0 .0.pop().unwrap();
        let prev = *self.0 .0.last().unwrap();
        self.0 .0.push(pt);

        let old_chain = Chain(replace(&mut self.0 .0, vec![prev, top]).into());
        let new_chain = Chain(vec![prev, pt].into());

        let lp1 = LineOrPoint::from((prev.into(), top.into()));
        let lp2 = LineOrPoint::from((prev.into(), pt.into()));
        if lp1 > lp2 {
            [old_chain, new_chain]
        } else {
            [new_chain, old_chain]
        }
    }

    pub fn push(&mut self, pt: Coord<T>) {
        self.0 .0.push(pt);
    }

    pub fn finish_with(mut self, mut other: Self, pt: Coord<T>) -> MonoPoly<T> {
        assert!(self.0 .0[0] == other.0 .0[0]);
        self.0 .0.push(pt);
        other.0 .0.push(pt);
        MonoPoly::new(other.0, self.0)
    }
}

#[derive(Debug)]
struct Info {
    next_is_inside: Cell<bool>,
    helper_chain: Cell<Option<usize>>,
    help: Cell<Option<[usize; 2]>>,
    chain_idx: Cell<usize>,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            next_is_inside: Default::default(),
            helper_chain: Default::default(),
            help: Default::default(),
            chain_idx: Default::default(),
        }
    }
}
