use crate::geometry::*;
use crate::relate::IntersectionMatrix;
use crate::relate::geomgraph::GeometryGraph;
use crate::relate::relateng::relate_ng::{PreparedRelateState, RelateNG};
use crate::relate::relateng::relate_predicate;
use crate::relate::relateng::topology_predicate::TopologyPredicate;
use crate::{BoundingRect, GeometryCow, HasDimensions};
use crate::{Contains, ContainsProperly, Covers, GeoFloat, Intersects, Relate};

use std::cell::OnceCell;
use std::fmt::{Debug, Formatter};

use crate::dimensions::Dimensions;
use rstar::RTreeNum;

/// A `PreparedGeometry` caches the spatial indexes and derived state that
/// topological comparisons use: a segment
/// [R-tree](https://en.wikipedia.org/wiki/R-tree), per-polygon
/// point-in-area locators, per-line envelopes and boundary points, and the
/// set of unique points. They are built on the first comparison that needs
/// them and are reused by later comparisons, so a `PreparedGeometry` can be more
/// efficient than a plain `Geometry` when it is compared many times.
///
/// The predicate traits [`Intersects`], [`Contains`], [`Covers`] and
/// [`ContainsProperly`] are implemented against any geometry that
/// implements [`Relate`]. They use the cached state and stop as soon as
/// the result is known, so they are cheaper than computing the full
/// matrix with [`Relate::relate`] and testing it.
///
/// ```
/// use geo::{Contains, Intersects, Relate, PreparedGeometry, wkt};
///
/// let polygon = wkt! { POLYGON((2.0 2.0,2.0 6.0,4.0 6.0)) };
/// let touching_line = wkt! { LINESTRING(0.0 0.0,2.0 2.0) };
/// let intersecting_line = wkt! { LINESTRING(0.0 0.0,3.0 3.0) };
/// let contained_line = wkt! { LINESTRING(2.0 2.0,3.0 5.0) };
///
/// let prepared_polygon = PreparedGeometry::from(polygon);
/// assert!(prepared_polygon.relate(&touching_line).is_touches());
/// assert!(prepared_polygon.intersects(&intersecting_line));
/// assert!(prepared_polygon.contains(&contained_line));
///
/// ```
pub struct PreparedGeometry<'a, G, F = f64>
where
    G: Into<GeometryCow<'a, F>>,
    F: GeoFloat + RTreeNum,
{
    pub(crate) geometry: G,
    geometry_cow: GeometryCow<'a, F>,
    /// The legacy noded graph, built on the first call of the deprecated
    /// `Relate::geometry_graph`.
    geometry_graph: OnceCell<GeometryGraph<'a, F>>,
    /// The RelateNG caches reused across `relate` calls: the geometry
    /// metadata, segment index, point-locator state, and unique points.
    relate_state: PreparedRelateState<F>,
}

impl<'a, G, F> Clone for PreparedGeometry<'a, G, F>
where
    G: Into<GeometryCow<'a, F>> + Clone,
    F: GeoFloat + RTreeNum,
{
    fn clone(&self) -> Self {
        // The caches are rebuildable; a clone starts with fresh (empty)
        // ones.
        Self {
            geometry: self.geometry.clone(),
            geometry_cow: self.geometry_cow.clone(),
            geometry_graph: OnceCell::new(),
            relate_state: PreparedRelateState::default(),
        }
    }
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

pub(crate) fn prepare_geometry<'a, F, T>(geometry: T) -> PreparedGeometry<'a, T, F>
where
    F: GeoFloat,
    T: Clone + Into<GeometryCow<'a, F>>,
{
    PreparedGeometry {
        geometry_cow: geometry.clone().into(),
        geometry,
        geometry_graph: OnceCell::new(),
        relate_state: PreparedRelateState::default(),
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
        self.relate_state.meta(&self.geometry_cow).envelope()
    }
}

impl<'a, G, F: GeoFloat> HasDimensions for PreparedGeometry<'a, G, F>
where
    F: GeoFloat,
    G: Into<GeometryCow<'a, F>>,
{
    fn is_empty(&self) -> bool {
        self.geometry_cow.is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        self.geometry_cow.dimensions()
    }

    fn boundary_dimensions(&self) -> Dimensions {
        self.geometry_cow.boundary_dimensions()
    }
}

impl<'a, G, F: GeoFloat> Relate<F> for PreparedGeometry<'a, G, F>
where
    F: GeoFloat,
    G: Into<GeometryCow<'a, F>>,
{
    /// Returns a copy of the cached [`GeometryGraph`], which is built on
    /// the first call.
    #[allow(deprecated)]
    fn geometry_graph(&self, arg_index: usize) -> GeometryGraph<'_, F> {
        self.geometry_graph
            .get_or_init(|| GeometryGraph::new(0, self.geometry_cow.clone()))
            .clone_for_arg_index(arg_index)
    }

    fn geometry_cow(&self) -> GeometryCow<'_, F> {
        self.geometry_cow.reborrow()
    }

    /// Relates against the B geometry with the cached prepared state: the
    /// A-side segment index, point-locator state and unique points are
    /// built once and reused across calls.
    fn relate(&self, other: &impl Relate<F>) -> IntersectionMatrix {
        let cow = self.geometry_cow();
        let engine = RelateNG::prepared(&cow, &self.relate_state);
        engine.evaluate_matrix(&other.geometry_cow())
    }
}

impl<'a, G, F> PreparedGeometry<'a, G, F>
where
    F: GeoFloat,
    G: Into<GeometryCow<'a, F>>,
{
    /// Evaluates a topological predicate against the B geometry with the
    /// cached prepared state, stopping as soon as the value is known.
    fn evaluate(&self, other: &impl Relate<F>, predicate: &mut dyn TopologyPredicate<F>) -> bool {
        let cow = self.geometry_cow();
        let engine = RelateNG::prepared(&cow, &self.relate_state);
        engine.evaluate(&other.geometry_cow(), predicate)
    }
}

/// Implements a predicate trait for `PreparedGeometry` against any
/// `Relate` geometry, evaluated with the cached prepared state.
macro_rules! impl_prepared_predicate {
    ($trait:ident, $method:ident, $predicate:expr) => {
        impl<'a, G, F, R> $trait<R> for PreparedGeometry<'a, G, F>
        where
            F: GeoFloat,
            G: Into<GeometryCow<'a, F>>,
            R: Relate<F>,
        {
            fn $method(&self, rhs: &R) -> bool {
                self.evaluate(rhs, &mut $predicate)
            }
        }
    };
}

impl_prepared_predicate!(Intersects, intersects, relate_predicate::intersects());
impl_prepared_predicate!(Contains, contains, relate_predicate::contains());
impl_prepared_predicate!(Covers, covers, relate_predicate::covers());
impl_prepared_predicate!(
    ContainsProperly,
    contains_properly,
    relate_predicate::contains_properly()
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::Relate;
    use crate::{polygon, wkt};

    #[test]
    fn clone_and_send() {
        fn send(_send: impl Send) {}
        let polygon: Polygon = wkt!(POLYGON EMPTY);
        let prepared = PreparedGeometry::from(polygon);
        send(prepared.clone())
    }

    #[test]
    fn moved_across_threads() {
        let polygon = wkt!(POLYGON((0.0 0.0,2.0 0.0,1.0 1.0,0.0 0.0)));
        let prepared = PreparedGeometry::from(polygon);

        std::thread::spawn(move || {
            let line = wkt!(LINESTRING(0.0 0.0,3.0 3.0));
            assert!(prepared.relate(&line).is_intersects());
        })
        .join()
        .unwrap();
    }

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

    // Not in JTS: repeated relate calls reuse the cached prepared state
    // (segment index, area locators, unique points) and must produce
    // results identical to unprepared evaluation, in both argument
    // positions and against multiple B geometries.
    #[test]
    fn repeated_relates_reuse_cached_state() {
        let a = wkt!(POLYGON((0.0 0.0, 10.0 0.0, 10.0 10.0, 0.0 10.0, 0.0 0.0)));
        let bs = [
            wkt!(POLYGON((2.0 2.0, 4.0 2.0, 4.0 4.0, 2.0 4.0, 2.0 2.0))),
            wkt!(POLYGON((8.0 8.0, 12.0 8.0, 12.0 12.0, 8.0 12.0, 8.0 8.0))),
            wkt!(POLYGON((20.0 20.0, 22.0 20.0, 22.0 22.0, 20.0 22.0, 20.0 20.0))),
        ];
        let prepared = PreparedGeometry::from(&a);
        for _pass in 0..2 {
            for b in &bs {
                assert_eq!(prepared.relate(b), a.relate(b));
                assert_eq!(b.relate(&prepared), b.relate(&a));
            }
        }
    }

    #[test]
    #[allow(deprecated)]
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

    // Not in JTS: the predicate traits on a prepared geometry must agree
    // with the full matrix and with the unprepared predicates, for every
    // argument type, and across repeated calls that reuse the cache.
    #[test]
    fn predicates_agree_with_relate_and_unprepared() {
        use crate::relate::relateng::relate_predicate::intersection_matrix_pattern::CONTAINS_PROPERLY;
        use crate::{Contains, ContainsProperly, Covers, Intersects};

        let a_geoms: Vec<Geometry> = vec![
            wkt!(POLYGON((0. 0., 10. 0., 10. 10., 0. 10., 0. 0.), (4. 4., 6. 4., 6. 6., 4. 6., 4. 4.))).into(),
            wkt!(MULTILINESTRING((0. 0., 10. 0.), (10. 0., 10. 10.), (2. 2., 8. 8.))).into(),
            Geometry::GeometryCollection(wkt!(GEOMETRYCOLLECTION(
                POLYGON((0. 0., 5. 0., 5. 5., 0. 5., 0. 0.)),
                POLYGON((5. 0., 10. 0., 10. 5., 5. 5., 5. 0.)),
                LINESTRING(0. 8., 10. 8.),
                POINT(1. 9.)
            ))),
            wkt!(MULTIPOINT(1. 1., 5. 5., 9. 9.)).into(),
        ];
        let b_geoms: Vec<Geometry> = vec![
            wkt!(POINT(5. 5.)).into(),
            wkt!(POINT(1. 9.)).into(),
            wkt!(POINT(20. 20.)).into(),
            wkt!(LINESTRING(1. 1., 3. 1.)).into(),
            wkt!(LINESTRING(5. 0., 5. 5.)).into(),
            wkt!(LINESTRING(0. 8., 10. 8.)).into(),
            wkt!(LINESTRING(-1. -1., 11. 11.)).into(),
            wkt!(POLYGON((1. 1., 3. 1., 3. 3., 1. 3., 1. 1.))).into(),
            wkt!(POLYGON((4.5 4.5, 5.5 4.5, 5.5 5.5, 4.5 5.5, 4.5 4.5))).into(),
            wkt!(POLYGON((0. 0., 10. 0., 10. 10., 0. 10., 0. 0.))).into(),
            wkt!(MULTIPOINT(1. 1., 5. 5.)).into(),
            wkt!(MULTIPOINT(1. 1., 20. 20.)).into(),
            Geometry::GeometryCollection(
                wkt!(GEOMETRYCOLLECTION(POINT(2. 2.), LINESTRING(2. 2., 3. 3.))),
            ),
            Geometry::GeometryCollection(wkt!(GEOMETRYCOLLECTION EMPTY)),
        ];
        for a in &a_geoms {
            let prepared = PreparedGeometry::from(a);
            for _pass in 0..2 {
                for b in &b_geoms {
                    let im = a.relate(b);
                    let ctx = format!("A = {a:?}, B = {b:?}");
                    assert_eq!(
                        prepared.intersects(b),
                        im.is_intersects(),
                        "intersects: {ctx}"
                    );
                    assert_eq!(prepared.contains(b), im.is_contains(), "contains: {ctx}");
                    assert_eq!(prepared.covers(b), im.is_covers(), "covers: {ctx}");
                    assert_eq!(
                        prepared.contains_properly(b),
                        im.matches(CONTAINS_PROPERLY).unwrap(),
                        "contains_properly: {ctx}"
                    );
                    assert_eq!(
                        prepared.intersects(b),
                        a.intersects(b),
                        "intersects vs unprepared: {ctx}"
                    );
                    assert_eq!(
                        prepared.contains(b),
                        a.contains(b),
                        "contains vs unprepared: {ctx}"
                    );
                }
            }
        }
    }

    #[test]
    fn predicates_accept_prepared_arguments() {
        let a = wkt!(POLYGON((0. 0., 10. 0., 10. 10., 0. 10., 0. 0.)));
        let b = wkt!(POLYGON((2. 2., 4. 2., 4. 4., 2. 4., 2. 2.)));
        let prepared_a = PreparedGeometry::from(&a);
        let prepared_b = PreparedGeometry::from(&b);
        assert!(prepared_a.contains(&prepared_b));
        assert!(prepared_a.contains_properly(&prepared_b));
        assert!(prepared_b.intersects(&prepared_a));
        assert!(!prepared_b.covers(&prepared_a));
    }
}
