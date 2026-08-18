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
//! The module is crate-private; the public entry points are the `Relate`
//! trait and the predicate traits built on it. The boundary node rule is
//! Mod-2 (the OGC SFS rule), matching the rest of the crate.

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
pub(crate) mod relate_ng;
pub(crate) mod relate_node;
pub(crate) mod relate_point_locator;
pub(crate) mod relate_predicate;
pub(crate) mod relate_segment_string;
pub(crate) mod topology_computer;
pub(crate) mod topology_predicate;

#[cfg(test)]
mod tests;

/// Implements a predicate trait for `$for` against each `$target` type by
/// evaluating a RelateNG predicate, which short-circuits as soon as its
/// value is known (unlike computing the full matrix).
macro_rules! impl_predicate_from_relate {
    ($trait:ident, $method:ident, $predicate:expr, $for:ty, [$($target:ty),*]) => {
        $(
            impl<T> $trait<$target> for $for
            where
                T: GeoFloat
            {
                fn $method(&self, target: &$target) -> bool {
                    use $crate::algorithm::Relate;
                    $crate::algorithm::relate::relateng::relate_ng::eval(
                        &self.geometry_cow(),
                        &target.geometry_cow(),
                        &mut $predicate,
                    )
                }
            }
        )*
    };
}
pub(crate) use impl_predicate_from_relate;
