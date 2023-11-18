//! Polygon monotone subdivision algorithm
//!
//! This implementation is based on these awesome [lecture notes] by David
//! Mount.  The broad idea is to run a left-right planar sweep on the segments
//! of the polygon and try to iteratively extend parallel monotone chains.
//!
//! [lecture notes]:
//! //www.cs.umd.edu/class/spring2020/cmsc754/Lects/lect05-triangulate.pdf

use super::{MonoPoly, SimpleSweep};
use crate::{
    sweep::{EventType, LineOrPoint, SweepPoint},
    *,
};
use std::{cell::Cell, mem::replace};

/// Construct a monotone subdivision (along the X-axis) of an iterator of polygons.
///
/// Returns the set of monotone polygons that make up the subdivision.  The
/// input polygon(s) must be a valid `MultiPolygon` (see the validity section in
/// [`MultiPolygon`]).  In particular, each polygon must be simple, not
/// self-intersect, and contain only finite coordinates;  further, the polygons
/// must have distinct interiors, and their boundaries may only intersect at
/// finite points.
pub fn monotone_subdivision<T: GeoNum, I: IntoIterator<Item = Polygon<T>>>(
    iter: I,
) -> Vec<MonoPoly<T>> {
    Builder::from_polygons_iter(iter).build()
}

pub(super) struct Builder<T: GeoNum> {
    sweep: SimpleSweep<T, Info>,
    chains: Vec<Option<Chain<T>>>,
    outputs: Vec<MonoPoly<T>>,
}

impl<T: GeoNum> Builder<T> {
    /// Create a new builder from a polygon.
    pub fn from_polygons_iter<I: IntoIterator<Item = Polygon<T>>>(iter: I) -> Self {
        let iter = iter.into_iter().flat_map(|polygon| {
            let (ext, ints) = polygon.into_inner();
            Some(ext)
                .into_iter()
                .chain(ints)
                .flat_map(|ls| -> Vec<_> { ls.lines().collect() })
                .filter_map(|line| {
                    if line.start == line.end {
                        None
                    } else {
                        let line = LineOrPoint::from(line);
                        debug!("adding line {:?}", line);
                        Some((line, Default::default()))
                    }
                })
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
                let rt = seg.line().right();
                incoming.push(seg);
                let chain_idx = incoming.last().unwrap().payload().chain_idx.get();
                self.chains[chain_idx].as_mut().unwrap().fix_top(*rt);
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

        info!(
            "\n\nprocessing point {:?}, #in={}, #out={}",
            pt,
            incoming.len(),
            outgoing.len()
        );

        // Step 2. Calculate region below the point, and if any previous point
        // registered a help.
        let bot_segment = self.sweep.prev_active_from_geom(pt.into());
        let (bot_region, bot_help) = bot_segment
            .as_ref()
            .map(|seg| (seg.payload().next_is_inside.get(), seg.payload().help.get()))
            .unwrap_or((false, None));
        debug!("bot region: {:?}", bot_region);
        debug!("bot segment: {:?}", bot_segment.as_ref().map(|s| s.line()));

        // Step 3. Reduce incoming segments.  Any two consecutive incoming
        // segment that encloses the input region should now complete a
        // mono-polygon; so we `finish` their chains.  Thus, we should be left
        // with at-most two incoming segments.
        if !incoming.is_empty() {
            let n = incoming.len();

            #[allow(clippy::bool_to_int_with_if)]
            let start_idx = if bot_region { 1 } else { 0 };
            let ub_idx = n - (n - start_idx) % 2;
            debug!("reducing incoming segments: {n} -> {start_idx}..{ub_idx}");

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
                    let mut fhc = self.chains[help[0]].take().unwrap();
                    let mut shc = self.chains[help[1]].take().unwrap();
                    fhc.push(*pt);
                    shc.push(*pt);
                    self.outputs.push(fc.finish_with(fhc));
                    self.outputs.push(shc.finish_with(sc));
                } else {
                    self.outputs.push(fc.finish_with(sc));
                }
            }
        }
        debug_assert!(incoming.len() <= 2);
        // Handle help on bot segment and reduce further to at-most two chain
        // indices that need to be extended.
        let in_chains = if let Some(h) = bot_help {
            debug!("serving to help: {h:?}");
            bot_segment.as_ref().unwrap().payload().help.set(None);
            if !incoming.is_empty() {
                let sc = self.chains[incoming[0].payload().chain_idx.get()]
                    .take()
                    .unwrap();
                let mut shc = self.chains[h[1]].take().unwrap();
                shc.push(*pt);
                self.chains[h[0]].as_mut().unwrap().push(*pt);
                self.outputs.push(shc.finish_with(sc));
                if incoming.len() == 1 {
                    (Some(h[0]), None)
                } else {
                    let last_idx = if let Some(h) = incoming[1].payload().help.get() {
                        let mut fhc = self.chains[h[0]].take().unwrap();
                        let fc = self.chains[incoming[1].payload().chain_idx.get()]
                            .take()
                            .unwrap();
                        fhc.push(*pt);
                        self.chains[h[1]].as_mut().unwrap().push(*pt);
                        self.outputs.push(fc.finish_with(fhc));
                        h[1]
                    } else {
                        incoming[1].payload().chain_idx.get()
                    };
                    (Some(h[0]), Some(last_idx))
                }
            } else {
                self.chains[h[0]].as_mut().unwrap().push(*pt);
                self.chains[h[1]].as_mut().unwrap().push(*pt);
                (Some(h[0]), Some(h[1]))
            }
        } else if incoming.is_empty() {
            (None, None)
        } else {
            let last_incoming = incoming.last().unwrap();
            let last_idx = if let Some(h) = last_incoming.payload().help.get() {
                let mut fhc = self.chains[h[0]].take().unwrap();
                let fc = self.chains[last_incoming.payload().chain_idx.get()]
                    .take()
                    .unwrap();
                fhc.push(*pt);
                self.chains[h[1]].as_mut().unwrap().push(*pt);
                self.outputs.push(fc.finish_with(fhc));
                h[1]
            } else {
                last_incoming.payload().chain_idx.get()
            };
            if incoming.len() == 1 {
                (Some(last_idx), None)
            } else {
                (Some(incoming[0].payload().chain_idx.get()), Some(last_idx))
            }
        };

        // Step 4. Reduce outgoing segments.  Any two consecutive outgoing
        // segment that encloses the input region can be started as a new
        // region.  This again reduces the outgoing list to atmost two segments.
        if !outgoing.is_empty() {
            let n = outgoing.len();
            #[allow(clippy::bool_to_int_with_if)]
            let start_idx = if bot_region { 1 } else { 0 };
            let ub_idx = n - (n - start_idx) % 2;
            debug!("reducing outgoing segments: {n} -> {start_idx}..{ub_idx}");
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

        // Step 5. Tie up incoming and outgoing as applicable
        debug!("in_chains: {in_chains:?}");
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
                    let mut new_chains = self.chains[idx].as_mut().unwrap().swap_at_top(*pt);
                    new_chains[0].push(*first.line().right());
                    new_chains[1].push(*second.line().right());
                    self.chains.extend(new_chains.map(Some));
                    first.payload().next_is_inside.set(false);
                    second.payload().next_is_inside.set(true);
                    first.payload().chain_idx.set(self.chains.len() - 2);
                    second.payload().chain_idx.set(self.chains.len() - 1);

                    bot_segment
                        .payload()
                        .helper_chain
                        .set(Some(self.chains.len() - 2));
                } else {
                    debug_assert!(!bot_region);
                }
            }
            (Some(idx), None) => {
                assert!(outgoing.len() == 1);
                let first = &outgoing[0];
                let bot = first.line().right();
                self.chains[idx].as_mut().unwrap().push(*bot);
                first.payload().next_is_inside.set(!bot_region);
                first.payload().chain_idx.set(idx);
                if let Some(b) = bot_segment {
                    b.payload().helper_chain.set(Some(idx))
                }
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
                    first.payload().next_is_inside.set(false);
                    second.payload().next_is_inside.set(true);
                    first.payload().chain_idx.set(idx);
                    second.payload().chain_idx.set(jdx);
                } else {
                    debug!("registering help: [{}, {}]", idx, jdx);
                    bot_segment
                        .as_ref()
                        .unwrap()
                        .payload()
                        .help
                        .set(Some([idx, jdx]));
                }
                if let Some(b) = bot_segment {
                    b.payload().helper_chain.set(Some(idx))
                }
            }
            _ => unreachable!(),
        }

        true
    }
}

pub(super) struct Chain<T: GeoNum>(LineString<T>);

impl<T: GeoNum + std::fmt::Debug> std::fmt::Debug for Chain<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bot: Vec<SweepPoint<T>> = self.0 .0.iter().map(|c| (*c).into()).collect();
        f.debug_tuple("Chain").field(&bot).finish()
    }
}

impl<T: GeoNum> Chain<T> {
    pub fn from_segment_pair(start: Coord<T>, first: Coord<T>, second: Coord<T>) -> [Self; 2] {
        let [x, y, z] = [SweepPoint::from(start), first.into(), second.into()];
        debug!("Creating chain from {x:?} -> {y:?}");
        debug!("                    {x:?} -> {z:?}");
        [
            Chain(line_string![start, first]),
            Chain(line_string![start, second]),
        ]
    }

    pub fn fix_top(&mut self, pt: Coord<T>) {
        *self.0 .0.last_mut().unwrap() = pt;
    }

    pub fn swap_at_top(&mut self, pt: Coord<T>) -> [Self; 2] {
        let top = self.0 .0.pop().unwrap();
        let prev = *self.0 .0.last().unwrap();
        debug!(
            "chain swap: {:?} -> {:?}",
            SweepPoint::from(top),
            SweepPoint::from(pt)
        );
        debug!("\tprev = {:?}", SweepPoint::from(prev));
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
        debug!("chain push: {:?} -> {:?}", self.0 .0.last().unwrap(), pt);
        self.0 .0.push(pt);
    }

    pub fn finish_with(self, other: Self) -> MonoPoly<T> {
        assert!(
            self.0 .0[0] == other.0 .0[0]
                && self.0 .0.last().unwrap() == other.0 .0.last().unwrap(),
            "chains must finish with same start/end points"
        );
        debug!("finishing {self:?} with {other:?}");
        MonoPoly::new(other.0, self.0)
    }
}

#[derive(Debug, Default, Clone)]
struct Info {
    next_is_inside: Cell<bool>,
    helper_chain: Cell<Option<usize>>,
    help: Cell<Option<[usize; 2]>>,
    chain_idx: Cell<usize>,
}
