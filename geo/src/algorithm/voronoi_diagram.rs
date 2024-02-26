use std::{f64::consts::FRAC_PI_2, fmt, result};

use geo_types::coord;

use crate::{Coord, EuclideanLength, GeoFloat, Line, MultiPoint, Point, Polygon, Rect, Triangle};

use crate::triangulate_delaunay::{DelaunayTriangle, DelaunayTriangulationError};
use crate::{BoundingRect, TriangulateDelaunay};

type Result<T> = result::Result<T, VoronoiDiagramError>;

/// The default expansion value for creating a clipping mask.
/// A bounding rectangle is computed for the points and the rectangle
/// is expanded by this value by default.
pub const DEFAULT_CLIPPING_MASK_EXPANSION: f64 = 20.;

/// The default value for determining if the slope of a line is
/// near zero or near infinite.
///
/// A near infinite slope is greater than this value.
///
/// A near zero slope is less than `1 / DEFAULT_SLOPE_THRESHOLD`.
pub const DEFAULT_SLOPE_THRESHOLD: f64 = 1e3;

/// Threshold value used to determine if a triangle is a right triangle
/// using the cosine rule.
const IS_RIGHT_TRIANGLE_THRESHOLD: f64 = 0.00001;

/// The components of a Voronoi Diagram.
#[derive(Debug, Clone)]
pub struct VoronoiComponents<T: GeoFloat> {
    /// The Delaunay Triangles used to construct the Voronoi diagram.
    pub delaunay_triangles: Vec<Triangle<T>>,
    /// The vertices of the Voronoi diagram.
    pub vertices: Vec<Coord<T>>,
    /// The lines of the Voronoi diagram.
    pub lines: Vec<Line<T>>,
}

/// The clipping mask used to trim the infinity lines of
/// the Voronoi diagram.
pub type ClippingMask<T> = Polygon<T>;

/// Compute the Voronoi diagram for a given set of points.
/// This method uses the property that Delaunay triangulation
/// [is a dual graph](https://en.wikipedia.org/wiki/Delaunay_triangulation#Relationship_with_the_Voronoi_diagram)
/// of the Voronoi diagram.
pub trait VoronoiDiagram<T: GeoFloat>
where
    f64: From<T>,
{
    /// Compute the Voronoi diagram components with a clipping mask to trim infinity lines.
    /// If `clipping_mask` is set to None, one will be computed by expanding the geometry's
    /// bounding box, using the [`DEFAULT_CLIPPING_MASK_EXPANSION`] to expand the size of the
    /// bounding box.
    ///
    /// The `slope_threshold` is used when creating the infinity lines to determine lines with a slope
    /// of near zero or near infinity.  If `slope_threshold` is set to None, [`DEFAULT_SLOPE_THRESHOLD`] will
    /// be used.
    ///
    /// # Example
    /// ```rust
    /// use geo::{coord, polygon, VoronoiDiagram};
    /// let poly = polygon![
    ///     coord!{ x: 10., y: 10.},
    ///     coord!{ x: 10., y: 20.},
    ///     coord!{ x: 20., y: 20.},
    ///     coord!{ x: 20., y: 10.},
    ///     coord!{ x: 10., y: 0.},
    ///     coord!{ x: 10., y: 0.},
    ///     coord!{ x: 0., y: 10.},
    ///     coord!{ x: 0., y: 20.},
    /// ];
    /// let voronoi = poly.compute_voronoi_components(None, None).unwrap();
    /// assert_eq!(voronoi.lines.len(), 12);
    /// assert_eq!(voronoi.vertices.len(), 6);
    /// ```
    fn compute_voronoi_components(
        &self,
        clipping_mask: Option<&ClippingMask<T>>,
        slope_threshold: Option<T>,
    ) -> Result<VoronoiComponents<T>>;
}

impl<T: GeoFloat> VoronoiDiagram<T> for Polygon<T>
where
    f64: From<T>,
{
    fn compute_voronoi_components(
        &self,
        clipping_mask: Option<&ClippingMask<T>>,
        slope_threshold: Option<T>,
    ) -> Result<VoronoiComponents<T>> {
        compute_voronoi_components(self, clipping_mask, slope_threshold)
    }
}
impl<T: GeoFloat> VoronoiDiagram<T> for MultiPoint<T>
where
    f64: From<T>,
{
    fn compute_voronoi_components(
        &self,
        clipping_mask: Option<&ClippingMask<T>>,
        slope_threshold: Option<T>,
    ) -> Result<VoronoiComponents<T>> {
        compute_voronoi_components(self, clipping_mask, slope_threshold)
    }
}

fn compute_voronoi_components<T: GeoFloat, U: TriangulateDelaunay<T>>(
    polygon: &U,
    clipping_mask: Option<&ClippingMask<T>>,
    slope_threshold: Option<T>,
) -> Result<VoronoiComponents<T>>
where
    f64: From<T>,
{
    let triangles = polygon
        .delaunay_triangulation()
        .map_err(VoronoiDiagramError::DelaunayError)?;

    if triangles.is_empty() {
        return Ok(VoronoiComponents {
            delaunay_triangles: vec![],
            vertices: vec![],
            lines: vec![],
        });
    }

    compute_voronoi_components_from_delaunay(&triangles, clipping_mask, slope_threshold)
}

/// Compute the Voronoi diagram from Delaunay triangles.
/// The Voronoi diagram is a [dual graph](https://en.wikipedia.org/wiki/Dual_graph)
/// of the [Delaunay triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation)
/// and thus the Voronoi diagram can be created from the Delaunay triangulation.
/// The `clipping_mask` is used to constrain the generated infinity lines,
/// if set to `None` a rectangle is determined by expanding the bounding box
/// constraining the points of the triangles by a factor of [`DEFAULT_CLIPPING_MASK_EXPANSION`].
/// The `slope_threshold` is used to determine if an infinity line has a slope of near infininty
/// or near zero.
/// When set to `None` the `slope_threshold` is set to a default value of `['DEFAULT_SLOPE_THRESHOLD`].
/// Changing the value of `slope_threshold` may affect the direction of some infinity lines and should
/// be modified with care.
fn compute_voronoi_components_from_delaunay<T: GeoFloat>(
    triangles: &[Triangle<T>],
    clipping_mask: Option<&Polygon<T>>,
    slope_threshold: Option<T>,
) -> Result<VoronoiComponents<T>>
where
    f64: From<T>,
{
    if triangles.is_empty() {
        return Ok(VoronoiComponents {
            delaunay_triangles: vec![],
            vertices: vec![],
            lines: vec![],
        });
    }

    let clipping_mask = match clipping_mask {
        Some(mask) => mask.to_owned(),
        None => create_clipping_mask(triangles)?,
    };

    let delaunay_triangles: Vec<DelaunayTriangle<T>> =
        triangles.iter().map(|x| (*x).into()).collect();

    // The centres of the delaunay circumcircles form the vertices of the
    // voronoi diagram
    let mut vertices: Vec<Coord<T>> = Vec::new();
    for tri in &delaunay_triangles {
        let vertex = tri
            .get_circumcircle_centre()
            .map_err(VoronoiDiagramError::DelaunayError)?;
        vertices.push(vertex);
    }

    // Find the shared edges
    let mut shared = find_shared_edges(&delaunay_triangles)?;

    // Create the lines joining the vertices
    let mut voronoi_lines: Vec<Line<T>> = Vec::new();
    for neighbour in &shared.neighbours {
        if let (Some(first_vertex), Some(second_vertex)) = (neighbour.0, neighbour.1) {
            voronoi_lines.push(Line::new(vertices[first_vertex], vertices[second_vertex]));
        }
    }

    // Create the lines to infinity
    voronoi_lines.extend(construct_edges_to_inf(
        &delaunay_triangles,
        &vertices,
        &mut shared.shared_edges,
        &shared.neighbours,
        &clipping_mask,
        slope_threshold,
    )?);

    Ok(VoronoiComponents {
        delaunay_triangles: triangles.to_vec(),
        vertices,
        lines: voronoi_lines,
    })
}

fn create_clipping_mask<T: GeoFloat>(triangles: &[Triangle<T>]) -> Result<Polygon<T>>
where
    f64: From<T>,
{
    let expand_factor = T::from(DEFAULT_CLIPPING_MASK_EXPANSION)
        .ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;

    let mut pts: Vec<Point<T>> = Vec::new();
    for tri in triangles {
        pts.extend(tri.to_array().iter().map(|x| Point::from(*x)));
    }
    let bounds = MultiPoint::new(pts)
        .bounding_rect()
        .ok_or(VoronoiDiagramError::CannotDetermineBoundsFromClipppingMask)?;

    let width = bounds.width() * expand_factor;
    let height = bounds.height() * expand_factor;
    let bounds_min = bounds.min();

    Ok(Polygon::from(Rect::new(
        coord! { x: bounds_min.x - width, y: bounds_min.y - height},
        coord! { x: bounds_min.x + width, y: bounds_min.y + height},
    )))
}

// types to make clippy happy
type Neighbour = (Option<usize>, Option<usize>);
type Neighbours = Vec<Neighbour>;

#[derive(Debug, Clone)]
struct SharedEdgesData<T: GeoFloat> {
    neighbours: Neighbours,
    shared_edges: Vec<Line<T>>,
}

fn find_shared_edges<T: GeoFloat>(triangles: &[DelaunayTriangle<T>]) -> Result<SharedEdgesData<T>>
where
    f64: From<T>,
{
    let mut neighbours: Vec<(Option<usize>, Option<usize>)> = Vec::new();
    let mut shared_edges: Vec<Line<T>> = Vec::new();

    // Search the delaunay triangles for neighbour triangles and shared edges
    for (tri_idx, tri) in triangles.iter().enumerate() {
        for (other_idx, other_tri) in triangles.iter().enumerate() {
            if tri_idx == other_idx {
                continue;
            }

            if let Some(shared_edge) = tri.shares_edge(other_tri) {
                if !neighbours.contains(&(Some(tri_idx), Some(other_idx)))
                    && !neighbours.contains(&(Some(other_idx), Some(tri_idx)))
                {
                    neighbours.push((Some(tri_idx), Some(other_idx)));
                }

                let flipped_edge = Line::new(shared_edge.end, shared_edge.start);

                if !shared_edges.contains(&shared_edge) && !shared_edges.contains(&flipped_edge) {
                    shared_edges.push(shared_edge);
                }
            }
        }
    }

    // For Voronoi diagrams, the triangles / circumcentres that are on the edge of the
    // diagram require connections to infinity to ensure separation of points between
    // voronoi cells. Voronoi cells on the borders can have 2 connections to infinity.
    // These connections to infinity will be bounded later, for now add the connections from infinity.
    let mut num_neighbours: Vec<usize> = vec![0; triangles.len()];
    for x in &neighbours {
        // unwrap here is safe as neighbours do not contain None values yet
        for n in [x.0.unwrap(), x.1.unwrap()] {
            num_neighbours[n] += 1;
        }
    }

    for (idx, val) in num_neighbours.iter().enumerate() {
        if *val > 3 {
            return Err(VoronoiDiagramError::InvalidTriangulation);
        }
        for _ in 0..3 - val {
            neighbours.push((None, Some(idx)));
        }
    }

    Ok(SharedEdgesData {
        neighbours,
        shared_edges,
    })
}

fn get_perpendicular_line<T: GeoFloat>(line: &Line<T>) -> Result<Line<T>>
where
    f64: From<T>,
{
    let two = T::from(2.).ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;
    let minus_one = T::from(-1.).ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;
    let slope = line.slope();

    // Vertical line
    if slope.is_infinite() {
        Ok(Line::new(
            coord! {x: line.start.x, y: line.start.y},
            coord! {x: line.start.x - line.dy(), y: line.start.y},
        ))
    } else if slope.is_zero() {
        Ok(Line::new(
            coord! {x: line.start.x, y: line.start.y},
            coord! {x: line.start.x, y: line.start.y + line.dx()},
        ))
    } else {
        let midpoint = coord! {
            x: line.start.x + line.dx() / two,
            y: line.start.y + line.dy() / two,
        };
        // y = mx + b
        let m = minus_one / slope;
        let b = m.mul_add(-midpoint.x, midpoint.y);
        let end_x = midpoint.x + line.dx();
        let end_y = m.mul_add(end_x, b);

        Ok(Line::new(midpoint, coord! {x: end_x, y: end_y}))
    }
}

fn cosine_rule<T: GeoFloat>(line_a: &Line<T>, line_b: &Line<T>, line_c: &Line<T>) -> Result<f64>
where
    f64: From<T>,
{
    let two = T::from(2.).ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;
    let a = line_a.euclidean_length();
    let b = line_b.euclidean_length();
    let c = line_c.euclidean_length();

    let arg = ((a * a) + (b * b) - (c * c)) / (two * a * b);
    let arg: f64 = arg.into();
    let ang = arg.acos();
    Ok(ang)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CircumCentreLocation {
    Inside,
    Outside,
    On,
}

impl CircumCentreLocation {
    fn from_triangle<T: GeoFloat>(triangle: &Triangle<T>) -> Result<CircumCentreLocation>
    where
        f64: From<T>,
    {
        // Determine if the triangle contains its circumcentre
        // https://en.wikipedia.org/wiki/Circumcircle#Location_relative_to_the_triangle
        // Use the cosine rule to determine the angles
        let corners = triangle.to_array();
        // Need to ensure the order of the lines is correct for
        // getting the angle between the lines
        let lines = [
            Line::new(corners[0], corners[1]),
            Line::new(corners[0], corners[2]),
            Line::new(corners[1], corners[2]),
        ];
        let angles = [
            cosine_rule(&lines[0], &lines[1], &lines[2])?,
            cosine_rule(&lines[1], &lines[2], &lines[0])?,
            cosine_rule(&lines[0], &lines[2], &lines[1])?,
        ];
        let is_obtuse_triangle = angles.iter().any(|x| *x > FRAC_PI_2);
        let is_acute_triangle = angles.iter().all(|x| *x < FRAC_PI_2);
        let is_right_triangle = angles
            .iter()
            .any(|x| ((*x) - FRAC_PI_2).abs() < IS_RIGHT_TRIANGLE_THRESHOLD);

        Ok(if is_right_triangle {
            CircumCentreLocation::On
        } else if is_acute_triangle {
            CircumCentreLocation::Inside
        } else if is_obtuse_triangle {
            CircumCentreLocation::Outside
        } else {
            return Err(VoronoiDiagramError::CannotDetermineCircumcentrePosition);
        })
    }
}

/// Get the infinity line from inside to outside the triangle.
/// Infinity lines that start from within the triangle to outside
/// need to start at the circumcentre and move towards infinity.
///
///```ascii
///       @           E
///      @ @
///     @   @
///    @     M
///   @   C   @
///  @         @
///  @ @ @ @ @ @
///
///  M: Midpoint
///  C: Circumecentre
///  E: End of infinity line
///```
fn get_inf_line_in_out_triangle<T: GeoFloat>(inf_line: Line<T>, circumcentre: &Coord<T>) -> Line<T>
where
    f64: From<T>,
{
    let slope = inf_line.slope();
    let end = if slope.is_infinite() {
        let end_x = circumcentre.x;
        let end_y = circumcentre.y + inf_line.dy();
        coord! {x: end_x, y: end_y}
    } else {
        let intercept = circumcentre.y - slope * circumcentre.x;
        let end_x = circumcentre.x + inf_line.dx();
        let end_y = slope * end_x + intercept;
        coord! {x: end_x, y: end_y}
    };
    Line::new(*circumcentre, end)
}

/// Determine is the slope of a line is near infinity or zero.
/// This function has been implemented to prevent issues with
/// very large or very small slopes that are not found to be
/// `is_infinite` or `is_zero` by type T.
fn is_slope_near_zero_or_inf<T: GeoFloat>(
    line: &Line<T>,
    slope_threshold: Option<T>,
) -> Result<(bool, bool)>
where
    f64: From<T>,
{
    let default_thresh = T::from(DEFAULT_SLOPE_THRESHOLD)
        .ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;

    let threshold = slope_threshold.unwrap_or(default_thresh);

    let slope_near_infinite = line.slope().is_infinite() || line.slope().abs() > threshold;
    let slope_near_zero = line.slope().is_zero() || line.slope().abs() < threshold.powi(-1);

    Ok((slope_near_zero, slope_near_infinite))
}

/// Construct the infinity line where the circumcentre lies outside of the triangle.
/// Infinity lines need to travel towards infinity passing through the midpoint.
/// This function constructs the infinity line, ensuring the infinity line travels outwards
/// from the triangle and does not intersect any of the other edges of the triangle.
///
///```ascii
///           @
///          @  @
///         @    @
///        @      @
///       @     @
///      @    @
///     @   M  
///    @  @    C
///   @ @
///  @
///                        E
///
///  M: Midpoint
///  C: Circumecentre
///  E: End of infinity line
///```
fn get_inf_outside_triangle<T: GeoFloat>(
    triangle: &Triangle<T>,
    circumcentre: &Coord<T>,
    midpoint: &Coord<T>,
    slope_threshold: Option<T>,
) -> Result<Line<T>>
where
    f64: From<T>,
{
    let two = T::from(2.).ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;

    // The circumcentre is outside the triangle so start by constructing a line from the
    // midpoint to the circumcentre.
    let inf_line = Line::new(*midpoint, *circumcentre);
    let (slope_near_zero, slope_near_infinite) =
        is_slope_near_zero_or_inf(&inf_line, slope_threshold)?;

    // If the infinity line crosses one of the other lines of the triangle
    // the direction of the infinity line needs to be flipped.
    // Get the other lines of the triangle
    let mut other_lines: Vec<_> = Vec::new();
    for line in triangle.to_lines() {
        let x = line.start + coord! {x: line.dx() / two, y: line.dy() / two};
        if x != *midpoint {
            other_lines.push(line);
        }
    }

    let mut intersections: bool = false;

    // Check for intersections
    for line in other_lines {
        if let Some(inter) = trim_line_to_intersection(&inf_line, &line) {
            let x_point = inter.end;
            // If the intersection point is lies within the bounds of the
            // infinity line and the other line then it intersects the triangle
            let inf_line_between_x = x_point.x > inf_line.start.x && x_point.x < inf_line.end.x
                || x_point.x > inf_line.end.x && x_point.x < inf_line.start.x;
            let inf_line_between_y = x_point.y > inf_line.start.y && x_point.y < inf_line.end.y
                || x_point.y > inf_line.end.y && x_point.y < inf_line.start.y;
            let line_between_x = x_point.x > line.start.x && x_point.x < line.end.x
                || x_point.x > line.end.x && x_point.x < line.start.x;
            let line_between_y = x_point.y > line.start.y && x_point.y < line.end.y
                || x_point.y > line.end.y && x_point.y < line.start.y;

            let intersects_tmp = if slope_near_zero {
                inf_line_between_x
            } else if slope_near_infinite {
                inf_line_between_y
            } else {
                inf_line_between_x && inf_line_between_y
            };
            let intersects_line = line_between_x && line_between_y;

            if intersects_line && intersects_tmp {
                intersections = true;
                break;
            }
        }
    }

    let tmp_line = if intersections {
        Line::new(*circumcentre, *midpoint)
    } else {
        inf_line
    };

    Ok(get_inf_line_in_out_triangle(tmp_line, circumcentre))
}

// A convenience wrapper to help testing
fn get_inf_inside_triangle<T: GeoFloat>(circumcentre: &Coord<T>, midpoint: &Coord<T>) -> Line<T>
where
    f64: From<T>,
{
    get_inf_line_in_out_triangle(Line::new(*circumcentre, *midpoint), circumcentre)
}

fn get_incentre<T: GeoFloat>(triangle: &Triangle<T>) -> Coord<T>
where
    f64: From<T>,
{
    let pt_a = triangle.0;
    let line_a = Line::new(triangle.1, triangle.2);
    let len_a = line_a.euclidean_length();
    let pt_b = triangle.1;
    let line_b = Line::new(triangle.0, triangle.2);
    let len_b = line_b.euclidean_length();
    let pt_c = triangle.2;
    let line_c = Line::new(triangle.0, triangle.1);
    let len_c = line_c.euclidean_length();

    let x = (len_a * pt_a.x + len_b * pt_b.x + len_c * pt_c.x) / (len_a + len_b + len_c);
    let y = (len_a * pt_a.y + len_b * pt_b.y + len_c * pt_c.y) / (len_a + len_b + len_c);

    coord! {x: x, y: y}
}

/// Compute the infinity line for a triangle where the circumcentre lies on the midpoint.
/// In this situations you cannot determine the correct direction of the infinity line
/// using the circumcentre and the midpoint.
/// The correct direction is the path away from the midpoints of the other edges
/// of the triangle.
/// This commonly occurs with right triangles.
///
///```ascii
///  @                   E
///  @ @
///  @  @
///  @   @
///  @    @
///  X     MC
///  @      @
///  @       @
///  @        @
///  @ @ X @ @ @
///
///
///  M: Midpoint
///  C: Circumecentre
///  E: End of infinity line
///  X: Midpoint of other edges of the triangle
///```
fn get_inf_on_midpoint_triangle<T: GeoFloat>(
    triangle: &Triangle<T>,
    edge: &Line<T>,
    circumcentre: &Coord<T>,
    midpoint: &Coord<T>,
) -> Result<Line<T>>
where
    f64: From<T>,
{
    // While the circumcentre and midpoint may equal on one edge of the triangle,
    // this is only the case for one of the edges of the triangle.
    // If this is not one of those cases break out earlier, using the existing
    // method.
    if midpoint != circumcentre {
        return Ok(get_inf_line_in_out_triangle(
            Line::new(*circumcentre, *midpoint),
            circumcentre,
        ));
    }
    // The midpoint is on the circumcentre so we need to use the other midpoints to determine direction
    // Construct the perpendicular line
    let line: Line<T> = get_perpendicular_line(edge)?;
    let incentre = get_incentre(triangle);
    let guiding_line = Line::new(incentre, *circumcentre);
    let end_x = if guiding_line.dx().is_negative() {
        circumcentre.x - line.dx().abs()
    } else {
        circumcentre.x + line.dx().abs()
    };
    let end_y = if guiding_line.dy().is_negative() {
        circumcentre.y - line.dy().abs()
    } else {
        circumcentre.y + line.dy().abs()
    };
    Ok(Line::new(*circumcentre, coord! {x: end_x, y: end_y}))
}

// Edges to infinity need to start from the circumcentre and travel
// outwards from the Delaunay triangle to infinity, passing through
// the midpoint of the outer line of the triangle.
// This function constructs these lines, accounting for the
// various positions of the midpoints and circumcentres.
fn define_edge_to_infinity<T: GeoFloat>(
    triangle: &DelaunayTriangle<T>,
    circumcentre: &Coord<T>,
    shared_edges: &mut Vec<Line<T>>,
    slope_threshold: Option<T>,
) -> Result<Option<Line<T>>>
where
    f64: From<T>,
{
    let two = T::from(2.).ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;

    let tri: Triangle<T> = triangle.clone().into();
    let midpoints: Vec<_> = tri
        .to_lines()
        .iter()
        .map(|edge| edge.start + coord! {x: edge.dx() / two, y: edge.dy() / two})
        .collect();

    for (edge, midpoint) in tri.to_lines().iter().zip(&midpoints) {
        let flipped_edge = Line::new(edge.end, edge.start);
        if shared_edges.contains(edge) || shared_edges.contains(&flipped_edge) {
            continue;
        }

        let circumcentre_location = CircumCentreLocation::from_triangle(&tri)?;

        let inf_line = match circumcentre_location {
            // The circumcentre lies inside the triangle so compute accordingly.
            CircumCentreLocation::Inside => get_inf_inside_triangle(circumcentre, midpoint),
            // The circumcentre lies outside the triangle, use the slope threshold to determine if the
            // lines are horizontal or vertical.
            CircumCentreLocation::Outside => {
                get_inf_outside_triangle(&tri, circumcentre, midpoint, slope_threshold)?
            }
            // The midpoint of the outer edge lies on the circumcentre of the triangle.
            // Need to ensure the line is passing outward and not through the triangle to
            // infinity.
            CircumCentreLocation::On => {
                get_inf_on_midpoint_triangle(&tri, edge, circumcentre, midpoint)?
            }
        };

        shared_edges.push(*edge);

        return Ok(Some(inf_line));
    }
    Ok(None)
}

fn trim_line_to_intersection<T: GeoFloat>(
    inf_line: &Line<T>,
    bounding_line: &Line<T>,
) -> Option<Line<T>>
where
    f64: From<T>,
{
    let (x1, y1) = inf_line.start.x_y();
    let (x2, y2) = inf_line.end.x_y();
    let (x3, y3) = bounding_line.start.x_y();
    let (x4, y4) = bounding_line.end.x_y();

    let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    if denom.is_zero() {
        return None;
    }

    let p_x = (x1 * y2 - y1 * x2) * (x3 - x4) - (x1 - x2) * (x3 * y4 - y3 * x4);
    let p_y = (x1 * y2 - y1 * x2) * (y3 - y4) - (y1 - y2) * (x3 * y4 - y3 * x4);

    let p_x = p_x / denom;
    let p_y = p_y / denom;

    // Trim the inf_line at the intersection location
    Some(Line::new(inf_line.start, coord! { x: p_x, y: p_y}))
}

// Construct the edges to infinity.
fn construct_edges_to_inf<T: GeoFloat>(
    triangles: &[DelaunayTriangle<T>],
    vertices: &[Coord<T>],
    edges: &mut Vec<Line<T>>,
    neighbours: &[(Option<usize>, Option<usize>)],
    clipping_mask: &Polygon<T>,
    slope_threshold: Option<T>,
) -> Result<Vec<Line<T>>>
where
    f64: From<T>,
{
    // Get the vertices with connections to infinity
    let mut inf_lines: Vec<Line<T>> = Vec::new();
    for neighbour in neighbours {
        if let (None, Some(tri_idx)) = (neighbour.0, neighbour.1) {
            let inf_edge = define_edge_to_infinity(
                &triangles[tri_idx],
                &vertices[tri_idx],
                edges,
                slope_threshold,
            )?
            .ok_or(VoronoiDiagramError::CannotComputeExpectedInfinityVertex)?;
            let inf_edge_dx_sign = inf_edge.dx().is_positive();
            let inf_edge_dy_sign = inf_edge.dy().is_positive();
            let (inf_edge_slope_near_zero, inf_edge_slope_near_inf) =
                is_slope_near_zero_or_inf(&inf_edge, slope_threshold)?;

            // Get the clipping mask line where the inf_vertex intersects
            let mut intersection_lines: Vec<Line<T>> = Vec::new();
            for line in clipping_mask.exterior().lines() {
                if let Some(inf_line) = trim_line_to_intersection(&inf_edge, &line) {
                    let same_dx = inf_line.dx().is_positive() == inf_edge_dx_sign;
                    let same_dy = inf_line.dy().is_positive() == inf_edge_dy_sign;
                    if (inf_edge_slope_near_zero && same_dx)
                        || (inf_edge_slope_near_inf && same_dy)
                        || (same_dx && same_dy)
                    {
                        intersection_lines.push(inf_line);
                    }
                }
            }
            let line_idx = if intersection_lines.len() > 1 {
                // get the shortest line
                let mut min_length: f64 = f64::INFINITY;
                let mut line_idx = 0;
                for (idx, line) in intersection_lines.iter().enumerate() {
                    let length = f64::from(line.euclidean_length());
                    if length < min_length {
                        min_length = length;
                        line_idx = idx;
                    }
                }
                line_idx
            } else {
                0
            };

            if let Some(line) = intersection_lines.get(line_idx) {
                inf_lines.push(*line);
            } else {
                // There is no intersection, the infinity line could be
                // outside the bounds of the clipping mask
                inf_lines.push(inf_edge);
            }
        }
    }
    Ok(inf_lines)
}

/// Voronoi diagram errors.
#[derive(Debug, PartialEq, Eq)]
pub enum VoronoiDiagramError {
    /// An error occurred when attempting to complete Delaunay triangulation
    /// before computing the Voronoi diagram.
    DelaunayError(DelaunayTriangulationError),
    /// This error occurs when a value cannot be converted to the core
    /// number type of the [`Polygon`] or [`MultiPoint`] being used to generate
    /// the voronoi diagram.
    /// Typically this would occur because a float being used in the construction
    /// cannot be converted into T when calculating the Voronoi diagram for either
    /// `Polygon<T>` or `MultiPoint<T>`.
    CannotConvertBetweenGeoGenerics,
    /// This error occurs when the bounding box cannot be determined when creating
    /// a clipping mask for the points.
    CannotDetermineBoundsFromClipppingMask,
    /// This error occurs when a vertex to infinity is expected but cannot be computed.
    CannotComputeExpectedInfinityVertex,
    /// This error occurs when the position of a [`Triangle`]'s circumcentre cannot be determined.
    CannotDetermineCircumcentrePosition,
    /// The delaunay triangulation is invalid.
    InvalidTriangulation,
}

impl fmt::Display for VoronoiDiagramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VoronoiDiagramError::DelaunayError(e) => {
                write!(f, "Delaunay Triangulation error: {}", e)
            }
            VoronoiDiagramError::CannotConvertBetweenGeoGenerics => {
                write!(f, "Cannot convert between Geo generic types")
            }
            VoronoiDiagramError::CannotDetermineBoundsFromClipppingMask => {
                write!(f, "Cannot determine the bounds from the clipping mask")
            }
            VoronoiDiagramError::CannotComputeExpectedInfinityVertex => {
                write!(f, "Cannot compute expected boundary to infinity")
            }
            VoronoiDiagramError::CannotDetermineCircumcentrePosition => {
                write!(
                    f,
                    "Cannot compute if the circumcentre is inside, outside or on the triangle"
                )
            }
            VoronoiDiagramError::InvalidTriangulation => {
                write!(
                    f,
                    "The provided triangles are not valid Delaunay Triangles. \
                    More than 3 connections have been found for a triangle vertex."
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use geo_types::{polygon, LineString};

    fn relative_voronoi_compare(
        voronoi: &VoronoiComponents<f64>,
        expected_vertices: &[Coord<f64>],
        expected_lines: &[Line<f64>],
    ) {
        assert_eq!(voronoi.vertices.len(), expected_vertices.len());
        for (vertex, expected) in voronoi.vertices.iter().zip(expected_vertices) {
            approx::assert_relative_eq!(
                *vertex,
                expected,
                max_relative = 0.3 // epsilon = 1e-3
            );
        }

        assert_eq!(voronoi.lines.len(), expected_lines.len());
        for (line, expected) in voronoi.lines.iter().zip(expected_lines) {
            let flipped_line = Line::new(line.end, line.start);
            let orig_eq = approx::relative_eq!(*line, expected, max_relative = 0.3);
            let flip_eq = approx::relative_eq!(flipped_line, expected, max_relative = 0.3);
            if !(orig_eq || flip_eq) {
                print!("");
            }
            assert!(orig_eq || flip_eq);
        }
    }

    #[test]
    fn test_find_shared_edge() {
        let triangles: Vec<DelaunayTriangle<f64>> = vec![
            Triangle::new(
                coord! {x: 0., y: 0.},
                coord! {x: 1., y: 1.},
                coord! {x: 2., y: 0.},
            )
            .into(),
            Triangle::new(
                coord! {x: 1., y: 1.},
                coord! {x: 2., y: 0.},
                coord! {x: 3., y: 1.},
            )
            .into(),
        ];

        let shared = find_shared_edges(&triangles).unwrap();

        assert_eq!(
            shared.neighbours,
            vec![
                (Some(0), Some(1)),
                (None, Some(0)),
                (None, Some(0)),
                (None, Some(1)),
                (None, Some(1))
            ]
        );

        assert_eq!(
            shared.shared_edges,
            vec![Line::new(coord! { x: 1.0, y: 1.0}, coord! {x: 2.0, y: 0.}),]
        );

        let triangles: Vec<DelaunayTriangle<f64>> = vec![
            Triangle::new(
                coord! {x: 0., y: 0.},
                coord! {x: 0., y: 1.},
                coord! {x: 1., y: 1.},
            )
            .into(),
            Triangle::new(
                coord! {x: 0., y: 1.},
                coord! {x: 1., y: 1.},
                coord! {x: -1., y: 2.},
            )
            .into(),
            Triangle::new(
                coord! {x: 0., y: 0.},
                coord! {x: 0., y: 1.},
                coord! {x: -1., y: 2.},
            )
            .into(),
        ];

        let shared = find_shared_edges(&triangles).unwrap();

        assert_eq!(
            shared.neighbours,
            vec![
                (Some(0), Some(1)),
                (Some(0), Some(2)),
                (Some(1), Some(2)),
                (None, Some(0)),
                (None, Some(1)),
                (None, Some(2))
            ]
        );

        assert_eq!(
            shared.shared_edges,
            vec![
                Line::new(coord! { x: 0.0, y: 1.0}, coord! {x: 1.0, y: 1.}),
                Line::new(coord! { x: 0.0, y: 0.0}, coord! {x: 0.0, y: 1.}),
                Line::new(coord! { x: -1.0, y: 2.0}, coord! {x: 0.0, y: 1.}),
            ]
        );
    }

    #[test]
    fn test_get_circumcircle_location() {
        // Acute triangle
        let triangle = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 3., y: 3.},
            coord! {x: 4., y: 0.},
        );

        assert_eq!(
            CircumCentreLocation::from_triangle(&triangle).unwrap(),
            CircumCentreLocation::Inside
        );

        // Obtuse triangle
        let triangle = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 3., y: 1.},
            coord! {x: 6., y: 0.},
        );

        assert_eq!(
            CircumCentreLocation::from_triangle(&triangle).unwrap(),
            CircumCentreLocation::Outside
        );

        // Right triangle
        let triangle = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 0., y: 3.},
            coord! {x: 4., y: 0.},
        );

        assert_eq!(
            CircumCentreLocation::from_triangle(&triangle).unwrap(),
            CircumCentreLocation::On
        );
    }

    #[test]
    fn test_get_inf_inside_triangle() {
        //Acute triangle pointing down
        let triangle: DelaunayTriangle<_> = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 3., y: 3.},
            coord! {x: 4., y: 0.},
        )
        .into();

        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 2., y: 0.};

        let inf_line = get_inf_inside_triangle(&circumcentre, &midpoint);
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 2., y: 0.}));

        //Acute triangle pointing up
        let triangle: DelaunayTriangle<_> = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 3., y: -3.},
            coord! {x: 4., y: 0.},
        )
        .into();

        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 2., y: 0.};

        let inf_line = get_inf_inside_triangle(&circumcentre, &midpoint);
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 2., y: 0.}));

        //Acute triangle pointing left
        let triangle: DelaunayTriangle<_> = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 3., y: -3.},
            coord! {x: 0., y: 4.},
        )
        .into();

        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 0., y: 2.};

        let inf_line = get_inf_inside_triangle(&circumcentre, &midpoint);
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 0., y: 2.}));

        //Acute triangle pointing right
        let triangle: DelaunayTriangle<_> = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: -4., y: 2.},
            coord! {x: 0., y: 4.},
        )
        .into();

        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 0., y: 2.};

        let inf_line = get_inf_inside_triangle(&circumcentre, &midpoint);
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 0., y: 2.}));

        //Acute triangle angle right
        let triangle: DelaunayTriangle<_> = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: -1., y: 1.},
            coord! {x: 2., y: 2.},
        )
        .into();

        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 1., y: 1.};

        let inf_line = get_inf_inside_triangle(&circumcentre, &midpoint);
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 1.0, y: 1.0}));

        //Acute triangle angle left
        let triangle: DelaunayTriangle<_> = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 3., y: 1.},
            coord! {x: 2., y: 2.},
        )
        .into();

        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 1., y: 1.};

        let inf_line = get_inf_inside_triangle(&circumcentre, &midpoint);
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 1.0, y: 1.0}));
    }

    #[test]
    fn test_get_inf_outside_triangle() {
        // Obtuse triangle inf line pointing down
        let triangle = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 3., y: 1.},
            coord! {x: 6., y: 0.},
        );

        assert_eq!(
            CircumCentreLocation::from_triangle(&triangle).unwrap(),
            CircumCentreLocation::Outside
        );

        let triangle: DelaunayTriangle<_> = triangle.into();
        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 3., y: 0.};
        let inf_line =
            get_inf_outside_triangle(&triangle.into(), &circumcentre, &midpoint, None).unwrap();
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 3., y: -8.}));

        // Obtuse triangle inf line pointing up
        let triangle = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: -3., y: 1.},
            coord! {x: 3., y: 1.},
        );
        assert_eq!(
            CircumCentreLocation::from_triangle(&triangle).unwrap(),
            CircumCentreLocation::Outside
        );

        let triangle: DelaunayTriangle<_> = triangle.into();
        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 0., y: 1.};
        let inf_line =
            get_inf_outside_triangle(&triangle.into(), &circumcentre, &midpoint, None).unwrap();
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 0., y: 9.0}));

        // Obtuse triangle inf line pointing left
        let triangle = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 0., y: 6.},
            coord! {x: 1., y: 3.},
        );
        assert_eq!(
            CircumCentreLocation::from_triangle(&triangle).unwrap(),
            CircumCentreLocation::Outside
        );

        let triangle: DelaunayTriangle<_> = triangle.into();
        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 0., y: 3.};
        let inf_line =
            get_inf_outside_triangle(&triangle.into(), &circumcentre, &midpoint, None).unwrap();
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: -8., y: 3.}));

        // Obtuse triangle at an angle pointing down
        let triangle = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 6., y: 6.},
            coord! {x: -1., y: 3.},
        );
        assert_eq!(
            CircumCentreLocation::from_triangle(&triangle).unwrap(),
            CircumCentreLocation::Outside
        );

        let triangle: DelaunayTriangle<_> = triangle.into();
        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 3., y: 3.};
        let inf_line =
            get_inf_outside_triangle(&triangle.into(), &circumcentre, &midpoint, None).unwrap();
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 3.5, y: 2.5}));

        // Obtuse triangle at an angle pointing up
        let triangle = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 6., y: 6.},
            coord! {x: 7., y: 3.},
        );
        assert_eq!(
            CircumCentreLocation::from_triangle(&triangle).unwrap(),
            CircumCentreLocation::Outside
        );

        let triangle: DelaunayTriangle<_> = triangle.into();
        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 3., y: 3.};
        let inf_line =
            get_inf_outside_triangle(&triangle.into(), &circumcentre, &midpoint, None).unwrap();
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 2.5, y: 3.5}));

        // Circumcentre is outside triangle but infinity line
        // needs to pass across the triangle.
        let triangle = Triangle::new(
            coord! {x: 15., y:19.},
            coord! {x: 17., y: 19.},
            coord! {x: 19., y: 17.},
        );
        assert_eq!(
            CircumCentreLocation::from_triangle(&triangle).unwrap(),
            CircumCentreLocation::Outside
        );

        let triangle: DelaunayTriangle<_> = triangle.into();
        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 18., y: 18.};
        let inf_line =
            get_inf_outside_triangle(&triangle.into(), &circumcentre, &midpoint, None).unwrap();
        assert_eq!(inf_line, Line::new(circumcentre, coord! {x: 18., y: 18.}));
    }

    #[test]
    fn test_inf_on_midpoint_triangle() {
        // The midpoint falls on the circumcentre for the hypotenuse of right triangles.
        let triangle = Triangle::new(
            coord! {x: 0., y:0.},
            coord! {x: 0., y: 3.},
            coord! {x: 4., y: 0.},
        );
        let tri = triangle.clone();

        let triangle: DelaunayTriangle<_> = triangle.into();

        let edge = Line::new(coord! {x: 4., y: 0.}, coord! {x: 0., y: 3.});
        let circumcentre = triangle.get_circumcircle_centre().unwrap();
        let midpoint = coord! {x: 2., y: 1.5};
        assert_eq!(circumcentre, midpoint);
        let inf_line = get_inf_on_midpoint_triangle(&tri, &edge, &circumcentre, &midpoint).unwrap();
        approx::assert_relative_eq!(
            inf_line,
            Line::new(circumcentre, coord! {x: 6., y: 6.8333}),
            max_relative = 0.3
        );
    }

    #[test]
    fn test_get_perpendicular_line() {
        // Vertical line
        let line = Line::new(coord! {x: 0., y: 0.}, coord! {x: 0., y: 1.});
        assert_eq!(
            get_perpendicular_line(&line).unwrap(),
            Line::new(coord! {x: 0., y: 0.}, coord! {x: -1.0, y: 0.})
        );
        // Horizontal line
        let line = Line::new(coord! {x: 0., y: 0.}, coord! {x: 1., y: 0.});
        assert_eq!(
            get_perpendicular_line(&line).unwrap(),
            Line::new(coord! {x: 0., y: 0.}, coord! {x: 0., y: 1.})
        );

        // Check a diagonal line with a new starting point
        let line = Line::new(coord! {x: 0., y: 0.}, coord! {x: 2., y: 2.});
        assert_eq!(
            get_perpendicular_line(&line).unwrap(),
            Line::new(coord! {x: 1., y: 1.}, coord! {x: 3., y: -1.})
        );

        let line = Line::new(coord! {x: 0., y: 0.}, coord! {x: 2., y: -1.});
        assert_eq!(
            get_perpendicular_line(&line).unwrap(),
            Line::new(coord! {x: 1., y: -0.5}, coord! {x: 3., y: 3.5})
        );
    }

    #[test]
    fn test_define_edge_to_infinity() {
        let tri: DelaunayTriangle<_> = Triangle::new(
            coord! {x: 0., y: 0.},
            coord! {x: 0., y: 1.},
            coord! {x: 1., y: 1.},
        )
        .into();
        let tri2: DelaunayTriangle<_> = Triangle::new(
            coord! {x: 0., y: 0.},
            coord! {x: 0., y: 1.},
            coord! {x: -1., y: 2.},
        )
        .into();
        let tri3: DelaunayTriangle<_> = Triangle::new(
            coord! {x: 0., y: 1.},
            coord! {x: -1., y: 2.},
            coord! {x: 1., y: 1.},
        )
        .into();

        let triangles = [tri, tri2, tri3];

        let circumcentres: Vec<_> = triangles
            .iter()
            .map(|x| x.get_circumcircle_centre().unwrap())
            .collect();

        let mut shared_edges = vec![
            Line::new(coord! {x: 0., y: 0.}, coord! {x: 0., y: 1. }),
            Line::new(coord! {x: 0., y: 1.}, coord! {x: 1., y: 1. }),
            Line::new(coord! {x: 0., y: 1.}, coord! {x: -1., y: 2. }),
        ];

        let expected_infintity_lines = [
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: 1.5, y: -0.5}),
            Line::new(coord! {x: -1.5, y: 0.5}, coord! {x: -2.5, y: 0.}),
            Line::new(coord! {x: 0.5, y: 2.5}, coord! {x: 1.0, y: 3.5}),
        ];

        for idx in 0..3 {
            let perpendicular_line = define_edge_to_infinity(
                &triangles[idx],
                &circumcentres[idx],
                &mut shared_edges,
                None,
            )
            .unwrap();
            assert_eq!(perpendicular_line.unwrap(), expected_infintity_lines[idx]);
        }
    }

    #[test]
    fn test_trim_lines_to_intersection() {
        let inf_line = Line::new(coord! { x: 0., y: 0.}, coord! { x: 100., y: 100.});
        let bounding_line = Line::new(coord! { x: 100., y: 0.}, coord! {x: 0., y: 100.});
        let trimmed_inf = trim_line_to_intersection(&inf_line, &bounding_line).unwrap();

        assert_eq!(
            trimmed_inf,
            Line::new(coord! {x: 0., y: 0.}, coord! {x: 50., y: 50.})
        );

        let inf_line = Line::new(coord! { x: -12., y: 9.}, coord! { x: 3., y: -21.});
        let bounding_line = Line::new(coord! { x: -10., y: -25.}, coord! {x: 10., y: 55.});
        let trimmed_inf = trim_line_to_intersection(&inf_line, &bounding_line).unwrap();

        assert_eq!(
            trimmed_inf,
            Line::new(coord! { x: -12., y: 9.}, coord! { x: -5., y: -5.})
        );

        // Parallel lines do not intersect and thus should return None
        let inf_line = Line::new(coord! { x: 0., y: 0.}, coord! { x: 100., y: 100.});
        let bounding_line = Line::new(coord! { x: 100., y: 100.}, coord! {x: 200., y: 200.});
        let trimmed_inf = trim_line_to_intersection(&inf_line, &bounding_line);

        assert!(trimmed_inf.is_none());
    }

    #[test]
    fn test_compute_voronoi_from_delaunay() {
        let poly: Polygon<_> = polygon![
            (x: 0., y: 0.),
            (x: 1., y: 1.),
            (x: 0., y: 1.),
            (x: -1., y: 2.),
        ];

        let delaunay_triangles = poly.delaunay_triangulation().unwrap();

        let voronoi =
            compute_voronoi_components_from_delaunay(&delaunay_triangles, None, None).unwrap();

        let expected_lines = vec![
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: 0.5, y: 2.5}),
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: -1.5, y: 0.5}),
            Line::new(coord! {x: 0.5, y: 2.5}, coord! {x: -1.5, y: 0.5}),
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: 39., y: -38.}),
            Line::new(coord! {x: 0.5, y: 2.5}, coord! {x: 19.25, y: 40.}),
            Line::new(coord! {x: -1.5, y: 0.5}, coord! {x: -41., y: -19.25}),
        ];

        let expected_vertices = vec![
            coord! { x: 0.5, y: 0.5},
            coord! {x: 0.5, y: 2.5},
            coord! {x: -1.5, y: 0.5},
        ];

        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }
    #[test]
    fn test_compute_voronoi_twin_inf_edges() {
        let poly: Polygon<_> = polygon![
            (x: 0., y: 0.),
            (x: 1., y: 1.),
            (x: 3., y: 1.),
            (x: 3., y: 0.),
            (x: 2., y: 0.),
        ];

        let triangles = poly.delaunay_triangulation().unwrap();
        let voronoi = compute_voronoi_components_from_delaunay(&triangles, None, None).unwrap();

        let expected_vertices = [
            coord! {x: 1.0, y: 0.0},
            coord! {x: 2.0, y: 1.0},
            coord! {x: 2.5, y: 0.5},
        ];
        let expected_lines = [
            Line::new(coord! {x: 1.0, y: 0.0}, coord! {x: 2.0, y: 1.0}),
            Line::new(coord! {x: 2.0, y: 1.0}, coord! {x: 2.5, y: 0.5}),
            Line::new(coord! {x: 1.0, y: 0.0}, coord! {x: -19., y: 20.0}),
            Line::new(coord! {x: 1.0, y: 0.0}, coord! {x: 1., y: -20.0}),
            Line::new(coord! {x: 2.0, y: 1.0}, coord! {x: 2., y: 20.0}),
            Line::new(coord! {x: 2.5, y: 0.5}, coord! {x: 60., y: 0.5}),
            Line::new(coord! {x: 2.5, y: 0.5}, coord! {x: 2.5, y: -20.}),
        ];
        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }

    #[test]
    fn test_voronoi_from_polygon() {
        let poly: Polygon<_> = polygon![
            (x: 0., y: 0.),
            (x: 1., y: 1.),
            (x: 0., y: 1.),
            (x: -1., y: 2.),
        ];

        let voronoi = poly.compute_voronoi_components(None, None).unwrap();

        let expected_lines = vec![
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: 0.5, y: 2.5}),
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: -1.5, y: 0.5}),
            Line::new(coord! {x: 0.5, y: 2.5}, coord! {x: -1.5, y: 0.5}),
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: 39., y: -38.}),
            Line::new(coord! {x: 0.5, y: 2.5}, coord! {x: 19.25, y: 40.}),
            Line::new(coord! {x: -1.5, y: 0.5}, coord! {x: -41., y: -19.25}),
        ];

        let expected_vertices = vec![
            coord! { x: 0.5, y: 0.5},
            coord! {x: 0.5, y: 2.5},
            coord! {x: -1.5, y: 0.5},
        ];

        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }

    // https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/VoronoiTest.cpp#L154
    #[test]
    fn test_single_point() {
        let poly = polygon![(x: 150., y: 200.)];
        let voronoi = poly.compute_voronoi_components(None, None).unwrap();

        assert!(voronoi.vertices.is_empty());
        assert!(voronoi.lines.is_empty());
    }

    // https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/VoronoiTest.cpp#L164
    #[test]
    fn test_simple() {
        let points = polygon![(x: 150., y: 200.), (x: 180., y: 270.), (x: 275., y: 163.)];

        let voronoi = points.compute_voronoi_components(None, None).unwrap();

        let expected_vertices = [coord! {x: 211.205, y: 210.911}];

        let expected_lines = vec![
            Line::new(expected_vertices[0], coord! {x: -2350.0, y: 1312.857}),
            Line::new(expected_vertices[0], coord! {x: -426.416, y: -1977.0}),
            Line::new(expected_vertices[0], coord! {x: 2577.558, y: 2303.0}),
        ];

        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }
    // https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/VoronoiTest.cpp#L174
    #[test]
    fn test_four_points() {
        let points = polygon![
            (x: 280., y: 300.),
            (x: 420., y: 330.),
            (x: 380., y: 230.),
            (x: 320., y: 160.)
        ];

        let voronoi = points.compute_voronoi_components(None, None).unwrap();

        let expected_vertices = [
            coord! {x: 353.516, y: 298.594},
            coord! {x: 306.875, y: 231.964},
        ];

        let expected_lines = [
            Line::new(
                coord! {x: 353.516, y: 298.594},
                coord! {x: 306.875, y: 231.964},
            ),
            Line::new(
                coord! {x: 353.516, y: 298.594},
                coord! {x: -345.571, y: 3556.0},
            ),
            Line::new(coord! {x: 353.516, y: 298.594}, coord! {x: 3080., y: -792.}),
            Line::new(
                coord! {x: 306.875, y: 231.964},
                coord! {x: -2520., y: -575.714},
            ),
            Line::new(
                coord! {x: 306.875, y: 231.964},
                coord! {x: 3080., y: -2145.},
            ),
        ];

        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }

    //https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/VoronoiTest.cpp#L189
    #[test]
    fn test_six_points() {
        let points = polygon![
            (x: 320., y: 170.),
            (x: 366., y: 246.),
            (x: 530., y: 230.),
            (x: 530., y: 300.),
            (x: 455., y: 277.),
            (x: 490., y: 160.)
        ];

        let voronoi = points.compute_voronoi_components(None, None).unwrap();

        let expected_vertices = [
            coord! {x: 499.707, y: 265.},
            coord! {x: 470.121, y: 217.788},
            coord! {x: 429.915, y: 205.761},
            coord! {x: 405.311, y: 170.286},
        ];

        let expected_lines = [
            Line::new(
                coord! {x: 499.70666666666665, y: 265.0 },
                coord! { x: 470.12061711079946, y: 217.7882187938289 },
            ),
            Line::new(
                coord! {x: 405.31091180866963, y: 170.28550074738416 },
                coord! { x: 429.9147677857019, y: 205.76082797008175 },
            ),
            Line::new(
                coord! {x: 470.12061711079946, y: 217.7882187938289 },
                coord! { x: 429.9147677857019, y: 205.76082797008175 },
            ),
            Line::new(
                coord! {x: 499.70666666666665, y: 265.0 },
                coord! { x: 4519.999999999999, y: 264.99999999999994 },
            ),
            Line::new(
                coord! {x: 499.70666666666665, y: 265.0 },
                coord! { x: -326.7599999999982, y: 2960.0 },
            ),
            Line::new(
                coord! {x: 405.31091180866963, y: 170.28550074738416 },
                coord! { x: -3880.0000000000005, y: 2764.0263157894765 },
            ),
            Line::new(
                coord! {x: 405.31091180866963, y: 170.28550074738416 },
                coord! { x: 239.99999999998627, y: -2640.0 },
            ),
            Line::new(
                coord! {x: 470.12061711079946, y: 217.7882187938289 },
                coord! { x: 4520.0, y: -2096.428571428572 },
            ),
            Line::new(
                coord! {x: 429.9147677857019, y: 205.76082797008175 },
                coord! { x: -529.42696629214, y: 2960.0 },
            ),
        ];

        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }

    #[test]
    fn test_rhombus() {
        let rhombus =
            polygon![(x: 10., y: 10.), (x: 11., y: 20.), (x: 20., y: 20.), (x: 19., y: 10.)];

        let voronoi = rhombus.compute_voronoi_components(None, None).unwrap();

        let expected_vertices = [coord! { x: 14.5, y: 14.6}, coord! { x: 15.5, y: 15.4}];
        let expected_lines = [
            Line::new(coord! {x: 14.5, y: 14.6}, coord! {x: 15.5, y: 15.4}),
            Line::new(coord! {x: 14.5, y: 14.6}, coord! {x: -190., y: 35.05}),
            Line::new(coord! {x: 14.5, y: 14.6}, coord! {x: 14.5, y: -190.0}),
            Line::new(coord! {x: 15.5, y: 15.4}, coord! {x: 15.5, y: 210.0}),
            Line::new(coord! {x: 15.5, y: 15.4}, coord! {x: 210., y: -4.05}),
        ];

        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }

    #[test]
    fn test_rotated_rhombus_lines() {
        let poly = Polygon::new(
            LineString::from(vec![
                coord! {x: 10., y: 10. },
                coord! {x: 10., y: 12. },
                coord! {x: 10., y: 14. },
                coord! {x: 10., y: 16. },
                coord! {x: 10., y: 18. },
                coord! {x: 10., y: 20. },
                coord! {x: 11.8, y: 19.8 },
                coord! {x: 13.6, y: 19.6 },
                coord! {x: 15.4, y: 19.4 },
                coord! {x: 17.2, y: 19.2 },
                coord! {x: 19., y: 19. },
                coord! {x: 19., y: 17. },
                coord! {x: 19., y: 15. },
                coord! {x: 19., y: 13. },
                coord! {x: 19., y: 11. },
                coord! {x: 17.2, y: 10.8 },
                coord! {x: 15.4, y: 10.6 },
                coord! {x: 13.6, y: 10.4 },
                coord! {x: 11.8, y: 10.2 },
                coord! {x: 10., y: 10. },
            ]),
            vec![],
        );

        let voronoi = poly.compute_voronoi_components(None, None).unwrap();

        let expected_vertices = [
            coord! { x: 10.8, y: 19.0 },
            coord! { x: 12.600000000000001, y: 17.0 },
            coord! { x: 12.440000000000001, y: 17.36 },
            coord! { x: 14.399999999999999, y: 15.0 },
            coord! { x: 14.059999999999995, y: 15.540000000000006 },
            coord! { x: 15.973333333333334, y: 16.36 },
            coord! { x: 17.977777777777778, y: 18.0 },
            coord! { x: 15.733333333333334, y: 16.0 },
            coord! { x: 17.977777777777778, y: 12.0 },
            coord! { x: 14.399999999999999, y: 15.0 },
            coord! { x: 14.511111111111113, y: 14.999999999999998 },
            coord! { x: 15.733333333333333, y: 14.0 },
            coord! { x: 15.973333333333331, y: 13.640000000000004 },
            coord! { x: 12.6, y: 13.0 },
            coord! { x: 14.060000000000002, y: 14.46 },
            coord! { x: 10.8, y: 11.0 },
            coord! { x: 12.44, y: 12.639999999999997 },
        ];
        let expected_lines = [
            Line::new(
                coord! { x: 10.8, y: 19.0 },
                coord! { x: 12.440000000000001, y: 17.36 },
            ),
            Line::new(
                coord! { x: 12.600000000000001, y: 17.0 },
                coord! { x: 12.440000000000001, y: 17.36 },
            ),
            Line::new(
                coord! { x: 12.600000000000001, y: 17.0 },
                coord! { x: 14.059999999999995, y: 15.540000000000006 },
            ),
            Line::new(
                coord! { x: 14.399999999999999, y: 15.0 },
                coord! { x: 14.059999999999995, y: 15.540000000000006 },
            ),
            Line::new(
                coord! { x: 14.399999999999999, y: 15.0 },
                coord! { x: 14.399999999999999, y: 15.0 },
            ),
            Line::new(
                coord! { x: 15.973333333333334, y: 16.36 },
                coord! { x: 17.977777777777778, y: 18.0 },
            ),
            Line::new(
                coord! { x: 15.973333333333334, y: 16.36 },
                coord! { x: 15.733333333333334, y: 16.0 },
            ),
            Line::new(
                coord! { x: 15.733333333333334, y: 16.0 },
                coord! { x: 14.511111111111113, y: 14.999999999999998 },
            ),
            Line::new(
                coord! { x: 17.977777777777778, y: 12.0 },
                coord! { x: 15.973333333333331, y: 13.640000000000004 },
            ),
            Line::new(
                coord! { x: 14.399999999999999, y: 15.0 },
                coord! { x: 14.511111111111113, y: 14.999999999999998 },
            ),
            Line::new(
                coord! { x: 14.399999999999999, y: 15.0 },
                coord! { x: 14.060000000000002, y: 14.46 },
            ),
            Line::new(
                coord! { x: 14.511111111111113, y: 14.999999999999998 },
                coord! { x: 15.733333333333333, y: 14.0 },
            ),
            Line::new(
                coord! { x: 15.733333333333333, y: 14.0 },
                coord! { x: 15.973333333333331, y: 13.640000000000004 },
            ),
            Line::new(
                coord! { x: 12.6, y: 13.0 },
                coord! { x: 14.060000000000002, y: 14.46 },
            ),
            Line::new(
                coord! { x: 12.6, y: 13.0 },
                coord! { x: 12.44, y: 12.639999999999997 },
            ),
            Line::new(
                coord! { x: 10.8, y: 11.0 },
                coord! { x: 12.44, y: 12.639999999999997 },
            ),
            Line::new(
                coord! { x: 10.8, y: 19.0 },
                coord! { x: -170.0, y: 19.000000000000007 },
            ),
            Line::new(
                coord! { x: 10.8, y: 19.0 },
                coord! { x: 32.02222222222237, y: 210.0 },
            ),
            Line::new(
                coord! { x: 12.600000000000001, y: 17.0 },
                coord! { x: -170.0, y: 17.0 },
            ),
            Line::new(
                coord! { x: 12.440000000000001, y: 17.36 },
                coord! { x: 33.84444444444426, y: 210.0 },
            ),
            Line::new(
                coord! { x: 14.399999999999999, y: 15.0 },
                coord! { x: -169.99999999999997, y: 14.999999999999998 },
            ),
            Line::new(
                coord! { x: 14.059999999999995, y: 15.540000000000006 },
                coord! { x: 35.666666666666934, y: 210.00000000000003 },
            ),
            Line::new(
                coord! { x: 15.973333333333334, y: 16.36 },
                coord! { x: 37.48888888888888, y: 210.0 },
            ),
            Line::new(
                coord! { x: 17.977777777777778, y: 18.0 },
                coord! { x: 39.311111111111494, y: 209.99999999999997 },
            ),
            Line::new(
                coord! { x: 17.977777777777778, y: 18.0 },
                coord! { x: 189.99999999999997, y: 17.99999999999998 },
            ),
            Line::new(
                coord! { x: 15.733333333333334, y: 16.0 },
                coord! { x: 190.0, y: 16.0 },
            ),
            Line::new(
                coord! { x: 17.977777777777778, y: 12.0 },
                coord! { x: 189.99999999999997, y: 11.999999999999986 },
            ),
            Line::new(
                coord! { x: 17.977777777777778, y: 12.0 },
                coord! { x: 40.42222222222257, y: -190.0 },
            ),
            Line::new(
                coord! { x: 15.733333333333333, y: 14.0 },
                coord! { x: 190.0, y: 14.000000000000002 },
            ),
            Line::new(
                coord! { x: 15.973333333333331, y: 13.640000000000004 },
                coord! { x: 38.600000000000314, y: -190.00000000000003 },
            ),
            Line::new(
                coord! { x: 12.6, y: 13.0 },
                coord! { x: -170.0, y: 12.999999999999996 },
            ),
            Line::new(
                coord! { x: 14.060000000000002, y: 14.46 },
                coord! { x: 36.77777777777766, y: -190.0 },
            ),
            Line::new(
                coord! { x: 10.8, y: 11.0 },
                coord! { x: -170.0, y: 11.000000000000004 },
            ),
            Line::new(
                coord! { x: 10.8, y: 11.0 },
                coord! { x: 33.13333333333312, y: -190.0 },
            ),
            Line::new(
                coord! { x: 12.44, y: 12.639999999999997 },
                coord! { x: 34.95555555555567, y: -189.99999999999997 },
            ),
        ];

        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }

    #[test]
    fn test_rhombus_lines() {
        let poly = Polygon::new(
            LineString::from(vec![
                coord! { x: 10.0, y: 10.0 },
                coord! { x: 10.166666666666666, y: 11.666666666666666 },
                coord! { x: 10.333333333333334, y: 13.333333333333332 },
                coord! { x: 10.5, y: 15.0 },
                coord! { x: 10.666666666666666, y: 16.666666666666664 },
                coord! { x: 10.833333333333334, y: 18.333333333333332 },
                coord! { x: 11.0, y: 20.0 },
                coord! { x: 12.8, y: 20.0 },
                coord! { x: 14.6, y: 20.0 },
                coord! { x: 16.4, y: 20.0 },
                coord! { x: 18.2, y: 20.0 },
                coord! { x: 20.0, y: 20.0 },
                coord! { x: 19.833333333333332, y: 18.333333333333332 },
                coord! { x: 19.666666666666668, y: 16.666666666666668 },
                coord! { x: 19.5, y: 15.0 },
                coord! { x: 19.333333333333332, y: 13.333333333333334 },
                coord! { x: 19.166666666666668, y: 11.666666666666668 },
                coord! { x: 19.0, y: 10.0 },
                coord! { x: 17.2, y: 10.0 },
                coord! { x: 15.4, y: 10.0 },
                coord! { x: 13.6, y: 10.0 },
                coord! { x: 11.8, y: 10.0 },
                coord! { x: 10.0, y: 10.0 },
            ]),
            vec![],
        );

        let voronoi = poly.compute_voronoi_components(None, None).unwrap();

        let expected_vertices = [
            coord! { x: 13.458641975308646, y: 17.22913580246913 },
            coord! { x: 11.9, y: 19.06833333333333 },
            coord! { x: 13.7, y: 17.074666666666666 },
            coord! { x: 19.1, y: 19.248333333333335 },
            coord! { x: 17.299999999999997, y: 17.614666666666665 },
            coord! { x: 17.541358024691355, y: 17.72086419753086 },
            coord! { x: 15.0, y: 15.391666666666666 },
            coord! { x: 15.022993827160494, y: 15.513533950617283 },
            coord! { x: 15.5, y: 15.980999999999998 },
            coord! { x: 15.862037037037041, y: 16.205462962962965 },
            coord! { x: 15.0, y: 14.608333333333336 },
            coord! { x: 16.541358024691363, y: 12.77086419753086 },
            coord! { x: 18.1, y: 10.931666666666668 },
            coord! { x: 14.977006172839506, y: 14.486466049382715 },
            coord! { x: 16.3, y: 12.925333333333333 },
            coord! { x: 14.137962962962964, y: 13.794537037037038 },
            coord! { x: 14.5, y: 14.019 },
            coord! { x: 12.458641975308637, y: 12.279135802469133 },
            coord! { x: 12.700000000000001, y: 12.385333333333334 },
            coord! { x: 10.9, y: 10.751666666666667 },
        ];
        let expected_lines = [
            Line::new(
                coord! { x: 13.458641975308646, y: 17.22913580246913 },
                coord! { x: 11.9, y: 19.06833333333333 },
            ),
            Line::new(
                coord! { x: 13.458641975308646, y: 17.22913580246913 },
                coord! { x: 13.7, y: 17.074666666666666 },
            ),
            Line::new(
                coord! { x: 13.7, y: 17.074666666666666 },
                coord! { x: 15.022993827160494, y: 15.513533950617283 },
            ),
            Line::new(
                coord! { x: 19.1, y: 19.248333333333335 },
                coord! { x: 17.541358024691355, y: 17.72086419753086 },
            ),
            Line::new(
                coord! { x: 17.299999999999997, y: 17.614666666666665 },
                coord! { x: 17.541358024691355, y: 17.72086419753086 },
            ),
            Line::new(
                coord! { x: 17.299999999999997, y: 17.614666666666665 },
                coord! { x: 15.862037037037041, y: 16.205462962962965 },
            ),
            Line::new(
                coord! { x: 15.0, y: 15.391666666666666 },
                coord! { x: 15.022993827160494, y: 15.513533950617283 },
            ),
            Line::new(
                coord! { x: 15.0, y: 15.391666666666666 },
                coord! { x: 15.0, y: 14.608333333333336 },
            ),
            Line::new(
                coord! { x: 15.022993827160494, y: 15.513533950617283 },
                coord! { x: 15.5, y: 15.980999999999998 },
            ),
            Line::new(
                coord! { x: 15.5, y: 15.980999999999998 },
                coord! { x: 15.862037037037041, y: 16.205462962962965 },
            ),
            Line::new(
                coord! { x: 15.0, y: 14.608333333333336 },
                coord! { x: 14.977006172839506, y: 14.486466049382715 },
            ),
            Line::new(
                coord! { x: 16.541358024691363, y: 12.77086419753086 },
                coord! { x: 18.1, y: 10.931666666666668 },
            ),
            Line::new(
                coord! { x: 16.541358024691363, y: 12.77086419753086 },
                coord! { x: 16.3, y: 12.925333333333333 },
            ),
            Line::new(
                coord! { x: 14.977006172839506, y: 14.486466049382715 },
                coord! { x: 16.3, y: 12.925333333333333 },
            ),
            Line::new(
                coord! { x: 14.977006172839506, y: 14.486466049382715 },
                coord! { x: 14.5, y: 14.019 },
            ),
            Line::new(
                coord! { x: 14.137962962962964, y: 13.794537037037038 },
                coord! { x: 14.5, y: 14.019 },
            ),
            Line::new(
                coord! { x: 14.137962962962964, y: 13.794537037037038 },
                coord! { x: 12.700000000000001, y: 12.385333333333334 },
            ),
            Line::new(
                coord! { x: 12.458641975308637, y: 12.279135802469133 },
                coord! { x: 12.700000000000001, y: 12.385333333333334 },
            ),
            Line::new(
                coord! { x: 12.458641975308637, y: 12.279135802469133 },
                coord! { x: 10.9, y: 10.751666666666667 },
            ),
            Line::new(
                coord! { x: 13.458641975308646, y: 17.22913580246913 },
                coord! { x: -189.99999999999997, y: 37.57500000000035 },
            ),
            Line::new(
                coord! { x: 11.9, y: 19.06833333333333 },
                coord! { x: -190.0, y: 39.25833333333322 },
            ),
            Line::new(
                coord! { x: 11.9, y: 19.06833333333333 },
                coord! { x: 11.89999999999999, y: 210.0 },
            ),
            Line::new(
                coord! { x: 13.7, y: 17.074666666666666 },
                coord! { x: 13.700000000000006, y: 210.00000000000003 },
            ),
            Line::new(
                coord! { x: 19.1, y: 19.248333333333335 },
                coord! { x: 19.099999999999955, y: 210.0 },
            ),
            Line::new(
                coord! { x: 19.1, y: 19.248333333333335 },
                coord! { x: 210.0, y: 0.1583333333323135 },
            ),
            Line::new(
                coord! { x: 17.299999999999997, y: 17.614666666666665 },
                coord! { x: 17.299999999999997, y: 210.0 },
            ),
            Line::new(
                coord! { x: 17.541358024691355, y: 17.72086419753086 },
                coord! { x: 210.00000000000003, y: -1.5249999999997563 },
            ),
            Line::new(
                coord! { x: 15.0, y: 15.391666666666666 },
                coord! { x: -190.0, y: 35.89166666666673 },
            ),
            Line::new(
                coord! { x: 15.5, y: 15.980999999999998 },
                coord! { x: 15.499999999999998, y: 210.0 },
            ),
            Line::new(
                coord! { x: 15.862037037037041, y: 16.205462962962965 },
                coord! { x: 210.00000000000003, y: -3.2083333333333326 },
            ),
            Line::new(
                coord! { x: 15.0, y: 14.608333333333336 },
                coord! { x: 210.00000000000003, y: -4.891666666666753 },
            ),
            Line::new(
                coord! { x: 16.541358024691363, y: 12.77086419753086 },
                coord! { x: 210.0, y: -6.574999999999776 },
            ),
            Line::new(
                coord! { x: 18.1, y: 10.931666666666668 },
                coord! { x: 209.99999999999997, y: -8.258333333333566 },
            ),
            Line::new(
                coord! { x: 18.1, y: 10.931666666666668 },
                coord! { x: 18.1, y: -190.0 },
            ),
            Line::new(
                coord! { x: 16.3, y: 12.925333333333333 },
                coord! { x: 16.3, y: -190.00000000000003 },
            ),
            Line::new(
                coord! { x: 14.137962962962964, y: 13.794537037037038 },
                coord! { x: -190.0, y: 34.208333333333215 },
            ),
            Line::new(
                coord! { x: 14.5, y: 14.019 },
                coord! { x: 14.499999999999996, y: -189.99999999999997 },
            ),
            Line::new(
                coord! { x: 12.458641975308637, y: 12.279135802469133 },
                coord! { x: -190.0, y: 32.52500000000029 },
            ),
            Line::new(
                coord! { x: 12.700000000000001, y: 12.385333333333334 },
                coord! { x: 12.699999999999852, y: -190.0 },
            ),
            Line::new(
                coord! { x: 10.9, y: 10.751666666666667 },
                coord! { x: -189.99999999999997, y: 30.841666666666274 },
            ),
            Line::new(
                coord! { x: 10.9, y: 10.751666666666667 },
                coord! { x: 10.900000000000002, y: -190.00000000000003 },
            ),
        ];

        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }

    #[test]
    fn test_polygon_self_intersecting() {
        let poly = Polygon::new(
            LineString::from(vec![
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 5.0, y: 0.0 },
                coord! { x: 10.0, y: 0.0 },
                coord! { x: 15.0, y: 0.0 },
                coord! { x: 20.0, y: 0.0 },
                coord! { x: 25.0, y: 0.0 },
                coord! { x: 30.0, y: 0.0 },
                coord! { x: 35.0, y: 0.0 },
                coord! { x: 40.0, y: 0.0 },
                coord! { x: 40.0, y: 5.0 },
                coord! { x: 40.0, y: 10.0 },
                coord! { x: 35.0, y: 10.0 },
                coord! { x: 30.0, y: 10.0 },
                coord! { x: 25.0, y: 10.0 },
                coord! { x: 20.0, y: 10.0 },
                coord! {
                    x: 15.000000000000004,
                    y: 10.0
                },
                coord! { x: 10.0, y: 10.0 },
                coord! { x: 10.0, y: 5.0 },
                coord! { x: 10.0, y: 0.0 },
                coord! {
                    x: 10.0,
                    y: -5.000000000000002
                },
                coord! { x: 10.0, y: -10.0 },
                coord! { x: 10.0, y: -15.0 },
                coord! {
                    x: 10.0,
                    y: -20.000000000000004
                },
                coord! { x: 10.0, y: -25.0 },
                coord! { x: 10.0, y: -30.0 },
                coord! { x: 10.0, y: -35.0 },
                coord! { x: 10.0, y: -40.0 },
                coord! { x: 5.0, y: -40.0 },
                coord! { x: 0.0, y: -40.0 },
                coord! { x: 0.0, y: -35.0 },
                coord! { x: 0.0, y: -30.0 },
                coord! { x: 0.0, y: -25.0 },
                coord! { x: 0.0, y: -20.0 },
                coord! { x: 0.0, y: -15.0 },
                coord! { x: 0.0, y: -10.0 },
                coord! { x: 0.0, y: -5.0 },
                coord! { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

        let expected_vertices = [
            coord! { x: 37.5, y: 2.5 },
            coord! { x: 32.5, y: 5.0 },
            coord! { x: 35.0, y: 5.0 },
            coord! { x: 37.5, y: 7.5 },
            coord! { x: 27.5, y: 5.0 },
            coord! { x: 32.5, y: 5.0 },
            coord! { x: 22.5, y: 5.0 },
            coord! { x: 27.5, y: 5.0 },
            coord! { x: 17.5, y: 5.0 },
            coord! { x: 22.5, y: 5.0 },
            coord! { x: 17.5, y: 4.999999999999999 },
            coord! { x: 2.5, y: 7.5 },
            coord! { x: 7.5, y: 2.5 },
            coord! { x: 2.5, y: 7.5 },
            coord! { x: 12.5, y: 2.5 },
            coord! { x: 15.0, y: 5.0 },
            coord! { x: 12.500000000000002, y: 7.5 },
            coord! { x: 7.5, y: -2.500000000000001 },
            coord! { x: 12.5, y: -2.5000000000000004 },
            coord! { x: 17.5, y: -7.5 },
            coord! { x: 22.5, y: -12.5 },
            coord! { x: 17.5, y: -7.5 },
            coord! { x: 22.5, y: -12.5 },
            coord! { x: 27.5, y: -17.500000000000004 },
            coord! { x: 27.50000000000001, y: -17.50000000000001 },
            coord! { x: 32.5, y: -22.5 },
            coord! { x: 37.5, y: -27.5 },
            coord! { x: 32.499999999999986, y: -22.499999999999993 },
            coord! { x: 37.5, y: -27.5 },
            coord! { x: 42.5, y: -32.5 },
            coord! { x: 48.333333333333336, y: -37.5 },
            coord! { x: 7.5, y: -37.5 },
            coord! { x: 5.0, y: -32.5 },
            coord! { x: 5.0, y: -35.0 },
            coord! { x: 2.5, y: -37.5 },
            coord! { x: 5.0, y: -27.5 },
            coord! { x: 5.0, y: -32.5 },
            coord! { x: 5.0, y: -22.5 },
            coord! { x: 5.0, y: -27.5 },
            coord! { x: 5.000000000000001, y: -17.5 },
            coord! { x: 5.0, y: -22.5 },
            coord! { x: 5.0, y: -12.5 },
            coord! { x: 5.0, y: -17.5 },
            coord! { x: 5.0, y: -7.500000000000001 },
            coord! { x: 5.0, y: -12.5 },
            coord! { x: 2.5, y: -2.5 },
            coord! { x: 5.0, y: -5.000000000000001 },
            coord! { x: 4.999999999999999, y: -7.5 },
        ];
        let expected_lines = [
            Line::new(coord! { x: 37.5, y: 2.5 }, coord! { x: 35.0, y: 5.0 }),
            Line::new(coord! { x: 37.5, y: 2.5 }, coord! { x: 37.5, y: -27.5 }),
            Line::new(coord! { x: 32.5, y: 5.0 }, coord! { x: 35.0, y: 5.0 }),
            Line::new(coord! { x: 32.5, y: 5.0 }, coord! { x: 32.5, y: 5.0 }),
            Line::new(coord! { x: 32.5, y: 5.0 }, coord! { x: 32.5, y: -22.5 }),
            Line::new(coord! { x: 35.0, y: 5.0 }, coord! { x: 37.5, y: 7.5 }),
            Line::new(coord! { x: 27.5, y: 5.0 }, coord! { x: 32.5, y: 5.0 }),
            Line::new(coord! { x: 27.5, y: 5.0 }, coord! { x: 27.5, y: 5.0 }),
            Line::new(
                coord! { x: 27.5, y: 5.0 },
                coord! { x: 27.5, y: -17.500000000000004 },
            ),
            Line::new(coord! { x: 22.5, y: 5.0 }, coord! { x: 27.5, y: 5.0 }),
            Line::new(coord! { x: 22.5, y: 5.0 }, coord! { x: 22.5, y: 5.0 }),
            Line::new(coord! { x: 22.5, y: 5.0 }, coord! { x: 22.5, y: -12.5 }),
            Line::new(coord! { x: 17.5, y: 5.0 }, coord! { x: 22.5, y: 5.0 }),
            Line::new(
                coord! { x: 17.5, y: 5.0 },
                coord! { x: 17.5, y: 4.999999999999999 },
            ),
            Line::new(coord! { x: 17.5, y: 5.0 }, coord! { x: 17.5, y: -7.5 }),
            Line::new(
                coord! { x: 17.5, y: 4.999999999999999 },
                coord! { x: 15.0, y: 5.0 },
            ),
            Line::new(coord! { x: 2.5, y: 7.5 }, coord! { x: 2.5, y: 7.5 }),
            Line::new(coord! { x: 2.5, y: 7.5 }, coord! { x: 2.5, y: -2.5 }),
            Line::new(coord! { x: 7.5, y: 2.5 }, coord! { x: 2.5, y: 7.5 }),
            Line::new(coord! { x: 7.5, y: 2.5 }, coord! { x: 12.5, y: 2.5 }),
            Line::new(
                coord! { x: 7.5, y: 2.5 },
                coord! { x: 7.5, y: -2.500000000000001 },
            ),
            Line::new(
                coord! { x: 2.5, y: 7.5 },
                coord! { x: 12.500000000000002, y: 7.5 },
            ),
            Line::new(coord! { x: 12.5, y: 2.5 }, coord! { x: 15.0, y: 5.0 }),
            Line::new(
                coord! { x: 12.5, y: 2.5 },
                coord! { x: 12.5, y: -2.5000000000000004 },
            ),
            Line::new(
                coord! { x: 15.0, y: 5.0 },
                coord! { x: 12.500000000000002, y: 7.5 },
            ),
            Line::new(
                coord! { x: 7.5, y: -2.500000000000001 },
                coord! { x: 12.5, y: -2.5000000000000004 },
            ),
            Line::new(
                coord! { x: 7.5, y: -2.500000000000001 },
                coord! { x: 5.0, y: -5.000000000000001 },
            ),
            Line::new(
                coord! { x: 12.5, y: -2.5000000000000004 },
                coord! { x: 17.5, y: -7.5 },
            ),
            Line::new(coord! { x: 17.5, y: -7.5 }, coord! { x: 17.5, y: -7.5 }),
            Line::new(coord! { x: 22.5, y: -12.5 }, coord! { x: 17.5, y: -7.5 }),
            Line::new(coord! { x: 22.5, y: -12.5 }, coord! { x: 22.5, y: -12.5 }),
            Line::new(
                coord! { x: 17.5, y: -7.5 },
                coord! { x: 5.0, y: -7.500000000000001 },
            ),
            Line::new(
                coord! { x: 22.5, y: -12.5 },
                coord! { x: 27.50000000000001, y: -17.50000000000001 },
            ),
            Line::new(coord! { x: 22.5, y: -12.5 }, coord! { x: 5.0, y: -12.5 }),
            Line::new(
                coord! { x: 27.5, y: -17.500000000000004 },
                coord! { x: 27.50000000000001, y: -17.50000000000001 },
            ),
            Line::new(
                coord! { x: 27.5, y: -17.500000000000004 },
                coord! { x: 32.5, y: -22.5 },
            ),
            Line::new(
                coord! { x: 27.50000000000001, y: -17.50000000000001 },
                coord! { x: 5.000000000000001, y: -17.5 },
            ),
            Line::new(
                coord! { x: 32.5, y: -22.5 },
                coord! { x: 32.499999999999986, y: -22.499999999999993 },
            ),
            Line::new(
                coord! { x: 37.5, y: -27.5 },
                coord! { x: 32.499999999999986, y: -22.499999999999993 },
            ),
            Line::new(coord! { x: 37.5, y: -27.5 }, coord! { x: 37.5, y: -27.5 }),
            Line::new(
                coord! { x: 32.499999999999986, y: -22.499999999999993 },
                coord! { x: 5.0, y: -22.5 },
            ),
            Line::new(coord! { x: 37.5, y: -27.5 }, coord! { x: 42.5, y: -32.5 }),
            Line::new(coord! { x: 37.5, y: -27.5 }, coord! { x: 5.0, y: -27.5 }),
            Line::new(
                coord! { x: 42.5, y: -32.5 },
                coord! { x: 48.333333333333336, y: -37.5 },
            ),
            Line::new(coord! { x: 42.5, y: -32.5 }, coord! { x: 5.0, y: -32.5 }),
            Line::new(
                coord! { x: 48.333333333333336, y: -37.5 },
                coord! { x: 7.5, y: -37.5 },
            ),
            Line::new(coord! { x: 7.5, y: -37.5 }, coord! { x: 5.0, y: -35.0 }),
            Line::new(coord! { x: 5.0, y: -32.5 }, coord! { x: 5.0, y: -35.0 }),
            Line::new(coord! { x: 5.0, y: -32.5 }, coord! { x: 5.0, y: -32.5 }),
            Line::new(coord! { x: 5.0, y: -35.0 }, coord! { x: 2.5, y: -37.5 }),
            Line::new(coord! { x: 5.0, y: -27.5 }, coord! { x: 5.0, y: -32.5 }),
            Line::new(coord! { x: 5.0, y: -27.5 }, coord! { x: 5.0, y: -27.5 }),
            Line::new(coord! { x: 5.0, y: -22.5 }, coord! { x: 5.0, y: -27.5 }),
            Line::new(coord! { x: 5.0, y: -22.5 }, coord! { x: 5.0, y: -22.5 }),
            Line::new(
                coord! { x: 5.000000000000001, y: -17.5 },
                coord! { x: 5.0, y: -22.5 },
            ),
            Line::new(
                coord! { x: 5.000000000000001, y: -17.5 },
                coord! { x: 5.0, y: -17.5 },
            ),
            Line::new(coord! { x: 5.0, y: -12.5 }, coord! { x: 5.0, y: -17.5 }),
            Line::new(coord! { x: 5.0, y: -12.5 }, coord! { x: 5.0, y: -12.5 }),
            Line::new(
                coord! { x: 5.0, y: -7.500000000000001 },
                coord! { x: 5.0, y: -12.5 },
            ),
            Line::new(
                coord! { x: 5.0, y: -7.500000000000001 },
                coord! { x: 4.999999999999999, y: -7.5 },
            ),
            Line::new(
                coord! { x: 2.5, y: -2.5 },
                coord! { x: 5.0, y: -5.000000000000001 },
            ),
            Line::new(
                coord! { x: 5.0, y: -5.000000000000001 },
                coord! { x: 4.999999999999999, y: -7.5 },
            ),
            Line::new(coord! { x: 37.5, y: 2.5 }, coord! { x: 800.0, y: 2.5 }),
            Line::new(coord! { x: 37.5, y: 7.5 }, coord! { x: 800.0, y: 7.5 }),
            Line::new(coord! { x: 37.5, y: 7.5 }, coord! { x: 37.5, y: 960.0 }),
            Line::new(coord! { x: 32.5, y: 5.0 }, coord! { x: 32.5, y: 960.0 }),
            Line::new(coord! { x: 27.5, y: 5.0 }, coord! { x: 27.5, y: 960.0 }),
            Line::new(coord! { x: 22.5, y: 5.0 }, coord! { x: 22.5, y: 960.0 }),
            Line::new(
                coord! { x: 17.5, y: 4.999999999999999 },
                coord! { x: 17.5, y: 959.9999999999999 },
            ),
            Line::new(coord! { x: 2.5, y: 7.5 }, coord! { x: -800.0, y: 810.0 }),
            Line::new(
                coord! { x: 12.500000000000002, y: 7.5 },
                coord! { x: 12.5, y: 960.0 },
            ),
            Line::new(
                coord! { x: 48.333333333333336, y: -37.5 },
                coord! { x: 800.0, y: -601.2499999999999 },
            ),
            Line::new(coord! { x: 7.5, y: -37.5 }, coord! { x: 7.5, y: -1040.0 }),
            Line::new(coord! { x: 2.5, y: -37.5 }, coord! { x: 2.5, y: -1040.0 }),
            Line::new(coord! { x: 2.5, y: -37.5 }, coord! { x: -800.0, y: -37.5 }),
            Line::new(coord! { x: 5.0, y: -32.5 }, coord! { x: -800.0, y: -32.5 }),
            Line::new(coord! { x: 5.0, y: -27.5 }, coord! { x: -800.0, y: -27.5 }),
            Line::new(coord! { x: 5.0, y: -22.5 }, coord! { x: -800.0, y: -22.5 }),
            Line::new(coord! { x: 5.0, y: -17.5 }, coord! { x: -800.0, y: -17.5 }),
            Line::new(coord! { x: 5.0, y: -12.5 }, coord! { x: -800.0, y: -12.5 }),
            Line::new(coord! { x: 2.5, y: -2.5 }, coord! { x: -800.0, y: -2.5 }),
            Line::new(
                coord! { x: 4.999999999999999, y: -7.5 },
                coord! { x: -800.0, y: -7.5 },
            ),
        ];

        let voronoi = poly.compute_voronoi_components(None, None).unwrap();

        relative_voronoi_compare(&voronoi, &expected_vertices, &expected_lines);
    }
}
