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
    pub fn polygons(&self) -> Box<dyn Iterator<Item = &Polygon<F>> + '_> {
        match self {
            Polygonal::Polygon(p) => Box::new(std::iter::once(p.as_ref())),
            Polygonal::MultiPolygon(mp) => Box::new(mp.0.iter()),
        }
    }

    /// Simple-mode location: the unindexed mod-2 locator.
    fn coordinate_position(&self, coord: Coord<F>) -> CoordPos {
        match self {
            Polygonal::Polygon(p) => p.coordinate_position(&coord),
            Polygonal::MultiPolygon(mp) => mp.coordinate_position(&coord),
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

/// The elements of an input geometry, decomposed for point location:
/// unique point coordinates, linear components, and polygonal components,
/// in document order.
pub(crate) struct GeometryElements<'a, F: GeoFloat> {
    points: BTreeSet<NodeKey<F>>,
    lines: Vec<Cow<'a, LineString<F>>>,
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

    fn add_line(&mut self, line: &'a LineString<F>) {
        if line.0.is_empty() {
            return;
        }
        self.lines.push(Cow::Borrowed(line));
    }

    fn add_line_segment(&mut self, start: Coord<F>, end: Coord<F>) {
        self.lines
            .push(Cow::Owned(LineString::new(vec![start, end])));
    }

    fn add_polygon(&mut self, polygon: &'a Polygon<F>) {
        if polygon.exterior().0.is_empty() {
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
        if mp.0.iter().all(|p| p.exterior().0.is_empty()) {
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

pub(crate) struct RelatePointLocator<'a, F: GeoFloat> {
    is_prepared: bool,
    is_empty: bool,
    elements: GeometryElements<'a, F>,
    /// Cached envelope per line, for fast rejection.
    line_envelopes: Vec<Option<Rect<F>>>,
    line_boundary: Option<LinearBoundary<F>>,
    /// Prepared mode: lazily built index per polygonal element.
    poly_locators: Vec<OnceCell<IntervalTreeMultiPolygon<F>>>,
    adj_edge_locator: OnceCell<AdjacentEdgeLocator<F>>,
}

impl<'a, F: GeoFloat> RelatePointLocator<'a, F> {
    pub fn new(geom: &'a GeometryCow<'a, F>) -> Self {
        Self::new_with_prepared(geom, false)
    }

    pub fn new_with_prepared(geom: &'a GeometryCow<'a, F>, is_prepared: bool) -> Self {
        let elements = GeometryElements::extract(geom);
        let is_empty = elements.is_empty();
        let line_envelopes = elements.lines.iter().map(|l| l.bounding_rect()).collect();
        let line_boundary = if elements.lines.is_empty() {
            None
        } else {
            Some(LinearBoundary::new(
                elements.lines.iter().map(|l| l.as_ref()),
            ))
        };
        let poly_locators = elements
            .polygonals
            .iter()
            .map(|_| OnceCell::new())
            .collect();
        Self {
            is_prepared,
            is_empty,
            elements,
            line_envelopes,
            line_boundary,
            poly_locators,
            adj_edge_locator: OnceCell::new(),
        }
    }

    /// Whether the linear components have any boundary points.
    pub fn has_boundary(&self) -> bool {
        self.line_boundary
            .as_ref()
            .is_some_and(|lb| lb.has_boundary())
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
        let is_boundary = self
            .line_boundary
            .as_ref()
            .is_some_and(|lb| lb.is_boundary(p));
        if is_boundary {
            DimensionLocation::LineBoundary
        } else {
            DimensionLocation::LineInterior
        }
    }

    /// Locates a point which is known to be a node of the geometry (a
    /// vertex or on an edge). `parent_polygonal_id` identifies the
    /// polygonal element the point is a node of, if any.
    pub fn locate_node(&self, p: Coord<F>, parent_polygonal_id: Option<usize>) -> CoordPos {
        self.locate_node_with_dim(p, parent_polygonal_id).location()
    }

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
        if self
            .line_boundary
            .as_ref()
            .is_some_and(|lb| lb.is_boundary(p))
        {
            return CoordPos::OnBoundary;
        }
        // A node must be on a line, in the interior.
        if is_node {
            return CoordPos::Inside;
        }

        for (line, env) in self.elements.lines.iter().zip(&self.line_envelopes) {
            // Every line has to be checked, since any or all may contain
            // the point.
            let loc = Self::locate_on_line(p, line, env);
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
                .adj_edge_locator
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
        if self.is_prepared {
            self.poly_locators[index]
                .get_or_init(|| polygonal.build_index())
                .containment_parity(p)
        } else {
            polygonal.coordinate_position(p)
        }
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
        let locator = RelatePointLocator::new(&cow);
        assert_eq!(locator.locate_with_dim(Coord { x, y }), expected);
        // Not in JTS: the prepared (indexed) mode must agree with the
        // simple mode.
        let prepared = RelatePointLocator::new_with_prepared(&cow, true);
        assert_eq!(prepared.locate_with_dim(Coord { x, y }), expected);
    }

    fn check_line_end_dim_location(
        gc: &GeometryCollection<f64>,
        x: f64,
        y: f64,
        expected: DimensionLocation,
    ) {
        let cow = GeometryCow::from(gc);
        let locator = RelatePointLocator::new(&cow);
        assert_eq!(locator.locate_line_end_with_dim(Coord { x, y }), expected);
    }

    fn check_node_location(gc: &GeometryCollection<f64>, x: f64, y: f64, expected: CoordPos) {
        let cow = GeometryCow::from(gc);
        let locator = RelatePointLocator::new(&cow);
        assert_eq!(locator.locate_node(Coord { x, y }, None), expected);
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
