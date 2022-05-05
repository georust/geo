use super::super::Edge;
use super::{EdgeSetIntersector, SegmentIntersector};
use crate::GeoFloat;

use std::cell::RefCell;
use std::rc::Rc;

use rstar::RTree;

pub(crate) struct RstarEdgeSetIntersector;

impl RstarEdgeSetIntersector {
    pub fn new() -> Self {
        RstarEdgeSetIntersector
    }
}

struct Segment<'a, F: GeoFloat + rstar::RTreeNum> {
    i: usize,
    edge: &'a RefCell<Edge<F>>,
    envelope: rstar::AABB<crate::Coordinate<F>>,
}

impl<'a, F> Segment<'a, F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    fn new(i: usize, edge: &'a RefCell<Edge<F>>) -> Self {
        use crate::rstar::RTreeObject;
        let p1 = edge.borrow().coords()[i];
        let p2 = edge.borrow().coords()[i + 1];
        Self {
            i,
            edge,
            envelope: rstar::AABB::from_corners(p1, p2),
        }
    }
}

impl<'a, F> rstar::RTreeObject for Segment<'a, F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<crate::Coordinate<F>>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl<F> EdgeSetIntersector<F> for RstarEdgeSetIntersector
where
    F: GeoFloat + rstar::RTreeNum,
{
    fn compute_intersections_within_set(
        &mut self,
        edges: &[Rc<RefCell<Edge<F>>>],
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let segments: Vec<Segment<F>> = edges
            .iter()
            .flat_map(|edge| {
                let start_of_final_segment: usize = RefCell::borrow(edge).coords().len() - 1;
                (0..start_of_final_segment).map(|segment_i| Segment::new(segment_i, edge))
            })
            .collect();
        let tree = RTree::bulk_load(segments);

        for (edge0, edge1) in tree.intersection_candidates_with_other_tree(&tree) {
            if check_for_self_intersecting_edges || edge0.edge.as_ptr() != edge1.edge.as_ptr() {
                segment_intersector.add_intersections(edge0.edge, edge0.i, edge1.edge, edge1.i);
            }
        }
    }

    fn compute_intersections_between_sets(
        &mut self,
        edges0: &[Rc<RefCell<Edge<F>>>],
        edges1: &[Rc<RefCell<Edge<F>>>],
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let segments0: Vec<Segment<F>> = edges0
            .iter()
            .flat_map(|edge| {
                let start_of_final_segment: usize = RefCell::borrow(edge).coords().len() - 1;
                (0..start_of_final_segment).map(|segment_i| Segment::new(segment_i, edge))
            })
            .collect();
        let tree_0 = RTree::bulk_load(segments0);

        let segments1: Vec<Segment<F>> = edges1
            .iter()
            .flat_map(|edge| {
                let start_of_final_segment: usize = RefCell::borrow(edge).coords().len() - 1;
                (0..start_of_final_segment).map(|segment_i| Segment::new(segment_i, edge))
            })
            .collect();
        let tree_1 = RTree::bulk_load(segments1);

        for (edge0, edge1) in tree_0.intersection_candidates_with_other_tree(&tree_1) {
            segment_intersector.add_intersections(edge0.edge, edge0.i, edge1.edge, edge1.i);
        }
    }
}
