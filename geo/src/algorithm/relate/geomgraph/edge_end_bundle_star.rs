use super::{
    Dimensions, Direction, EdgeEnd, EdgeEndBundle, EdgeEndKey, GeometryGraph, IntersectionMatrix,
    LabeledEdgeEndBundle,
};
use crate::algorithm::coordinate_position::{CoordPos, CoordinatePosition};
use crate::{Coordinate, GeoFloat, GeometryCow};

/// An ordered list of [`EdgeEndBundle`]s around a [`RelateNodeFactory::Node`].
///
/// They are maintained in CCW order (starting with the positive x-axis) around the node
/// for efficient lookup and topology building.
///
/// This is based on [JTS's `EdgeEndBundleStar` as of 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/operation/relate/EdgeEndBundleStar.java)
#[derive(Clone, Debug)]
pub(crate) struct EdgeEndBundleStar<F>
where
    F: GeoFloat,
{
    edge_map: std::collections::BTreeMap<EdgeEndKey<F>, EdgeEndBundle<F>>,
    point_in_area_location: Option<[CoordPos; 2]>,
}

#[derive(Clone, Debug)]
pub(crate) struct LabeledEdgeEndBundleStar<F>
where
    F: GeoFloat,
{
    edges: Vec<LabeledEdgeEndBundle<F>>,
}

impl<F: GeoFloat> LabeledEdgeEndBundleStar<F> {
    pub(crate) fn new(
        edges: Vec<LabeledEdgeEndBundle<F>>,
        graph_a: &GeometryGraph<F>,
        graph_b: &GeometryGraph<F>,
    ) -> Self {
        let mut labeled_bundle_star = Self { edges };
        labeled_bundle_star.compute_labeling(graph_a, graph_b);
        labeled_bundle_star
    }

    /// Compute a label for the star based on the labels of its EdgeEndBundles.
    fn compute_labeling(&mut self, graph_a: &GeometryGraph<F>, graph_b: &GeometryGraph<F>) {
        self.propagate_side_labels(0);
        self.propagate_side_labels(1);
        let mut has_dimensional_collapse_edge = [false, false];
        for edge_end in self.edges.iter() {
            let label = edge_end.label();
            for (geom_index, is_collapsed) in has_dimensional_collapse_edge.iter_mut().enumerate() {
                *is_collapsed = label.is_line(geom_index)
                    && label.on_position(geom_index) == Some(CoordPos::OnBoundary);
            }
        }
        for edge_end_bundle in &mut self.edges {
            let coord = *edge_end_bundle.coordinate();
            let label = edge_end_bundle.label_mut();
            for (geom_index, is_dimensionally_collapsed) in
                has_dimensional_collapse_edge.iter().enumerate()
            {
                if label.is_any_empty(geom_index) {
                    let position: CoordPos = if *is_dimensionally_collapsed {
                        CoordPos::Outside
                    } else {
                        // PERF: In JTS this is memoized, but that gets a little tricky with rust's
                        // borrow checker. Let's wait to see if it's a hotspot.
                        let geometry = match geom_index {
                            0 => graph_a.geometry(),
                            1 => graph_b.geometry(),
                            _ => unreachable!("invalid geom_index"),
                        };
                        use crate::algorithm::dimensions::HasDimensions;
                        if geometry.dimensions() == Dimensions::TwoDimensional {
                            geometry.coordinate_position(&coord)
                        } else {
                            // if geometry is *not* an area, Coord is always Outside
                            CoordPos::Outside
                        }
                    };
                    label.set_all_positions_if_empty(geom_index, position);
                }
            }
        }
        debug!("edge_end_bundle_star: {:?}", self);
    }

    fn propagate_side_labels(&mut self, geom_index: usize) {
        let mut start_position = None;

        for edge_ends in self.edge_end_bundles_iter() {
            let label = edge_ends.label();
            if label.is_geom_area(geom_index) {
                if let Some(position) = label.position(geom_index, Direction::Left) {
                    start_position = Some(position);
                }
            }
        }
        if start_position.is_none() {
            return;
        }
        let mut current_position = start_position.unwrap();

        for edge_ends in self.edge_end_bundles_iter_mut() {
            let label = edge_ends.label_mut();
            if label.position(geom_index, Direction::On).is_none() {
                label.set_position(geom_index, Direction::On, current_position);
            }
            if label.is_geom_area(geom_index) {
                let left_position = label.position(geom_index, Direction::Left);
                let right_position = label.position(geom_index, Direction::Right);

                if let Some(right_position) = right_position {
                    debug_assert!(right_position == current_position, "side_location conflict with coordinate: {:?}, right_location: {:?}, current_location: {:?}", edge_ends.coordinate(), right_position, current_position);
                    assert!(left_position.is_some(), "found single null side");
                    current_position = left_position.unwrap();
                } else {
                    debug_assert!(label.position(geom_index, Direction::Left).is_none());
                    label.set_position(geom_index, Direction::Right, current_position);
                    label.set_position(geom_index, Direction::Left, current_position);
                }
            }
        }
    }

    fn edge_end_bundles_iter(&self) -> impl Iterator<Item = &LabeledEdgeEndBundle<F>> {
        self.edges.iter()
    }

    fn edge_end_bundles_iter_mut(&mut self) -> impl Iterator<Item = &mut LabeledEdgeEndBundle<F>> {
        self.edges.iter_mut()
    }

    pub fn update_intersection_matrix(&self, intersection_matrix: &mut IntersectionMatrix) {
        for edge_end_bundle in self.edge_end_bundles_iter() {
            edge_end_bundle.update_intersection_matrix(intersection_matrix);
            debug!(
                "updated intersection_matrix: {:?} from edge_end_bundle: {:?}",
                intersection_matrix, edge_end_bundle
            );
        }
    }
}

impl<F> EdgeEndBundleStar<F>
where
    F: GeoFloat,
{
    pub(crate) fn new() -> Self {
        EdgeEndBundleStar {
            edge_map: std::collections::BTreeMap::new(),
            point_in_area_location: None,
        }
    }

    pub(crate) fn insert(&mut self, edge_end: EdgeEnd<F>) {
        let bundle = self
            .edge_map
            .entry(edge_end.key().clone())
            .or_insert_with(|| EdgeEndBundle::new(*edge_end.coordinate()));
        bundle.insert(edge_end);
    }

    fn edge_end_bundles_iter(&self) -> impl Iterator<Item = &EdgeEndBundle<F>> {
        self.edge_map.values()
    }

    fn edge_end_bundles_iter_mut(&mut self) -> impl Iterator<Item = &mut EdgeEndBundle<F>> {
        self.edge_map.values_mut()
    }

    /// Compute labeling for the star's EdgeEndBundles, and use them to compute an overall label
    /// for the star.
    ///
    /// Implementation Note: This is a bit of a divergence from JTS in two ways.
    ///
    /// Firstly, JTS doesn't leverage optionals, and sets nullable `Label`s, whereas here we convert
    /// to an explicitly labeled type to avoid unwrapping optionals later.
    ///
    /// Secondly, in JTS this functionality does not live directly on EdgeEndBundleStar, but rather
    /// on it's parent class [EdgeEndStar](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/EdgeEndStar.java#L117)
    ///
    /// Since we're only using this one subclass (EdgeEndBundleStar), we skip the
    /// complexity of mapping Java inheritance to Rust and implement this functionality
    /// on EdgeEndBundleStar directly.
    ///
    /// If/When we implement overlay operations we might consider extracting the superclass
    /// behavior.
    pub(crate) fn into_labeled(
        self,
        graph_a: &GeometryGraph<F>,
        graph_b: &GeometryGraph<F>,
    ) -> LabeledEdgeEndBundleStar<F> {
        debug!("edge_end_bundle_star: {:?}", self);
        let labeled_edges = self
            .edge_map
            .into_iter()
            .map(|(_k, v)| v.into_labeled())
            .collect();
        LabeledEdgeEndBundleStar::new(labeled_edges, graph_a, graph_b)
    }
}
