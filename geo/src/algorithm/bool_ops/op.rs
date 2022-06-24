use std::{cell::Cell, cmp::Ordering, fmt::Debug};

use super::*;
use crate::{
    sweep::{Cross, Crossing, CrossingsIter, LineOrPoint},
    winding_order::WindingOrder,
    CoordsIter, GeoFloat as Float, LineString, Polygon,
};

#[derive(Debug, Clone)]
pub struct Op<T: Float> {
    ty: OpType,
    edges: Vec<Edge<T>>,
}

impl<T: Float> Op<T> {
    pub fn new(ty: OpType, capacity: usize) -> Self {
        Op {
            ty,
            edges: Vec::with_capacity(capacity),
        }
    }

    // is_first -> whether it is from first input or second input
    pub(crate) fn add_multi_polygon(&mut self, mp: &MultiPolygon<T>, is_first: bool) {
        mp.0.iter().for_each(|p| self.add_polygon(p, is_first));
    }

    // is_first -> whether it is from first input or second input
    pub(crate) fn add_polygon(&mut self, poly: &Polygon<T>, is_first: bool) {
        self.add_closed_ring(poly.exterior(), is_first, false);
        for hole in poly.interiors() {
            self.add_closed_ring(hole, is_first, true);
        }
    }
    // is_first -> whether it is from first input or second input
    // _is_hole is not used rn; remove it once we fully handle fp issues
    fn add_closed_ring(&mut self, ring: &LineString<T>, is_first: bool, _is_hole: bool) {
        assert!(ring.is_closed());
        if ring.coords_count() <= 3 {
            return;
        }

        for line in ring.lines() {
            let lp: LineOrPoint<_> = line.into();
            if !lp.is_line() {
                continue;
            }

            debug!("processing: {lp:?}");

            let region = Region::infinity(self.ty);
            self.edges.push(Edge {
                geom: lp,
                is_first,
                _region: region.into(),
                _region_2: region.into(),
            });
        }
    }

    pub fn sweep(&self) -> Vec<Ring<T>> {
        let mut iter = CrossingsIter::from_iter(self.edges.iter());
        let mut rings = Rings::default();

        while let Some(pt) = iter.next() {
            trace!(
                "\n\nSweep point: {pt:?}, {n} intersection segments",
                n = iter.intersections_mut().len()
            );
            fn compare_crossings<X: Cross>(a: &Crossing<X>, b: &Crossing<X>) -> Ordering {
                a.at_left.cmp(&b.at_left).then_with(|| {
                    let ord = a.line.partial_cmp(&b.line).unwrap();
                    if a.at_left {
                        ord
                    } else {
                        ord.reverse()
                    }
                })
            }
            iter.intersections_mut().sort_unstable_by(compare_crossings);

            // Process all end-segments.
            let mut idx = 0;
            let mut next_region = None;
            while idx < iter.intersections().len() {
                let c = &iter.intersections()[idx];
                // If we hit a start-segment, we are done.
                if c.at_left {
                    break;
                }
                let cross = c.cross;
                if next_region.is_none() {
                    next_region = Some(cross.get_region(c.line));
                    trace!(
                        "get_region: {geom:?}: {next_region:?}",
                        next_region = next_region.unwrap(),
                        geom = c.line,
                    );
                }
                next_region.as_mut().unwrap().cross(cross.is_first);
                let has_overlap = (idx + 1) < iter.intersections().len()
                    && compare_crossings(c, &iter.intersections()[idx + 1]) == Ordering::Equal;
                if !has_overlap {
                    let prev_region = cross.get_region(c.line);
                    trace!(
                        "check_add: {geom:?}: {prev_region:?} -> {next_region:?}",
                        geom = c.line,
                        next_region = next_region.unwrap()
                    );
                    let next_is_ty = next_region.unwrap().is_ty(self.ty);
                    if prev_region.is_ty(self.ty) ^ next_is_ty {
                        trace!("\tfull_geom: {geom:?}", geom = c.cross.geom);
                        rings.add_edge(
                            c.line,
                            if !next_is_ty {
                                WindingOrder::CounterClockwise
                            } else {
                                WindingOrder::Clockwise
                            },
                        )
                    }
                    next_region = None;
                }
                idx += 1;
            }

            if idx >= iter.intersections_mut().len() {
                continue;
            }
            let botmost_start_segment = iter.intersections_mut()[idx].clone();
            debug_assert!(botmost_start_segment.at_left);

            trace!(
                "Bottom most start-edge: {botmost:?}",
                botmost = botmost_start_segment.line,
            );

            let prev = iter.prev_active(&botmost_start_segment);
            trace!(
                "prev-active(bot-most): {prev:?}",
                prev = prev.map(|(_, p)| p.geom)
            );

            let mut region = prev
                .as_ref()
                .map(|(g, c)| c.get_region(*g))
                .unwrap_or_else(|| Region::infinity(self.ty));
            trace!("bot region: {region:?}");

            while idx < iter.intersections().len() {
                let mut c = &iter.intersections()[idx];
                let mut jdx = idx;
                loop {
                    region.cross(c.cross.is_first);
                    let has_overlap = (idx + 1) < iter.intersections().len()
                        && compare_crossings(c, &iter.intersections()[idx + 1]) == Ordering::Equal;
                    if !has_overlap {
                        break;
                    }
                    idx += 1;
                    c = &iter.intersections()[idx];
                }
                trace!(
                    "set_region: {geom:?} / {geom2:?} => {region:?} ({c} counts)",
                    geom2 = c.cross.geom,
                    geom = iter.intersections_mut()[jdx].line,
                    c = idx - jdx + 1,
                );
                while jdx <= idx {
                    let gpiece = iter.intersections()[jdx].line;
                    iter.intersections()[jdx].cross.set_region(region, gpiece);
                    jdx += 1;
                }
                idx += 1;
            }
        }

        rings.finish()
    }
}

#[derive(Clone, Copy)]
struct Region {
    is_first: bool,
    is_second: bool,
}
impl Debug for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{f}{s}]",
            f = if self.is_first { "A" } else { "" },
            s = if self.is_second { "B" } else { "" },
        )
    }
}

impl Region {
    fn infinity(ty: OpType) -> Self {
        Region {
            is_first: false,
            is_second: matches!(ty, OpType::Difference),
        }
    }
    fn cross(&mut self, first: bool) {
        if first {
            self.is_first = !self.is_first;
        } else {
            self.is_second = !self.is_second;
        }
    }
    fn is_ty(&self, ty: OpType) -> bool {
        match ty {
            OpType::Intersection | OpType::Difference => self.is_first && self.is_second,
            OpType::Union => self.is_first || self.is_second,
            OpType::Xor => self.is_first ^ self.is_second,
        }
    }
}

#[derive(Clone)]
struct Edge<T: Float> {
    geom: LineOrPoint<T>,
    is_first: bool,
    _region: Cell<Region>,
    _region_2: Cell<Region>,
}

impl<T: Float> Edge<T> {
    fn get_region(&self, piece: LineOrPoint<T>) -> Region {
        // Note: This is related to the ordering of intersection
        // with respect to the complete geometry. Due to
        // finite-precision errors, intersection points might lie
        // outside the end-points in lexicographic ordering.
        //
        // Thus, while processing, the segment, we might be looking at it from
        // end-to-start as opposed to the typical start-to-end (with respect to
        // the complete geom. the segment is a part of).
        //
        // In this case, the region set/get procedure queries different sides of
        // the segment. Thus, we detect this and store both sides of the region.
        // Finally, note that we need to store both sides of the segment, as
        // this cannot be computed from the current edge alone (it may depend on
        // more overlapping edges).
        if piece.left() < self.geom.right() {
            self._region.get()
        } else {
            debug!("getting region_2");
            self._region_2.get()
        }
    }
    fn set_region(&self, region: Region, piece: LineOrPoint<T>) {
        if piece.left() < self.geom.right() {
            self._region.set(region);
        } else {
            debug!("setting region_2");
            self._region_2.set(region);
        }
    }
}

impl<T: Float> std::fmt::Debug for Edge<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = self.geom.line();
        f.debug_struct("Edge")
            .field(
                "geom",
                &format!(
                    "({:?},{:?}) <-> ({:?},{:?})",
                    line.start.x, line.start.y, line.end.x, line.end.y
                ),
            )
            .field("is_first", &self.is_first)
            .field("region", &self._region)
            .finish()
    }
}

impl<T: Float> Cross for Edge<T> {
    type Scalar = T;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        self.geom
    }
}
