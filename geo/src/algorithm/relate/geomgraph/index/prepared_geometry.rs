use super::Segment;
use crate::geometry::Polygon;
use crate::relate::geomgraph::index::PreparedRStarEdgeSetIntersector;
use crate::relate::geomgraph::GeometryGraph;
use crate::GeoFloat;
use crate::GeometryCow;
use rstar::{RTree, RTreeNum};

// TODO: other types
impl<'a, F: GeoFloat> From<&'a Polygon<F>> for PreparedGeometry<'a, F> {
    fn from(polygon: &'a Polygon<F>) -> Self {
        use std::cell::RefCell;
        let geometry = GeometryCow::from(polygon);
        let mut geometry_graph = GeometryGraph::new(0, geometry);
        let segments: Vec<Segment<F>> = geometry_graph
            .edges()
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
        geometry_graph.set_tree(tree);
        Self { geometry_graph }
    }
}

pub struct PreparedGeometry<'a, F: GeoFloat + RTreeNum = f64> {
    geometry_graph: GeometryGraph<'a, F>,
}

impl<'a, F> PreparedGeometry<'a, F>
where
    F: GeoFloat + RTreeNum,
{
    pub(crate) fn geometry_graph(&'a self, arg_index: usize) -> GeometryGraph<'a, F> {
        self.geometry_graph.clone_for_arg_index(arg_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::Relate;
    use crate::polygon;

    #[test]
    fn relate() {
        let p1 = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let p2 = polygon![(x: 0.5, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let prepared_1 = PreparedGeometry::from(&p1);
        let prepared_2 = PreparedGeometry::from(&p2);
        assert!(prepared_1.relate(&prepared_2).is_contains());
        assert!(prepared_2.relate(&prepared_1).is_within());
    }

    #[test]
    fn swap_arg_index() {
        let poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let prepared_geom = PreparedGeometry::from(&poly);

        let poly_cow = GeometryCow::from(&poly);

        let cached_graph = prepared_geom.geometry_graph(0);
        let fresh_graph = GeometryGraph::new(0, poly_cow.clone());
        cached_graph.assert_eq_graph(&fresh_graph);

        let cached_graph = prepared_geom.geometry_graph(1);
        let fresh_graph = GeometryGraph::new(1, poly_cow);
        cached_graph.assert_eq_graph(&fresh_graph);
    }
}
