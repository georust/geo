//! Locates a point on a geometry, including mixed-type collections,
//! reporting the dimension of the containing element as well.
//!
//! Port of JTS `RelatePointLocator`.
//!
//! GeometryCollections are handled with union semantics: the location of a
//! point is the location of that point on the union of the elements of the
//! collection. For a mixed-dimension collection a point may lie on two
//! elements with different dimensions; the location on the
//! largest-dimension element is reported. For a collection with
//! overlapping or adjacent polygons, points on polygon element boundaries
//! may lie in the effective interior of the collection.
//!
//! Prepared mode uses a lazily built [`IntervalTreeMultiPolygon`] per
//! polygonal element (the JTS `IndexedPointInAreaLocator` role); simple
//! mode locates via the unindexed [`CoordinatePosition`] trait (the JTS
//! `SimplePointInAreaLocator` role). Where JTS identifies the parent
//! polygonal element of a node by reference, this port uses the element's
//! index in document order; `RelateGeometry` assigns the same ids during
//! segment-string extraction.

use std::borrow::Cow;
use std::cell::OnceCell;
use std::collections::BTreeSet;

use crate::bounding_rect::BoundingRect;
use crate::coordinate_position::{CoordPos, CoordinatePosition};
use crate::dimensions::HasDimensions;
use crate::geometry_cow::GeometryCow;
use crate::indexed::IntervalTreeMultiPolygon;
use crate::relate::geomgraph::node_map::NodeKey;
use crate::{
    Coord, GeoFloat, Geometry, Intersects, LineString, LinesIter, MultiPolygon, Polygon, Rect,
};

use super::adjacent_edge_locator::AdjacentEdgeLocator;
use super::dimension_location::DimensionLocation;
use super::linear_boundary::LinearBoundary;

/// A polygonal element of the input geometry: a Polygon or MultiPolygon.
/// Rect and Triangle inputs are converted to owned Polygons.
#[derive(Clone)]
pub(crate) enum Polygonal<'a, F: GeoFloat> {
    Polygon(Cow<'a, Polygon<F>>),
    MultiPolygon(&'a MultiPolygon<F>),
}

impl<'a, F: GeoFloat> Polygonal<'a, F> {
    pub fn polygons(&self) -> &[Polygon<F>] {
        match self {
            Polygonal::Polygon(p) => std::slice::from_ref(p.as_ref()),
            Polygonal::MultiPolygon(mp) => &mp.0,
        }
    }

    /// Simple-mode location, with JTS `SimplePointInAreaLocator` member
    /// semantics: the first non-exterior member location wins. The
    /// `CoordinatePosition` impl for `MultiPolygon` is deliberately not
    /// used: it applies the mod-2 rule across member boundaries, so a
    /// point where two members touch would be classified exterior instead
    /// of on the boundary.
    fn coordinate_position(&self, coord: Coord<F>) -> CoordPos {
        match self {
            Polygonal::Polygon(p) => p.coordinate_position(&coord),
            Polygonal::MultiPolygon(mp) => {
                for polygon in &mp.0 {
                    let loc = polygon.coordinate_position(&coord);
                    if loc != CoordPos::Outside {
                        return loc;
                    }
                }
                CoordPos::Outside
            }
        }
    }

    /// Prepared-mode index over the element's segments.
    fn build_index(&self) -> IntervalTreeMultiPolygon<F> {
        match self {
            Polygonal::Polygon(p) => IntervalTreeMultiPolygon::from_polygon(p),
            Polygonal::MultiPolygon(mp) => IntervalTreeMultiPolygon::new(mp),
        }
    }
}

/// A linear element of the input geometry, with its envelope cached for
/// fast rejection. A Line input is promoted to a two-point LineString.
pub(crate) struct LineElement<'a, F: GeoFloat> {
    pub line: Cow<'a, LineString<F>>,
    pub env: Option<Rect<F>>,
}

/// The elements of an input geometry, decomposed for point location:
/// unique point coordinates, linear components, and polygonal components,
/// in document order.
pub(crate) struct GeometryElements<'a, F: GeoFloat> {
    points: BTreeSet<NodeKey<F>>,
    lines: Vec<LineElement<'a, F>>,
    polygonals: Vec<Polygonal<'a, F>>,
    is_polygonal_input: bool,
}

impl<'a, F: GeoFloat> GeometryElements<'a, F> {
    pub fn extract(geom: &'a GeometryCow<'a, F>) -> Self {
        let mut elements = Self {
            points: BTreeSet::new(),
            lines: Vec::new(),
            polygonals: Vec::new(),
            is_polygonal_input: matches!(
                geom,
                GeometryCow::Polygon(_)
                    | GeometryCow::MultiPolygon(_)
                    | GeometryCow::Rect(_)
                    | GeometryCow::Triangle(_)
            ),
        };
        match geom {
            GeometryCow::Point(p) => elements.add_point(p.0),
            GeometryCow::Line(l) => elements.add_line_segment(l.start, l.end),
            GeometryCow::LineString(ls) => elements.add_line(ls),
            GeometryCow::Polygon(p) => elements.add_polygon(p),
            GeometryCow::MultiPoint(mp) => {
                for p in &mp.0 {
                    elements.add_point(p.0);
                }
            }
            GeometryCow::MultiLineString(mls) => {
                for ls in &mls.0 {
                    elements.add_line(ls);
                }
            }
            GeometryCow::MultiPolygon(mp) => elements.add_multi_polygon(mp),
            GeometryCow::Rect(r) => elements.add_owned_polygon(r.to_polygon()),
            GeometryCow::Triangle(t) => elements.add_owned_polygon(t.to_polygon()),
            GeometryCow::GeometryCollection(gc) => {
                for g in &gc.0 {
                    elements.extract_geometry(g);
                }
            }
        }
        elements
    }

    /// Collection members are walked over `Geometry` rather than a
    /// borrowed `GeometryCow` view so that the element borrows keep the
    /// input lifetime.
    fn extract_geometry(&mut self, geom: &'a Geometry<F>) {
        match geom {
            Geometry::Point(p) => self.add_point(p.0),
            Geometry::Line(l) => self.add_line_segment(l.start, l.end),
            Geometry::LineString(ls) => self.add_line(ls),
            Geometry::Polygon(p) => self.add_polygon(p),
            Geometry::MultiPoint(mp) => {
                for p in &mp.0 {
                    self.add_point(p.0);
                }
            }
            Geometry::MultiLineString(mls) => {
                for ls in &mls.0 {
                    self.add_line(ls);
                }
            }
            Geometry::MultiPolygon(mp) => self.add_multi_polygon(mp),
            Geometry::Rect(r) => self.add_owned_polygon(r.to_polygon()),
            Geometry::Triangle(t) => self.add_owned_polygon(t.to_polygon()),
            Geometry::GeometryCollection(gc) => {
                for g in &gc.0 {
                    self.extract_geometry(g);
                }
            }
        }
    }

    fn add_point(&mut self, p: Coord<F>) {
        self.points.insert(NodeKey(p));
    }

    // Envelopes are filled in by the locator, which may take them from
    // the prepared cache instead of walking the coordinates.
    fn add_line(&mut self, line: &'a LineString<F>) {
        if line.0.is_empty() {
            return;
        }
        self.lines.push(LineElement {
            line: Cow::Borrowed(line),
            env: None,
        });
    }

    fn add_line_segment(&mut self, start: Coord<F>, end: Coord<F>) {
        self.lines.push(LineElement {
            line: Cow::Owned(LineString::new(vec![start, end])),
            env: None,
        });
    }

    fn add_polygon(&mut self, polygon: &'a Polygon<F>) {
        if polygon.is_empty() {
            return;
        }
        self.polygonals
            .push(Polygonal::Polygon(Cow::Borrowed(polygon)));
    }

    fn add_owned_polygon(&mut self, polygon: Polygon<F>) {
        self.polygonals
            .push(Polygonal::Polygon(Cow::Owned(polygon)));
    }

    fn add_multi_polygon(&mut self, mp: &'a MultiPolygon<F>) {
        if mp.is_empty() {
            return;
        }
        self.polygonals.push(Polygonal::MultiPolygon(mp));
    }

    pub fn polygonals(&self) -> &[Polygonal<'a, F>] {
        &self.polygonals
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty() && self.lines.is_empty() && self.polygonals.is_empty()
    }
}

/// The parts of a point locator that cost a walk of the geometry's
/// coordinates, owned by prepared state and reused by every locator
/// constructed over the same geometry. Each part is built on first use.
/// Per-element entries are in document order, so they line up with the
/// elements a locator extracts from the same geometry.
///
/// The locator itself borrows the geometry and so cannot be stored next
/// to it in a prepared geometry; with this cache a per-evaluation locator
/// costs a walk of the elements rather than of the coordinates.
pub(crate) struct PreparedLocatorCache<F: GeoFloat> {
    /// The envelope of each linear element.
    line_envelopes: OnceCell<Vec<Option<Rect<F>>>>,
    /// The boundary points of the linear elements; `None` when there are
    /// no linear elements.
    line_boundary: OnceCell<Option<LinearBoundary<F>>>,
    /// One area index cell per polygonal element.
    area_locators: OnceCell<Vec<OnceCell<IntervalTreeMultiPolygon<F>>>>,
    adj_edge_locator: OnceCell<AdjacentEdgeLocator<F>>,
}

impl<F: GeoFloat> Default for PreparedLocatorCache<F> {
    fn default() -> Self {
        Self {
            line_envelopes: OnceCell::new(),
            line_boundary: OnceCell::new(),
            area_locators: OnceCell::new(),
            adj_edge_locator: OnceCell::new(),
        }
    }
}

/// How an input geometry is evaluated.
#[derive(Clone, Copy)]
pub(crate) enum Mode<'a, F: GeoFloat> {
    /// A single evaluation: point-in-area location scans the rings, and
    /// the edge index is filtered to the opposing envelope.
    Simple,
    /// Repeated evaluations against the same geometry: point-in-area
    /// location is indexed, and the locator state that costs a coordinate
    /// walk is taken from a cache that outlives the evaluation.
    Prepared(&'a PreparedLocatorCache<F>),
}

impl<F: GeoFloat> Mode<'_, F> {
    pub fn is_prepared(self) -> bool {
        matches!(self, Mode::Prepared(_))
    }
}

pub(crate) struct RelatePointLocator<'a, F: GeoFloat> {
    is_empty: bool,
    elements: GeometryElements<'a, F>,
    /// Owned in simple mode, borrowed from the cache in prepared mode.
    line_boundary: Cow<'a, Option<LinearBoundary<F>>>,
    /// Prepared mode: the lazily built index per polygonal element.
    poly_locators: Option<&'a [OnceCell<IntervalTreeMultiPolygon<F>>]>,
    /// Prepared mode: the cached adjacent-edge locator cell.
    shared_adj_edge_locator: Option<&'a OnceCell<AdjacentEdgeLocator<F>>>,
    /// Simple mode: the adjacent-edge locator cell.
    own_adj_edge_locator: OnceCell<AdjacentEdgeLocator<F>>,
}

impl<'a, F: GeoFloat> RelatePointLocator<'a, F> {
    /// In prepared mode the cache must belong to the same geometry.
    pub fn new(geom: &'a GeometryCow<'a, F>, mode: Mode<'a, F>) -> Self {
        let mut elements = GeometryElements::extract(geom);
        let is_empty = elements.is_empty();
        let lines = &mut elements.lines;
        let (line_boundary, poly_locators, shared_adj_edge_locator) = match mode {
            Mode::Simple => {
                for elem in lines.iter_mut() {
                    elem.env = elem.line.bounding_rect();
                }
                (Cow::Owned(line_boundary(lines)), None, None)
            }
            Mode::Prepared(cache) => {
                let envs = cache
                    .line_envelopes
                    .get_or_init(|| lines.iter().map(|l| l.line.bounding_rect()).collect());
                for (elem, env) in lines.iter_mut().zip(envs) {
                    elem.env = *env;
                }
                let boundary = cache.line_boundary.get_or_init(|| line_boundary(lines));
                let cells = cache.area_locators.get_or_init(|| {
                    (0..elements.polygonals.len())
                        .map(|_| OnceCell::new())
                        .collect()
                });
                (
                    Cow::Borrowed(boundary),
                    Some(cells.as_slice()),
                    Some(&cache.adj_edge_locator),
                )
            }
        };
        Self {
            is_empty,
            elements,
            line_boundary,
            poly_locators,
            shared_adj_edge_locator,
            own_adj_edge_locator: OnceCell::new(),
        }
    }

    pub fn lines(&self) -> &[LineElement<'a, F>] {
        &self.elements.lines
    }

    pub fn polygonals(&self) -> &[Polygonal<'a, F>] {
        self.elements.polygonals()
    }

    fn line_boundary(&self) -> Option<&LinearBoundary<F>> {
        Option::as_ref(&self.line_boundary)
    }

    /// Whether the linear components have any boundary points.
    pub fn has_boundary(&self) -> bool {
        self.line_boundary().is_some_and(|lb| lb.has_boundary())
    }

    pub fn locate(&self, p: Coord<F>) -> CoordPos {
        self.locate_with_dim(p).location()
    }

    /// Locates a line endpoint. In a mixed-dimension collection the line
    /// end point may also lie in an area; in that case the area location
    /// is reported. Otherwise the location is the line boundary or
    /// interior, depending on the endpoint valence.
    pub fn locate_line_end_with_dim(&self, p: Coord<F>) -> DimensionLocation {
        // If a collection with areas, check for the point on an area.
        if !self.elements.polygonals.is_empty() {
            let loc_poly = self.locate_on_polygons(p, false, None);
            if loc_poly != CoordPos::Outside {
                return DimensionLocation::from_location_area(loc_poly);
            }
        }
        // Not in an area, so return the line end location.
        let is_boundary = self.line_boundary().is_some_and(|lb| lb.is_boundary(p));
        if is_boundary {
            DimensionLocation::LineBoundary
        } else {
            DimensionLocation::LineInterior
        }
    }

    /// Locates a point which is known to be a node of the geometry (a
    /// vertex or on an edge). `parent_polygonal_id` identifies the
    /// polygonal element the point is a node of, if any.
    pub fn locate_node_with_dim(
        &self,
        p: Coord<F>,
        parent_polygonal_id: Option<usize>,
    ) -> DimensionLocation {
        self.locate_with_dim_impl(p, true, parent_polygonal_id)
    }

    pub fn locate_with_dim(&self, p: Coord<F>) -> DimensionLocation {
        self.locate_with_dim_impl(p, false, None)
    }

    fn locate_with_dim_impl(
        &self,
        p: Coord<F>,
        is_node: bool,
        parent_polygonal_id: Option<usize>,
    ) -> DimensionLocation {
        if self.is_empty {
            return DimensionLocation::Exterior;
        }

        // In a purely polygonal geometry a node must be on the boundary.
        // (This is not the case for a mixed collection, since the node may
        // be in the interior of a polygon.)
        if is_node && self.elements.is_polygonal_input {
            return DimensionLocation::AreaBoundary;
        }

        self.compute_dim_location(p, is_node, parent_polygonal_id)
    }

    /// Checks the dimensions in order of precedence: area, line, point.
    fn compute_dim_location(
        &self,
        p: Coord<F>,
        is_node: bool,
        parent_polygonal_id: Option<usize>,
    ) -> DimensionLocation {
        if !self.elements.polygonals.is_empty() {
            let loc_poly = self.locate_on_polygons(p, is_node, parent_polygonal_id);
            if loc_poly != CoordPos::Outside {
                return DimensionLocation::from_location_area(loc_poly);
            }
        }
        if !self.elements.lines.is_empty() {
            let loc_line = self.locate_on_lines(p, is_node);
            if loc_line != CoordPos::Outside {
                return DimensionLocation::from_location_line(loc_line);
            }
        }
        if !self.elements.points.is_empty() {
            let loc_pt = self.locate_on_points(p);
            if loc_pt != CoordPos::Outside {
                return DimensionLocation::from_location_point(loc_pt);
            }
        }
        DimensionLocation::Exterior
    }

    fn locate_on_points(&self, p: Coord<F>) -> CoordPos {
        if self.elements.points.contains(&NodeKey(p)) {
            CoordPos::Inside
        } else {
            CoordPos::Outside
        }
    }

    fn locate_on_lines(&self, p: Coord<F>, is_node: bool) -> CoordPos {
        if self.line_boundary().is_some_and(|lb| lb.is_boundary(p)) {
            return CoordPos::OnBoundary;
        }
        // A node must be on a line, in the interior.
        if is_node {
            return CoordPos::Inside;
        }

        for elem in &self.elements.lines {
            // Every line has to be checked, since any or all may contain
            // the point.
            let loc = Self::locate_on_line(p, &elem.line, &elem.env);
            if loc != CoordPos::Outside {
                return loc;
            }
        }
        CoordPos::Outside
    }

    fn locate_on_line(p: Coord<F>, line: &LineString<F>, env: &Option<Rect<F>>) -> CoordPos {
        // Bounding-box fast rejection.
        if let Some(env) = env
            && !env.intersects(&p)
        {
            return CoordPos::Outside;
        }
        if line.lines_iter().any(|seg| seg.intersects(&p)) {
            CoordPos::Inside
        } else {
            CoordPos::Outside
        }
    }

    fn locate_on_polygons(
        &self,
        p: Coord<F>,
        is_node: bool,
        parent_polygonal_id: Option<usize>,
    ) -> CoordPos {
        let mut num_boundaries = 0;
        for i in 0..self.elements.polygonals.len() {
            let loc = self.locate_on_polygonal(p, is_node, parent_polygonal_id, i);
            match loc {
                CoordPos::Inside => return CoordPos::Inside,
                CoordPos::OnBoundary => num_boundaries += 1,
                CoordPos::Outside => {}
            }
        }
        if num_boundaries == 1 {
            CoordPos::OnBoundary
        } else if num_boundaries > 1 {
            // The point lies on more than one polygon boundary: determine
            // the effective location from the adjacent edges.
            let adj_locator = self
                .shared_adj_edge_locator
                .unwrap_or(&self.own_adj_edge_locator)
                .get_or_init(|| AdjacentEdgeLocator::new(&self.elements.polygonals));
            adj_locator.locate(p)
        } else {
            CoordPos::Outside
        }
    }

    fn locate_on_polygonal(
        &self,
        p: Coord<F>,
        is_node: bool,
        parent_polygonal_id: Option<usize>,
        index: usize,
    ) -> CoordPos {
        if is_node && parent_polygonal_id == Some(index) {
            return CoordPos::OnBoundary;
        }
        let polygonal = &self.elements.polygonals[index];
        match self.poly_locators {
            Some(cells) => cells[index]
                .get_or_init(|| polygonal.build_index())
                .containment_parity(p),
            None => polygonal.coordinate_position(p),
        }
    }
}

/// The boundary of the linear elements; `None` when there are none.
fn line_boundary<F: GeoFloat>(lines: &[LineElement<'_, F>]) -> Option<LinearBoundary<F>> {
    if lines.is_empty() {
        None
    } else {
        Some(LinearBoundary::new(lines.iter().map(|l| l.line.as_ref())))
    }
}

#[cfg(test)]
mod tests {
    // Tests ported from JTS RelatePointLocatorTest.java (master, ab57bff).
    use super::*;
    use crate::GeometryCollection;
    use crate::wkt;

    fn gc_pla() -> GeometryCollection<f64> {
        wkt!(GEOMETRYCOLLECTION (
            POINT (1. 1.),
            POINT (2. 1.),
            LINESTRING (3. 1., 3. 9.),
            LINESTRING (4. 1., 5. 4., 7. 1., 4. 1.),
            LINESTRING (12. 12., 14. 14.),
            POLYGON ((6. 5., 6. 9., 9. 9., 9. 5., 6. 5.)),
            POLYGON ((10. 10., 10. 16., 16. 16., 16. 10., 10. 10.)),
            POLYGON ((11. 11., 11. 17., 17. 17., 17. 11., 11. 11.)),
            POLYGON ((12. 12., 12. 16., 16. 16., 16. 12., 12. 12.))
        ))
    }

    fn check_dim_location(
        gc: &GeometryCollection<f64>,
        x: f64,
        y: f64,
        expected: DimensionLocation,
    ) {
        let cow = GeometryCow::from(gc);
        let locator = RelatePointLocator::new(&cow, Mode::Simple);
        assert_eq!(locator.locate_with_dim(Coord { x, y }), expected);
        // Not in JTS: the prepared (indexed) mode must agree with the
        // simple mode, both when the cache is empty and when a second
        // locator reuses the cache filled by the first.
        let cache = PreparedLocatorCache::default();
        let prepared = RelatePointLocator::new(&cow, Mode::Prepared(&cache));
        assert_eq!(prepared.locate_with_dim(Coord { x, y }), expected);
        let reused = RelatePointLocator::new(&cow, Mode::Prepared(&cache));
        assert_eq!(reused.locate_with_dim(Coord { x, y }), expected);
    }

    fn check_line_end_dim_location(
        gc: &GeometryCollection<f64>,
        x: f64,
        y: f64,
        expected: DimensionLocation,
    ) {
        let cow = GeometryCow::from(gc);
        let locator = RelatePointLocator::new(&cow, Mode::Simple);
        assert_eq!(locator.locate_line_end_with_dim(Coord { x, y }), expected);
    }

    fn check_node_location(gc: &GeometryCollection<f64>, x: f64, y: f64, expected: CoordPos) {
        let cow = GeometryCow::from(gc);
        let locator = RelatePointLocator::new(&cow, Mode::Simple);
        assert_eq!(
            locator
                .locate_node_with_dim(Coord { x, y }, None)
                .location(),
            expected
        );
    }

    #[test]
    fn test_point() {
        let gc = gc_pla();
        check_dim_location(&gc, 1., 1., DimensionLocation::PointInterior);
        check_dim_location(&gc, 0., 1., DimensionLocation::Exterior);
    }

    #[test]
    fn test_point_in_line() {
        check_dim_location(&gc_pla(), 3., 8., DimensionLocation::LineInterior);
    }

    #[test]
    fn test_point_in_area() {
        check_dim_location(&gc_pla(), 8., 8., DimensionLocation::AreaInterior);
    }

    #[test]
    fn test_line() {
        let gc = gc_pla();
        check_dim_location(&gc, 3., 3., DimensionLocation::LineInterior);
        check_dim_location(&gc, 3., 1., DimensionLocation::LineBoundary);
    }

    #[test]
    fn test_line_in_area() {
        let gc = gc_pla();
        check_dim_location(&gc, 11., 11., DimensionLocation::AreaInterior);
        check_dim_location(&gc, 14., 14., DimensionLocation::AreaInterior);
    }

    #[test]
    fn test_area() {
        let gc = gc_pla();
        check_dim_location(&gc, 8., 8., DimensionLocation::AreaInterior);
        check_dim_location(&gc, 9., 9., DimensionLocation::AreaBoundary);
    }

    #[test]
    fn test_area_in_area() {
        let gc = gc_pla();
        check_dim_location(&gc, 11., 11., DimensionLocation::AreaInterior);
        check_dim_location(&gc, 12., 12., DimensionLocation::AreaInterior);
        check_dim_location(&gc, 10., 10., DimensionLocation::AreaBoundary);
        check_dim_location(&gc, 16., 16., DimensionLocation::AreaInterior);
    }

    #[test]
    fn test_line_node() {
        check_node_location(&gc_pla(), 3., 1., CoordPos::OnBoundary);
    }

    #[test]
    fn test_line_end_in_gc_la() {
        let gc: GeometryCollection<f64> = wkt!(GEOMETRYCOLLECTION (
            POLYGON ((0. 0., 10. 0., 10. 10., 0. 10., 0. 0.)),
            LINESTRING (12. 2., 0. 2., 0. 5., 5. 5.),
            LINESTRING (12. 10., 12. 2.)
        ));
        check_line_end_dim_location(&gc, 5., 5., DimensionLocation::AreaInterior);
        check_line_end_dim_location(&gc, 12., 2., DimensionLocation::LineInterior);
        check_line_end_dim_location(&gc, 12., 10., DimensionLocation::LineBoundary);
    }
}
