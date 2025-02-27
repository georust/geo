use super::Segment;
use crate::geometry::*;
use crate::relate::geomgraph::{GeometryGraph, RobustLineIntersector};
use crate::{BoundingRect, GeometryCow, HasDimensions};
use crate::{GeoFloat, Relate};

use std::cell::RefCell;
use std::rc::Rc;

use crate::dimensions::Dimensions;
use rstar::{RTree, RTreeNum};

/// A `PreparedGeometry` can be more efficient than a plain Geometry when performing
/// multiple topological comparisons against the `PreparedGeometry`.
///
/// ```
/// use geo::{Relate, PreparedGeometry, wkt};
///
/// let polygon = wkt! { POLYGON((2.0 2.0,2.0 6.0,4.0 6.0)) };
/// let touching_line = wkt! { LINESTRING(0.0 0.0,2.0 2.0) };
/// let intersecting_line = wkt! { LINESTRING(0.0 0.0,3.0 3.0) };
/// let contained_line = wkt! { LINESTRING(2.0 2.0,3.0 5.0) };
///
/// let prepared_polygon = PreparedGeometry::from(polygon);
/// assert!(prepared_polygon.relate(&touching_line).is_touches());
/// assert!(prepared_polygon.relate(&intersecting_line).is_intersects());
/// assert!(prepared_polygon.relate(&contained_line).is_contains());
///
/// ```
pub struct PreparedGeometry<'a, F: GeoFloat + RTreeNum = f64> {
    pub(crate) geometry_graph: GeometryGraph<'a, F>,
    pub(crate) bounding_rect: Option<Rect<F>>,
}

mod conversions {
    use crate::geometry_cow::GeometryCow;
    use crate::relate::geomgraph::{GeometryGraph, RobustLineIntersector};
    use crate::{BoundingRect, GeoFloat, PreparedGeometry};
    use geo_types::{
        Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint, MultiPolygon,
        Point, Polygon, Rect, Triangle,
    };
    use rstar::Envelope;
    use std::rc::Rc;

    impl<'a, F: GeoFloat> From<GeometryCow<'a, F>> for PreparedGeometry<'a, F> {
        fn from(geometry: GeometryCow<'a, F>) -> Self {
            let mut geometry_graph = GeometryGraph::new(0, geometry);
            let r_tree = geometry_graph.build_tree();

            let envelope = r_tree.root().envelope();

            // geo and rstar have different conventions for how to represet an empty bounding boxes
            let bounding_rect: Option<Rect<F>> = if envelope == rstar::AABB::new_empty() {
                None
            } else {
                Some(Rect::new(envelope.lower(), envelope.upper()))
            };
            // They should be equal - but we can save the computation in the `--release` case
            // by using the bounding_rect from the RTree
            debug_assert_eq!(bounding_rect, geometry_graph.geometry().bounding_rect());

            geometry_graph.set_tree(Rc::new(r_tree));

            // TODO: don't pass in line intersector here - in theory we'll want pluggable line intersectors
            // and the type (Robust) shouldn't be hard coded here.
            geometry_graph.compute_self_nodes(Box::new(RobustLineIntersector::new()));
            Self {
                geometry_graph,
                bounding_rect,
            }
        }
    }
}

impl<F> PreparedGeometry<'_, F>
where
    F: GeoFloat + RTreeNum,
{
    pub(crate) fn geometry(&self) -> &GeometryCow<F> {
        self.geometry_graph.geometry()
    }
}

impl<F: GeoFloat> BoundingRect<F> for PreparedGeometry<'_, F> {
    type Output = Option<Rect<F>>;

    fn bounding_rect(&self) -> Option<Rect<F>> {
        self.bounding_rect
    }
}

impl<F: GeoFloat> HasDimensions for PreparedGeometry<'_, F> {
    fn is_empty(&self) -> bool {
        self.geometry_graph.geometry().is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        self.geometry_graph.geometry().dimensions()
    }

    fn boundary_dimensions(&self) -> Dimensions {
        self.geometry_graph.geometry().boundary_dimensions()
    }
}

impl<F: GeoFloat> Relate<F> for PreparedGeometry<'_, F> {
    /// Efficiently builds a [`GeometryGraph`] which can then be used for topological
    /// computations.
    fn geometry_graph(&self, arg_index: usize) -> GeometryGraph<F> {
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
    fn prepared_with_unprepared() {
        let p1 = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let p2 = polygon![(x: 0.5, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let prepared_1 = PreparedGeometry::from(&p1);
        assert!(prepared_1.relate(&p2).is_contains());
        assert!(p2.relate(&prepared_1).is_within());
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
