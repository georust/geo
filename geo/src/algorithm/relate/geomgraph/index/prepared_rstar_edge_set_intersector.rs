use super::super::{Edge, PlanarGraph};
use super::{EdgeSetIntersector, SegmentIntersector};
use crate::{Coordinate, GeoFloat};

use std::cell::RefCell;
use std::rc::Rc;

use rstar::RTree;

pub(crate) struct PreparedRStarEdgeSetIntersector<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    tree: RTree<Segment<F>>,
}

impl<F: GeoFloat> PreparedRStarEdgeSetIntersector<F> {
    pub fn new(graph: &PlanarGraph<F>) -> Self {
        let edges: &[Rc<RefCell<Edge<F>>>] = graph.edges();
        let segments: Vec<Segment<F>> = edges
            .iter()
            .enumerate()
            .flat_map(|(edge_idx, edge)| {
                let edge = RefCell::borrow(edge);
                let start_of_final_segment: usize = edge.coords().len() - 1;
                (0..start_of_final_segment).map(move |segment_idx| {
                    let p1 = edge.coords()[segment_idx];
                    let p2 = edge.coords()[segment_idx + 1];
                    Segment::new(edge_idx, segment_idx, p1, p2)
                })
            })
            .collect();

        let tree = RTree::bulk_load(segments);

        PreparedRStarEdgeSetIntersector { tree }
    }
}

struct Segment<F: GeoFloat + rstar::RTreeNum> {
    edge_idx: usize,
    segment_idx: usize,
    envelope: rstar::AABB<Coordinate<F>>,
}

impl<F> Segment<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    fn new(edge_idx: usize, segment_idx: usize, p1: Coordinate<F>, p2: Coordinate<F>) -> Self {
        use crate::rstar::RTreeObject;
        Self {
            edge_idx,
            segment_idx,
            envelope: rstar::AABB::from_corners(p1, p2),
        }
    }
}

impl<'a, F> rstar::RTreeObject for Segment<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Coordinate<F>>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl<F> EdgeSetIntersector<F> for PreparedRStarEdgeSetIntersector<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    fn compute_intersections_within_set(
        &self,
        edges: &[Rc<RefCell<Edge<F>>>],
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let segments: Vec<Segment<F>> = edges
            .iter()
            .enumerate()
            .flat_map(|(edge_idx, edge)| {
                let edge = RefCell::borrow(edge);
                let start_of_final_segment: usize = edge.coords().len() - 1;
                (0..start_of_final_segment).map(move |segment_idx| {
                    let p1 = edge.coords()[segment_idx];
                    let p2 = edge.coords()[segment_idx + 1];
                    Segment::new(edge_idx, segment_idx, p1, p2)
                })
            })
            .collect();

        let tree = RTree::bulk_load(segments);

        for (segment_0, segment_1) in tree.intersection_candidates_with_other_tree(&tree) {
            if check_for_self_intersecting_edges || segment_0.edge_idx != segment_1.edge_idx {
                let edge_0 = &edges[segment_0.edge_idx];
                let edge_1 = &edges[segment_1.edge_idx];
                segment_intersector.add_intersections(
                    edge_0,
                    segment_0.segment_idx,
                    edge_1,
                    segment_1.segment_idx,
                );
            }
        }
    }

    fn compute_intersections_between_sets(
        &self,
        edges_0: &[Rc<RefCell<Edge<F>>>],
        edges_1: &[Rc<RefCell<Edge<F>>>],
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let segments0: Vec<Segment<F>> = edges_0
            .iter()
            .enumerate()
            .flat_map(|(edge_idx, edge)| {
                let edge = RefCell::borrow(edge);
                let start_of_final_segment: usize = edge.coords().len() - 1;
                (0..start_of_final_segment).map(move |segment_idx| {
                    let p1 = edge.coords()[segment_idx];
                    let p2 = edge.coords()[segment_idx + 1];
                    Segment::new(edge_idx, segment_idx, p1, p2)
                })
            })
            .collect();
        let tree_0 = RTree::bulk_load(segments0);

        let segments1: Vec<Segment<F>> = edges_1
            .iter()
            .enumerate()
            .flat_map(|(edge_idx, edge)| {
                let edge = RefCell::borrow(edge);
                let start_of_final_segment: usize = edge.coords().len() - 1;
                (0..start_of_final_segment).map(move |segment_idx| {
                    let p1 = edge.coords()[segment_idx];
                    let p2 = edge.coords()[segment_idx + 1];
                    Segment::new(edge_idx, segment_idx, p1, p2)
                })
            })
            .collect();
        let tree_1 = RTree::bulk_load(segments1);

        for (segment_0, segment_1) in tree_0.intersection_candidates_with_other_tree(&tree_1) {
            let edge_0 = &edges_0[segment_0.edge_idx];
            let edge_1 = &edges_1[segment_1.edge_idx];
            segment_intersector.add_intersections(
                edge_0,
                segment_0.segment_idx,
                edge_1,
                segment_1.segment_idx,
            );
        }
    }
}

// impl<F> EdgeSetIntersector<F> for PreparedRStarEdgeSetIntersector<F>
//     where
//         F: GeoFloat + rstar::RTreeNum,
// {
//     fn compute_intersections_within_set(&mut self, edges: &[Rc<RefCell<Edge<F>>>], check_for_self_intersecting_edges: bool, segment_intersector: &mut SegmentIntersector<F>) {
//         for (segment_0, segment_1) in self.tree.intersection_candidates_with_other_tree(&self.tree) {
//             if check_for_self_intersecting_edges || segment_0.edge_idx != segment_1.edge_idx {
//                 let edge_0 = &edges[segment_0.edge_idx];
//                 let edge_1 = &edges[segment_1.edge_idx];
//                 segment_intersector.add_intersections(
//                     edge_0,
//                     segment_0.segment_idx,
//                     edge_1,
//                     segment_1.segment_idx,
//                 );
//             }
//         }
//     }
//
//     fn compute_intersections_between_sets(&mut self, edges0: &[Rc<RefCell<Edge<F>>>], edges1: &[Rc<RefCell<Edge<F>>>], segment_intersector: &mut SegmentIntersector<F>) {
//         todo!()
//     }
// }
