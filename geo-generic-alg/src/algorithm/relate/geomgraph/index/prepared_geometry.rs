use super::Segment;
use crate::geometry::*;
use crate::relate::geomgraph::{GeometryGraph, RobustLineIntersector};
use crate::{BoundingRect, GeometryCow, HasDimensions};
use crate::{GeoFloat, Relate};

use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::dimensions::Dimensions;
use rstar::{Envelope, RTree, RTreeNum};

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
pub struct PreparedGeometry<'a, G, F = f64>
where
    G: Into<GeometryCow<'a, F>>,
    F: GeoFloat + RTreeNum,
{
    pub(crate) geometry: G,
    pub(crate) geometry_graph: GeometryGraph<'a, F>,
    pub(crate) bounding_rect: Option<Rect<F>>,
}

impl<'a, G, F> Debug for PreparedGeometry<'a, G, F>
where
    G: Into<GeometryCow<'a, F>> + Debug,
    F: GeoFloat + RTreeNum,
{
    /// ```
    /// use geo::{wkt, PreparedGeometry};
    /// let poly = wkt!(POLYGON((0.0 0.0,2.0 0.0,1.0 1.0,0.0 0.0)));
    /// let prepared_geom = PreparedGeometry::from(&poly);
    ///
    /// let debug = format!("debug output is: {prepared_geom:?}");
    /// assert_eq!(
    ///     debug,
    ///     "debug output is: PreparedGeometry(POLYGON((0.0 0.0,2.0 0.0,1.0 1.0,0.0 0.0)))"
    /// );
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PreparedGeometry")
            .field(&self.geometry)
            .finish()
    }
}

use crate::geometry::*;
pub(crate) fn prepare_geometry<'a, F, T>(geometry: T) -> PreparedGeometry<'a, T, F>
where
    F: GeoFloat,
    T: Clone + Into<GeometryCow<'a, F>>,
{
    let mut geometry_graph = GeometryGraph::new(0, geometry.clone().into());
    let r_tree = geometry_graph.build_tree();
    geometry_graph.set_tree(Rc::new(r_tree));
    let bounding_rect = geometry_graph.geometry().bounding_rect();

    // TODO: don't pass in line intersector here - in theory we'll want pluggable line intersectors
    // and the type (Robust) shouldn't be hard coded here.
    geometry_graph.compute_self_nodes(Box::new(RobustLineIntersector::new()));
    PreparedGeometry {
        geometry,
        geometry_graph,
        bounding_rect,
    }
}

impl<'a, G, F> PreparedGeometry<'a, G, F>
where
    F: GeoFloat + RTreeNum,
    G: Into<GeometryCow<'a, F>>,
{
    pub fn geometry(&self) -> &G {
        &self.geometry
    }
    pub fn into_geometry(self) -> G {
        self.geometry
    }
}

impl<'a, G, F> BoundingRect<F> for PreparedGeometry<'a, G, F>
where
    F: GeoFloat,
    G: Into<GeometryCow<'a, F>>,
{
    type Output = Option<Rect<F>>;

    fn bounding_rect(&self) -> Option<Rect<F>> {
        self.bounding_rect
    }
}

impl<'a, G, F: GeoFloat> HasDimensions for PreparedGeometry<'a, G, F>
where
    F: GeoFloat,
    G: Into<GeometryCow<'a, F>>,
{
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

impl<'a, G, F: GeoFloat> Relate<F> for PreparedGeometry<'a, G, F>
where
    F: GeoFloat,
    G: Into<GeometryCow<'a, F>>,
{
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
    use crate::{polygon, wkt};

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

    #[test]
    fn get_geometry() {
        let poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let prepared_geom = PreparedGeometry::from(&poly);
        assert_eq!(&poly, *prepared_geom.geometry());
        assert_eq!(&poly, prepared_geom.into_geometry());

        let prepared_geom = PreparedGeometry::from(poly.clone());
        assert_eq!(&poly, prepared_geom.geometry());
        assert_eq!(poly, prepared_geom.into_geometry());
    }

    #[test]
    fn zero_dimensional_point() {
        let poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 2.0)];
        let prepared_poly = PreparedGeometry::from(&poly);
        let point = crate::point!(x: 1.0, y: 1.0);
        let prepared_point = PreparedGeometry::from(&point);

        let im = poly.relate(&point);
        assert!(im.matches("0F2FF1FF2").unwrap(), "got {im:?}");

        let im = prepared_poly.relate(&point);
        assert!(im.matches("0F2FF1FF2").unwrap(), "got {im:?}");

        let im = poly.relate(&prepared_point);
        assert!(im.matches("0F2FF1FF2").unwrap(), "got {im:?}");

        let im = prepared_poly.relate(&prepared_point);
        assert!(im.matches("0F2FF1FF2").unwrap(), "got {im:?}");
    }

    #[test]
    fn zero_dimensional_multipoint() {
        let poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 2.0)];
        let prepared_poly = PreparedGeometry::from(&poly);
        let multi_point = wkt!(MULTIPOINT(1. 1.));
        let prepared_multi_point = PreparedGeometry::from(&multi_point);

        let im = poly.relate(&multi_point);
        assert!(im.matches("0F2FF1FF2").unwrap(), "got {im:?}");

        let im = prepared_poly.relate(&multi_point);
        assert!(im.matches("0F2FF1FF2").unwrap(), "got {im:?}");

        let im = poly.relate(&prepared_multi_point);
        assert!(im.matches("0F2FF1FF2").unwrap(), "got {im:?}");

        let im = prepared_poly.relate(&prepared_multi_point);
        assert!(im.matches("0F2FF1FF2").unwrap(), "got {im:?}");
    }
}
