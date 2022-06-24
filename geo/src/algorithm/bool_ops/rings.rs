use crate::utils::EitherIter as Either;
use crate::{
    winding_order::{Winding, WindingOrder},
    Coordinate, GeoFloat, GeoNum, LineString,
};
use log::{debug, trace};
use std::{
    collections::{BTreeMap, VecDeque},
    ops::Deref,
};

use crate::sweep::{LineOrPoint, SweepPoint};

#[derive(Debug, Clone)]
pub struct Rings<T: GeoNum> {
    chains: Vec<Chain<T>>,
    end_points: BTreeMap<SweepPoint<T>, Link>,
}

impl<T: GeoNum> Default for Rings<T> {
    fn default() -> Self {
        Self {
            chains: Default::default(),
            end_points: Default::default(),
        }
    }
}
impl<T: GeoFloat> Rings<T> {
    pub fn finish(mut self) -> Vec<Ring<T>> {
        let mut output = vec![];
        let mut curr_chain_idx = self.chains.len();
        trace!("finalizing {n} chains", n = curr_chain_idx);

        let mut history = BTreeMap::new();
        while curr_chain_idx > 0 {
            curr_chain_idx -= 1;
            if self.chains[curr_chain_idx].items.is_empty() {
                continue;
            }

            let mut ls: Vec<Coordinate<T>> = vec![];
            let mut loop_link = Link {
                idx: curr_chain_idx,
                to_front: true,
            };

            // re-use btree-map
            history.clear();
            let winding = self.chains[curr_chain_idx].winding;

            loop {
                trace!("traversing chain: {loop_link:?}");
                let iter = {
                    let iter = self.chains[loop_link.idx].items.iter();
                    if loop_link.to_front {
                        Either::A(iter)
                    } else {
                        Either::B(iter.rev())
                    }
                };

                for pt in iter {
                    trace!("\tpt: {pt:?}");
                    if let Some(idx) = history.get(pt) {
                        trace!("intersects at {idx}");
                        // [idx..ls.len()) is a separate loop.
                        let new_ls = ls
                            .drain(*idx..)
                            .inspect(|pt: &Coordinate<_>| {
                                history.remove(&((*pt).into())).unwrap();
                            })
                            .collect();
                        output.push(Ring::from_coords(new_ls, &winding));
                    }
                    history.insert(*pt, ls.len());
                    trace!("insert: {pt:?} @ idx {ln}", ln = ls.len());
                    ls.push(*pt.deref());
                }

                self.chains[loop_link.idx].items.clear();
                loop_link = if loop_link.to_front {
                    self.chains[loop_link.idx].next_back.unwrap()
                } else {
                    self.chains[loop_link.idx].next_front.unwrap()
                };

                if loop_link.idx == curr_chain_idx {
                    debug_assert!(loop_link.to_front);
                    break;
                }
            }
            trace!("ring {idx}:", idx = output.len());
            trace!("\t{ls:?}");
            output.push(Ring::from_coords(ls, &winding));
        }
        output
    }

    pub fn add_edge(&mut self, geom: LineOrPoint<T>, winding: WindingOrder) {
        trace!("Rings.add_edge: {geom:?} {winding:?}");
        let left = geom.left();
        let right = geom.right();

        let (le, re) = (
            self.end_points.get(&left).copied(),
            self.end_points.get(&right).copied(),
        );

        match (le, re) {
            (None, None) => {
                // New chain.
                let idx = self.chains.len();
                self.chains.push(Chain::new(left, right, winding));
                self.end_points.insert(
                    left,
                    Link {
                        idx,
                        to_front: true,
                    },
                );
                self.end_points.insert(
                    right,
                    Link {
                        idx,
                        to_front: false,
                    },
                );
                trace!("\tnew chain: {idx}: [{left:?} {right:?}]");
            }
            (None, Some(i)) => {
                // right matched against of chains[i]
                self.push_link(i, left, winding.inverse());
                self.end_points.remove(&right).unwrap();
                self.end_points.insert(left, i);
                trace!("\tadded to {i:?}: {left:?}");
            }
            (Some(i), None) => {
                // left matched against of chains[i]
                self.push_link(i, right, winding);
                self.end_points.remove(&left).unwrap();
                self.end_points.insert(right, i);
                trace!("\tadded to {i:?}: {right:?}");
            }
            (Some(i), Some(j)) => {
                // Since both end points matched, we don't
                // need to add to the end_points tree, nor
                // push_link.  Only need to connect the two chains.
                self.link_chains(i, j, winding);
                self.end_points.remove(&left).unwrap();
                self.end_points.remove(&right).unwrap();
                trace!("\tconnected chains {i:?} and {j:?}");
            }
        }
    }

    fn push_link(&mut self, l: Link, pt: SweepPoint<T>, winding: WindingOrder) {
        if l.to_front {
            self.chains[l.idx].push_front(pt);
            debug_assert_eq!(self.chains[l.idx].winding, winding.inverse());
        } else {
            self.chains[l.idx].push_back(pt);
            debug_assert_eq!(self.chains[l.idx].winding, winding);
        }
    }

    fn link_chains(&mut self, i: Link, j: Link, winding: WindingOrder) {
        if i.to_front {
            self.chains[i.idx].link_front(j);
            debug_assert_eq!(self.chains[i.idx].winding, winding.inverse());
        } else {
            self.chains[i.idx].link_back(j);
            debug_assert_eq!(self.chains[i.idx].winding, winding);
        }
        if j.to_front {
            self.chains[j.idx].link_front(i);
            debug_assert_eq!(self.chains[j.idx].winding, winding);
        } else {
            self.chains[j.idx].link_back(i);
            debug_assert_eq!(self.chains[j.idx].winding, winding.inverse());
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ring<T: GeoNum> {
    coords: LineString<T>,
    is_hole: bool,
}

impl<T: GeoNum> Ring<T> {
    fn from_coords(coords: Vec<Coordinate<T>>, region_winding: &WindingOrder) -> Self {
        debug!("new ring:");
        debug!("\tregion_winding: {region_winding:?}");
        debug!("\tcoords: {coords:?}");
        let mut ls = LineString(coords);
        ls.close();
        let winding = ls.winding_order().expect("ring must have a winding order");
        Self {
            coords: ls,
            is_hole: &winding != region_winding,
        }
    }

    /// Get the ring's is hole.
    #[must_use]
    pub fn is_hole(&self) -> bool {
        self.is_hole
    }

    /// Get a reference to the ring's coords.
    #[must_use]
    pub fn coords(&self) -> &LineString<T> {
        &self.coords
    }
}

#[derive(Debug, Clone)]
struct Chain<T: GeoNum> {
    winding: WindingOrder,
    items: VecDeque<SweepPoint<T>>,
    next_front: Option<Link>,
    next_back: Option<Link>,
}

impl<T: GeoNum> Chain<T> {
    pub fn new(start: SweepPoint<T>, end: SweepPoint<T>, winding: WindingOrder) -> Self {
        let mut this = Self {
            winding,
            items: VecDeque::with_capacity(8),
            next_front: None,
            next_back: None,
        };
        this.items.push_back(start);
        this.items.push_back(end);
        this
    }
    pub fn push_front(&mut self, pt: SweepPoint<T>) {
        debug_assert!(self.next_front.is_none());
        self.items.push_front(pt);
    }
    pub fn push_back(&mut self, pt: SweepPoint<T>) {
        debug_assert!(self.next_back.is_none());
        self.items.push_back(pt)
    }
    pub fn link_front(&mut self, link: Link) {
        debug_assert!(self.next_front.is_none());
        self.next_front = Some(link);
    }
    pub fn link_back(&mut self, link: Link) {
        debug_assert!(self.next_back.is_none());
        self.next_back = Some(link);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Link {
    idx: usize,
    to_front: bool,
}
