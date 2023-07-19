use super::Segment;
use crate::geometry::*;
use crate::relate::geomgraph::index::PreparedRStarEdgeSetIntersector;
use crate::relate::geomgraph::{GeometryGraph, RobustLineIntersector};
use crate::GeoFloat;
use crate::GeometryCow;

use std::cell::RefCell;
use std::rc::Rc;

use rstar::{RTree, RTreeNum};

impl<'a, F: GeoFloat> From<Point<F>> for PreparedGeometry<'a, F> {
    fn from(point: Point<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(point))
    }
}
impl<'a, F: GeoFloat> From<Line<F>> for PreparedGeometry<'a, F> {
    fn from(line: Line<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(line))
    }
}
impl<'a, F: GeoFloat> From<LineString<F>> for PreparedGeometry<'a, F> {
    fn from(line_string: LineString<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(line_string))
    }
}
impl<'a, F: GeoFloat> From<Polygon<F>> for PreparedGeometry<'a, F> {
    fn from(polygon: Polygon<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(polygon))
    }
}
impl<'a, F: GeoFloat> From<MultiPoint<F>> for PreparedGeometry<'a, F> {
    fn from(multi_point: MultiPoint<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(multi_point))
    }
}
impl<'a, F: GeoFloat> From<MultiLineString<F>> for PreparedGeometry<'a, F> {
    fn from(multi_line_string: MultiLineString<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(multi_line_string))
    }
}
impl<'a, F: GeoFloat> From<MultiPolygon<F>> for PreparedGeometry<'a, F> {
    fn from(multi_polygon: MultiPolygon<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(multi_polygon))
    }
}
impl<'a, F: GeoFloat> From<Rect<F>> for PreparedGeometry<'a, F> {
    fn from(rect: Rect<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(rect))
    }
}
impl<'a, F: GeoFloat> From<Triangle<F>> for PreparedGeometry<'a, F> {
    fn from(triangle: Triangle<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(triangle))
    }
}
impl<'a, F: GeoFloat> From<GeometryCollection<F>> for PreparedGeometry<'a, F> {
    fn from(geometry_collection: GeometryCollection<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(geometry_collection))
    }
}
impl<'a, F: GeoFloat> From<Geometry<F>> for PreparedGeometry<'a, F> {
    fn from(geometry: Geometry<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(geometry))
    }
}

impl<'a, F: GeoFloat> From<&'a Point<F>> for PreparedGeometry<'a, F> {
    fn from(point: &'a Point<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(point))
    }
}
impl<'a, F: GeoFloat> From<&'a Line<F>> for PreparedGeometry<'a, F> {
    fn from(line: &'a Line<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(line))
    }
}
impl<'a, F: GeoFloat> From<&'a LineString<F>> for PreparedGeometry<'a, F> {
    fn from(line_string: &'a LineString<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(line_string))
    }
}
impl<'a, F: GeoFloat> From<&'a Polygon<F>> for PreparedGeometry<'a, F> {
    fn from(polygon: &'a Polygon<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(polygon))
    }
}
impl<'a, F: GeoFloat> From<&'a MultiPoint<F>> for PreparedGeometry<'a, F> {
    fn from(multi_point: &'a MultiPoint<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(multi_point))
    }
}
impl<'a, F: GeoFloat> From<&'a MultiLineString<F>> for PreparedGeometry<'a, F> {
    fn from(multi_line_string: &'a MultiLineString<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(multi_line_string))
    }
}
impl<'a, F: GeoFloat> From<&'a MultiPolygon<F>> for PreparedGeometry<'a, F> {
    fn from(multi_polygon: &'a MultiPolygon<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(multi_polygon))
    }
}
impl<'a, F: GeoFloat> From<&'a GeometryCollection<F>> for PreparedGeometry<'a, F> {
    fn from(geometry_collection: &'a GeometryCollection<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(geometry_collection))
    }
}
impl<'a, F: GeoFloat> From<&'a Rect<F>> for PreparedGeometry<'a, F> {
    fn from(rect: &'a Rect<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(rect))
    }
}
impl<'a, F: GeoFloat> From<&'a Triangle<F>> for PreparedGeometry<'a, F> {
    fn from(triangle: &'a Triangle<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(triangle))
    }
}
impl<'a, F: GeoFloat> From<&'a Geometry<F>> for PreparedGeometry<'a, F> {
    fn from(geometry: &'a Geometry<F>) -> Self {
        PreparedGeometry::from(GeometryCow::from(geometry))
    }
}

impl<'a, F: GeoFloat> From<GeometryCow<'a, F>> for PreparedGeometry<'a, F> {
    fn from(geometry: GeometryCow<'a, F>) -> Self {
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
        geometry_graph.set_tree(Rc::new(tree));

        // TODO: don't pass in line intersector here - in theory we'll want pluggable line intersectors
        // and the type (Robust) shouldn't be hard coded here.
        geometry_graph.compute_self_nodes(Box::new(RobustLineIntersector::new()));

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
    pub(crate) fn geometry(&self) -> &GeometryCow<F> {
        self.geometry_graph.geometry()
    }

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
