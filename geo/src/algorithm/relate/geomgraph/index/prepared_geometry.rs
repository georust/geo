use super::Segment;
use crate::geometry::*;
use crate::relate::geomgraph::{GeometryGraph, RobustLineIntersector};
use crate::GeometryCow;
use crate::{GeoFloat, Relate};

use std::cell::RefCell;
use std::sync::Arc;

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
#[derive(Clone)]
pub struct PreparedGeometry<'a, F: GeoFloat + RTreeNum = f64> {
    geometry_graph: GeometryGraph<'a, F>,
}

mod conversions {
    use crate::geometry_cow::GeometryCow;
    use crate::relate::geomgraph::{GeometryGraph, RobustLineIntersector};
    use crate::{GeoFloat, PreparedGeometry};
    use geo_types::{
        Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint, MultiPolygon,
        Point, Polygon, Rect, Triangle,
    };
    use std::sync::Arc;

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
            geometry_graph.update_tree(); // TODO: maybe unecessary

            // TODO: don't pass in line intersector here - in theory we'll want pluggable line intersectors
            // and the type (Robust) shouldn't be hard coded here.
            geometry_graph.compute_self_nodes(Box::new(RobustLineIntersector::new()));

            Self { geometry_graph }
        }
    }
}

impl<'a, F> PreparedGeometry<'a, F>
where
    F: GeoFloat + RTreeNum,
{
    pub(crate) fn geometry(&self) -> &GeometryCow<F> {
        self.geometry_graph.geometry()
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
