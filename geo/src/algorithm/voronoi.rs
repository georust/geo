//! This module provides the [`Voronoi`] trait for computing Voronoi diagrams from
//! the vertices of input geometries.
//!
//! # Edges vs Cells
//!
//! - [`Voronoi::voronoi_edges`] returns the boundaries of the Voronoi diagram as
//!   line segments (`Vec<Line>`), similar to PostGIS `ST_VoronoiLines`
//! - [`Voronoi::voronoi_cells`] returns the regions as polygons (`Vec<Polygon>`),
//!   one per input vertex, similar to PostGIS `ST_VoronoiPolygons`
//!
//! Use `_edges` when you need the diagram structure; use `_cells` when you need
//! spatial regions for proximity queries.
//!
//! # Supported Types
//!
//! Both methods work on any geometry implementing [`CoordsIter`](self::CoordsIter), including:
//! - `Point`, `MultiPoint`: typical use case for site-based Voronoi
//! - `LineString`, `MultiLineString`: uses vertices as sites
//! - `Polygon`, `MultiPolygon`: uses exterior/interior ring vertices as sites
//! - `Rect`, `Triangle`, `Line`
//! - `GeometryCollection`, `Geometry`
//! - `Vec<G>` where `G` implements the trait
//!
//! # Examples
//!
//! ```
//! use geo::{Voronoi, MultiPoint, point};
//!
//! // MultiPoint: each point becomes a site
//! let sites = MultiPoint::from(vec![
//!     point!(x: 0.0, y: 0.0),
//!     point!(x: 1.0, y: 0.0),
//!     point!(x: 0.5, y: 1.0),
//! ]);
//! let edges = sites.voronoi_edges().unwrap();
//! assert_eq!(edges.len(), 3); // Three edges radiating from circumcentre
//!
//! let cells = sites.voronoi_cells().unwrap();
//! assert_eq!(cells.len(), 3); // One cell per site
//! ```
//!
//! # Configuration
//!
//! Use [`VoronoiParams`] to configure tolerance and clipping behaviour:
//!
//! ```
//! use geo::{Voronoi, VoronoiParams, VoronoiClip, MultiPoint, point, Polygon, LineString, Coord};
//!
//! let sites = MultiPoint::from(vec![
//!     point!(x: 0.0, y: 0.0),
//!     point!(x: 1.0, y: 0.0),
//!     point!(x: 0.5, y: 1.0),
//! ]);
//!
//! // Clip to exact bounding box instead of 50% padded default
//! let cells = sites.voronoi_cells_with_params(
//!     VoronoiParams::new().clip(VoronoiClip::Envelope)
//! ).unwrap();
//!
//! // Clip to custom boundary polygon
//! let boundary = Polygon::new(
//!     LineString::from(vec![
//!         Coord { x: -1.0, y: -1.0 },
//!         Coord { x: 2.0, y: -1.0 },
//!         Coord { x: 2.0, y: 2.0 },
//!         Coord { x: -1.0, y: 2.0 },
//!         Coord { x: -1.0, y: -1.0 },
//!     ]),
//!     vec![],
//! );
//! let cells = sites.voronoi_cells_with_params(
//!     VoronoiParams::new().clip(VoronoiClip::Polygon(&boundary))
//! ).unwrap();
//! ```
//!
//
// TODO: Cell clipping currently uses BooleanOps intersection. Alternative approaches that
// could simplify the implementation or improve performance:
//   1. Clip edges first, build cells from clipped edges: `voronoi_edges` already clips rays
//      to the bounding box using line-rectangle intersection. Could construct cells from
//      those clipped edges rather than building oversized cells then clipping.
//   2. Direct cell clipping during construction: clip rays to the boundary and insert bbox
//      corners as needed during cell construction, rather than extending rays and using
//      BooleanOps afterwards.
//   3. Sutherland-Hodgman polygon clipping: a simpler algorithm for clipping convex polygons
//      against convex clip regions, potentially faster than general BooleanOps for the
//      common case of rectangular clips.

use crate::algorithm::triangulate_delaunay::{
    DelaunayTriangulationConfig, SpadeTriangulationFloat,
};
use crate::algorithm::vector_ops::Vector2DOps;
use crate::line_intersection::{LineIntersection, line_intersection};
use crate::utils::lex_cmp;
use geo_types::{Coord, Line, LineString, Point, Polygon, Rect};
use num_traits::Float;
use spade::Triangulation;
use spade::handles::{VoronoiVertex::Inner, VoronoiVertex::Outer};

use crate::algorithm::bool_ops::BoolOpsNum;
use crate::algorithm::triangulate_delaunay::TriangulationError;
use crate::{
    BooleanOps, BoundingRect, Contains, Distance, Euclidean, TriangulateDelaunayUnconstrained,
};

/// Error type for Voronoi diagram computation.
///
/// This wraps [`TriangulationError`] (which can occur during the underlying
/// Delaunay triangulation) and adds Voronoi-specific error variants.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoronoiError {
    /// Error during the underlying Delaunay triangulation.
    Triangulation(TriangulationError),
    /// Input points are collinear; Voronoi cells cannot be computed.
    /// Use [`Voronoi::voronoi_edges`] instead to get perpendicular bisectors.
    CollinearInput,
    /// Fewer than 2 unique vertices were provided.
    /// At least 2 vertices are required to compute a Voronoi diagram.
    InsufficientVertices,
}

impl std::fmt::Display for VoronoiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoronoiError::Triangulation(e) => write!(f, "triangulation error: {e}"),
            VoronoiError::CollinearInput => {
                write!(
                    f,
                    "input points are collinear; Voronoi cells cannot be computed"
                )
            }
            VoronoiError::InsufficientVertices => {
                write!(
                    f,
                    "fewer than 2 unique vertices; cannot compute Voronoi diagram"
                )
            }
        }
    }
}

impl std::error::Error for VoronoiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VoronoiError::Triangulation(e) => Some(e),
            VoronoiError::CollinearInput | VoronoiError::InsufficientVertices => None,
        }
    }
}

impl From<TriangulationError> for VoronoiError {
    fn from(e: TriangulationError) -> Self {
        VoronoiError::Triangulation(e)
    }
}

/// Clipping mode for Voronoi diagrams.
///
/// Controls how infinite Voronoi edges are bounded.
#[derive(Debug, Clone, Default)]
pub enum VoronoiClip<'a, T: SpadeTriangulationFloat> {
    /// Extend to a bounding box with 50 % padding around input vertices.
    ///
    /// This is the default, matching PostGIS `ST_VoronoiPolygons` when called
    /// without an `extend_to` geometry.
    #[default]
    Padded,

    /// Clip to the exact bounding box of the input vertices.
    ///
    /// Useful when you want cells that don't extend beyond the input extent.
    Envelope,

    /// Clip to a custom polygon boundary.
    ///
    /// Allows clipping to arbitrary shapes, such as a study area polygon.
    Polygon(&'a Polygon<T>),
}

/// Configuration parameters for Voronoi diagram computation.
///
/// Use the builder pattern to configure options:
///
/// ```
/// use geo::{VoronoiParams, VoronoiClip};
///
/// let params = VoronoiParams::<f64>::new()
///     .tolerance(0.001)
///     .clip(VoronoiClip::Envelope);
/// ```
#[derive(Debug, Clone)]
pub struct VoronoiParams<'a, T: SpadeTriangulationFloat> {
    /// Points within this distance are snapped together before triangulation.
    ///
    /// Similar to PostGIS `ST_VoronoiPolygons` tolerance parameter.
    /// Default is 0.0 (no snapping).
    pub tolerance: T,

    /// How to clip / bound the Voronoi diagram.
    ///
    /// Default is [`VoronoiClip::Padded`] (50 % padding around input bounds).
    pub clip: VoronoiClip<'a, T>,
}

impl<'a, T: SpadeTriangulationFloat> Default for VoronoiParams<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: SpadeTriangulationFloat> VoronoiParams<'a, T> {
    /// Create new parameters with defaults (no tolerance, 50 % padded clipping).
    pub fn new() -> Self {
        Self {
            tolerance: T::zero(),
            clip: VoronoiClip::Padded,
        }
    }

    /// Set the tolerance for snapping nearby points.
    ///
    /// Points within this distance are merged before triangulation.
    pub fn tolerance(mut self, tolerance: T) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Set the clipping mode.
    pub fn clip(mut self, clip: VoronoiClip<'a, T>) -> Self {
        self.clip = clip;
        self
    }
}

/// Produce Voronoi diagrams from Delaunay triangulations of input geometries.
///
/// The trait is automatically implemented for any geometry type that implements
/// [`CoordsIter`](crate::algorithm::coords_iter::CoordsIter), including `Point`, `MultiPoint`, `LineString`, `Polygon`,
/// and collections thereof.
///
/// See the [module documentation](self) for detailed examples.
pub trait Voronoi<'a, T>
where
    T: SpadeTriangulationFloat,
{
    /// Compute Voronoi edges with default parameters.
    ///
    /// Returns the Voronoi diagram boundaries as a `Vec<Line>`. Each edge
    /// is guaranteed to have exactly two endpoints. Semi-infinite edges are
    /// clipped to a bounding box with 50% padding around the input vertices
    /// (matching PostGIS default).
    ///
    /// Similar to PostGIS `ST_VoronoiLines(geom)`.
    ///
    /// # Converting to `MultiLineString`
    ///
    /// If you need a `MultiLineString` (e.g. for serialisation or to use
    /// geometry traits), collect the edges:
    ///
    /// ```
    /// use geo::{Voronoi, MultiPoint, MultiLineString, point};
    ///
    /// let sites = MultiPoint::from(vec![
    ///     point!(x: 0.0, y: 0.0),
    ///     point!(x: 1.0, y: 0.0),
    ///     point!(x: 0.5, y: 1.0),
    /// ]);
    /// let edges = sites.voronoi_edges().unwrap();
    /// let mls = MultiLineString::from_iter(edges);
    /// ```
    ///
    /// # Example
    /// ```
    /// use geo::{Voronoi, MultiPoint, point};
    ///
    /// let sites = MultiPoint::from(vec![
    ///     point!(x: 0.0, y: 0.0),
    ///     point!(x: 1.0, y: 0.0),
    ///     point!(x: 0.5, y: 1.0),
    /// ]);
    /// let edges = sites.voronoi_edges().unwrap();
    /// assert_eq!(edges.len(), 3);
    /// ```
    fn voronoi_edges(&'a self) -> Result<Vec<Line<T>>, VoronoiError>;

    /// Compute Voronoi edges with custom parameters.
    ///
    /// See [`VoronoiParams`] for available configuration options.
    fn voronoi_edges_with_params(
        &'a self,
        params: VoronoiParams<'a, T>,
    ) -> Result<Vec<Line<T>>, VoronoiError>;

    /// Compute Voronoi cells with default parameters.
    ///
    /// Returns one polygon per input vertex (site). Each cell contains all
    /// points closer to that site than to any other. Unbounded cells on the
    /// convex hull are clipped to a bounding box with 50% padding.
    ///
    /// Equivalent to PostGIS `ST_VoronoiPolygons(geom)`.
    ///
    /// # Returns
    /// Returns `Vec<Polygon<T>>` rather than `MultiPolygon<T>` to preserve the
    /// one-to-one correspondence between cells and input vertices, allowing
    /// indexed access. Use `MultiPolygon::new(cells)` if a single geometry is needed.
    ///
    /// # Errors
    /// Returns [`VoronoiError::CollinearInput`] if all input points are
    /// collinear. Use [`voronoi_edges`](Self::voronoi_edges) instead to get
    /// perpendicular bisectors for collinear input.
    ///
    /// # Example
    /// ```
    /// use geo::{Voronoi, Polygon, LineString, Coord};
    ///
    /// let triangle = Polygon::new(
    ///     LineString::from(vec![
    ///         Coord { x: 0.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 0.0 },
    ///         Coord { x: 0.5, y: 1.0 },
    ///         Coord { x: 0.0, y: 0.0 },
    ///     ]),
    ///     vec![],
    /// );
    ///
    /// let cells = triangle.voronoi_cells().unwrap();
    /// assert_eq!(cells.len(), 3); // One cell per vertex
    /// ```
    fn voronoi_cells(&'a self) -> Result<Vec<Polygon<T>>, VoronoiError>
    where
        T: BoolOpsNum;

    /// Compute Voronoi cells with custom parameters.
    ///
    /// See [`VoronoiParams`] for available configuration options including
    /// tolerance and clipping modes.
    ///
    /// # Example
    /// ```
    /// use geo::{Voronoi, VoronoiParams, VoronoiClip, MultiPoint, point};
    ///
    /// let sites = MultiPoint::from(vec![
    ///     point!(x: 0.0, y: 0.0),
    ///     point!(x: 1.0, y: 0.0),
    ///     point!(x: 0.5, y: 1.0),
    /// ]);
    ///
    /// // Clip to exact bounding box
    /// let cells = sites.voronoi_cells_with_params(
    ///     VoronoiParams::new().clip(VoronoiClip::Envelope)
    /// ).unwrap();
    /// ```
    fn voronoi_cells_with_params(
        &'a self,
        params: VoronoiParams<'a, T>,
    ) -> Result<Vec<Polygon<T>>, VoronoiError>
    where
        T: BoolOpsNum;
}

// Everything that can be triangulated with Spade automatically gets Voronoi functionality
impl<'a, T, G> Voronoi<'a, T> for G
where
    T: SpadeTriangulationFloat + 'a,
    G: TriangulateDelaunayUnconstrained<'a, T>,
{
    fn voronoi_edges(&'a self) -> Result<Vec<Line<T>>, VoronoiError> {
        self.voronoi_edges_with_params(VoronoiParams::new())
    }

    fn voronoi_edges_with_params(
        &'a self,
        params: VoronoiParams<'a, T>,
    ) -> Result<Vec<Line<T>>, VoronoiError> {
        // 1. Build Delaunay triangulation from input geometry vertices
        // 2. Compute bounding box for clipping infinite edges
        // 3. For each undirected Voronoi edge, handle three cases:
        //    a) Inner-Inner: Both endpoints are circumcenters, emit finite edge directly
        //    b) Inner-Outer: One endpoint is infinite, extend ray from circumcenter,
        //       find closest intersection with bounding box, emit clipped edge
        //    c) Outer-Outer: Both endpoints infinite (collinear points), create
        //       perpendicular bisector line between sorted input points
        // 4. Return all edges as Vec<Line>

        let triangulation = if params.tolerance > T::zero() {
            self.unconstrained_triangulation_raw_with_config(DelaunayTriangulationConfig {
                snap_radius: params.tolerance,
            })?
        } else {
            self.unconstrained_triangulation_raw()?
        };

        let base_bounds = compute_bounds_from_vertices(triangulation.vertices().map(|v| *v.data()));

        // Use padded bounds for clipping (50 % padding matches PostGIS default)
        let bounds = padded_bounds(base_bounds, <T as num_traits::NumCast>::from(0.5).unwrap());
        let width = base_bounds.width();
        let height = base_bounds.height();

        let mut edges = Vec::new();
        for edge in triangulation.undirected_voronoi_edges() {
            match edge.vertices() {
                [Inner(from), Inner(to)] => {
                    let from_point = from.circumcenter();
                    let to_point = to.circumcenter();
                    edges.push(Line::new(
                        Coord {
                            x: from_point.x,
                            y: from_point.y,
                        },
                        Coord {
                            x: to_point.x,
                            y: to_point.y,
                        },
                    ));
                }
                [Inner(from), Outer(edge)] | [Outer(edge), Inner(from)] => {
                    let start = from.circumcenter();
                    let dir = edge.direction_vector();

                    // Create a line extending beyond bounds in direction of outer edge
                    let extended_end = Point::new(
                        start.x + dir.x * (width + height),
                        start.y + dir.y * (width + height),
                    );
                    let infinite_voronoi_edge =
                        Line::new(Point::new(start.x, start.y), extended_end);

                    // Manual line-bbox intersection is simpler than BooleanOps here:
                    // BooleanOps operates on polygons, not line strings, and for a single
                    // ray-rectangle intersection the geometric calculation is straightforward.
                    //
                    // Check intersection with each bounding box edge
                    // Improper intersections (at bbox corners or ray origin) are handled correctly:
                    // - Corner hits produce duplicate points; min_by distance deduplicates
                    // - Collinear rays (edge parallel to bbox side) are dropped; this case is
                    //   geometrically degenerate and would require exact float alignment,
                    //   so is almost certainly not a problem for non-adversarial inputs
                    let intersections = bounds
                        .to_lines()
                        .iter()
                        .filter_map(|edge| line_intersection(infinite_voronoi_edge, *edge))
                        .filter_map(|inter| match inter {
                            LineIntersection::SinglePoint {
                                intersection,
                                is_proper: _,
                            } => Some(intersection),
                            LineIntersection::Collinear { intersection: _ } => None,
                        })
                        .collect::<Vec<_>>();
                    // Because we extend our infinite edge well beyond the bounding box
                    // we should always expect to see at least one infinite edge intersection
                    debug_assert!(
                        !intersections.is_empty(),
                        "No infinite edges intersect the bounding box. Degenerate or invalid geometry?"
                    );

                    // Take the closest intersection to the start point
                    if let Some(intersection) = intersections.into_iter().min_by(|a, b| {
                        let dist_a: T = Euclidean
                            .distance(&Point::new(start.x, start.y), &Point::new(a.x, a.y));
                        let dist_b: T = Euclidean
                            .distance(&Point::new(start.x, start.y), &Point::new(b.x, b.y));
                        dist_a.total_cmp(&dist_b)
                    }) {
                        edges.push(Line::new(
                            Coord {
                                x: start.x,
                                y: start.y,
                            },
                            intersection,
                        ));
                    }
                }
                [Outer(edge1), Outer(_)] => {
                    // Outer-Outer edges occur only when all input points are collinear.
                    // For N collinear points, we get N-1 perpendicular bisectors.
                    // Use edges.len() as the bisector index to place each correctly.
                    let mut points: Vec<_> = triangulation.vertices().map(|v| *v.data()).collect();
                    points.sort_by(|a, b| lex_cmp(a, b));

                    let bisector_index = edges.len();
                    if bisector_index + 1 >= points.len() {
                        // Safety: skip if we've exhausted point pairs
                        continue;
                    }

                    let p_a = points[bisector_index];
                    let p_b = points[bisector_index + 1];
                    let midpoint = (p_a + p_b) / <T as num_traits::NumCast>::from(2.0).unwrap();

                    let dir = edge1.direction_vector();
                    let extension = width + height;
                    let extended_start = Coord {
                        x: midpoint.x - dir.x * extension,
                        y: midpoint.y - dir.y * extension,
                    };
                    let extended_end = Coord {
                        x: midpoint.x + dir.x * extension,
                        y: midpoint.y + dir.y * extension,
                    };

                    edges.push(Line::new(extended_start, extended_end))
                }
            }
        }
        Ok(edges)
    }

    fn voronoi_cells(&'a self) -> Result<Vec<Polygon<T>>, VoronoiError>
    where
        T: BoolOpsNum,
    {
        self.voronoi_cells_with_params(VoronoiParams::new())
    }

    fn voronoi_cells_with_params(
        &'a self,
        params: VoronoiParams<'a, T>,
    ) -> Result<Vec<Polygon<T>>, VoronoiError>
    where
        T: BoolOpsNum,
    {
        let (raw_cells, base_bounds) = build_raw_voronoi_cells(self, &params)?;

        // Determine clipping polygon based on params
        let clip_poly: Polygon<T> = match &params.clip {
            VoronoiClip::Padded => {
                padded_bounds(base_bounds, <T as num_traits::NumCast>::from(0.5).unwrap())
                    .to_polygon()
            }
            VoronoiClip::Envelope => base_bounds.to_polygon(),
            VoronoiClip::Polygon(boundary) => (*boundary).clone(),
        };

        // Intersect each cell individually rather than batching into a MultiPolygon:
        // adjacent Voronoi cells share edges, so batch intersection could merge them,
        // losing the 1:1 correspondence between input sites and output cells.
        //
        // Optimisation: skip intersection for cells entirely within the clip bounds.
        // Interior cells (not on the convex hull) often fall entirely within the
        // clipping region, so we can avoid the BooleanOps overhead for those.
        let clip_rect = clip_poly.bounding_rect();

        Ok(raw_cells
            .into_iter()
            .flat_map(|cell| {
                // Skip intersection if cell is entirely within clip bounds
                let contained_by_clip = clip_rect
                    .as_ref()
                    .zip(cell.bounding_rect())
                    .is_some_and(|(cr, cell_rect)| cr.contains(&cell_rect));

                if contained_by_clip {
                    vec![cell]
                } else {
                    cell.intersection(&clip_poly).0
                }
            })
            .collect())
    }
}

/// Build raw Voronoi cells with extended rays, without clipping.
///
/// Returns `(raw_cells, base_bounds)` where `base_bounds` is the tight bounding box
/// of the input points. Each raw cell has infinite rays extended far beyond any
/// reasonable clipping boundary, so callers must clip with `intersection()`.
fn build_raw_voronoi_cells<'a, T, G>(
    geom: &'a G,
    params: &VoronoiParams<'a, T>,
) -> Result<(Vec<Polygon<T>>, Rect<T>), VoronoiError>
where
    T: SpadeTriangulationFloat + BoolOpsNum,
    G: TriangulateDelaunayUnconstrained<'a, T>,
{
    let triangulation = if params.tolerance > T::zero() {
        geom.unconstrained_triangulation_raw_with_config(DelaunayTriangulationConfig {
            snap_radius: params.tolerance,
        })?
    } else {
        geom.unconstrained_triangulation_raw()?
    };

    let num_vertices = triangulation.num_vertices();
    if num_vertices < 2 {
        return Err(VoronoiError::InsufficientVertices);
    }

    let base_bounds = compute_bounds_from_vertices(triangulation.vertices().map(|v| *v.data()));

    // Use padded bounds for extension distance calculation
    let padded = padded_bounds(base_bounds, <T as num_traits::NumCast>::from(0.5).unwrap());
    let extension =
        (padded.width() + padded.height()) * <T as num_traits::NumCast>::from(2.0).unwrap();

    let mut raw_cells: Vec<Polygon<T>> = Vec::new();

    for face in triangulation.voronoi_faces() {
        let edges: Vec<_> = face.adjacent_edges().collect();
        if edges.is_empty() {
            continue;
        }

        let site_coord = *face.as_delaunay_vertex().data();

        // Collect circumcenters and ray info
        let mut circumcenters: Vec<Coord<T>> = Vec::new();
        let mut rays: Vec<(Coord<T>, Coord<T>)> = Vec::new(); // (origin, direction)

        for edge in &edges {
            let from_vertex = edge.from();
            let to_vertex = edge.to();

            if let Inner(inner_face) = &from_vertex {
                let cc = inner_face.circumcenter();
                let coord = Coord { x: cc.x, y: cc.y };
                if !circumcenters.contains(&coord) {
                    circumcenters.push(coord);
                }
            }
            if let Inner(inner_face) = &to_vertex {
                let cc = inner_face.circumcenter();
                let coord = Coord { x: cc.x, y: cc.y };
                if !circumcenters.contains(&coord) {
                    circumcenters.push(coord);
                }
            }

            // Collect ray information
            if let (Inner(inner_face), Outer(outer_edge)) = (&from_vertex, &to_vertex) {
                let ref_pt = inner_face.circumcenter();
                let dir = outer_edge.direction_vector();
                rays.push((
                    Coord {
                        x: ref_pt.x,
                        y: ref_pt.y,
                    },
                    Coord { x: dir.x, y: dir.y },
                ));
            }

            if let (Outer(outer_edge), Inner(inner_face)) = (&from_vertex, &to_vertex) {
                let ref_pt = inner_face.circumcenter();
                let dir = outer_edge.direction_vector();
                rays.push((
                    Coord {
                        x: ref_pt.x,
                        y: ref_pt.y,
                    },
                    Coord { x: dir.x, y: dir.y },
                ));
            }
        }

        // Build cell vertices
        let mut vertices: Vec<Coord<T>> = circumcenters.clone();

        if rays.is_empty() {
            // Interior cell: just circumcenters
            if vertices.len() < 3 {
                continue;
            }
        } else {
            // Boundary cell: extend rays far beyond bbox
            for (origin, direction) in &rays {
                // Normalise direction to unit vector so all rays extend the same distance.
                // Skip degenerate zero-length or non-finite directions.
                let Some(unit_dir) = direction.try_normalize() else {
                    continue;
                };

                // Add a point far beyond the bbox in the ray direction
                let extended = *origin + unit_dir * extension;
                vertices.push(extended);
            }
        }

        if vertices.len() < 3 {
            continue;
        }

        // Sort vertices by angle around the site
        vertices.sort_by(|a, b| {
            let angle_a = Float::atan2(a.y - site_coord.y, a.x - site_coord.x);
            let angle_b = Float::atan2(b.y - site_coord.y, b.x - site_coord.x);
            angle_a.total_cmp(&angle_b)
        });

        vertices.push(vertices[0]);
        let poly = Polygon::new(LineString::new(vertices), vec![]);

        raw_cells.push(poly);
    }

    // Collinear input produces no cells (only perpendicular bisector lines).
    // Return an error rather than silently returning an empty result.
    if raw_cells.is_empty() && num_vertices >= 2 {
        return Err(VoronoiError::CollinearInput);
    }

    Ok((raw_cells, base_bounds))
}

/// Compute bounding box from an iterator of Coord vertices.
fn compute_bounds_from_vertices<T, I>(vertices: I) -> Rect<T>
where
    T: SpadeTriangulationFloat,
    I: Iterator<Item = Coord<T>>,
{
    let (min_x, min_y, max_x, max_y) = vertices.fold(
        (
            Float::max_value(),
            Float::max_value(),
            Float::min_value(),
            Float::min_value(),
        ),
        |(min_x, min_y, max_x, max_y), p| {
            (
                Float::min(min_x, p.x),
                Float::min(min_y, p.y),
                Float::max(max_x, p.x),
                Float::max(max_y, p.y),
            )
        },
    );
    Rect::new((min_x, min_y), (max_x, max_y))
}

/// Compute a padded bounding box around the base bounds.
///
/// Padding is calculated as `padding_factor * max(width, height)` and applied
/// uniformly on all sides. A factor of 0.5 matches PostGIS ST_VoronoiPolygons default.
fn padded_bounds<T: SpadeTriangulationFloat>(base: Rect<T>, padding_factor: T) -> Rect<T> {
    let padding = Float::max(base.width(), base.height()) * padding_factor;
    Rect::new(
        (base.min().x - padding, base.min().y - padding),
        (base.max().x + padding, base.max().y + padding),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Relate;
    use crate::algorithm::Validation;
    use crate::wkt;
    use approx::assert_relative_eq;
    use geo_types::{MultiLineString, MultiPolygon};

    #[test]
    fn voronoi_test_simple_triangle() {
        // Right triangle: vertices at (0,0), (1,0), (0,1)
        // Circumcenter at midpoint of hypotenuse = (0.5, 0.5)
        // Bounding box with 50% padding: (-0.5, -0.5) to (1.5, 1.5)
        let triangle = wkt!(POLYGON((0.0 0.0, 1.0 0.0, 0.0 1.0, 0.0 0.0)));

        let voronoi = triangle.voronoi_edges().unwrap();
        let result: MultiLineString<_> = voronoi.into_iter().collect();

        // 3 edges from circumcenter (0.5, 0.5) to clipped boundary
        let expected = wkt!(MULTILINESTRING(
            (0.5 0.5, 0.5 -0.5),
            (0.5 0.5, -0.5 0.5),
            (0.5 0.5, 1.5 1.5)
        ));
        assert!(
            result.relate(&expected).is_equal_topo(),
            "Expected {expected:?}, got {result:?}"
        );
    }

    #[test]
    fn voronoi_test_square() {
        // Unit square: vertices at (0,0), (1,0), (1,1), (0,1)
        // Triangulated with diagonal, both triangles have circumcenter at (0.5, 0.5)
        // Bounding box with 50% padding: (-0.5, -0.5) to (1.5, 1.5)
        let square = wkt!(POLYGON((0.0 0.0, 1.0 0.0, 1.0 1.0, 0.0 1.0, 0.0 0.0)));

        let voronoi = square.voronoi_edges().unwrap();
        let result: MultiLineString<_> = voronoi.into_iter().collect();

        // 4 edges from centre to clipped boundary, plus 1 degenerate edge (both
        // right triangles share the same circumcenter)
        let expected = wkt!(MULTILINESTRING(
            (0.5 0.5, 0.5 -0.5),
            (0.5 0.5, 1.5 0.5),
            (0.5 0.5, 0.5 0.5),
            (0.5 0.5, 0.5 1.5),
            (0.5 0.5, -0.5 0.5)
        ));
        assert!(
            result.relate(&expected).is_equal_topo(),
            "Expected {expected:?}, got {result:?}"
        );
    }

    #[test]
    fn voronoi_test_collinear_points() {
        // Three collinear points on x-axis at (-1,0), (0,0), (1,0)
        // Perpendicular bisectors at x = -0.5 and x = 0.5
        let collinear = wkt!(POLYGON((-1.0 0.0, 0.0 0.0, 1.0 0.0, -1.0 0.0)));

        let voronoi = collinear.voronoi_edges().unwrap();
        let result: MultiLineString<_> = voronoi.into_iter().collect();

        // Two vertical bisector lines
        let expected = wkt!(MULTILINESTRING(
            (-0.5 -2.0, -0.5 2.0),
            (0.5 2.0, 0.5 -2.0)
        ));
        assert!(
            result.relate(&expected).is_equal_topo(),
            "Expected {expected:?}, got {result:?}"
        );
    }

    #[test]
    fn voronoi_cells_collinear_returns_error() {
        // Collinear points cannot produce Voronoi cells (only infinite strips)
        let collinear = wkt!(LINESTRING(0.0 0.0, 1.0 0.0, 2.0 0.0));

        let Err(VoronoiError::CollinearInput) = collinear.voronoi_cells() else {
            panic!("Expected CollinearInput error");
        };

        // Edges should still work for collinear input
        let edges = collinear.voronoi_edges().unwrap();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn voronoi_test_two_points() {
        // Two points at (0,0) and (1,0)
        // Perpendicular bisector at x = 0.5
        let points = wkt!(POLYGON((0.0 0.0, 1.0 0.0, 0.0 0.0)));

        let voronoi = points.voronoi_edges().unwrap();
        let result: MultiLineString<_> = voronoi.into_iter().collect();

        // Single vertical bisector line
        let expected = wkt!(MULTILINESTRING((0.5 -1.0, 0.5 1.0)));
        assert!(
            result.relate(&expected).is_equal_topo(),
            "Expected {expected:?}, got {result:?}"
        );
    }

    // Tests for voronoi_cells()

    #[test]
    fn voronoi_cells_triangle() {
        // Right triangle should produce 3 cells
        let triangle = wkt!(POLYGON((0.0 0.0, 1.0 0.0, 0.0 1.0, 0.0 0.0)));

        let cells = triangle.voronoi_cells().unwrap();
        assert_eq!(cells.len(), 3);

        // Each cell should be a valid polygon with at least 3 vertices (plus closing)
        for cell in &cells {
            assert!(cell.exterior().0.len() >= 4);
            // Verify cell is valid (no self-intersections, correct winding)
            assert!(
                cell.is_valid(),
                "Voronoi cell should be a valid polygon: {:?}",
                cell.exterior().0
            );
        }
    }

    #[test]
    fn voronoi_cells_square() {
        // Unit square should produce 4 cells
        let square = wkt!(POLYGON((0.0 0.0, 1.0 0.0, 1.0 1.0, 0.0 1.0, 0.0 0.0)));

        let cells = square.voronoi_cells().unwrap();
        assert_eq!(cells.len(), 4);

        // Verify all cells are valid polygons
        for cell in &cells {
            assert!(
                cell.is_valid(),
                "Voronoi cell should be a valid polygon: {:?}",
                cell.exterior().0
            );
        }
    }

    #[test]
    fn voronoi_cells_clipped_square() {
        // Test clipping functionality
        let square = wkt!(POLYGON((0.0 0.0, 1.0 0.0, 1.0 1.0, 0.0 1.0, 0.0 0.0)));

        let clipped = square
            .voronoi_cells_with_params(VoronoiParams::new().clip(VoronoiClip::Envelope))
            .unwrap();
        // After clipping to the bounding box, we should still have cells
        // The exact count may vary due to how the boolean operations handle edge cases
        assert!(!clipped.is_empty());
    }

    #[test]
    fn voronoi_cells_clipped_to_custom_boundary() {
        // Test clipping to a custom boundary
        let points = wkt!(POLYGON((0.0 0.0, 2.0 0.0, 2.0 2.0, 0.0 2.0, 0.0 0.0)));

        // Clip to a smaller square
        let clip_boundary = wkt!(POLYGON((0.5 0.5, 1.5 0.5, 1.5 1.5, 0.5 1.5, 0.5 0.5)));

        let clipped = points
            .voronoi_cells_with_params(
                VoronoiParams::new().clip(VoronoiClip::Polygon(&clip_boundary)),
            )
            .unwrap();
        // Should have cells, clipped to the boundary
        assert!(!clipped.is_empty());
    }

    #[test]
    fn voronoi_cells_count_matches_input_points() {
        // Create a polygon with 8 coordinates (7 unique after closing point)
        let polygon = wkt!(POLYGON((
            0.0 0.0, 1.0 0.0, 2.0 0.5, 2.0 1.5, 1.5 2.0, 0.5 2.0, 0.0 1.5, 0.0 0.0
        )));

        // 7 unique vertices (8 coords minus 1 closing duplicate)
        let cells = polygon.voronoi_cells().unwrap();
        assert_eq!(
            cells.len(),
            7,
            "voronoi_cells() should produce one cell per unique input vertex"
        );

        let clipped = polygon
            .voronoi_cells_with_params(VoronoiParams::new().clip(VoronoiClip::Envelope))
            .unwrap();
        assert_eq!(
            clipped.len(),
            7,
            "voronoi_cells_with_params(Envelope) should produce one cell per unique input vertex"
        );
    }

    #[test]
    fn voronoi_cells_union_covers_bounding_box() {
        use crate::Area;

        // Create a simple set of points (unit square vertices)
        let square = wkt!(POLYGON((0.0 0.0, 1.0 0.0, 1.0 1.0, 0.0 1.0, 0.0 0.0)));

        let cells = square.voronoi_cells().unwrap();
        assert_eq!(cells.len(), 4);

        // Verify all cells are valid polygons (no self-intersections)
        for cell in &cells {
            assert!(
                cell.is_valid(),
                "Voronoi cell should be a valid polygon: {:?}",
                cell.exterior().0
            );
        }

        // Compute the expected padded bounding box (50% padding)
        // Input bounds: (0,0) to (1,1), so width=height=1
        // Padding = max(1,1) * 0.5 = 0.5
        // Padded bounds: (-0.5, -0.5) to (1.5, 1.5)
        let expected_bounds = Rect::new(Coord { x: -0.5, y: -0.5 }, Coord { x: 1.5, y: 1.5 });
        let expected_area = expected_bounds.to_polygon().unsigned_area();

        // Union all cells together
        let mut union_result = MultiPolygon(vec![cells[0].clone()]);
        for cell in cells.iter().skip(1) {
            union_result = union_result.union(&MultiPolygon(vec![cell.clone()]));
        }

        // The union should have the same area as the expected bounding box
        let union_area = union_result.unsigned_area();
        assert_relative_eq!(union_area, expected_area, epsilon = Float::epsilon());

        // The union should be a single polygon (the bounding box)
        assert_eq!(
            union_result.0.len(),
            1,
            "Union of Voronoi cells should form a single polygon"
        );
    }

    #[test]
    fn voronoi_cells_clipped_islington_fills_bbox() {
        use crate::Area;
        use crate::algorithm::bounding_rect::BoundingRect;
        use geo_types::MultiPoint;

        // Load Islington post box locations from test fixture (151 total, 147 unique)
        let points_mp: MultiPoint<f64> = geo_test_fixtures::islington_post_boxes();

        // Convert MultiPoint to Polygon for Voronoi computation
        let coords: Vec<Coord<f64>> = points_mp.iter().map(|p| p.0).collect();
        let unique_coords: std::collections::HashSet<_> = coords
            .iter()
            .map(|c| (c.x.to_bits(), c.y.to_bits()))
            .collect();
        let num_unique_points = unique_coords.len();
        let mut ring_coords = coords.clone();
        ring_coords.push(coords[0]);
        let points = Polygon::new(LineString::from(ring_coords), vec![]);

        let clipped_cells = points
            .voronoi_cells_with_params(VoronoiParams::new().clip(VoronoiClip::Envelope))
            .unwrap();

        // Get the bounding box of the input points
        let bbox = points.bounding_rect().unwrap();
        let bbox_area = bbox.to_polygon().unsigned_area();

        // Check cell count matches unique input points
        assert_eq!(
            clipped_cells.len(),
            num_unique_points,
            "Should have one cell per unique input vertex"
        );

        // Sum of individual cell areas should equal bbox area (no gaps or overlaps)
        let individual_sum: f64 = clipped_cells.iter().map(|c| c.unsigned_area()).sum();
        assert_relative_eq!(individual_sum, bbox_area, epsilon = 1e-9);
    }

    #[test]
    fn voronoi_test_vertical_collinear_points() {
        // Three collinear points on y-axis at (0,0), (0,1), (0,2)
        // Perpendicular bisectors at y = 0.5 and y = 1.5
        let collinear = wkt!(POLYGON((0.0 0.0, 0.0 1.0, 0.0 2.0, 0.0 0.0)));

        let voronoi = collinear.voronoi_edges().unwrap();
        let result: MultiLineString<_> = voronoi.into_iter().collect();

        // Two horizontal bisector lines
        let expected = wkt!(MULTILINESTRING(
            (2.0 0.5, -2.0 0.5),
            (-2.0 1.5, 2.0 1.5)
        ));
        assert!(
            result.relate(&expected).is_equal_topo(),
            "Expected {expected:?}, got {result:?}"
        );
    }
}
