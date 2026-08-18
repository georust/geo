//! Next-generation relate engine, a port of JTS's `operation.relateng`
//! package (Martin Davis, JTS 1.20; ported from master at `ab57bff`).
//!
//! RelateNG evaluates DE-9IM topological predicates and intersection
//! matrices without building a noded topology graph. It locates points and
//! line ends on the opposing geometry, examines each intersection node from
//! local segment-pair tests with robust orientation predicates, and never
//! constructs intersection points into shared topology. This avoids the
//! robustness failures of the graph-based engine when an intersection point
//! is not exactly representable (see georust/geo issue #1585), supports
//! GeometryCollections with union semantics, and allows named predicates to
//! short-circuit as soon as their value is known.
//!
//! The module is crate-private while the port is in progress; see
//! RELATENG_PLAN.md at the workspace root for the implementation plan and
//! progress log. The boundary node rule is Mod-2 (the OGC SFS rule),
//! matching the rest of the crate.

// The module is consumed incrementally as the port proceeds; the allowance
// is removed when the RelateNG driver lands.
#![allow(dead_code)]

pub(crate) mod adjacent_edge_locator;
pub(crate) mod dimension_location;
pub(crate) mod edge_segment_intersector;
pub(crate) mod im_predicate;
pub(crate) mod linear_boundary;
pub(crate) mod node_section;
pub(crate) mod node_sections;
pub(crate) mod polygon_node_converter;
pub(crate) mod relate_edge;
pub(crate) mod relate_geometry;
pub(crate) mod relate_node;
pub(crate) mod relate_point_locator;
pub(crate) mod relate_predicate;
pub(crate) mod relate_segment_string;
pub(crate) mod topology_computer;
pub(crate) mod topology_predicate;
