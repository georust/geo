use std::{cell::Cell, cmp::Ordering, iter::FromIterator};

use crate::GeoFloat as Float;
use crate::{GeoFloat, Line, Polygon};
use log::trace;

use crate::sweep::{Cross, Crossing, CrossingsIter, LineOrPoint};

use super::Ring;

pub fn assemble<T: Float>(rings: Vec<Ring<T>>) -> Vec<Polygon<T>> {
    let mut parents = vec![0; rings.len()];
    let edges: Vec<Edge<_>> = rings
        .iter()
        .enumerate()
        .flat_map(|(idx, ring)| {
            debug_assert!(ring.coords().is_closed());
            ring.coords().lines().map(move |l| (idx, l))
        })
        .map(|(ring_idx, line)| Edge {
            geom: line.into(),
            ring_idx,
            region: None.into(),
        })
        .collect();

    let mut sweep = CrossingsIter::from_iter(edges.iter());
    while let Some(pt) = sweep.next() {
        fn compare_crossings<X: Cross>(a: &Crossing<X>, b: &Crossing<X>) -> Ordering {
            a.at_left.cmp(&b.at_left).reverse().then_with(|| {
                if !a.at_left {
                    Ordering::Equal
                } else {
                    LineOrPoint::from(a.line)
                        .partial_cmp(&b.line.into())
                        .unwrap()
                }
            })
        }
        sweep.intersections_mut().sort_by(compare_crossings);

        let mut below = {
            let first = sweep.intersections().first().unwrap();
            if !first.at_left {
                continue;
            }
            let active = sweep.prev_active(first);
            trace!(
                "active of {first_geom:?}: {active:?}",
                first_geom = first.cross.geom
            );
            active.map(|(_, b)| b.region.get()).unwrap_or(None)
        };
        trace!("pt: {pt:?}\n\tbelow: {below:?}");

        sweep.intersections().iter().position(|edge| {
            if !edge.at_left {
                return true;
            }
            debug_assert!(!edge.has_overlap, "laminar are non-overlapping");
            let ring_idx = edge.cross.ring_idx;
            let ring = &rings[ring_idx];
            below = match below {
                Some(ext_idx) => {
                    if ring.is_hole() {
                        parents[ring_idx] = ext_idx;
                    } else {
                        debug_assert_eq!(ring_idx, ext_idx, "matching idx");
                    }
                    None
                }
                None => {
                    if ring.is_hole() {
                        Some(parents[ring_idx])
                    } else {
                        Some(ring_idx)
                    }
                }
            };
            trace!("setting: {geom:?} -> {below:?}", geom = edge.cross.geom);
            edge.cross.region.set(below);

            false
        });
    }

    let mut polygons = vec![None; rings.len()];
    rings.iter().enumerate().for_each(|(idx, r)| {
        if r.is_hole() {
            let p_idx = parents[idx];
            if polygons[p_idx].is_none() {
                polygons[p_idx] = Some(Polygon::new(rings[p_idx].coords().clone(), vec![]));
            }
            polygons[p_idx]
                .as_mut()
                .unwrap()
                .interiors_push(r.coords().clone());
        } else {
            if polygons[idx].is_none() {
                polygons[idx] = Some(Polygon::new(rings[idx].coords().clone(), vec![]));
            }
        }
    });

    polygons.into_iter().filter_map(|p| p).collect()
}

#[derive(Debug, Clone)]
struct Edge<T: GeoFloat> {
    geom: LineOrPoint<T>,
    ring_idx: usize,
    region: Cell<Option<usize>>,
}

impl<T: GeoFloat> Cross for Edge<T> {
    type Scalar = T;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        self.geom
    }
}
