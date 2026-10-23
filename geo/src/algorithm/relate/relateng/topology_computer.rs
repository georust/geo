//! Accumulates topological information from point, line-end, area-vertex
//! and edge-intersection interactions between the two input geometries,
//! updating the predicate as evaluation proceeds.
//!
//! Port of JTS `TopologyComputer`.

use std::collections::BTreeMap;

use crate::algorithm::polygon_node_topology::is_crossing;
use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::relate::geomgraph::Direction;
use crate::relate::geomgraph::node_map::NodeKey;
use crate::{Coord, GeoFloat};

use super::node_section::NodeSection;
use super::node_sections::NodeSections;
use super::relate_geometry::RelateGeometry;
use super::relate_node::RelateNode;
use super::topology_predicate::{InputIndex, TopologyPredicate};

const MSG_GEOMETRY_DIMENSION_UNEXPECTED: &str = "Unexpected combination of geometry dimensions";

pub(crate) struct TopologyComputer<'r, 'a, 'b, F: GeoFloat> {
    predicate: &'r mut dyn TopologyPredicate<F>,
    geom_a: &'r RelateGeometry<'a, F>,
    geom_b: &'r RelateGeometry<'b, F>,
    node_map: BTreeMap<NodeKey<F>, NodeSections<F>>,
}

impl<'r, 'a, 'b, F: GeoFloat> TopologyComputer<'r, 'a, 'b, F> {
    pub fn new(
        predicate: &'r mut dyn TopologyPredicate<F>,
        geom_a: &'r RelateGeometry<'a, F>,
        geom_b: &'r RelateGeometry<'b, F>,
    ) -> Self {
        let mut computer = Self {
            predicate,
            geom_a,
            geom_b,
            node_map: BTreeMap::new(),
        };
        computer.init_exterior_dims();
        computer
    }

    /// Determines a-priori partial exterior topology based on the real
    /// dimensions of the inputs.
    fn init_exterior_dims(&mut self) {
        let dim_real_a = self.geom_a.dimension_real();
        let dim_real_b = self.geom_b.dimension_real();
        use Dimensions::{Empty, OneDimensional, TwoDimensional, ZeroDimensional};

        if dim_real_a == ZeroDimensional && dim_real_b == OneDimensional {
            // For the P/L case, the point exterior intersects the line
            // interior.
            self.update_dim(CoordPos::Outside, CoordPos::Inside, OneDimensional);
        } else if dim_real_a == OneDimensional && dim_real_b == ZeroDimensional {
            self.update_dim(CoordPos::Inside, CoordPos::Outside, OneDimensional);
        } else if dim_real_a == ZeroDimensional && dim_real_b == TwoDimensional {
            // For the P/A case, the area interior and boundary intersect
            // the point exterior.
            self.update_dim(CoordPos::Outside, CoordPos::Inside, TwoDimensional);
            self.update_dim(CoordPos::Outside, CoordPos::OnBoundary, OneDimensional);
        } else if dim_real_a == TwoDimensional && dim_real_b == ZeroDimensional {
            self.update_dim(CoordPos::Inside, CoordPos::Outside, TwoDimensional);
            self.update_dim(CoordPos::OnBoundary, CoordPos::Outside, OneDimensional);
        } else if dim_real_a == OneDimensional && dim_real_b == TwoDimensional {
            self.update_dim(CoordPos::Outside, CoordPos::Inside, TwoDimensional);
        } else if dim_real_a == TwoDimensional && dim_real_b == OneDimensional {
            self.update_dim(CoordPos::Inside, CoordPos::Outside, TwoDimensional);
        } else if dim_real_a == Empty || dim_real_b == Empty {
            // Cases where one geometry is empty.
            if dim_real_a != Empty {
                self.init_exterior_empty(InputIndex::A);
            }
            if dim_real_b != Empty {
                self.init_exterior_empty(InputIndex::B);
            }
        }
    }

    /// Seeds the matrix entries of a non-empty geometry against an empty
    /// one (everything of the non-empty geometry lies in the exterior of
    /// the empty one).
    fn init_exterior_empty(&mut self, geom_non_empty: InputIndex) {
        // The type-based dimension, as in JTS.
        let dim_non_empty = self.dimension(geom_non_empty);
        match dim_non_empty {
            Dimensions::ZeroDimensional => {
                self.update_dim_for(
                    geom_non_empty,
                    CoordPos::Inside,
                    CoordPos::Outside,
                    Dimensions::ZeroDimensional,
                );
            }
            Dimensions::OneDimensional => {
                if self.geometry_has_boundary(geom_non_empty) {
                    self.update_dim_for(
                        geom_non_empty,
                        CoordPos::OnBoundary,
                        CoordPos::Outside,
                        Dimensions::ZeroDimensional,
                    );
                }
                self.update_dim_for(
                    geom_non_empty,
                    CoordPos::Inside,
                    CoordPos::Outside,
                    Dimensions::OneDimensional,
                );
            }
            Dimensions::TwoDimensional => {
                self.update_dim_for(
                    geom_non_empty,
                    CoordPos::OnBoundary,
                    CoordPos::Outside,
                    Dimensions::OneDimensional,
                );
                self.update_dim_for(
                    geom_non_empty,
                    CoordPos::Inside,
                    CoordPos::Outside,
                    Dimensions::TwoDimensional,
                );
            }
            Dimensions::Empty => {}
        }
    }

    fn geometry_has_boundary(&self, input: InputIndex) -> bool {
        match input {
            InputIndex::A => self.geom_a.has_boundary(),
            InputIndex::B => self.geom_b.has_boundary(),
        }
    }

    fn geometry_is_empty(&self, input: InputIndex) -> bool {
        match input {
            InputIndex::A => self.geom_a.is_empty(),
            InputIndex::B => self.geom_b.is_empty(),
        }
    }

    /// The type-based dimension of an input.
    pub fn dimension(&self, input: InputIndex) -> Dimensions {
        match input {
            InputIndex::A => self.geom_a.dimension(),
            InputIndex::B => self.geom_b.dimension(),
        }
    }

    /// Whether the input geometries require self-noding for correct
    /// evaluation of the predicate.
    ///
    /// Self-noding is required for geometries which may have self-crossing
    /// linework, or lines lying in the boundary of an area. It causes the
    /// coordinates of nodes created by crossing segments to be computed
    /// explicitly, so node locations match in situations where a
    /// self-crossing and a mutual crossing occur at the same logical
    /// location (the canonical example being a self-crossing line tested
    /// against a segment identical to one of the crossed segments).
    ///
    /// Requiring self-noding prevents cached-index reuse, so the cases are
    /// kept limited: A inputs which may self-cross, and B inputs which are
    /// mixed area/line collections (a linear B does not require it when A
    /// is polygonal; JTS PR #1099).
    pub fn is_self_noding_required(&self) -> bool {
        if !self.predicate.requires_self_noding() {
            return false;
        }
        if self.geom_a.is_self_noding_required() {
            return true;
        }
        // If B is a mixed collection with areas and lines, full noding is
        // required.
        self.geom_b.has_area_and_line()
    }

    pub fn is_exterior_check_required(&self, input: InputIndex) -> bool {
        self.predicate.requires_exterior_check(input)
    }

    fn update_dim(&mut self, loc_a: CoordPos, loc_b: CoordPos, dimension: Dimensions) {
        self.predicate.update_dimension(loc_a, loc_b, dimension);
    }

    /// Updates an entry with the locations given in the order (source,
    /// target); the source input determines the matrix orientation.
    fn update_dim_for(
        &mut self,
        source: InputIndex,
        loc1: CoordPos,
        loc2: CoordPos,
        dimension: Dimensions,
    ) {
        match source {
            InputIndex::A => self.update_dim(loc1, loc2, dimension),
            // The locations are ordered B/A, so swap them.
            InputIndex::B => self.update_dim(loc2, loc1, dimension),
        }
    }

    pub fn is_result_known(&self) -> bool {
        self.predicate.is_known()
    }

    pub fn result(&self) -> bool {
        self.predicate.value()
    }

    /// Finalises the evaluation.
    pub fn finish(&mut self) {
        self.predicate.finish();
    }

    /// Records an intersection between segments of the inputs: updates
    /// direct topology for A/B interactions and accumulates the node
    /// sections for later node evaluation.
    pub fn add_intersection(&mut self, a: NodeSection<F>, b: NodeSection<F>) {
        if !a.is_same_geometry(&b) {
            self.update_intersection_ab(&a, &b);
        }
        // Add the edges to the node, to allow full topology evaluation
        // later.
        self.add_node_sections(a, b);
    }

    fn update_intersection_ab(&mut self, a: &NodeSection<F>, b: &NodeSection<F>) {
        if NodeSection::is_area_area(a, b) {
            self.update_area_area_cross(a, b);
        }
        self.update_node_location(a, b);
    }

    /// Updates topology for an A/B area-area crossing node. Sections cross
    /// at a node if the intersection is proper (in the interior of both
    /// segments), or otherwise if the linework on either side of the node
    /// crosses. In these situations the area interiors intersect in
    /// dimension 2.
    fn update_area_area_cross(&mut self, a: &NodeSection<F>, b: &NodeSection<F>) {
        let is_proper = NodeSection::is_proper_pair(a, b);
        if is_proper
            || is_crossing(
                a.node_pt(),
                a.vertex(0).expect("area section has both vertices"),
                a.vertex(1).expect("area section has both vertices"),
                b.vertex(0).expect("area section has both vertices"),
                b.vertex(1).expect("area section has both vertices"),
            )
        {
            self.update_dim(
                CoordPos::Inside,
                CoordPos::Inside,
                Dimensions::TwoDimensional,
            );
        }
    }

    /// Updates topology for the point location of an A/B edge intersection
    /// node.
    fn update_node_location(&mut self, a: &NodeSection<F>, b: &NodeSection<F>) {
        let pt = a.node_pt();
        let loc_a = self.geom_a.locate_node(pt, a.polygonal_id());
        let loc_b = self.geom_b.locate_node(pt, b.polygonal_id());
        self.update_dim(loc_a, loc_b, Dimensions::ZeroDimensional);
    }

    fn add_node_sections(&mut self, ns0: NodeSection<F>, ns1: NodeSection<F>) {
        let sections = self
            .node_map
            .entry(NodeKey(ns0.node_pt()))
            .or_insert_with(|| NodeSections::new(ns0.node_pt()));
        sections.add_node_section(ns0);
        sections.add_node_section(ns1);
    }

    pub fn add_point_on_point_interior(&mut self) {
        self.update_dim(
            CoordPos::Inside,
            CoordPos::Inside,
            Dimensions::ZeroDimensional,
        );
    }

    pub fn add_point_on_point_exterior(&mut self, source: InputIndex) {
        self.update_dim_for(
            source,
            CoordPos::Inside,
            CoordPos::Outside,
            Dimensions::ZeroDimensional,
        );
    }

    pub fn add_point_on_geometry(
        &mut self,
        source: InputIndex,
        loc_target: CoordPos,
        dim_target: Dimensions,
        _pt: Coord<F>,
    ) {
        // Update the entry for the point interior.
        self.update_dim_for(
            source,
            CoordPos::Inside,
            loc_target,
            Dimensions::ZeroDimensional,
        );

        // An empty geometry has no points to infer entries from.
        if self.geometry_is_empty(source.other()) {
            return;
        }

        match dim_target {
            Dimensions::ZeroDimensional => {}
            Dimensions::OneDimensional => {
                // Because zero-length lines are handled, a point lying in
                // the exterior of the line target may imply either P or L
                // for the exterior interaction, so nothing further can be
                // inferred.
            }
            Dimensions::TwoDimensional => {
                // If a point intersects an area target, then the area
                // interior and boundary must extend beyond the point and
                // thus interact with its exterior.
                self.update_dim_for(
                    source,
                    CoordPos::Outside,
                    CoordPos::Inside,
                    Dimensions::TwoDimensional,
                );
                self.update_dim_for(
                    source,
                    CoordPos::Outside,
                    CoordPos::OnBoundary,
                    Dimensions::OneDimensional,
                );
            }
            Dimensions::Empty => unreachable!("{}", MSG_GEOMETRY_DIMENSION_UNEXPECTED),
        }
    }

    /// Adds topology for a line end. The line end point must be
    /// "significant": not contained in an area if the source is a
    /// mixed-dimension collection.
    pub fn add_line_end_on_geometry(
        &mut self,
        source: InputIndex,
        loc_line_end: CoordPos,
        loc_target: CoordPos,
        dim_target: Dimensions,
        pt: Coord<F>,
    ) {
        // Record topology at the line end point.
        self.update_dim_for(
            source,
            loc_line_end,
            loc_target,
            Dimensions::ZeroDimensional,
        );

        // An empty geometry has no points to infer entries from.
        if self.geometry_is_empty(source.other()) {
            return;
        }

        // Line and area targets may have additional topology.
        match dim_target {
            Dimensions::ZeroDimensional => {}
            Dimensions::OneDimensional => {
                self.add_line_end_on_line(source, loc_line_end, loc_target, pt);
            }
            Dimensions::TwoDimensional => {
                self.add_line_end_on_area(source, loc_line_end, loc_target, pt);
            }
            Dimensions::Empty => unreachable!("{}", MSG_GEOMETRY_DIMENSION_UNEXPECTED),
        }
    }

    fn add_line_end_on_line(
        &mut self,
        source: InputIndex,
        _loc_line_end: CoordPos,
        loc_line: CoordPos,
        _pt: Coord<F>,
    ) {
        // When a line end is in the exterior of a line, some length of the
        // source line interior is also in the target line exterior. This
        // holds for zero-length lines as well.
        if loc_line == CoordPos::Outside {
            self.update_dim_for(
                source,
                CoordPos::Inside,
                CoordPos::Outside,
                Dimensions::OneDimensional,
            );
        }
    }

    fn add_line_end_on_area(
        &mut self,
        source: InputIndex,
        _loc_line_end: CoordPos,
        loc_area: CoordPos,
        _pt: Coord<F>,
    ) {
        if loc_area != CoordPos::OnBoundary {
            // When a line end is in an area interior or exterior, some
            // length of the source line interior, and the exterior of the
            // line, is also in that location of the target. This assumes
            // the line end is not also in an area of a mixed-dimension
            // collection.
            self.update_dim_for(
                source,
                CoordPos::Inside,
                loc_area,
                Dimensions::OneDimensional,
            );
            self.update_dim_for(
                source,
                CoordPos::Outside,
                loc_area,
                Dimensions::TwoDimensional,
            );
        }
    }

    /// Adds topology for an area vertex interaction with a target geometry
    /// element. Assumes the target element has the highest dimension of
    /// those the point lies on (the semantic provided by the point
    /// locator).
    ///
    /// In a GeometryCollection containing overlapping or adjacent
    /// polygons, the area vertex location may be interior instead of
    /// boundary.
    pub fn add_area_vertex(
        &mut self,
        source: InputIndex,
        loc_area: CoordPos,
        loc_target: CoordPos,
        dim_target: Dimensions,
        pt: Coord<F>,
    ) {
        if loc_target == CoordPos::Outside {
            self.update_dim_for(
                source,
                CoordPos::Inside,
                CoordPos::Outside,
                Dimensions::TwoDimensional,
            );
            // If the area vertex is on the boundary, further topology can
            // be deduced from the neighbourhood around the boundary
            // vertex. This is always the case for polygonal geometries;
            // for collections the vertex may be on the boundary or in the
            // interior (of overlapping or adjacent polygons).
            if loc_area == CoordPos::OnBoundary {
                self.update_dim_for(
                    source,
                    CoordPos::OnBoundary,
                    CoordPos::Outside,
                    Dimensions::OneDimensional,
                );
                self.update_dim_for(
                    source,
                    CoordPos::Outside,
                    CoordPos::Outside,
                    Dimensions::TwoDimensional,
                );
            }
            return;
        }
        match dim_target {
            Dimensions::ZeroDimensional => self.add_area_vertex_on_point(source, loc_area, pt),
            Dimensions::OneDimensional => {
                self.add_area_vertex_on_line(source, loc_area, loc_target, pt)
            }
            Dimensions::TwoDimensional => {
                self.add_area_vertex_on_area(source, loc_area, loc_target, pt)
            }
            Dimensions::Empty => unreachable!("{}", MSG_GEOMETRY_DIMENSION_UNEXPECTED),
        }
    }

    /// An area vertex (in the interior or on the boundary) intersecting a
    /// point. Because the largest-dimension intersecting target was
    /// determined, the point is not part of any other target element, so
    /// its neighbourhood is in the target exterior.
    fn add_area_vertex_on_point(&mut self, source: InputIndex, loc_area: CoordPos, _pt: Coord<F>) {
        // The vertex location intersects the point.
        self.update_dim_for(
            source,
            loc_area,
            CoordPos::Inside,
            Dimensions::ZeroDimensional,
        );
        // The area interior intersects the point's exterior neighbourhood.
        self.update_dim_for(
            source,
            CoordPos::Inside,
            CoordPos::Outside,
            Dimensions::TwoDimensional,
        );
        // If the area vertex is on the boundary, the area boundary and
        // exterior also intersect the point's exterior neighbourhood.
        if loc_area == CoordPos::OnBoundary {
            self.update_dim_for(
                source,
                CoordPos::OnBoundary,
                CoordPos::Outside,
                Dimensions::OneDimensional,
            );
            self.update_dim_for(
                source,
                CoordPos::Outside,
                CoordPos::Outside,
                Dimensions::TwoDimensional,
            );
        }
    }

    /// An area vertex intersecting a line: all that is known is the
    /// point intersection (the line may or may not be collinear with the
    /// area boundary or intersect the area interior); full topology is
    /// determined later by node analysis.
    fn add_area_vertex_on_line(
        &mut self,
        source: InputIndex,
        loc_area: CoordPos,
        loc_target: CoordPos,
        _pt: Coord<F>,
    ) {
        self.update_dim_for(source, loc_area, loc_target, Dimensions::ZeroDimensional);
        if loc_area == CoordPos::Inside {
            // The area interior intersects the line's exterior
            // neighbourhood.
            self.update_dim_for(
                source,
                CoordPos::Inside,
                CoordPos::Outside,
                Dimensions::TwoDimensional,
            );
        }
    }

    fn add_area_vertex_on_area(
        &mut self,
        source: InputIndex,
        loc_area: CoordPos,
        loc_target: CoordPos,
        _pt: Coord<F>,
    ) {
        if loc_target == CoordPos::OnBoundary {
            if loc_area == CoordPos::OnBoundary {
                // B/B topology is fully computed later by node analysis.
                self.update_dim_for(
                    source,
                    CoordPos::OnBoundary,
                    CoordPos::OnBoundary,
                    Dimensions::ZeroDimensional,
                );
            } else {
                // The area vertex is in the interior.
                self.update_dim_for(
                    source,
                    CoordPos::Inside,
                    CoordPos::Inside,
                    Dimensions::TwoDimensional,
                );
                self.update_dim_for(
                    source,
                    CoordPos::Inside,
                    CoordPos::OnBoundary,
                    Dimensions::OneDimensional,
                );
                self.update_dim_for(
                    source,
                    CoordPos::Inside,
                    CoordPos::Outside,
                    Dimensions::TwoDimensional,
                );
            }
        } else {
            // The target location is interior or exterior.
            self.update_dim_for(
                source,
                CoordPos::Inside,
                loc_target,
                Dimensions::TwoDimensional,
            );
            // If the area vertex is on the boundary, further topology can
            // be deduced from the neighbourhood around the boundary
            // vertex (see add_area_vertex).
            if loc_area == CoordPos::OnBoundary {
                self.update_dim_for(
                    source,
                    CoordPos::OnBoundary,
                    loc_target,
                    Dimensions::OneDimensional,
                );
                self.update_dim_for(
                    source,
                    CoordPos::Outside,
                    loc_target,
                    Dimensions::TwoDimensional,
                );
            }
        }
    }

    /// Evaluates the accumulated intersection nodes: builds each node with
    /// interaction between the inputs, finishes its edge topology, and
    /// reads the edge locations into the matrix.
    pub fn evaluate_nodes(&mut self) {
        let Self {
            predicate,
            geom_a,
            geom_b,
            node_map,
        } = self;
        for sections in node_map.values_mut() {
            if sections.has_interaction_ab() {
                evaluate_node(*predicate, geom_a, geom_b, sections);
                if predicate.is_known() {
                    return;
                }
            }
        }
    }
}

fn evaluate_node<F: GeoFloat>(
    predicate: &mut dyn TopologyPredicate<F>,
    geom_a: &RelateGeometry<'_, F>,
    geom_b: &RelateGeometry<'_, F>,
    sections: &mut NodeSections<F>,
) {
    let p = sections.coordinate();
    let mut node = sections.create_node();
    // The node must have edges for its own geometry, but may also lie in
    // the interior of an overlapping polygon of a collection.
    let is_area_interior_a = geom_a.is_node_in_area(p, sections.polygonal_id(InputIndex::A));
    let is_area_interior_b = geom_b.is_node_in_area(p, sections.polygonal_id(InputIndex::B));
    node.finish(is_area_interior_a, is_area_interior_b);
    evaluate_node_edges(predicate, geom_a, geom_b, &node);
}

fn evaluate_node_edges<F: GeoFloat>(
    predicate: &mut dyn TopologyPredicate<F>,
    geom_a: &RelateGeometry<'_, F>,
    geom_b: &RelateGeometry<'_, F>,
    node: &RelateNode<F>,
) {
    let is_area_area = geom_a.dimension() == Dimensions::TwoDimensional
        && geom_b.dimension() == Dimensions::TwoDimensional;
    for e in node.edges() {
        let loc = |input, dir| {
            e.location(input, dir)
                .expect("finished node edges have all locations")
        };
        // Side updates are only needed for the area/area case.
        if is_area_area {
            predicate.update_dimension(
                loc(InputIndex::A, Direction::Left),
                loc(InputIndex::B, Direction::Left),
                Dimensions::TwoDimensional,
            );
            predicate.update_dimension(
                loc(InputIndex::A, Direction::Right),
                loc(InputIndex::B, Direction::Right),
                Dimensions::TwoDimensional,
            );
        }
        predicate.update_dimension(
            loc(InputIndex::A, Direction::On),
            loc(InputIndex::B, Direction::On),
            Dimensions::OneDimensional,
        );
    }
}
