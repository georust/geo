use std::cell::Cell;

use crate::{
    sweep::{EventType, LineOrPoint, SweepPoint},
    *,
};

use super::{MonoPoly, SimpleSweep};

pub(super) struct Chain<T: GeoNum> {
    chain: LineString<T>,
}

impl<T: GeoNum> Chain<T> {
    pub fn from_segment_pair(start: Coord<T>, first: Coord<T>, second: Coord<T>) -> [Self; 2] {
        [
            Chain {
                chain: line_string![start, first],
            },
            Chain {
                chain: line_string![start, second],
            },
        ]
    }

    pub fn push(&mut self, pt: Coord<T>) {
        self.chain.0.push(pt);
    }

    pub fn finish_with(self, other: Self, pt: Coord<T>) -> MonoPoly<T> {
        todo!()
    }
}

#[derive(Default, Debug)]
struct Info {
    next_is_inside: Cell<bool>,
    helper_idx: Cell<Option<usize>>,
    chain_idx: Cell<usize>,
}

pub(super) struct Builder<T: GeoNum> {
    sweep: SimpleSweep<T, Info>,
    chains: Vec<Option<Chain<T>>>,
    outputs: Vec<MonoPoly<T>>,
}

impl<T: GeoNum> Builder<T> {
    /// Create a new builder from a polygon.
    pub fn new(polygon: Polygon<T>) -> Self {
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

    fn churn(&mut self) -> bool {
        let mut incoming = vec![];
        let mut outgoing = vec![];

        let pt = if let Some(pt) = self.sweep.next_point(|seg, ev| match ev {
            EventType::LineLeft => {
                incoming.push(seg);
            }
            EventType::LineRight => {
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

        let bot_region = if !outgoing.is_empty() {
            self.sweep
                .prev_active(&outgoing[0])
                .map(|seg| seg.payload().next_is_inside.get())
                .unwrap_or(false)
        } else if !incoming.is_empty() {
            todo!()
        } else {
            // Nothing to process.
            return true;
        };

        if !incoming.is_empty() {
            let n = incoming.len();
            let start_idx = if incoming[0].payload().next_is_inside.get() {
                0
            } else {
                1
            };

            let ub_idx = n - (n - start_idx) % 2;
            let mut iter = incoming.drain(start_idx..ub_idx);
            while let Some(first) = iter.next() {
                let second = iter.next().unwrap();

                let fc = self.chains[first.payload().chain_idx.get()].take().unwrap();
                let sc = self.chains[second.payload().chain_idx.get()]
                    .take()
                    .unwrap();
                self.outputs.push(fc.finish_with(sc, *pt));
            }
        }
        debug_assert!(incoming.len() <= 2);

        if !outgoing.is_empty() {
            let n = outgoing.len();
            let start_idx = if bot_region { 1 } else { 0 };
            let ub_idx = n - (n - start_idx) % 2;
            let mut iter = outgoing.drain(start_idx..ub_idx);
            while let Some(first) = iter.next() {
                let second = iter.next().unwrap();

                let bot = first.line().right();
                let top = second.line().right();
                self.chains.extend(
                    Chain::from_segment_pair(*pt, *bot, *top)
                        .into_iter()
                        .map(Some),
                );
                first.payload().next_is_inside.set(true);
                second.payload().next_is_inside.set(false);
                first.payload().chain_idx.set(self.chains.len() - 1);
                second.payload().chain_idx.set(self.chains.len());
            }
        }
        debug_assert!(outgoing.len() <= 2);

        let mut curr_region = bot_region;
        outgoing.iter().for_each(|seg| {
            curr_region = !curr_region;
            seg.payload().next_is_inside.set(curr_region);
        });

        true
    }
}
