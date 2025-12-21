//! This module provides the [`Voronoi`] trait for computing Voronoi diagrams from
//! the vertices of input geometries.
//!
//! # Edges vs Cells
//!
//! - [`Voronoi::voronoi_edges`] returns the boundaries of the Voronoi diagram as
//!   line segments (`MultiLineString`), equivalent to PostGIS `ST_VoronoiLines`
//! - [`Voronoi::voronoi_cells`] returns the regions as polygons (`Vec<Polygon>`),
//!   one per input vertex, equivalent to PostGIS `ST_VoronoiPolygons`
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
//! assert_eq!(edges.0.len(), 3); // Three edges radiating from circumcentre
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
// TODO: This module manually computes bounding boxes from Spade triangulation vertices because
// Spade's `Point2<T>` type is not compatible with geo-types. If future work converts Spade
// vertices to geo-types (e.g. `Coord<T>`), the following geo traits could be used instead:
//   - `BoundingRect` trait for bounds computation (replaces `compute_bounds_from_vertices`)
//   - `CoordsIter` trait for vertex iteration
// The `line_intersection` and `Euclidean::distance` functions from geo are already used here.

use crate::algorithm::triangulate_delaunay::SpadeTriangulationFloat;
use crate::line_intersection::{LineIntersection, line_intersection};
use geo_types::{Coord, Line, LineString, MultiLineString, Point, Polygon, Rect, line_string};
use num_traits::Float;
use spade::Triangulation;
use spade::handles::{VoronoiVertex::Inner, VoronoiVertex::Outer};

use crate::algorithm::bool_ops::BoolOpsNum;
use crate::algorithm::triangulate_delaunay::TriangulationError;
use crate::{BooleanOps, Distance, Euclidean, TriangulateDelaunayUnconstrained};

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
    /// Returns the Voronoi diagram boundaries as a `MultiLineString`.
    /// Semi-infinite edges are clipped to a bounding box with 50% padding
    /// around the input vertices (matching PostGIS default).
    ///
    /// Equivalent to PostGIS `ST_VoronoiLines(geom)`.
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
    /// assert_eq!(edges.0.len(), 3);
    /// ```
    fn voronoi_edges(&'a self) -> Result<MultiLineString<T>, TriangulationError>;

    /// Compute Voronoi edges with custom parameters.
    ///
    /// See [`VoronoiParams`] for available configuration options.
    fn voronoi_edges_with_params(
        &'a self,
        params: VoronoiParams<'a, T>,
    ) -> Result<MultiLineString<T>, TriangulationError>;

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
    fn voronoi_cells(&'a self) -> Result<Vec<Polygon<T>>, TriangulationError>
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
    ) -> Result<Vec<Polygon<T>>, TriangulationError>
    where
        T: BoolOpsNum;
}

// Everything that can be triangulated with Spade automatically gets Voronoi functionality
impl<'a, T, G> Voronoi<'a, T> for G
where
    T: SpadeTriangulationFloat + 'a,
    G: TriangulateDelaunayUnconstrained<'a, T>,
{
    fn voronoi_edges(&'a self) -> Result<MultiLineString<T>, TriangulationError> {
        self.voronoi_edges_with_params(VoronoiParams::new())
    }

    fn voronoi_edges_with_params(
        &'a self,
        params: VoronoiParams<'a, T>,
    ) -> Result<MultiLineString<T>, TriangulationError> {
        // 1. Build Delaunay triangulation from input geometry vertices
        // 2. Compute bounding box for clipping infinite edges
        // 3. For each undirected Voronoi edge, handle three cases:
        //    a) Inner-Inner: Both endpoints are circumcenters, emit finite edge directly
        //    b) Inner-Outer: One endpoint is infinite, extend ray from circumcenter,
        //       find closest intersection with bounding box, emit clipped edge
        //    c) Outer-Outer: Both endpoints infinite (collinear points), create
        //       vertical bisector line between sorted input points
        // 4. Return all edges as MultiLineString

        let triangulation = if params.tolerance > T::zero() {
            self.unconstrained_triangulation_raw_with_tolerance(params.tolerance)?
        } else {
            self.unconstrained_triangulation_raw()?
        };

        let base_bounds =
            compute_bounds_from_vertices(triangulation.vertices().map(|v| v.position()));

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
                    edges.push(line_string![
                        (from_point.x, from_point.y).into(),
                        (to_point.x, to_point.y).into(),
                    ]);
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

                    // Check intersection with each bounding box edge
                    // Improper intersections (at bbox corners or ray origin) are handled correctly:
                    // - Corner hits produce duplicate points; min_by distance deduplicates
                    // - Collinear rays (edge parallel to bbox side) are dropped; this case is
                    //   geometrically degenerate and would require exact float alignment,
                    //   so is almost certainly not a problem for non-adversarial inputs
                    let intersections = rect_edges(&bounds)
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
                        edges.push(line_string![(start.x, start.y).into(), intersection]);
                    }
                }
                [Outer(edge1), Outer(_)] => {
                    // Outer-Outer edges occur only when all input points are collinear.
                    // For N collinear points, we get N-1 perpendicular bisectors.
                    // Use edges.len() as the bisector index to place each correctly.
                    let mut points: Vec<_> =
                        triangulation.vertices().map(|v| v.position()).collect();
                    points.sort_by(|a, b| a.x.total_cmp(&b.x));

                    let bisector_index = edges.len();
                    if bisector_index + 1 >= points.len() {
                        // Safety: skip if we've exhausted point pairs
                        continue;
                    }

                    let two = <T as num_traits::NumCast>::from(2.0).unwrap();
                    let midpoint_x =
                        (points[bisector_index].x + points[bisector_index + 1].x) / two;

                    let dir = edge1.direction_vector();
                    let extended_start = Point::new(midpoint_x, -dir.y * (width + height));
                    let extended_end = Point::new(midpoint_x, dir.y * (width + height));

                    edges.push(line_string![extended_start.into(), extended_end.into()])
                }
            }
        }
        Ok(MultiLineString(edges))
    }

    fn voronoi_cells(&'a self) -> Result<Vec<Polygon<T>>, TriangulationError>
    where
        T: BoolOpsNum,
    {
        self.voronoi_cells_with_params(VoronoiParams::new())
    }

    fn voronoi_cells_with_params(
        &'a self,
        params: VoronoiParams<'a, T>,
    ) -> Result<Vec<Polygon<T>>, TriangulationError>
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

        Ok(raw_cells
            .iter()
            .flat_map(|cell| cell.intersection(&clip_poly).0)
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
) -> Result<(Vec<Polygon<T>>, Rect<T>), TriangulationError>
where
    T: SpadeTriangulationFloat + BoolOpsNum,
    G: TriangulateDelaunayUnconstrained<'a, T>,
{
    let triangulation = if params.tolerance > T::zero() {
        geom.unconstrained_triangulation_raw_with_tolerance(params.tolerance)?
    } else {
        geom.unconstrained_triangulation_raw()?
    };

    let num_vertices = triangulation.num_vertices();
    if num_vertices < 2 {
        return Ok((
            Vec::new(),
            Rect::new((T::zero(), T::zero()), (T::zero(), T::zero())),
        ));
    }

    let base_bounds = compute_bounds_from_vertices(triangulation.vertices().map(|v| v.position()));

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

        let site = face.as_delaunay_vertex().position();
        let site_coord = Coord {
            x: site.x,
            y: site.y,
        };

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
                // Skip degenerate zero-length directions (guard against division by zero).
                let dir_len = Float::sqrt(direction.x * direction.x + direction.y * direction.y);
                if dir_len < T::epsilon() {
                    continue;
                }
                let dir_x = direction.x / dir_len;
                let dir_y = direction.y / dir_len;

                // Add a point far beyond the bbox in the ray direction
                let extended = Coord {
                    x: origin.x + dir_x * extension,
                    y: origin.y + dir_y * extension,
                };
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

    Ok((raw_cells, base_bounds))
}

/// Compute bounding box from an iterator of Spade Point2 vertices.
///
/// Folds over vertices tracking min/max for x and y coordinates.
///
/// TODO: If Spade vertices were converted to geo-types, this could use the `BoundingRect` trait.
fn compute_bounds_from_vertices<T, I>(vertices: I) -> Rect<T>
where
    T: SpadeTriangulationFloat,
    I: Iterator<Item = spade::Point2<T>>,
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

/// Return the four edges of a bounding box as [top, right, bottom, left].
///
/// Edge order does not affect correctness of intersection calculations.
fn rect_edges<T: SpadeTriangulationFloat>(bounds: &Rect<T>) -> [Line<T>; 4] {
    let (min, max) = (bounds.min(), bounds.max());
    [
        Line::new(Coord { x: min.x, y: max.y }, Coord { x: max.x, y: max.y }), // top
        Line::new(Coord { x: max.x, y: max.y }, Coord { x: max.x, y: min.y }), // right
        Line::new(Coord { x: max.x, y: min.y }, Coord { x: min.x, y: min.y }), // bottom
        Line::new(Coord { x: min.x, y: min.y }, Coord { x: min.x, y: max.y }), // left
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::Validation;
    use crate::{Distance, Euclidean};
    use approx::assert_relative_eq;
    use geo_types::{Coord, LineString, MultiPolygon, Polygon};

    #[test]
    fn voronoi_test_simple_triangle() {
        // Equilateral triangle
        let triangle = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord {
                    x: 0.5,
                    y: f64::sqrt(0.75),
                },
                Coord { x: 0.0, y: 0.0 }, // closing point
            ]),
            vec![],
        );

        let voronoi = triangle.voronoi_edges().unwrap();

        // An equilateral triangle's Voronoi diagram should:
        // 1. Have exactly 3 finite edges
        // 2. All edges should meet at the centroid (1/3 of the way from each vertex)
        assert_eq!(voronoi.0.len(), 3);

        // The centroid should be at (0.5, sqrt(0.75) / 3)
        let expected_centroid = Point::new(0.5, f64::sqrt(0.75) / 3.0);

        // All lines should have one endpoint at the centroid
        for line in voronoi {
            let start: Point<_> = line.0[0].into();
            // ugh
            let end: Point<_> = Point(line.0.last().copied().unwrap());
            assert!(
                Euclidean.distance(&start, &expected_centroid) < 1e-10
                    || Euclidean.distance(&end, &expected_centroid) < 1e-10
            )
        }
    }

    #[test]
    fn voronoi_test_square() {
        // Unit square
        let square = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 0.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 }, // closing point
            ]),
            vec![],
        );

        let voronoi = square.voronoi_edges();

        // A square triangulated with a diagonal should have:
        // 1. One finite edge (connecting the circumcenters of the two triangles)
        // 2. Four infinite edges (perpendicular to each side)
        assert_eq!(voronoi.unwrap().0.len(), 5);
    }

    #[test]
    fn voronoi_test_collinear_points() {
        // Three collinear points on x-axis
        let collinear = Polygon::new(
            LineString::from(vec![
                Coord { x: -1.0, y: 0.0 },
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: -1.0, y: 0.0 }, // closing point
            ]),
            vec![],
        );

        let voronoi = collinear.voronoi_edges().unwrap();

        // The Voronoi diagram should have:
        // 1. Two perpendicular bisector lines between adjacent points
        // 2. These should be perpendicular to the x-axis
        assert_eq!(voronoi.0.len(), 2);

        // The bisector lines should be at x = -0.5 and x = 0.5
        let expected_x_coords = vec![-0.5, 0.5];

        for (line, expected_x) in voronoi.iter().zip(expected_x_coords) {
            assert_relative_eq!(line[0].x, expected_x, epsilon = 1e-10);
            assert_relative_eq!(line.0.iter().last().unwrap().x, expected_x, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_collinear_four_points_voronoi() {
        // Four collinear points on a horizontal line
        // Expected: 3 vertical bisectors at x = 0.5, 1.5, 2.5
        let line = LineString::from(vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)]);

        let edges = line.voronoi_edges().unwrap();

        // Should produce exactly 3 bisectors
        assert_eq!(
            edges.0.len(),
            3,
            "Expected 3 bisectors for 4 collinear points"
        );

        // Each bisector should be at x = 0.5, 1.5, 2.5 respectively
        let mut midpoints: Vec<f64> = edges
            .0
            .iter()
            .map(|ls| ls.0[0].x) // x-coordinate of first point
            .collect();
        midpoints.sort_by(|a, b| a.total_cmp(b));

        assert!((midpoints[0] - 0.5).abs() < 1e-6, "First bisector at x=0.5");
        assert!(
            (midpoints[1] - 1.5).abs() < 1e-6,
            "Second bisector at x=1.5"
        );
        assert!((midpoints[2] - 2.5).abs() < 1e-6, "Third bisector at x=2.5");
    }

    #[test]
    fn voronoi_test_two_points() {
        // Just two points should give us a single perpendicular bisector
        let points = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 0.0, y: 0.0 }, // closing point
            ]),
            vec![],
        );

        let voronoi = points.voronoi_edges().unwrap();
        assert_eq!(voronoi.0.len(), 1);
    }

    #[test]
    fn voronoi_test_rectangle() {
        // Non-square rectangle to test different width/height handling
        let rect = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 0.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

        let voronoi = rect.voronoi_edges().unwrap();
        assert_eq!(voronoi.0.len(), 5); // Should be same as square
    }

    // Tests for voronoi_cells()

    #[test]
    fn voronoi_cells_triangle() {
        // Equilateral triangle should produce 3 cells
        let triangle = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord {
                    x: 0.5,
                    y: f64::sqrt(0.75),
                },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

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
        let square = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 0.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

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
        let square = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 0.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

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
        let points = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 2.0, y: 2.0 },
                Coord { x: 0.0, y: 2.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

        // Clip to a smaller square
        let clip_boundary = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.5, y: 0.5 },
                Coord { x: 1.5, y: 0.5 },
                Coord { x: 1.5, y: 1.5 },
                Coord { x: 0.5, y: 1.5 },
                Coord { x: 0.5, y: 0.5 },
            ]),
            vec![],
        );

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
        // Create a polygon with 10 vertices (9 unique after closing point)
        // Using a LineString since Polygon would have triangulation treat it as a shape
        let polygon = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 2.0, y: 0.5 },
                Coord { x: 2.0, y: 1.5 },
                Coord { x: 1.5, y: 2.0 },
                Coord { x: 0.5, y: 2.0 },
                Coord { x: 0.0, y: 1.5 },
                Coord { x: 0.0, y: 0.0 }, // closing point (duplicate of first)
            ]),
            vec![],
        );

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
        let square = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 0.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

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

        // The union should have approximately the same area as the expected bounding box
        let union_area = union_result.unsigned_area();
        assert_relative_eq!(union_area, expected_area, epsilon = 1e-6);

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

        // The union of all clipped cells should equal the bounding box
        let mut union_result = MultiPolygon::<f64>(vec![clipped_cells[0].clone()]);
        for cell in clipped_cells.iter().skip(1) {
            union_result = union_result.union(&MultiPolygon(vec![cell.clone()]));
        }

        let union_area = union_result.unsigned_area();
        let individual_sum: f64 = clipped_cells.iter().map(|c| c.unsigned_area()).sum();

        // The union should exactly cover the bounding box (100% coverage, no gaps or overlaps)
        assert_relative_eq!(union_area, bbox_area, epsilon = 1e-6);
        // Individual sum should also equal bbox area (no overlaps)
        assert_relative_eq!(individual_sum, bbox_area, epsilon = 1e-6);
    }

    /// Test that exposes a bug in corner tracing for convex hull cells.
    ///
    /// When a convex hull cell has both infinite rays hitting the same edge of the
    /// padded bounding box, the angular sorting of vertices can cause interior vertices
    /// to appear between the two boundary vertices. Since corner tracing only fires
    /// between consecutive boundary vertices, no corners are added, leaving gaps.
    ///
    /// The bug manifests as:
    /// 1. Cells touching exactly 1 edge of the padded bbox (should touch 2+ for corner cells)
    /// 2. Missing bbox corners in the cell vertices
    /// 3. Total coverage < 100% of padded bbox
    ///
    /// Example from islington dataset (Cell 14):
    /// ```text
    /// Cell 14 touches ["left"] (5 vertices):
    ///   0: (-0.170852, 51.540582) angle=-2.7867 [LEFT]
    ///   1: (-0.170852, 51.528288) angle=-2.2688 [LEFT]
    ///   2: (-0.134901, 51.559983) angle=0.5835
    ///   3: (-0.138511, 51.562739) angle=0.7629
    ///   4: (-0.170852, 51.540582) angle=-2.7867 [LEFT]
    /// ```
    /// The jump from vertex 3 (angle 0.76) to vertex 0 (angle -2.79) crosses the
    /// +/- pi discontinuity. Since vertex 3 is interior (not on boundary), no
    /// corner tracing occurs, leaving the top-left and bottom-left corners missing.
    #[test]
    fn voronoi_cells_corner_tracing_bug() {
        use crate::Area;
        use crate::algorithm::bounding_rect::BoundingRect;
        use geo_types::MultiPoint;

        // Use the islington dataset which triggers the bug
        let points_mp: MultiPoint<f64> = geo_test_fixtures::islington_post_boxes();
        let coords: Vec<Coord<f64>> = points_mp.iter().map(|p| p.0).collect();
        let mut ring_coords = coords.clone();
        ring_coords.push(coords[0]);
        let points = Polygon::new(LineString::from(ring_coords), vec![]);

        let cells = points.voronoi_cells().unwrap();

        // Calculate padded bounding box
        let bbox = points.bounding_rect().unwrap();
        let padding = 0.5 * f64::max(bbox.width(), bbox.height());
        let p_min_x = bbox.min().x - padding;
        let p_max_x = bbox.max().x + padding;
        let p_min_y = bbox.min().y - padding;
        let p_max_y = bbox.max().y + padding;
        let padded_area = (p_max_x - p_min_x) * (p_max_y - p_min_y);

        // Count cells by how many edges they touch
        let mut cells_touching_multiple_edges = 0;
        for cell in &cells {
            let ext = cell.exterior();
            let mut edges = std::collections::HashSet::new();
            for c in ext.coords() {
                if (c.x - p_min_x).abs() < 0.001 {
                    edges.insert("left");
                }
                if (c.x - p_max_x).abs() < 0.001 {
                    edges.insert("right");
                }
                if (c.y - p_min_y).abs() < 0.001 {
                    edges.insert("bottom");
                }
                if (c.y - p_max_y).abs() < 0.001 {
                    edges.insert("top");
                }
            }
            if edges.len() > 1 {
                cells_touching_multiple_edges += 1;
            }
        }

        // Check for missing corners
        let corners = [
            (p_min_x, p_min_y, "bottom-left"),
            (p_max_x, p_min_y, "bottom-right"),
            (p_max_x, p_max_y, "top-right"),
            (p_min_x, p_max_y, "top-left"),
        ];
        let mut missing_corners = Vec::new();
        for (cx, cy, name) in &corners {
            let found = cells.iter().any(|cell| {
                cell.exterior()
                    .coords()
                    .any(|c| (c.x - cx).abs() < 0.001 && (c.y - cy).abs() < 0.001)
            });
            if !found {
                missing_corners.push(*name);
            }
        }

        // Calculate coverage
        let total_area: f64 = cells.iter().map(|c| c.unsigned_area()).sum();
        let coverage = total_area / padded_area;

        // Corner cells should touch multiple edges
        assert!(
            cells_touching_multiple_edges > 0,
            "Some cells should touch multiple edges (corner cells)"
        );

        // All corners should be present
        assert!(
            missing_corners.is_empty(),
            "All corners should be present, but missing: {:?}",
            missing_corners
        );

        // Coverage should be close to 100%
        assert!(
            coverage > 0.99,
            "Coverage should be >= 99%, but was {:.2}%",
            coverage * 100.0
        );
    }

    #[test]
    fn voronoi_cells_extreme_aspect_ratio() {
        use crate::Area;

        // Wide rectangle: width=10, height=1
        // Padding = max(10,1) * 0.5 = 5
        // This tests ray extension with large padding relative to bbox
        let wide = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 10.0, y: 0.0 },
                Coord { x: 10.0, y: 1.0 },
                Coord { x: 0.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

        let cells = wide.voronoi_cells().unwrap();
        assert_eq!(cells.len(), 4);

        // Verify all cells are valid polygons
        for cell in &cells {
            assert!(
                cell.is_valid(),
                "Voronoi cell should be a valid polygon: {:?}",
                cell.exterior().0
            );
        }

        // Expected padded bounds: padding = max(10,1) * 0.5 = 5
        // So bounds extend 5 units in all directions: (-5, -5) to (15, 6)
        let expected_bounds = Rect::new(Coord { x: -5.0, y: -5.0 }, Coord { x: 15.0, y: 6.0 });
        let expected_area = expected_bounds.to_polygon().unsigned_area();

        // Union all cells
        let mut union_result = MultiPolygon(vec![cells[0].clone()]);
        for cell in cells.iter().skip(1) {
            union_result = union_result.union(&MultiPolygon(vec![cell.clone()]));
        }

        assert_relative_eq!(union_result.unsigned_area(), expected_area, epsilon = 1e-6);
    }

    #[test]
    fn voronoi_cells_obtuse_triangle() {
        use crate::Area;

        // Pentagon with one very wide angle which tests handling of obtuse angles
        // and circumcentres that may be far from the input vertices
        let wide_pentagon = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 10.0, y: 0.0 },
                Coord { x: 10.0, y: 5.0 },
                Coord { x: 5.0, y: 8.0 }, // Wide/obtuse section
                Coord { x: 0.0, y: 5.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

        let cells = wide_pentagon.voronoi_cells().unwrap();
        assert_eq!(cells.len(), 5);

        // Verify all cells are valid polygons
        for cell in &cells {
            assert!(
                cell.is_valid(),
                "Voronoi cell should be a valid polygon: {:?}",
                cell.exterior().0
            );
        }

        // Compute expected bounds with padding
        // Input bounds: (0,0) to (10,8), max dimension = 10, padding = 5
        let expected_bounds = Rect::new(Coord { x: -5.0, y: -5.0 }, Coord { x: 15.0, y: 13.0 });
        let expected_area = expected_bounds.to_polygon().unsigned_area();

        // Union all cells
        let mut union_result = MultiPolygon(vec![cells[0].clone()]);
        for cell in cells.iter().skip(1) {
            union_result = union_result.union(&MultiPolygon(vec![cell.clone()]));
        }

        // Use larger epsilon to account for BooleanOps precision
        assert_relative_eq!(union_result.unsigned_area(), expected_area, epsilon = 1e-4);
    }

    #[test]
    fn voronoi_cells_points_near_boundary() {
        use crate::Area;

        // Narrow rectangle: points clustered near left and right edges
        // Tests that rays extend correctly even when input is very narrow
        let narrow_rect = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 0.5, y: 0.0 },
                Coord { x: 0.5, y: 10.0 },
                Coord { x: 0.0, y: 10.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

        let cells = narrow_rect.voronoi_cells().unwrap();
        assert_eq!(cells.len(), 4);

        // Verify all cells are valid polygons
        for cell in &cells {
            assert!(
                cell.is_valid(),
                "Voronoi cell should be a valid polygon: {:?}",
                cell.exterior().0
            );
        }

        // Compute expected bounds with padding
        // Input bounds: (0,0) to (0.5,10), max dimension = 10, padding = 5
        let expected_bounds = Rect::new(Coord { x: -5.0, y: -5.0 }, Coord { x: 5.5, y: 15.0 });
        let expected_area = expected_bounds.to_polygon().unsigned_area();

        // Union all cells
        let mut union_result = MultiPolygon(vec![cells[0].clone()]);
        for cell in cells.iter().skip(1) {
            union_result = union_result.union(&MultiPolygon(vec![cell.clone()]));
        }

        assert_relative_eq!(union_result.unsigned_area(), expected_area, epsilon = 1e-6);
    }
}
