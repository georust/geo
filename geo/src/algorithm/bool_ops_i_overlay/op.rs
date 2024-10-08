use std::{cell::Cell, cmp::Ordering, fmt::Debug};

use super::{MultiPolygon, Spec};
use crate::{
    sweep::{compare_crossings, Cross, Crossing, CrossingsIter, LineOrPoint, SweepPoint},
    CoordsIter, GeoFloat as Float, LineString, Polygon,
};

/// The state for computing the output shape of a collection of shapes (with
/// possibly intersecting geometry) by applying `S`.
#[derive(Debug, Clone)]
pub struct Proc<T: Float, S: Spec<T>> {
    spec: S,
    edges: Vec<Edge<T, S>>,
}

impl<T: Float, S: Spec<T>> Proc<T, S> {
    pub fn new(spec: S, capacity: usize) -> Self {
        Proc {
            spec,
            edges: Vec::with_capacity(capacity),
        }
    }

    /// Adds `mp` to the procedure, with the shape index as `idx`. `idx` should
    /// be either 0 or 1 depending on whether it is the first or second input.
    pub(crate) fn add_multi_polygon(&mut self, mp: &MultiPolygon<T>, idx: usize) {
        mp.0.iter().for_each(|p| self.add_polygon(p, idx));
    }

    /// Adds `poly` to the procedure, with the shape index as `idx`. `idx`
    /// should be either 0 or 1 depending on whether it is the first or second
    /// input.
    pub(crate) fn add_polygon(&mut self, poly: &Polygon<T>, idx: usize) {
        self.add_closed_ring(poly.exterior(), idx, false);
        for hole in poly.interiors() {
            self.add_closed_ring(hole, idx, true);
        }
    }

    /// Adds `ls` to the procedure, with the shape index as `idx`. `idx` should
    /// be either 0 or 1 depending on whether it is the first or second input.
    pub(crate) fn add_line_string(&mut self, ls: &LineString<T>, idx: usize) {
        for line in ls.lines() {
            let lp: LineOrPoint<_> = line.into();
            if !lp.is_line() {
                continue;
            }

            debug!("processing: {lp:?}");
            let region = self.spec.infinity();
            self.edges.push(Edge {
                geom: lp,
                idx,
                _region: region.into(),
                _region_2: region.into(),
            });
        }
    }

    /// Adds `ring` to the procedure, with the shape index as `idx`, and
    /// `_is_hole` if this ring belongs to a hole in the original polygon. `idx`
    /// should be either 0 or 1 depending on whether it is the first or
    /// second input. `_is_hole` is not used right now; remove it once we fully
    /// handle floating-point issues.
    fn add_closed_ring(&mut self, ring: &LineString<T>, idx: usize, _is_hole: bool) {
        assert!(ring.is_closed());
        if ring.coords_count() <= 3 {
            return;
        }

        self.add_line_string(ring, idx);
    }

    /// Sweeps across the edges, splits them, then passes the resulting regions
    /// and edges to the spec, to compute the result shape.
    pub fn sweep(mut self) -> S::Output {
        let mut iter = CrossingsIter::from_iter(self.edges.iter());

        // Iterate through the intersection points (including end points).
        while let Some(pt) = iter.next() {
            debug!(
                "\n\nSweep point: {pt:?}, {n} intersection segments",
                n = iter.intersections_mut().len(),
                pt = SweepPoint::from(pt),
            );
            // Sort crossings at `pt`. This means the end of segments will be ordered before
            // the start of any segments, and start/end segments will be ordered from bottom
            // to top.
            iter.intersections_mut().sort_unstable_by(compare_crossings);

            // Trace the crossings for debugging.
            for (idx, it) in iter.intersections().iter().enumerate() {
                let it: &Crossing<_> = it;
                trace!("{idx}: {geom:?} of {cr:?}", geom = it.line, cr = it.cross);
            }

            // Process all end-segments.
            let mut idx = 0;
            let mut next_region = None;
            trace!("end segments:");
            while idx < iter.intersections().len() {
                let c = &iter.intersections()[idx];
                // If we hit a start-segment, we are done with all the end segments.
                if c.at_left {
                    break;
                }
                trace!("{idx}: {geom:?}", geom = c.line);
                let cross = c.cross;
                if next_region.is_none() {
                    // This is the first segment in a group of overlapping segments (including when
                    // the group only includes this one segment).
                    next_region = Some(cross.get_region(c.line));
                    trace!(
                        "get_region: {geom:?}: {next_region:?}",
                        next_region = next_region.unwrap(),
                        geom = c.line,
                    );
                }
                // Update the region with the new edge.
                next_region = Some(self.spec.cross(next_region.unwrap(), cross.idx));
                trace!("next_region: {reg:?}", reg = next_region.unwrap());
                // We only want to output one edge per group of overlapping segments (since the
                // output shape should not have overlapping segments).
                let has_overlap = (idx + 1) < iter.intersections().len()
                    && c.line.partial_cmp(&iter.intersections()[idx + 1].line)
                        == Some(Ordering::Equal);
                if !has_overlap {
                    let prev_region = cross.get_region(c.line);
                    debug!(
                        "check_add: {geom:?}: {prev_region:?} -> {next_region:?}",
                        geom = c.line,
                        next_region = next_region.unwrap()
                    );
                    // Output the segment once we've reached the end of the segment. We must wait
                    // until the end of the segment (here), since overlapping segments can affect if
                    // an edge is in the output or not.
                    self.spec
                        .output([prev_region, next_region.unwrap()], c.line, c.cross.idx);
                    next_region = None;
                }
                idx += 1;
            }

            // We've processed the last crossing. Just skip as an optimization.
            if idx >= iter.intersections_mut().len() {
                continue;
            }
            let botmost_start_segment = iter.intersections_mut()[idx].clone();
            debug_assert!(botmost_start_segment.at_left);

            trace!(
                "Bottom most start-edge: {botmost:?} of {cr:?}",
                botmost = botmost_start_segment.line,
                cr = botmost_start_segment.cross,
            );

            // Get the region of the previous edge in the output, or use the "infinity
            // point".
            let prev = iter.prev_active(&botmost_start_segment);
            trace!(
                "prev-active(bot-most): {prev:?}",
                prev = prev.map(|(_, p)| p.geom)
            );
            let mut region = prev
                .as_ref()
                .map(|(g, c)| c.get_region(*g))
                .unwrap_or_else(|| self.spec.infinity());
            trace!("bot region: {region:?}");

            // Process all start-segments.
            while idx < iter.intersections().len() {
                let mut c = &iter.intersections()[idx];
                let mut jdx = idx;
                // Loop over all the segments in the intersection that are overlapping with `c`.
                loop {
                    // Update the region with the new edge.
                    region = self.spec.cross(region, c.cross.idx);
                    let has_overlap = (idx + 1) < iter.intersections().len()
                        && c.line.partial_cmp(&iter.intersections()[idx + 1].line)
                            == Some(Ordering::Equal);
                    if !has_overlap {
                        // The next segment is not intersecting, so leave that to the next iteration
                        // of the outer loop.
                        break;
                    }
                    // The next edge is overlapping, so keep looping.
                    idx += 1;
                    c = &iter.intersections()[idx];
                }
                // We have "skipped over" all the overlapping segments of `c`.
                trace!(
                    "set_region: {geom:?} / {geom2:?} => {region:?} ({c} counts)",
                    geom2 = c.cross.geom,
                    geom = iter.intersections_mut()[jdx].line,
                    c = idx - jdx + 1,
                );
                // Set the region of all overlapping segments of `c` to `region`, since the
                // "result" of all of these segments is `region`.
                while jdx <= idx {
                    let gpiece = iter.intersections()[jdx].line;
                    iter.intersections()[jdx].cross.set_region(region, gpiece);
                    jdx += 1;
                }
                idx += 1;
            }
        }
        self.spec.finish()
    }
}

/// An edge of a shape.
#[derive(Clone)]
struct Edge<T: Float, S: Spec<T>> {
    /// The geometry of the edge.
    geom: LineOrPoint<T>,
    /// The index of the shape this edge belongs to.
    idx: usize,
    /// The region of this edge when the piece is going in the same direction as
    /// `geom`.
    _region: Cell<S::Region>,
    _region_2: Cell<S::Region>,
}

impl<T: Float, S: Spec<T>> Edge<T, S> {
    /// Gets the region for this edge based on the provided `piece`. `piece`
    /// must be some part (i.e., subspan) of `self.geom`.
    fn get_region(&self, piece: LineOrPoint<T>) -> S::Region {
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
    /// Sets the region for this edge to `region` based onm the provided
    /// `piece`. `piece` must be some part (i.e., subspan) of `self.geom`.
    fn set_region(&self, region: S::Region, piece: LineOrPoint<T>) {
        if piece.left() < self.geom.right() {
            self._region.set(region);
        } else {
            debug!("setting region_2");
            self._region_2.set(region);
        }
    }
}

impl<T: Float, S: Spec<T>> std::fmt::Debug for Edge<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = self.geom.line();
        f.debug_struct("Edge")
            .field(
                "geom",
                &format!(
                    "({:?}, {:?}) <-> ({:?}, {:?})",
                    line.start.x, line.start.y, line.end.x, line.end.y
                ),
            )
            .field("idx", &self.idx)
            .field("region", &self._region)
            .finish()
    }
}

impl<T: Float, S: Spec<T>> Cross for Edge<T, S> {
    type Scalar = T;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        self.geom
    }
}
