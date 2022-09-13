use std::{
    cell::Cell,
    collections::{BTreeMap, HashMap, VecDeque},
};

use crate::{
    sweep::{compare_crossings, Cross, CrossingsIter, LineOrPoint, SweepPoint},
    utils::EitherIter,
    winding_order::WindingOrder,
    GeoFloat,
};
use geo_types::{LineString, MultiPolygon, Polygon};

/// Assemble polygons from boundary segments of the output region.
///
/// Implements the construction of the final geometry from boundary
/// line-segments obtained via the sweep. The line-segments are guaranteed to
/// describe a bounded region, do not intersect in their interior, and are not
/// degenerate (not a point).
#[derive(Debug)]
pub struct RegionAssembly<T: GeoFloat> {
    segments: Vec<Segment<T>>,
}

impl<T: GeoFloat> Default for RegionAssembly<T> {
    fn default() -> Self {
        Self {
            segments: Default::default(),
        }
    }
}

impl<T: GeoFloat> RegionAssembly<T> {
    pub fn add_edge(&mut self, edge: LineOrPoint<T>) {
        debug_assert!(edge.is_line());
        trace!("add_edge: {edge:?}");
        self.segments.push(edge.into());
    }
    pub fn finish(self) -> MultiPolygon<T> {
        let mut iter = CrossingsIter::new_simple(self.segments.iter());
        let mut snakes = vec![];

        while let Some(pt) = iter.next() {
            let num_segments = iter.intersections().len();
            debug_assert!(num_segments % 2 == 0, "assembly segments must be eulierian");
            iter.intersections_mut().sort_unstable_by(compare_crossings);

            let first = &iter.intersections()[0];
            let (prev_region, mut parent_snake_idx) = if first.at_left {
                // No segment ends here.
                // We should read prev_region via `prev_active`
                iter.prev_active(first)
                    .map(|(_, seg)| (seg.region.get(), Some(seg.snake_idx.get())))
                    .unwrap_or_else(|| (false, Some(0)))
            } else {
                (first.cross.region.get(), None)
            };

            // Connect consecutive segments
            let mut idx = if prev_region { 1 } else { 0 };

            while idx < iter.intersections().len() {
                let c = &iter.intersections()[idx];
                let d = if idx == num_segments - 1 {
                    &iter.intersections()[0]
                } else {
                    &iter.intersections()[idx + 1]
                };
                idx += 2;

                if c.at_left {
                    c.cross.region.set(true);
                    if parent_snake_idx.is_none() {
                        parent_snake_idx = Some(
                            iter.prev_active(c)
                                .map(|(_, seg)| seg.snake_idx.get())
                                .unwrap_or(0),
                        );
                    }
                }
                if d.at_left {
                    d.cross.region.set(false);
                    if parent_snake_idx.is_none() {
                        parent_snake_idx = Some(
                            iter.prev_active(d)
                                .map(|(_, seg)| seg.snake_idx.get())
                                .unwrap_or(0),
                        );
                    }
                }

                match (c.at_left, d.at_left) {
                    (true, true) => {
                        // Create new snakes
                        let l = snakes.len();
                        snakes.push(Snake::new(
                            pt.into(),
                            c.line.right(),
                            l + 1,
                            WindingOrder::CounterClockwise,
                            parent_snake_idx.unwrap(),
                        ));
                        c.cross.snake_idx.set(l);
                        snakes.push(Snake::new(
                            pt.into(),
                            d.line.right(),
                            l,
                            WindingOrder::Clockwise,
                            parent_snake_idx.unwrap(),
                        ));
                        d.cross.snake_idx.set(l + 1);
                    }
                    (true, false) => {
                        // Connect d -> c
                        let s_idx = d.cross.snake_idx.get();
                        snakes[s_idx].push(c.line.right());
                        c.cross.snake_idx.set(s_idx);
                    }
                    (false, true) => {
                        // Connect c -> d
                        let s_idx = c.cross.snake_idx.get();
                        snakes[s_idx].push(d.line.right());
                        d.cross.snake_idx.set(s_idx);
                    }
                    (false, false) => {
                        let c_idx = c.cross.snake_idx.get();
                        let d_idx = d.cross.snake_idx.get();
                        debug_assert_ne!(c_idx, d_idx);
                        snakes[c_idx].finish(d_idx);
                        snakes[d_idx].finish(c_idx);
                    }
                }
            }
        }

        let (rings, snakes_idx_map) = rings_from_snakes(&mut snakes[..]);

        let mut polygons = vec![];
        let mut children = HashMap::new();
        for (ring_idx, ring) in rings.iter().enumerate() {
            if ring.is_hole {
                let mut parent_ring_idx;
                let mut parent_snake_idx = ring.parent_snake_idx;
                loop {
                    parent_ring_idx = snakes_idx_map[&parent_snake_idx];
                    let parent = &rings[parent_ring_idx];
                    if !parent.is_hole {
                        break;
                    }
                    parent_snake_idx = parent.parent_snake_idx;
                }
                let this_children = children.entry(parent_ring_idx).or_insert(vec![]);
                this_children.push(ring_idx);
            } else {
                continue;
            }
        }

        for (ring_idx, ring) in rings.iter().enumerate() {
            if ring.is_hole {
                continue;
            }
            let mut holes = vec![];
            for child_idx in children.remove(&ring_idx).unwrap_or_default() {
                let ls = split_ring(&rings[child_idx].ls, |ls| holes.push(ls));
                holes.push(ls);
            }
            debug!("ext: {ext:?}", ext = ring.ls);
            let exterior = split_ring(&ring.ls, |ls| holes.push(ls));
            polygons.push(Polygon::new(exterior, holes));
        }

        polygons.into()
    }
}

#[derive(Debug)]
pub struct LineAssembly<T: GeoFloat> {
    segments: Vec<VecDeque<SweepPoint<T>>>,
    end_points: BTreeMap<(usize, SweepPoint<T>), (usize, bool)>,
}

impl<T: GeoFloat> LineAssembly<T> {
    pub fn add_edge(&mut self, geom: LineOrPoint<T>, geom_idx: usize) {
        // Try to find a line-string with either end-point
        if let Some((seg_idx, at_front)) = self.end_points.remove(&(geom_idx, geom.left())) {
            if at_front {
                self.segments[seg_idx].push_front(geom.right());
            } else {
                self.segments[seg_idx].push_back(geom.right());
            }
            self.end_points
                .insert((geom_idx, geom.right()), (seg_idx, at_front));
        } else if let Some((seg_idx, at_front)) = self.end_points.remove(&(geom_idx, geom.right()))
        {
            if at_front {
                self.segments[seg_idx].push_front(geom.left());
            } else {
                self.segments[seg_idx].push_back(geom.left());
            }
            self.end_points
                .insert((geom_idx, geom.left()), (seg_idx, at_front));
        } else {
            let idx = self.segments.len();
            self.segments
                .push(VecDeque::from_iter([geom.left(), geom.right()]));
            self.end_points.insert((geom_idx, geom.left()), (idx, true));
            self.end_points
                .insert((geom_idx, geom.right()), (idx, false));
        }
    }
    pub fn finish(self) -> Vec<LineString<T>> {
        self.segments
            .into_iter()
            .map(|pts| LineString::from_iter(pts.into_iter().map(|pt| *pt)))
            .collect()
    }
}

impl<T: GeoFloat> Default for LineAssembly<T> {
    fn default() -> Self {
        Self {
            segments: Default::default(),
            end_points: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct Ring<T: GeoFloat> {
    ls: LineString<T>,
    is_hole: bool,
    parent_snake_idx: usize,
}

fn split_ring<T: GeoFloat, F: FnMut(LineString<T>)>(
    ls: &LineString<T>,
    mut cb: F,
) -> LineString<T> {
    let mut pts_map = BTreeMap::new();
    let mut exterior = vec![];
    for coord in ls.0.iter().copied() {
        if let Some(idx) = pts_map.get(&SweepPoint::from(coord)) {
            let new_ls: LineString<_> = exterior
                .drain(idx..)
                .inspect(|pt| {
                    pts_map.remove(&SweepPoint::from(*pt)).unwrap();
                })
                .collect();
            cb(new_ls);
        }
        pts_map.insert(SweepPoint::from(coord), exterior.len());
        exterior.push(coord);
    }
    LineString::from(exterior)
}

fn rings_from_snakes<T: GeoFloat>(
    snakes: &mut [Snake<T>],
) -> (Vec<Ring<T>>, HashMap<usize, usize>) {
    let mut snake_idx_map = HashMap::new();
    let mut rings = vec![];
    for idx in 0..snakes.len() {
        if let Some(ls) = Snake::into_ring(snakes, idx, |midx| {
            snake_idx_map.insert(midx, rings.len());
        }) {
            rings.push(ls);
        }
    }
    (rings, snake_idx_map)
}

#[derive(Debug, Clone)]
struct Snake<T: GeoFloat> {
    points: Vec<SweepPoint<T>>,
    start_pair: usize,
    end_pair: Option<usize>,
    region: WindingOrder,
    parent_snake_idx: usize,
}

impl<T: GeoFloat> Snake<T> {
    pub fn new(
        start: SweepPoint<T>,
        end: SweepPoint<T>,
        pair: usize,
        region: WindingOrder,
        parent_snake_idx: usize,
    ) -> Self {
        Snake {
            points: vec![start, end],
            start_pair: pair,
            end_pair: None,
            region,
            parent_snake_idx,
        }
    }
    pub fn push(&mut self, right: SweepPoint<T>) {
        debug_assert!(self.end_pair.is_none());
        self.points.push(right)
    }
    pub fn finish(&mut self, other: usize) {
        self.end_pair = Some(other)
    }

    pub fn into_ring<F: FnMut(usize)>(
        slice: &mut [Self],
        start_idx: usize,
        mut idx_cb: F,
    ) -> Option<Ring<T>> {
        let mut output = vec![];

        let mut idx = start_idx;
        let mut at_start = true;
        let (parent_snake_idx, is_hole) = {
            let el = &slice[idx];
            if el.points.is_empty() {
                return None;
            }
            let last_el = &slice[el.start_pair];

            let start_l = LineOrPoint::new(el.points[0], el.points[1]);
            let end_l = LineOrPoint::new(el.points[0], last_el.points[1]);
            use std::cmp::Ordering;
            let ls_winding = match start_l.partial_cmp(&end_l).unwrap() {
                Ordering::Less => WindingOrder::CounterClockwise,
                Ordering::Greater => WindingOrder::Clockwise,
                _ => unreachable!(),
            };
            (el.parent_snake_idx, el.region != ls_winding)
        };
        loop {
            let el = &mut slice[idx];
            debug_assert!(!el.points.is_empty());
            idx_cb(idx);

            let iter = el.points.drain(..);
            let iter = if at_start {
                idx = el.end_pair.unwrap();
                EitherIter::A(iter)
            } else {
                idx = el.start_pair;
                EitherIter::B(iter.rev())
            }
            .skip(1);
            at_start = !at_start;
            output.extend(iter.map(|pt| *pt));
            if idx == start_idx {
                break;
            }
        }

        let ls = LineString::new(output);
        Some(Ring {
            ls,
            is_hole,
            parent_snake_idx,
        })
    }
}

#[derive(Debug, Clone)]
struct Segment<T: GeoFloat> {
    geom: LineOrPoint<T>,
    region: Cell<bool>,
    snake_idx: Cell<usize>,
}

impl<T: GeoFloat> From<LineOrPoint<T>> for Segment<T> {
    fn from(geom: LineOrPoint<T>) -> Self {
        Segment {
            geom,
            region: Cell::new(false),
            snake_idx: Cell::new(0),
        }
    }
}

impl<T: GeoFloat> Cross for Segment<T> {
    type Scalar = T;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        self.geom
    }
}
