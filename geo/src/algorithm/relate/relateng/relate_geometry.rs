//! One input geometry of a relate evaluation, with cached metadata and the
//! machinery to locate points on it and extract its edges.
//!
//! Port of JTS `RelateGeometry`.
//!
//! Dimension semantics follow JTS: `dimension()` is the type-based
//! dimension (an empty polygon still has dimension 2, and empty elements
//! of a collection count); `dimension_real()` is the dimension of the
//! non-empty content, with all-zero-length linear geometry demoted to
//! dimension 0.
//!
//! Polygonal elements are identified by their index in document order;
//! [`super::relate_point_locator::GeometryElements`] assigns the same ids,
//! so node sections and the point locator agree on which element a node
//! belongs to.

use std::cell::OnceCell;
use std::collections::BTreeSet;

use crate::bounding_rect::BoundingRect;
use crate::coordinate_position::CoordPos;
use crate::dimensions::{Dimensions, HasDimensions};
use crate::geometry_cow::GeometryCow;
use crate::relate::geomgraph::node_map::NodeKey;
use crate::winding_order::Winding;
use crate::{Coord, GeoFloat, Intersects, LineString, MultiPolygon, Polygon, Rect};

use super::dimension_location::DimensionLocation;
use super::relate_point_locator::{LineElement, Mode, Polygonal, RelatePointLocator};
use super::relate_segment_string::RelateSegmentString;
use super::topology_predicate::InputIndex;

/// The metadata of a relate input that costs a walk of its coordinates:
/// the envelope and the dimension analysis. A prepared geometry computes
/// it once and reuses it across evaluations.
#[derive(Clone, Copy)]
pub(crate) struct GeometryMeta<F: GeoFloat> {
    env: Option<Rect<F>>,
    geom_dim: Dimensions,
    has_points: bool,
    has_lines: bool,
    has_areas: bool,
    is_line_zero_len: bool,
    is_geom_empty: bool,
}

impl<F: GeoFloat> GeometryMeta<F> {
    pub fn of(geom: &GeometryCow<'_, F>) -> Self {
        let is_geom_empty = geom.is_empty();
        let mut analysis = DimensionAnalysis {
            // The type-based dimension, which counts empty elements.
            dim: type_dimension(geom),
            has_points: false,
            has_lines: false,
            has_areas: false,
        };
        if !is_geom_empty {
            analysis.analyze(geom);
        }
        let geom_dim = analysis.dim;
        Self {
            env: geom.bounding_rect(),
            geom_dim,
            has_points: analysis.has_points,
            has_lines: analysis.has_lines,
            has_areas: analysis.has_areas,
            is_line_zero_len: geom_dim == Dimensions::OneDimensional && is_zero_length(geom),
            is_geom_empty,
        }
    }

    /// The geometry envelope; `None` for an empty geometry.
    pub fn envelope(&self) -> Option<Rect<F>> {
        self.env
    }
}

pub(crate) struct RelateGeometry<'a, F: GeoFloat> {
    geom: &'a GeometryCow<'a, F>,
    mode: Mode<'a, F>,
    meta: GeometryMeta<F>,
    unique_points: OnceCell<BTreeSet<NodeKey<F>>>,
    locator: OnceCell<RelatePointLocator<'a, F>>,
}

impl<'a, F: GeoFloat> RelateGeometry<'a, F> {
    pub fn new(geom: &'a GeometryCow<'a, F>, mode: Mode<'a, F>) -> Self {
        Self::with_meta(geom, mode, GeometryMeta::of(geom))
    }

    /// An instance over metadata computed earlier for the same geometry.
    pub fn with_meta(
        geom: &'a GeometryCow<'a, F>,
        mode: Mode<'a, F>,
        meta: GeometryMeta<F>,
    ) -> Self {
        Self {
            geom,
            mode,
            meta,
            unique_points: OnceCell::new(),
            locator: OnceCell::new(),
        }
    }

    pub fn is_prepared(&self) -> bool {
        self.mode.is_prepared()
    }

    /// The geometry envelope; `None` for an empty geometry (the JTS null
    /// envelope).
    pub fn envelope(&self) -> Option<Rect<F>> {
        self.meta.env
    }

    /// The type-based dimension of the geometry.
    pub fn dimension(&self) -> Dimensions {
        self.meta.geom_dim
    }

    pub fn has_dimension(&self, dim: Dimensions) -> bool {
        match dim {
            Dimensions::ZeroDimensional => self.meta.has_points,
            Dimensions::OneDimensional => self.meta.has_lines,
            Dimensions::TwoDimensional => self.meta.has_areas,
            Dimensions::Empty => false,
        }
    }

    pub fn has_area_and_line(&self) -> bool {
        self.meta.has_areas && self.meta.has_lines
    }

    /// The actual non-empty dimension of the geometry. Zero-length
    /// linestrings are treated as points; an empty geometry has dimension
    /// `Empty`.
    pub fn dimension_real(&self) -> Dimensions {
        if self.meta.is_geom_empty {
            return Dimensions::Empty;
        }
        if self.meta.geom_dim == Dimensions::OneDimensional && self.meta.is_line_zero_len {
            return Dimensions::ZeroDimensional;
        }
        if self.meta.has_areas {
            return Dimensions::TwoDimensional;
        }
        if self.meta.has_lines {
            return Dimensions::OneDimensional;
        }
        Dimensions::ZeroDimensional
    }

    pub fn has_edges(&self) -> bool {
        self.meta.has_lines || self.meta.has_areas
    }

    fn locator(&self) -> &RelatePointLocator<'a, F> {
        self.locator
            .get_or_init(|| RelatePointLocator::new(self.geom, self.mode))
    }

    /// The non-empty linear elements, in document order.
    pub fn lines(&self) -> &[LineElement<'a, F>] {
        self.locator().lines()
    }

    /// The non-empty polygonal elements, in document order.
    pub fn polygonals(&self) -> &[Polygonal<'a, F>] {
        self.locator().polygonals()
    }

    /// Whether a node point lies in the interior of the geometry's area,
    /// excluding the polygonal element the node belongs to. This occurs
    /// for nodes of a geometry inside an overlapping polygon of a
    /// GeometryCollection.
    pub fn is_node_in_area(&self, node_pt: Coord<F>, parent_polygonal_id: Option<usize>) -> bool {
        let dim_loc = self.locate_node_with_dim(node_pt, parent_polygonal_id);
        dim_loc == DimensionLocation::AreaInterior
    }

    /// Locates a node point, hoisting the locator's own O(1) fast paths so
    /// the locator is not built when they decide the answer: an empty
    /// geometry has only exterior, and a node of a purely polygonal
    /// geometry is always on its boundary. The conditions mirror
    /// `RelatePointLocator::locate_with_dim_impl` exactly.
    fn locate_node_with_dim(
        &self,
        pt: Coord<F>,
        parent_polygonal_id: Option<usize>,
    ) -> DimensionLocation {
        if self.meta.is_geom_empty {
            return DimensionLocation::Exterior;
        }
        if self.is_polygonal() {
            return DimensionLocation::AreaBoundary;
        }
        self.locator().locate_node_with_dim(pt, parent_polygonal_id)
    }

    pub fn locate_line_end_with_dim(&self, p: Coord<F>) -> DimensionLocation {
        self.locator().locate_line_end_with_dim(p)
    }

    /// Locates a vertex of a polygon. A vertex of a Polygon or
    /// MultiPolygon is on the boundary, but a vertex of an overlapped
    /// polygon in a GeometryCollection may be in the interior.
    pub fn locate_area_vertex(&self, pt: Coord<F>) -> CoordPos {
        // No parent element needs to be passed, because the point is an
        // exact vertex, which is detected as being on the boundary of its
        // own polygon.
        self.locate_node(pt, None)
    }

    pub fn locate_node(&self, pt: Coord<F>, parent_polygonal_id: Option<usize>) -> CoordPos {
        self.locate_node_with_dim(pt, parent_polygonal_id)
            .location()
    }

    pub fn locate_with_dim(&self, pt: Coord<F>) -> DimensionLocation {
        // Envelope fast rejection: a point outside the (cached) envelope is
        // exterior without a locator walk. This restores the performance of
        // full-matrix evaluation on envelope-disjoint inputs, where every
        // point-phase locate lands here.
        match self.meta.env {
            Some(env) if env.intersects(&pt) => self.locator().locate_with_dim(pt),
            _ => DimensionLocation::Exterior,
        }
    }

    /// Whether the geometry requires self-noding for correct evaluation of
    /// specific spatial predicates. Self-noding is required for geometries
    /// which may self-cross: lines, and overlapping elements in
    /// GeometryCollections. Polygonal geometries do not require it, since
    /// their rings can only touch at vertices.
    pub fn is_self_noding_required(&self) -> bool {
        match self.geom {
            GeometryCow::Point(_)
            | GeometryCow::MultiPoint(_)
            | GeometryCow::Polygon(_)
            | GeometryCow::MultiPolygon(_)
            | GeometryCow::Rect(_)
            | GeometryCow::Triangle(_) => false,
            GeometryCow::GeometryCollection(gc) => {
                // A collection with a single polygonal element does not
                // need noding; neither does one with only points.
                if self.meta.has_areas && gc.0.len() == 1 {
                    return false;
                }
                self.meta.has_areas || self.meta.has_lines
            }
            // A single Line segment cannot self-cross, but is treated as a
            // LineString for consistency with JTS.
            GeometryCow::Line(_) | GeometryCow::LineString(_) | GeometryCow::MultiLineString(_) => {
                true
            }
        }
    }

    /// Whether the geometry has polygonal topology. This is not the case
    /// for a GeometryCollection containing polygons, since they may
    /// overlap or be adjacent. Polygonal topology allows more assumptions
    /// about the location of boundary vertices.
    pub fn is_polygonal(&self) -> bool {
        matches!(
            self.geom,
            GeometryCow::Polygon(_)
                | GeometryCow::MultiPolygon(_)
                | GeometryCow::Rect(_)
                | GeometryCow::Triangle(_)
        )
    }

    pub fn is_empty(&self) -> bool {
        self.meta.is_geom_empty
    }

    pub fn has_boundary(&self) -> bool {
        self.locator().has_boundary()
    }

    /// The distinct representative coordinates of the geometry's point and
    /// line components (JTS `ComponentCoordinateExtracter`: one coordinate
    /// per component). Zero-length lines are effectively points, so their
    /// coordinate is included. Cached for reuse in prepared mode. Only
    /// used for geometries with real dimension 0.
    pub fn unique_points(&self) -> &BTreeSet<NodeKey<F>> {
        self.unique_points
            .get_or_init(|| self.compute_unique_points())
    }

    /// Computes the unique points without caching; prepared state caches
    /// the result across evaluations.
    pub fn compute_unique_points(&self) -> BTreeSet<NodeKey<F>> {
        let mut set = BTreeSet::new();
        collect_component_coords(self.geom, &mut |c| {
            set.insert(NodeKey(c));
        });
        set
    }

    /// The point-element coordinates that are not covered by a
    /// higher-dimension element of the geometry.
    pub fn effective_points(&self) -> Vec<Coord<F>> {
        let mut pts = Vec::new();
        collect_point_coords(self.geom, &mut |c| pts.push(c));

        if pts.is_empty() || self.dimension_real() <= Dimensions::ZeroDimensional {
            return pts;
        }
        // Only return points not covered by another element.
        pts.into_iter()
            .filter(|&p| self.locate_with_dim(p).dimension() == Dimensions::ZeroDimensional)
            .collect()
    }

    /// Extracts the segment strings of the geometry which intersect a given
    /// envelope. If the envelope is `None`, all edges are extracted.
    ///
    /// Polygonal element ids are assigned in document order over the whole
    /// geometry, independent of the envelope filter, so they are stable
    /// across extractions and match the point locator's element ids.
    pub fn extract_segment_strings(
        &self,
        input: InputIndex,
        env: Option<&Rect<F>>,
    ) -> Vec<RelateSegmentString<F>> {
        let mut extractor = SegmentStringExtractor {
            input,
            env,
            element_id: 0,
            polygonal_count: 0,
            seg_strings: Vec::new(),
        };
        extractor.extract(self.geom);
        extractor.seg_strings
    }
}

struct SegmentStringExtractor<'e, F: GeoFloat> {
    input: InputIndex,
    env: Option<&'e Rect<F>>,
    element_id: i32,
    polygonal_count: usize,
    seg_strings: Vec<RelateSegmentString<F>>,
}

impl<F: GeoFloat> SegmentStringExtractor<'_, F> {
    fn extract(&mut self, geom: &GeometryCow<'_, F>) {
        match geom {
            GeometryCow::Point(_) | GeometryCow::MultiPoint(_) => {}
            GeometryCow::Line(l) => {
                if self.env_intersects(Some(l.bounding_rect())) {
                    self.push_line(vec![l.start, l.end]);
                }
            }
            GeometryCow::LineString(ls) => self.extract_line(ls),
            GeometryCow::Polygon(p) => self.extract_polygon(p),
            GeometryCow::MultiLineString(mls) => {
                for ls in &mls.0 {
                    self.extract_line(ls);
                }
            }
            GeometryCow::MultiPolygon(mp) => self.extract_multi_polygon(mp),
            GeometryCow::Rect(r) => self.extract_polygon(&r.to_polygon()),
            GeometryCow::Triangle(t) => self.extract_polygon(&t.to_polygon()),
            GeometryCow::GeometryCollection(gc) => {
                for g in &gc.0 {
                    self.extract(&GeometryCow::from(g));
                }
            }
        }
    }

    fn extract_line(&mut self, line: &LineString<F>) {
        if line.0.is_empty() || !self.env_intersects(line.bounding_rect()) {
            return;
        }
        self.push_line(line.0.clone());
    }

    fn push_line(&mut self, coords: Vec<Coord<F>>) {
        self.element_id += 1;
        self.seg_strings.push(RelateSegmentString::create_line(
            coords,
            self.input,
            self.element_id,
        ));
    }

    /// A polygon which is a direct element (its own polygonal element).
    fn extract_polygon(&mut self, polygon: &Polygon<F>) {
        if polygon.is_empty() {
            return;
        }
        self.polygonal_count += 1;
        let polygonal_id = self.polygonal_count - 1;
        self.extract_polygon_rings(polygon, polygonal_id);
    }

    /// A MultiPolygon is one polygonal element; its member polygons share
    /// its id.
    fn extract_multi_polygon(&mut self, mp: &MultiPolygon<F>) {
        if mp.is_empty() {
            return;
        }
        self.polygonal_count += 1;
        let polygonal_id = self.polygonal_count - 1;
        for polygon in &mp.0 {
            if polygon.is_empty() {
                continue;
            }
            self.extract_polygon_rings(polygon, polygonal_id);
        }
    }

    fn extract_polygon_rings(&mut self, polygon: &Polygon<F>, polygonal_id: usize) {
        if !self.env_intersects(polygon.exterior().bounding_rect()) {
            // The shell envelope contains the whole polygon.
            return;
        }
        self.element_id += 1;
        self.extract_ring(polygon.exterior(), 0, polygonal_id);
        for (i, hole) in polygon.interiors().iter().enumerate() {
            self.extract_ring(hole, (i + 1) as i32, polygonal_id);
        }
    }

    fn extract_ring(&mut self, ring: &LineString<F>, ring_id: i32, polygonal_id: usize) {
        if ring.0.is_empty() || !self.env_intersects(ring.bounding_rect()) {
            return;
        }
        // Orient the points if required: shells CW, holes CCW.
        let require_cw = ring_id == 0;
        let pts = oriented_ring_coords(ring, require_cw);
        self.seg_strings.push(RelateSegmentString::create_ring(
            pts,
            self.input,
            self.element_id,
            ring_id,
            polygonal_id,
        ));
    }

    /// Whether an element with the given envelope passes the extraction
    /// filter. Without a filter every element passes; with one, an empty
    /// element (no envelope) is rejected.
    fn env_intersects(&self, elem_env: Option<Rect<F>>) -> bool {
        match (self.env, elem_env) {
            (None, _) => true,
            (Some(env), Some(elem_env)) => env.intersects(&elem_env),
            (Some(_), None) => false,
        }
    }
}

/// A copy of the ring's coordinates, oriented CW (for shells) or CCW (for
/// holes). Port of JTS `RelateGeometry.orient`.
pub(crate) fn oriented_ring_coords<F: GeoFloat>(
    ring: &LineString<F>,
    require_cw: bool,
) -> Vec<Coord<F>> {
    let mut ring = ring.clone();
    if require_cw {
        ring.make_cw_winding();
    } else {
        ring.make_ccw_winding();
    }
    ring.0
}

/// The type-based dimension of the geometry: empty geometries and empty
/// collection members count with their type's dimension (JTS
/// `Geometry.getDimension`).
fn type_dimension<F: GeoFloat>(geom: &GeometryCow<'_, F>) -> Dimensions {
    match geom {
        GeometryCow::Point(_) | GeometryCow::MultiPoint(_) => Dimensions::ZeroDimensional,
        GeometryCow::Line(_) | GeometryCow::LineString(_) | GeometryCow::MultiLineString(_) => {
            Dimensions::OneDimensional
        }
        GeometryCow::Polygon(_)
        | GeometryCow::MultiPolygon(_)
        | GeometryCow::Rect(_)
        | GeometryCow::Triangle(_) => Dimensions::TwoDimensional,
        GeometryCow::GeometryCollection(gc) => {
            gc.0.iter()
                .map(|g| type_dimension(&GeometryCow::from(g)))
                .max()
                .unwrap_or(Dimensions::Empty)
        }
    }
}

/// The dimension analysis of a geometry: the dimension raised to that of
/// the highest non-empty element, and which dimensions have non-empty
/// elements (JTS `RelateGeometry.analyzeDimensions`).
struct DimensionAnalysis {
    dim: Dimensions,
    has_points: bool,
    has_lines: bool,
    has_areas: bool,
}

impl DimensionAnalysis {
    fn found(&mut self, dim: Dimensions) {
        match dim {
            Dimensions::ZeroDimensional => self.has_points = true,
            Dimensions::OneDimensional => self.has_lines = true,
            Dimensions::TwoDimensional => self.has_areas = true,
            Dimensions::Empty => {}
        }
        if self.dim < dim {
            self.dim = dim;
        }
    }

    fn analyze<F: GeoFloat>(&mut self, geom: &GeometryCow<'_, F>) {
        match geom {
            GeometryCow::Point(_) => self.found(Dimensions::ZeroDimensional),
            GeometryCow::MultiPoint(mp) => {
                if !mp.0.is_empty() {
                    self.found(Dimensions::ZeroDimensional);
                }
            }
            GeometryCow::Line(_) => self.found(Dimensions::OneDimensional),
            GeometryCow::LineString(ls) => {
                if !ls.0.is_empty() {
                    self.found(Dimensions::OneDimensional);
                }
            }
            GeometryCow::MultiLineString(mls) => {
                if mls.0.iter().any(|ls| !ls.0.is_empty()) {
                    self.found(Dimensions::OneDimensional);
                }
            }
            GeometryCow::Polygon(p) => {
                if !p.is_empty() {
                    self.found(Dimensions::TwoDimensional);
                }
            }
            GeometryCow::MultiPolygon(mp) => {
                if !mp.is_empty() {
                    self.found(Dimensions::TwoDimensional);
                }
            }
            GeometryCow::Rect(_) | GeometryCow::Triangle(_) => {
                self.found(Dimensions::TwoDimensional)
            }
            GeometryCow::GeometryCollection(gc) => {
                for g in &gc.0 {
                    self.analyze(&GeometryCow::from(g));
                }
            }
        }
    }
}

/// Tests if all linear elements are zero-length. For efficiency the test
/// avoids computing actual length.
fn is_zero_length<F: GeoFloat>(geom: &GeometryCow<'_, F>) -> bool {
    match geom {
        GeometryCow::Line(l) => l.start == l.end,
        GeometryCow::LineString(ls) => is_zero_length_line(ls),
        GeometryCow::MultiLineString(mls) => mls.0.iter().all(is_zero_length_line),
        GeometryCow::GeometryCollection(gc) => {
            gc.0.iter().all(|g| is_zero_length(&GeometryCow::from(g)))
        }
        _ => true,
    }
}

fn is_zero_length_line<F: GeoFloat>(line: &LineString<F>) -> bool {
    if line.0.len() >= 2 {
        let p0 = line.0[0];
        // Most non-zero-length lines trigger this right away.
        return line.0.iter().all(|&p| p == p0);
    }
    true
}

/// Applies `f` to one representative coordinate (the first) of every
/// point and line component of the geometry, in document order (the JTS
/// `ComponentCoordinateExtracter` semantic). Polygonal components are not
/// visited; the callers only use this for zero-dimensional geometries.
fn collect_component_coords<F: GeoFloat>(geom: &GeometryCow<'_, F>, f: &mut impl FnMut(Coord<F>)) {
    match geom {
        GeometryCow::Point(p) => f(p.0),
        GeometryCow::Line(l) => f(l.start),
        GeometryCow::LineString(ls) => {
            if let Some(&c) = ls.0.first() {
                f(c);
            }
        }
        GeometryCow::MultiPoint(mp) => {
            for p in &mp.0 {
                f(p.0);
            }
        }
        GeometryCow::MultiLineString(mls) => {
            for ls in &mls.0 {
                if let Some(&c) = ls.0.first() {
                    f(c);
                }
            }
        }
        GeometryCow::GeometryCollection(gc) => {
            for g in &gc.0 {
                collect_component_coords(&GeometryCow::from(g), f);
            }
        }
        _ => {}
    }
}

/// Applies `f` to the coordinate of every point element of the geometry,
/// in document order.
fn collect_point_coords<F: GeoFloat>(geom: &GeometryCow<'_, F>, f: &mut impl FnMut(Coord<F>)) {
    match geom {
        GeometryCow::Point(p) => f(p.0),
        GeometryCow::MultiPoint(mp) => {
            for p in &mp.0 {
                f(p.0);
            }
        }
        GeometryCow::GeometryCollection(gc) => {
            for g in &gc.0 {
                collect_point_coords(&GeometryCow::from(g), f);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    // Tests ported from JTS RelateGeometryTest.java (master, ab57bff).
    use super::*;
    use crate::wkt;

    fn relate_geom_dims(geom: &GeometryCow<'_, f64>) -> (Dimensions, Dimensions) {
        let rgeom = RelateGeometry::new(geom, Mode::Simple);
        (rgeom.dimension(), rgeom.dimension_real())
    }

    #[test]
    fn test_unique_points() {
        let geom = wkt!(MULTIPOINT(0. 0., 5. 5., 5. 0., 0. 0.));
        let cow = GeometryCow::from(&geom);
        let rgeom = RelateGeometry::new(&cow, Mode::Simple);
        assert_eq!(rgeom.unique_points().len(), 3, "Unique pts size");
    }

    #[test]
    fn test_boundary() {
        let geom = wkt!(MULTILINESTRING ((0. 0., 9. 9.), (9. 9., 5. 1.)));
        let cow = GeometryCow::from(&geom);
        let rgeom = RelateGeometry::new(&cow, Mode::Simple);
        assert!(rgeom.has_boundary(), "hasBoundary");
    }

    #[test]
    fn test_has_dimension() {
        let geom = wkt!(GEOMETRYCOLLECTION (
            POLYGON ((1. 9., 5. 9., 5. 5., 1. 5., 1. 9.)),
            LINESTRING (1. 1., 5. 4.),
            POINT (6. 5.)
        ));
        let cow = GeometryCow::from(&geom);
        let rgeom = RelateGeometry::new(&cow, Mode::Simple);
        assert!(rgeom.has_dimension(Dimensions::ZeroDimensional), "dim 0");
        assert!(rgeom.has_dimension(Dimensions::OneDimensional), "dim 1");
        assert!(rgeom.has_dimension(Dimensions::TwoDimensional), "dim 2");
    }

    #[test]
    fn test_dimension() {
        use Dimensions::{OneDimensional, TwoDimensional, ZeroDimensional};

        let point = wkt!(POINT (0. 0.));
        let cow = GeometryCow::from(&point);
        assert_eq!(relate_geom_dims(&cow), (ZeroDimensional, ZeroDimensional));

        let zero_len_line = wkt!(LINESTRING (0. 0., 0. 0.));
        let cow = GeometryCow::from(&zero_len_line);
        assert_eq!(relate_geom_dims(&cow), (OneDimensional, ZeroDimensional));

        let line = wkt!(LINESTRING (0. 0., 9. 9.));
        let cow = GeometryCow::from(&line);
        assert_eq!(relate_geom_dims(&cow), (OneDimensional, OneDimensional));

        let line_repeated = wkt!(LINESTRING (0. 0., 0. 0., 9. 9.));
        let cow = GeometryCow::from(&line_repeated);
        assert_eq!(relate_geom_dims(&cow), (OneDimensional, OneDimensional));

        let polygon = wkt!(POLYGON ((1. 9., 5. 9., 5. 5., 1. 5., 1. 9.)));
        let cow = GeometryCow::from(&polygon);
        assert_eq!(relate_geom_dims(&cow), (TwoDimensional, TwoDimensional));

        let gc = wkt!(GEOMETRYCOLLECTION (
            POLYGON ((1. 9., 5. 9., 5. 5., 1. 5., 1. 9.)),
            LINESTRING (1. 1., 5. 4.),
            POINT (6. 5.)
        ));
        let cow = GeometryCow::from(&gc);
        assert_eq!(relate_geom_dims(&cow), (TwoDimensional, TwoDimensional));

        // An empty polygon still contributes its type dimension to
        // dimension(), but not to dimension_real().
        let gc_empty_poly = wkt!(GEOMETRYCOLLECTION (
            POLYGON EMPTY,
            LINESTRING (1. 1., 5. 4.),
            POINT (6. 5.)
        ));
        let cow = GeometryCow::from(&gc_empty_poly);
        assert_eq!(relate_geom_dims(&cow), (TwoDimensional, OneDimensional));
    }

    // Not in JTS: the polygonal element ids assigned by segment-string
    // extraction must match the document-order ids used by the point
    // locator.
    #[test]
    fn test_polygonal_ids_are_stable_under_env_filter() {
        let gc = wkt!(GEOMETRYCOLLECTION (
            POLYGON ((0. 0., 2. 0., 2. 2., 0. 2., 0. 0.)),
            POLYGON ((10. 10., 12. 10., 12. 12., 10. 12., 10. 10.)),
            POLYGON ((20. 20., 22. 20., 22. 22., 20. 22., 20. 20.))
        ));
        let cow = GeometryCow::from(&gc);
        let rgeom = RelateGeometry::new(&cow, Mode::Simple);

        // Filter to the last polygon only: its section id must still be 2.
        let env = Rect::new(Coord { x: 19., y: 19. }, Coord { x: 23., y: 23. });
        let filtered = rgeom
            .extract_segment_strings(super::super::topology_predicate::InputIndex::A, Some(&env));
        assert_eq!(filtered.len(), 1);
        let section = filtered[0].create_node_section(0, Coord { x: 21., y: 20. });
        assert_eq!(section.polygonal_id(), Some(2));
    }
}
