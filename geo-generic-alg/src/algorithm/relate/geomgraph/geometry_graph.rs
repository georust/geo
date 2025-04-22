use super::{
    index::{
        EdgeSetIntersector, RStarEdgeSetIntersector, Segment, SegmentIntersector,
        SimpleEdgeSetIntersector,
    },
    CoordNode, CoordPos, Direction, Edge, Label, LineIntersector, PlanarGraph, TopologyPosition,
};

use crate::HasDimensions;
use crate::{Coord, GeoFloat, GeometryCow, Line, LineString, Point, Polygon};

use rstar::{RTree, RTreeNum};
use std::cell::RefCell;
use std::rc::Rc;

/// The computation of the [`IntersectionMatrix`](crate::algorithm::relate::IntersectionMatrix) relies on the use of a
/// structure called a "topology graph". The topology graph contains nodes (CoordNode) and
/// edges (Edge) corresponding to the nodes and line segments of a [`Geometry`](crate::Geometry). Each
/// node and edge in the graph is labeled with its topological location
/// relative to the source geometry.
///
/// Note that there is no requirement that points of self-intersection be a
/// vertex. Thus, to obtain a correct topology graph, [`Geometry`](crate::Geometry) must be
/// self-noded before constructing their graphs.
///
/// Two fundamental operations are supported by topology graphs:
///   - Computing the intersections between all the edges and nodes of a single graph
///   - Computing the intersections between the edges and nodes of two different graphs
///
/// GeometryGraph is based on [JTS's `GeomGraph` as of 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/GeometryGraph.java)
#[derive(Clone)]
pub struct GeometryGraph<'a, F>
where
    F: GeoFloat,
{
    arg_index: usize,
    parent_geometry: GeometryCow<'a, F>,
    tree: Option<Rc<RTree<Segment<F>>>>,
    use_boundary_determination_rule: bool,
    has_computed_self_nodes: bool,
    planar_graph: PlanarGraph<F>,
}

///  PlanarGraph delegations
///
/// In JTS, which is written in Java, GeometryGraph inherits from PlanarGraph. Here in Rust land we
/// use composition and delegation to the same effect.
impl<F> GeometryGraph<'_, F>
where
    F: GeoFloat,
{
    pub(crate) fn set_tree(&mut self, tree: Rc<RTree<Segment<F>>>) {
        self.tree = Some(tree);
    }

    pub(crate) fn get_or_build_tree(&self) -> Rc<RTree<Segment<F>>> {
        self.tree
            .clone()
            .unwrap_or_else(|| Rc::new(self.build_tree()))
    }

    pub(crate) fn build_tree(&self) -> RTree<Segment<F>> {
        let segments: Vec<Segment<F>> = self
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
        RTree::bulk_load(segments)
    }

    pub(crate) fn assert_eq_graph(&self, other: &Self) {
        assert_eq!(self.arg_index, other.arg_index);
        assert_eq!(
            self.use_boundary_determination_rule,
            other.use_boundary_determination_rule
        );
        assert_eq!(self.parent_geometry, other.parent_geometry);
        self.planar_graph.assert_eq_graph(&other.planar_graph);
    }

    pub(crate) fn clone_for_arg_index(&self, arg_index: usize) -> Self {
        debug_assert!(
            self.has_computed_self_nodes,
            "should only be called after computing self nodes"
        );
        let planar_graph = self
            .planar_graph
            .clone_for_arg_index(self.arg_index, arg_index);
        Self {
            arg_index,
            parent_geometry: self.parent_geometry.clone(),
            tree: self.tree.clone(),
            use_boundary_determination_rule: self.use_boundary_determination_rule,
            has_computed_self_nodes: true,
            planar_graph,
        }
    }

    pub(crate) fn edges(&self) -> &[Rc<RefCell<Edge<F>>>] {
        self.planar_graph.edges()
    }

    pub(crate) fn insert_edge(&mut self, edge: Edge<F>) {
        self.planar_graph.insert_edge(edge)
    }

    pub(crate) fn is_boundary_node(&self, coord: Coord<F>) -> bool {
        self.planar_graph.is_boundary_node(self.arg_index, coord)
    }

    pub(crate) fn add_node_with_coordinate(&mut self, coord: Coord<F>) -> &mut CoordNode<F> {
        self.planar_graph.add_node_with_coordinate(coord)
    }

    pub(crate) fn nodes_iter(&self) -> impl Iterator<Item = &CoordNode<F>> {
        self.planar_graph.nodes.iter()
    }
}

impl<'a, F> GeometryGraph<'a, F>
where
    F: GeoFloat + RTreeNum,
{
    pub(crate) fn new(arg_index: usize, parent_geometry: GeometryCow<'a, F>) -> Self {
        let mut graph = GeometryGraph {
            arg_index,
            parent_geometry,
            use_boundary_determination_rule: true,
            tree: None,
            has_computed_self_nodes: false,
            planar_graph: PlanarGraph::new(),
        };
        graph.add_geometry(&graph.parent_geometry.clone());
        graph
    }

    pub(crate) fn geometry(&self) -> &GeometryCow<F> {
        &self.parent_geometry
    }

    /// Determine whether a component (node or edge) that appears multiple times in elements
    /// of a Multi-Geometry is in the boundary or the interior of the Geometry
    pub fn determine_boundary(boundary_count: usize) -> CoordPos {
        // For now, we only support the SFS "Mod-2 Rule"
        // We could make this configurable if we wanted to support alternative boundary rules.
        if boundary_count % 2 == 1 {
            CoordPos::OnBoundary
        } else {
            CoordPos::Inside
        }
    }

    fn boundary_nodes(&self) -> impl Iterator<Item = &CoordNode<F>> {
        self.planar_graph.boundary_nodes(self.arg_index)
    }

    pub(crate) fn add_geometry(&mut self, geometry: &GeometryCow<F>) {
        if geometry.is_empty() {
            return;
        }
        match geometry {
            GeometryCow::Line(line) => self.add_line(line),
            GeometryCow::Rect(rect) => {
                // PERF: avoid this conversion/clone?
                self.add_polygon(&rect.to_polygon());
            }
            GeometryCow::Point(point) => {
                self.add_point(point);
            }
            GeometryCow::Polygon(polygon) => self.add_polygon(polygon),
            GeometryCow::Triangle(triangle) => {
                // PERF: avoid this conversion/clone?
                self.add_polygon(&triangle.to_polygon());
            }
            GeometryCow::LineString(line_string) => self.add_line_string(line_string),
            GeometryCow::MultiPoint(multi_point) => {
                for point in &multi_point.0 {
                    self.add_point(point);
                }
            }
            GeometryCow::MultiPolygon(multi_polygon) => {
                // check if this Geometry should obey the Boundary Determination Rule
                // all collections except MultiPolygons obey the rule
                self.use_boundary_determination_rule = false;
                for polygon in &multi_polygon.0 {
                    self.add_polygon(polygon);
                }
            }
            GeometryCow::MultiLineString(multi_line_string) => {
                for line_string in &multi_line_string.0 {
                    self.add_line_string(line_string);
                }
            }
            GeometryCow::GeometryCollection(geometry_collection) => {
                for geometry in geometry_collection.iter() {
                    self.add_geometry(&GeometryCow::from(geometry));
                }
            }
        }
    }

    fn add_polygon_ring(
        &mut self,
        linear_ring: &LineString<F>,
        cw_left: CoordPos,
        cw_right: CoordPos,
    ) {
        debug_assert!(linear_ring.is_closed());
        if linear_ring.is_empty() {
            return;
        }

        let mut coords: Vec<Coord<F>> = Vec::with_capacity(linear_ring.0.len());
        // remove repeated coords
        for coord in &linear_ring.0 {
            if coords.last() != Some(coord) {
                coords.push(*coord)
            }
        }

        if coords.len() < 4 {
            // TODO: we could return an Err here, but this has ramifications for how we can
            // use this code in other operations - do we want all our methods, like `contains` to
            // return a Result?
            warn!("encountered invalid ring, which has undefined results");
        }
        let first_point = coords[0];

        use crate::winding_order::{Winding, WindingOrder};
        let (left, right) = match linear_ring.winding_order() {
            Some(WindingOrder::Clockwise) => (cw_left, cw_right),
            Some(WindingOrder::CounterClockwise) => (cw_right, cw_left),
            None => {
                warn!("polygon had no winding order. Results are undefined.");
                (cw_left, cw_right)
            }
        };

        let edge = Edge::new(
            coords,
            Label::new(
                self.arg_index,
                TopologyPosition::area(CoordPos::OnBoundary, left, right),
            ),
        );
        self.insert_edge(edge);

        // insert the endpoint as a node, to mark that it is on the boundary
        self.insert_point(self.arg_index, first_point, CoordPos::OnBoundary);
    }

    fn add_polygon(&mut self, polygon: &Polygon<F>) {
        self.add_polygon_ring(polygon.exterior(), CoordPos::Outside, CoordPos::Inside);
        // Holes are topologically labeled opposite to the shell, since
        // the interior of the polygon lies on their opposite side
        // (on the left, if the hole is oriented CW)
        for hole in polygon.interiors() {
            self.add_polygon_ring(hole, CoordPos::Inside, CoordPos::Outside)
        }
    }

    fn add_line_string(&mut self, line_string: &LineString<F>) {
        if line_string.is_empty() {
            return;
        }

        let mut coords: Vec<Coord<F>> = Vec::with_capacity(line_string.0.len());
        for coord in &line_string.0 {
            if coords.last() != Some(coord) {
                coords.push(*coord)
            }
        }

        if coords.len() < 2 {
            warn!("Treating invalid linestring as point, which has undefined results");
            self.add_point(&coords[0].into());
            return;
        }

        self.insert_boundary_point(*coords.first().unwrap());
        self.insert_boundary_point(*coords.last().unwrap());

        let edge = Edge::new(
            coords,
            Label::new(
                self.arg_index,
                TopologyPosition::line_or_point(CoordPos::Inside),
            ),
        );
        self.insert_edge(edge);
    }

    fn add_line(&mut self, line: &Line<F>) {
        self.insert_boundary_point(line.start);
        self.insert_boundary_point(line.end);

        let edge = Edge::new(
            vec![line.start, line.end],
            Label::new(
                self.arg_index,
                TopologyPosition::line_or_point(CoordPos::Inside),
            ),
        );

        self.insert_edge(edge);
    }

    /// Add a point computed externally.  The point is assumed to be a
    /// Point Geometry part, which has a location of INTERIOR.
    fn add_point(&mut self, point: &Point<F>) {
        self.insert_point(self.arg_index, (*point).into(), CoordPos::Inside);
    }

    /// Compute self-nodes, taking advantage of the Geometry type to minimize the number of
    /// intersection tests.  (E.g. rings are not tested for self-intersection, since they are
    /// assumed to be valid).
    ///
    /// `line_intersector` the [`LineIntersector`] to use to determine intersection
    pub(crate) fn compute_self_nodes(&mut self, line_intersector: Box<dyn LineIntersector<F>>) {
        if self.has_computed_self_nodes {
            return;
        }
        self.has_computed_self_nodes = true;

        let mut segment_intersector = SegmentIntersector::new(line_intersector, true);

        // optimize intersection search for valid Polygons and LinearRings
        let is_rings = match self.geometry() {
            GeometryCow::LineString(ls) => ls.is_closed(),
            GeometryCow::MultiLineString(ls) => ls.is_closed(),
            GeometryCow::Polygon(_) | GeometryCow::MultiPolygon(_) => true,
            _ => false,
        };
        let check_for_self_intersecting_edges = !is_rings;

        let edge_set_intersector = RStarEdgeSetIntersector;
        edge_set_intersector.compute_intersections_within_set(
            self,
            check_for_self_intersecting_edges,
            &mut segment_intersector,
        );
        self.add_self_intersection_nodes();
    }

    pub(crate) fn compute_edge_intersections(
        &self,
        other: &GeometryGraph<F>,
        line_intersector: Box<dyn LineIntersector<F>>,
    ) -> SegmentIntersector<F> {
        let mut segment_intersector = SegmentIntersector::new(line_intersector, false);
        segment_intersector.set_boundary_nodes(
            self.boundary_nodes().cloned().collect(),
            other.boundary_nodes().cloned().collect(),
        );

        let edge_set_intersector = RStarEdgeSetIntersector;
        edge_set_intersector.compute_intersections_between_sets(
            self,
            other,
            &mut segment_intersector,
        );

        segment_intersector
    }

    fn insert_point(&mut self, arg_index: usize, coord: Coord<F>, position: CoordPos) {
        let node: &mut CoordNode<F> = self.add_node_with_coordinate(coord);
        node.label_mut().set_on_position(arg_index, position);
    }

    /// Add the boundary points of 1-dim (line) geometries.
    fn insert_boundary_point(&mut self, coord: Coord<F>) {
        let arg_index = self.arg_index;
        let node: &mut CoordNode<F> = self.add_node_with_coordinate(coord);

        let label: &mut Label = node.label_mut();

        // determine the current location for the point (if any)
        let boundary_count = {
            #[allow(clippy::bool_to_int_with_if)]
            let prev_boundary_count =
                if Some(CoordPos::OnBoundary) == label.position(arg_index, Direction::On) {
                    1
                } else {
                    0
                };
            prev_boundary_count + 1
        };

        let new_position = Self::determine_boundary(boundary_count);
        label.set_on_position(arg_index, new_position);
    }

    fn add_self_intersection_nodes(&mut self) {
        let positions_and_intersections: Vec<(CoordPos, Vec<Coord<F>>)> = self
            .edges()
            .iter()
            .map(|cell| cell.borrow())
            .map(|edge| {
                let position = edge
                    .label()
                    .on_position(self.arg_index)
                    .expect("all edge labels should have an `on` position by now");
                let coordinates = edge
                    .edge_intersections()
                    .iter()
                    .map(|edge_intersection| edge_intersection.coordinate());

                (position, coordinates.collect())
            })
            .collect();

        for (position, edge_intersection_coordinates) in positions_and_intersections {
            for coordinate in edge_intersection_coordinates {
                self.add_self_intersection_node(coordinate, position)
            }
        }
    }

    /// Add a node for a self-intersection.
    ///
    /// If the node is a potential boundary node (e.g. came from an edge which is a boundary), then
    /// insert it as a potential boundary node.  Otherwise, just add it as a regular node.
    fn add_self_intersection_node(&mut self, coord: Coord<F>, position: CoordPos) {
        // if this node is already a boundary node, don't change it
        if self.is_boundary_node(coord) {
            return;
        }

        if position == CoordPos::OnBoundary && self.use_boundary_determination_rule {
            self.insert_boundary_point(coord)
        } else {
            self.insert_point(self.arg_index, coord, position)
        }
    }
}
