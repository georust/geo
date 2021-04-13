use super::{EdgeEndBuilder, IntersectionMatrix};
use crate::algorithm::coordinate_position::CoordinatePosition;
use crate::algorithm::dimensions::{Dimensions, HasDimensions};
use crate::algorithm::relate::geomgraph::{
    index::SegmentIntersector,
    node_map::{NodeFactory, NodeMap},
    CoordNode, CoordPos, Edge, EdgeEnd, EdgeEndBundleStar, GeometryGraph, LabeledEdgeEndBundleStar,
    RobustLineIntersector,
};
use crate::{Coordinate, GeoFloat, GeometryCow};

use std::cell::RefCell;
use std::rc::Rc;

/// Computes an [`IntersectionMatrix`] describing the topological relationship between two
/// Geometries.
///
/// `RelateOperation` does not currently support [`GeometryCollection`]s with overlapping Polygons,
/// and may provide surprising results in that case.
///
/// This implementation relies heavily on the functionality of [`GeometryGraph`].
///
/// Based on [JTS's `RelateComputer` as of 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/operation/relate/RelateComputer.java)
pub(crate) struct RelateOperation<'a, F>
where
    F: GeoFloat,
{
    graph_a: GeometryGraph<'a, F>,
    graph_b: GeometryGraph<'a, F>,
    nodes: NodeMap<F, RelateNodeFactory>,
    line_intersector: RobustLineIntersector,
    isolated_edges: Vec<Rc<RefCell<Edge<F>>>>,
}

pub(crate) struct RelateNodeFactory;
impl<F> NodeFactory<F> for RelateNodeFactory
where
    F: GeoFloat,
{
    type Node = (CoordNode<F>, EdgeEndBundleStar<F>);
    fn create_node(coordinate: Coordinate<F>) -> Self::Node {
        (CoordNode::new(coordinate), EdgeEndBundleStar::new())
    }
}

impl<'a, F> RelateOperation<'a, F>
where
    F: GeoFloat,
{
    pub(crate) fn new(
        geom_a: &'a GeometryCow<'a, F>,
        geom_b: &'a GeometryCow<'a, F>,
    ) -> RelateOperation<'a, F> {
        Self {
            graph_a: GeometryGraph::new(0, geom_a),
            graph_b: GeometryGraph::new(1, geom_b),
            nodes: NodeMap::new(),
            isolated_edges: vec![],
            line_intersector: RobustLineIntersector::new(),
        }
    }

    pub(crate) fn compute_intersection_matrix(&mut self) -> IntersectionMatrix {
        let mut intersection_matrix = IntersectionMatrix::empty();
        // since Geometries are finite and embedded in a 2-D space,
        // the `(Outside, Outside)` element must always be 2-D
        intersection_matrix.set(
            CoordPos::Outside,
            CoordPos::Outside,
            Dimensions::TwoDimensional,
        );

        use crate::algorithm::bounding_rect::BoundingRect;
        use crate::algorithm::intersects::Intersects;
        match (
            self.graph_a.geometry().bounding_rect(),
            self.graph_b.geometry().bounding_rect(),
        ) {
            (Some(bounding_rect_a), Some(bounding_rect_b))
                if bounding_rect_a.intersects(&bounding_rect_b) => {}
            _ => {
                // since Geometries don't overlap, we can skip most of the work
                self.compute_disjoint_intersection_matrix(&mut intersection_matrix);
                return intersection_matrix;
            }
        }

        // Since changes to topology are inspected at nodes, we must crate a node for each
        // intersection.
        self.graph_a
            .compute_self_nodes(Box::new(self.line_intersector.clone()));
        self.graph_b
            .compute_self_nodes(Box::new(self.line_intersector.clone()));

        // compute intersections between edges of the two input geometries
        let segment_intersector = self
            .graph_a
            .compute_edge_intersections(&self.graph_b, Box::new(self.line_intersector.clone()));

        self.compute_intersection_nodes(0);
        self.compute_intersection_nodes(1);
        // Copy the labelling for the nodes in the parent Geometries.  These override any labels
        // determined by intersections between the geometries.
        self.copy_nodes_and_labels(0);
        self.copy_nodes_and_labels(1);
        // complete the labelling for any nodes which only have a label for a single geometry
        self.label_isolated_nodes();
        // If a proper intersection was found, we can set a lower bound on the IM.
        self.compute_proper_intersection_im(&segment_intersector, &mut intersection_matrix);
        // Now process improper intersections
        // (eg where one or other of the geometries has a vertex at the intersection point)
        // We need to compute the edge graph at all nodes to determine the IM.
        let edge_end_builder = EdgeEndBuilder::new();
        let edge_ends_a: Vec<_> = edge_end_builder.compute_ends_for_edges(self.graph_a.edges());
        self.insert_edge_ends(edge_ends_a);
        let edge_ends_b: Vec<_> = edge_end_builder.compute_ends_for_edges(self.graph_b.edges());
        self.insert_edge_ends(edge_ends_b);

        let mut nodes = NodeMap::new();
        std::mem::swap(&mut self.nodes, &mut nodes);
        let labeled_node_edges = nodes
            .into_iter()
            .map(|(node, edges)| (node, edges.into_labeled(&self.graph_a, &self.graph_b)))
            .collect();

        // Compute the labeling for "isolated" components
        //
        // Isolated components are components that do not touch any other components in the graph.
        //
        // They can be identified by the fact that their labels will have only one non-empty
        // element, the one for their parent geometry.
        //
        // We only need to check components contained in the input graphs, since, by definition,
        // isolated components will not have been replaced by new components formed by
        // intersections.
        self.label_isolated_edges(0, 1);
        self.label_isolated_edges(1, 0);

        debug!(
            "before update_intersection_matrix: {:?}",
            &intersection_matrix
        );
        self.update_intersection_matrix(labeled_node_edges, &mut intersection_matrix);

        intersection_matrix
    }

    fn insert_edge_ends(&mut self, edge_ends: Vec<EdgeEnd<F>>) {
        for edge_end in edge_ends {
            let (_node, edges) = self
                .nodes
                .insert_node_with_coordinate(*edge_end.coordinate());
            edges.insert(edge_end);
        }
    }

    fn compute_proper_intersection_im(
        &mut self,
        segment_intersector: &SegmentIntersector<F>,
        intersection_matrix: &mut IntersectionMatrix,
    ) {
        // If a proper intersection is found, we can set a lower bound on the IM.
        let dim_a = self.graph_a.geometry().dimensions();
        let dim_b = self.graph_b.geometry().dimensions();

        let has_proper = segment_intersector.has_proper_intersection();
        let has_proper_interior = segment_intersector.has_proper_interior_intersection();

        debug_assert!(
            (dim_a != Dimensions::ZeroDimensional && dim_b != Dimensions::ZeroDimensional)
                || (!has_proper && !has_proper_interior)
        );

        match (dim_a, dim_b) {
            // If edge segments of Areas properly intersect, the areas must properly overlap.
            (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                if has_proper {
                    intersection_matrix
                        .set_at_least_from_string("212101212")
                        .expect("error in hardcoded dimensions");
                }
            }

            // If a Line segment properly intersects an edge segment of an Area, it follows that
            // the Interior of the Line intersects the Boundary of the Area.
            // If the intersection is a proper *interior* intersection, then there is an
            // Interior-Interior intersection too.
            // Note that it does not follow that the Interior of the Line intersects the Exterior
            // of the Area, since there may be another Area component which contains the rest of the Line.
            (Dimensions::TwoDimensional, Dimensions::OneDimensional) => {
                if has_proper {
                    intersection_matrix
                        .set_at_least_from_string("FFF0FFFF2")
                        .expect("error in hardcoded dimensions");
                }

                if has_proper_interior {
                    intersection_matrix
                        .set_at_least_from_string("1FFFFF1FF")
                        .expect("error in hardcoded dimensions");
                }
            }

            (Dimensions::OneDimensional, Dimensions::TwoDimensional) => {
                if has_proper {
                    intersection_matrix
                        .set_at_least_from_string("F0FFFFFF2")
                        .expect("error in hardcoded dimensions");
                }

                if has_proper_interior {
                    intersection_matrix
                        .set_at_least_from_string("1F1FFFFFF")
                        .expect("error in hardcoded dimensions");
                }
            }

            // If edges of LineStrings properly intersect *in an interior point*, all we can deduce
            // is that the interiors intersect.  (We can NOT deduce that the exteriors intersect,
            // since some other segments in the geometries might cover the points in the
            // neighbourhood of the intersection.)
            // It is important that the point be known to be an interior point of both Geometries,
            // since it is possible in a self-intersecting geometry to have a proper intersection
            // on one segment that is also a boundary point of another segment.
            (Dimensions::OneDimensional, Dimensions::OneDimensional) => {
                if has_proper_interior {
                    intersection_matrix
                        .set_at_least_from_string("0FFFFFFFF")
                        .expect("error in hardcoded dimensions");
                }
            }
            _ => {}
        }
    }

    /// Copy all nodes from an arg geometry into this graph.
    ///
    /// The node label in the arg geometry overrides any previously computed label for that
    /// argIndex.  (E.g. a node may be an intersection node with a computed label of BOUNDARY, but
    /// in the original arg Geometry it is actually in the interior due to the Boundary
    /// Determination Rule)
    fn copy_nodes_and_labels(&mut self, geom_index: usize) {
        let graph = if geom_index == 0 {
            &self.graph_a
        } else {
            assert!(geom_index == 1);
            &self.graph_b
        };
        for graph_node in graph.nodes_iter() {
            let new_node = self
                .nodes
                .insert_node_with_coordinate(*graph_node.coordinate());

            let on_position = graph_node
                .label()
                .on_position(geom_index)
                .expect("node should have been labeled by now");

            new_node.0.set_label_on_position(geom_index, on_position);
        }
    }

    /// Insert nodes for all intersections on the edges of a Geometry.  
    ///
    /// Label the created nodes the same as the edge label if they do not already have a label.
    /// This allows nodes created by either self-intersections or mutual intersections to be
    /// labeled.
    ///
    /// Endpoint nodes will already be labeled from when they were inserted.
    fn compute_intersection_nodes(&mut self, geom_index: usize) {
        let graph = if geom_index == 0 {
            &self.graph_a
        } else {
            assert!(geom_index == 1);
            &self.graph_b
        };

        for edge in graph.edges() {
            let edge = edge.borrow();

            let edge_position = edge.label().on_position(geom_index);
            for edge_intersection in edge.edge_intersections() {
                let (new_node, _edges) = self
                    .nodes
                    .insert_node_with_coordinate(edge_intersection.coordinate());

                if edge_position == Some(CoordPos::OnBoundary) {
                    new_node.set_label_boundary(geom_index);
                } else if new_node.label().is_empty(geom_index) {
                    new_node.set_label_on_position(geom_index, CoordPos::Inside);
                }
            }
        }
    }

    /// If the Geometries are disjoint, we need to enter their dimension and boundary dimension in
    /// the `Outside` rows in the IM
    fn compute_disjoint_intersection_matrix(&self, intersection_matrix: &mut IntersectionMatrix) {
        {
            let geometry_a = self.graph_a.geometry();
            let dimensions = geometry_a.dimensions();
            if dimensions != Dimensions::Empty {
                intersection_matrix.set(CoordPos::Inside, CoordPos::Outside, dimensions);

                let boundary_dimensions = geometry_a.boundary_dimensions();
                if boundary_dimensions != Dimensions::Empty {
                    intersection_matrix.set(
                        CoordPos::OnBoundary,
                        CoordPos::Outside,
                        boundary_dimensions,
                    );
                }
            }
        }

        {
            let geometry_b = self.graph_b.geometry();
            let dimensions = geometry_b.dimensions();
            if dimensions != Dimensions::Empty {
                intersection_matrix.set(CoordPos::Outside, CoordPos::Inside, dimensions);

                let boundary_dimensions = geometry_b.boundary_dimensions();
                if boundary_dimensions != Dimensions::Empty {
                    intersection_matrix.set(
                        CoordPos::Outside,
                        CoordPos::OnBoundary,
                        boundary_dimensions,
                    );
                }
            }
        }
    }

    fn update_intersection_matrix(
        &self,
        labeled_node_edges: Vec<(CoordNode<F>, LabeledEdgeEndBundleStar<F>)>,
        intersection_matrix: &mut IntersectionMatrix,
    ) {
        debug!(
            "before updated_intersection_matrix(isolated_edges): {:?}",
            intersection_matrix
        );
        for isolated_edge in &self.isolated_edges {
            let edge = isolated_edge.borrow();
            Edge::<F>::update_intersection_matrix(edge.label(), intersection_matrix);
            debug!(
                "after updated_intersection_matrix(isolated_edge: {:?}, label: {:?}): {:?}",
                edge,
                edge.label(),
                intersection_matrix
            );
        }

        for (node, edges) in labeled_node_edges.iter() {
            node.update_intersection_matrix(intersection_matrix);
            edges.update_intersection_matrix(intersection_matrix);
        }
    }

    /// Processes isolated edges by computing their labelling and adding them to the isolated edges
    /// list.
    ///
    /// By definition, "isolated" edges are guaranteed not to touch the boundary of the target
    /// (since if they did, they would have caused an intersection to be computed and hence would
    /// not be isolated).
    fn label_isolated_edges(&mut self, this_index: usize, target_index: usize) {
        let (this_graph, target_graph) = if this_index == 0 {
            (&self.graph_a, &self.graph_b)
        } else {
            (&self.graph_b, &self.graph_a)
        };

        for edge in this_graph.edges() {
            let mut mut_edge = edge.borrow_mut();
            if mut_edge.is_isolated() {
                Self::label_isolated_edge(&mut mut_edge, target_index, target_graph.geometry());
                self.isolated_edges.push(edge.clone());
            }
        }
    }

    /// Label an isolated edge of a graph with its relationship to the target geometry.
    /// If the target has dim 2 or 1, the edge can either be in the interior or the exterior.
    /// If the target has dim 0, the edge must be in the exterior
    fn label_isolated_edge(edge: &mut Edge<F>, target_index: usize, target: &GeometryCow<F>) {
        if target.dimensions() > Dimensions::ZeroDimensional {
            // An isolated edge doesn't cross any boundary, so it's either wholly inside, or wholly
            // outside of the geometry. As such, we can use any point from the edge to infer the
            // position of the edge as a whole.
            let coord = edge.coords().first().expect("can't create empty edge");
            let position = target.coordinate_position(coord);
            edge.label_mut().set_all_positions(target_index, position);
        } else {
            edge.label_mut()
                .set_all_positions(target_index, CoordPos::Outside);
        }
    }

    /// Isolated nodes are nodes whose labels are incomplete (e.g. the location for one Geometry is
    /// null).  
    /// This is the case because nodes in one graph which don't intersect nodes in the other
    /// are not completely labeled by the initial process of adding nodes to the nodeList.  To
    /// complete the labelling we need to check for nodes that lie in the interior of edges, and in
    /// the interior of areas.
    fn label_isolated_nodes(&mut self) {
        let geometry_a = self.graph_a.geometry();
        let geometry_b = self.graph_b.geometry();
        for (node, _edges) in self.nodes.iter_mut() {
            let label = node.label();
            // isolated nodes should always have at least one geometry in their label
            debug_assert!(label.geometry_count() > 0, "node with empty label found");
            if node.is_isolated() {
                if label.is_empty(0) {
                    Self::label_isolated_node(node, 0, geometry_a)
                } else {
                    Self::label_isolated_node(node, 1, geometry_b)
                }
            }
        }
    }

    fn label_isolated_node(
        node: &mut CoordNode<F>,
        target_index: usize,
        geometry: &GeometryCow<F>,
    ) {
        let position = geometry.coordinate_position(node.coordinate());
        node.label_mut().set_all_positions(target_index, position);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use geo_types::{polygon, Geometry};
    use std::str::FromStr;

    #[test]
    fn test_disjoint() {
        let square_a: Geometry<f64> = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_b: Geometry<f64> = polygon![
            (x: 55., y: 55.),
            (x: 50., y: 60.),
            (x: 60., y: 60.),
            (x: 60., y: 55.),
            (x: 55., y: 55.),
        ]
        .into();

        let gc1 = GeometryCow::from(&square_a);
        let gc2 = GeometryCow::from(&square_b);
        let mut relate_computer = RelateOperation::new(&gc1, &gc2);
        let intersection_matrix = relate_computer.compute_intersection_matrix();
        assert_eq!(
            intersection_matrix,
            IntersectionMatrix::from_str("FF2FF1212").unwrap()
        );
    }

    #[test]
    fn test_a_contains_b() {
        let square_a: Geometry<f64> = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_b: Geometry<f64> = polygon![
            (x: 5., y: 5.),
            (x: 5., y: 10.),
            (x: 10., y: 10.),
            (x: 10., y: 5.),
            (x: 5., y: 5.),
        ]
        .into();

        let gca = GeometryCow::from(&square_a);
        let gcb = GeometryCow::from(&square_b);
        let mut relate_computer = RelateOperation::new(&gca, &gcb);
        let intersection_matrix = relate_computer.compute_intersection_matrix();
        assert_eq!(
            intersection_matrix,
            IntersectionMatrix::from_str("212FF1FF2").unwrap()
        );
    }

    #[test]
    fn test_a_overlaps_b() {
        let square_a: Geometry<f64> = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 20.),
            (x: 20., y: 20.),
            (x: 20., y: 0.),
            (x: 0., y: 0.),
        ]
        .into();

        let square_b: Geometry<f64> = polygon![
            (x: 5., y: 5.),
            (x: 5., y: 30.),
            (x: 30., y: 30.),
            (x: 30., y: 5.),
            (x: 5., y: 5.),
        ]
        .into();

        let gca = &GeometryCow::from(&square_a);
        let gcb = &GeometryCow::from(&square_b);
        let mut relate_computer = RelateOperation::new(gca, gcb);
        let intersection_matrix = relate_computer.compute_intersection_matrix();
        assert_eq!(
            intersection_matrix,
            IntersectionMatrix::from_str("212101212").unwrap()
        );
    }
}
