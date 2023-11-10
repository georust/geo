use std::{fmt, result};

use crate::{coord, Coord, GeoFloat, Line, MultiPoint, Point, Polygon, Rect, Triangle};

use crate::triangulate_delaunay::{
    DelaunayTriangle, DelaunayTriangulationError, DEFAULT_SUPER_TRIANGLE_EXPANSION,
};
use crate::{BoundingRect, TriangulateDelaunay};

type Result<T> = result::Result<T, VoronoiDiagramError>;

#[derive(Debug, Clone)]
pub struct VoronoiComponents<T: GeoFloat> {
    pub delaunay_triangles: Vec<Triangle<T>>,
    pub vertices: Vec<Coord<T>>,
    pub lines: Vec<Line<T>>,
    pub neighbours: Vec<(Option<usize>, Option<usize>)>,
}

pub type ClippingMask<T> = Polygon<T>;

pub trait VoronoiDiagram<T: GeoFloat>
where
    f64: From<T>,
{
    fn compute_voronoi_components(
        &self,
        clipping_mask: Option<&ClippingMask<T>>,
    ) -> Result<VoronoiComponents<T>>;
    fn compute_voronoi_diagram(
        &self,
        clipping_mask: Option<&ClippingMask<T>>,
    ) -> Result<Vec<Polygon>>;
}

impl<T: GeoFloat> VoronoiDiagram<T> for Polygon<T>
where
    f64: From<T>,
{
    fn compute_voronoi_components(
        &self,
        clipping_mask: Option<&ClippingMask<T>>,
    ) -> Result<VoronoiComponents<T>> {
        compute_voronoi_components(self, clipping_mask)
    }

    fn compute_voronoi_diagram(
        &self,
        _clipping_mask: Option<&ClippingMask<T>>,
    ) -> Result<Vec<Polygon>> {
        todo!("Need to map the components to voronoi cells");
    }
}
impl<T: GeoFloat> VoronoiDiagram<T> for MultiPoint<T>
where
    f64: From<T>,
{
    fn compute_voronoi_components(
        &self,
        clipping_mask: Option<&ClippingMask<T>>,
    ) -> Result<VoronoiComponents<T>> {
        compute_voronoi_components(self, clipping_mask)
    }

    fn compute_voronoi_diagram(
        &self,
        _clipping_mask: Option<&ClippingMask<T>>,
    ) -> Result<Vec<Polygon>> {
        todo!("Need to map the components to voronoi cells");
    }
}

fn compute_voronoi_components<T: GeoFloat, U: TriangulateDelaunay<T>>(
    polygon: &U,
    clipping_mask: Option<&ClippingMask<T>>,
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
            neighbours: vec![],
        });
    }

    compute_voronoi_components_from_delaunay(&triangles, clipping_mask)
}

/// Compute the Voronoi Diagram from Delaunay Triangles.
/// The Voronoi Diagram is a [dual graph](https://en.wikipedia.org/wiki/Dual_graph)
/// of the [Delaunay Triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation)
/// and thus the Voronoi Diagram can be created from the Delaunay Triangulation.
pub fn compute_voronoi_components_from_delaunay<T: GeoFloat>(
    triangles: &[Triangle<T>],
    clipping_mask: Option<&Polygon<T>>,
) -> Result<VoronoiComponents<T>>
where
    f64: From<T>,
{
    if triangles.is_empty() {
        return Ok(VoronoiComponents {
            delaunay_triangles: vec![],
            vertices: vec![],
            lines: vec![],
            neighbours: vec![],
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
        vertices.push(
            tri.get_circumcircle_center()
                .map_err(VoronoiDiagramError::DelaunayError)?,
        );
    }

    // Find the shared edges
    let mut shared = find_shared_edges(&delaunay_triangles);

    // Create the lines joining the vertices
    let mut voronoi_lines: Vec<Line<T>> = Vec::new();
    for neighbour in &shared.neighbours {
        if let (Some(first_vertex), Some(second_vertex)) = (neighbour.0, neighbour.1) {
            voronoi_lines.push(Line::new(vertices[first_vertex], vertices[second_vertex]));
        }
    }

    voronoi_lines.extend(construct_edges_to_inf(
        &delaunay_triangles,
        &vertices,
        &mut shared.shared_edges,
        &shared.neighbours,
        &clipping_mask,
    )?);

    Ok(VoronoiComponents {
        delaunay_triangles: triangles.to_vec(),
        vertices,
        lines: voronoi_lines,
        neighbours: shared.neighbours,
    })
}

fn create_clipping_mask<T: GeoFloat>(triangles: &[Triangle<T>]) -> Result<Polygon<T>>
where
    f64: From<T>,
{
    let mut pts: Vec<Point<T>> = Vec::new();
    for tri in triangles {
        pts.extend(tri.to_array().iter().map(|x| Point::from(*x)));
    }
    let bounds = MultiPoint::new(pts)
        .bounding_rect()
        .ok_or(VoronoiDiagramError::CannotDetermineBoundsFromClipppingMask)?;

    let expand_factor = T::from(DEFAULT_SUPER_TRIANGLE_EXPANSION)
        .ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;

    let width = bounds.width() * expand_factor;
    let height = bounds.height() * expand_factor;
    let bounds_min = bounds.min();

    Ok(Polygon::from(Rect::new(
        coord! { x: bounds_min.x - width, y: bounds_min.y - height},
        coord! { x: bounds_min.x + width, y: bounds_min.y + height},
    )))
}

type Neighbor = (Option<usize>, Option<usize>);
type Neighbors = Vec<Neighbor>;

#[derive(Debug, Clone)]
struct SharedEdgesData<T: GeoFloat> {
    neighbours: Neighbors,
    shared_edges: Vec<Line<T>>,
}

fn find_shared_edges<T: GeoFloat>(triangles: &[DelaunayTriangle<T>]) -> SharedEdgesData<T>
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

    // For Voronoi diagrams, the triangles / circumcenters that are on the edge of the
    // diagram require connections to infinity to ensure separation of points between
    // voronoi cells. Voronoi cells on the borders can have 2 connections to infinity.
    // These connections to infinity will be bounded later, for now add the connections from infinity.
    let mut num_neighbours: Vec<usize> = triangles.iter().map(|_| 0).collect();
    for x in &neighbours {
        // unwrap here is safe as neighbours do not contain None values yet
        for n in [x.0.unwrap(), x.1.unwrap()] {
            num_neighbours[n] += 1;
        }
    }

    num_neighbours.iter().enumerate().for_each(|(idx, val)| {
        for _ in 0..3 - val {
            neighbours.push((None, Some(idx)));
        }
    });

    SharedEdgesData {
        neighbours,
        shared_edges,
    }
}

fn get_perpendicular_line<T: GeoFloat>(line: &Line<T>) -> Result<Line<T>>
where
    f64: From<T>,
{
    let slope = f64::from(line.slope());

    // Vertical line
    if slope.is_infinite() {
        Ok(Line::new(
            coord! {x: line.start.x, y: line.start.y},
            coord! {x: line.start.x - line.dy(), y: line.start.y},
        ))
    } else if slope == 0. {
        Ok(Line::new(
            coord! {x: line.start.x, y: line.start.y},
            coord! {x: line.start.x, y: line.start.y + line.dx()},
        ))
    } else {
        let midpoint = coord! {
            x: f64::from(line.start.x) + f64::from(line.dx()) / 2.,
            y: f64::from(line.start.y) + f64::from(line.dy()) / 2.,
        };
        // y = mx + b
        let m = -1. / slope;
        let b = m.mul_add(-midpoint.x, midpoint.y);
        let x = midpoint.x + f64::from(line.dx());
        let y = m.mul_add(x, b);

        let end_x = T::from(x).ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;
        let end_y = T::from(y).ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;
        let start_x =
            T::from(midpoint.x).ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;
        let start_y =
            T::from(midpoint.y).ok_or(VoronoiDiagramError::CannotConvertBetweenGeoGenerics)?;
        Ok(Line::new(
            coord! {x: start_x, y: start_y},
            coord! {x: end_x, y: end_y},
        ))
    }
}

#[derive(Debug, Clone)]
enum GuidingDirection {
    Left,
    Right,
    Up,
    Down,
}

impl GuidingDirection {
    // The common point is the end co-ordinate of both lines
    fn from_midpoints<T: GeoFloat>(line_a: &Line<T>, line_b: &Line<T>) -> Self {
        let dx_a = line_a.dx().is_sign_positive();
        let dy_a = line_a.dy().is_sign_positive();
        let dx_b = line_b.dx().is_sign_positive();
        let dy_b = line_b.dy().is_sign_positive();

        // In a triangle a midpoint must either be
        // to the left or right of the other midpoints, or
        // above of below the other midpoints
        if (dx_a == dx_b) && dx_a {
            Self::Right
        } else if (dx_a == dx_b) && !dx_a {
            Self::Left
        } else if (dy_a == dy_b) && dy_a {
            Self::Up
        } else {
            Self::Down
        }
    }
}

fn define_edge_to_infinity<T: GeoFloat>(
    triangle: &DelaunayTriangle<T>,
    circumcenter: &Coord<T>,
    shared_edges: &mut Vec<Line<T>>,
    clipping_mask: &Rect<T>,
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

        // Get the line that passes from the circumcenter and is perpendicular to
        // the edge without a 3rd vertex
        let line: Line<T> = get_perpendicular_line(edge)?;
        let slope = line.slope();

        // Get the width and height of the clipping mask to ensure intersection with
        // the lines of infinity. 2 x the width or height should be sufficient.
        // Add the values to each other to prevent the need for another trait
        let width_factor = clipping_mask.width() + clipping_mask.width();
        let height_factor = clipping_mask.height() + clipping_mask.height();

        // We want to move away from the other two edges of the triangle towards infinity.
        // We will compare the three midpoints of the triangle to determine the correct direction.
        let rel_to_midpoint: Vec<_> = midpoints
            .iter()
            .filter(|x| *x != midpoint)
            .map(|x| Line::new(*x, *midpoint))
            .collect();
        let guiding_direction =
            GuidingDirection::from_midpoints(&rel_to_midpoint[0], &rel_to_midpoint[1]);

        // It is a vertical line so we need to ensure it moves up or down correctly
        let end = if slope.is_infinite() {
            match guiding_direction {
                GuidingDirection::Up => {
                    coord! {x: circumcenter.x, y: circumcenter.y + line.dy().abs() * height_factor}
                }
                GuidingDirection::Down => {
                    coord! {x: circumcenter.x, y: circumcenter.y - line.dy().abs() * height_factor}
                }
                _ => return Err(VoronoiDiagramError::UnexpectedDirectionForInfinityVertex),
            }
        } else {
            let intercept = circumcenter.y - line.slope() * circumcenter.x;
            let end_x_neg = circumcenter.x - line.dx().abs() * width_factor;
            let end_y_neg = line.slope() * end_x_neg + intercept;
            let end_x_pos = circumcenter.x + line.dx().abs() * width_factor;
            let end_y_pos = line.slope() * end_x_pos + intercept;

            match guiding_direction {
                GuidingDirection::Left => {
                    coord! {x: end_x_neg, y: end_y_neg}
                }
                GuidingDirection::Right => {
                    coord! {x: end_x_pos, y: end_y_pos}
                }
                GuidingDirection::Up => {
                    if end_y_neg.is_positive() {
                        coord! {x: end_x_neg, y: end_y_neg}
                    } else {
                        coord! {x: end_x_pos, y: end_y_pos}
                    }
                }
                GuidingDirection::Down => {
                    if end_y_neg.is_negative() {
                        coord! {x: end_x_neg, y: end_y_neg}
                    } else {
                        coord! {x: end_x_pos, y: end_y_pos}
                    }
                }
            }
        };

        shared_edges.push(*edge);

        return Ok(Some(Line::new(*circumcenter, end)));
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

fn construct_edges_to_inf<T: GeoFloat>(
    triangles: &[DelaunayTriangle<T>],
    vertices: &[Coord<T>],
    edges: &mut Vec<Line<T>>,
    neighbours: &[(Option<usize>, Option<usize>)],
    clipping_mask: &Polygon<T>,
) -> Result<Vec<Line<T>>>
where
    f64: From<T>,
{
    // Find the mean vertex to determine the direction
    // of edges going to infinity.
    let clipping_bounds = clipping_mask
        .bounding_rect()
        .ok_or(VoronoiDiagramError::CannotDetermineBoundsFromClipppingMask)?;
    let clipping_min = clipping_bounds.min();
    let clipping_max = clipping_bounds.max();

    // Get the vertices with connections to infinity
    let mut inf_lines: Vec<Line<T>> = Vec::new();
    for neighbour in neighbours {
        if let (None, Some(tri_idx)) = (neighbour.0, neighbour.1) {
            let inf_edge = define_edge_to_infinity(
                &triangles[tri_idx],
                &vertices[tri_idx],
                edges,
                &clipping_bounds,
            )?
            .ok_or(VoronoiDiagramError::CannotComputeExpectedInfinityVertex)?;
            let inf_edge_dx_sign = inf_edge.dx().is_sign_positive();
            let inf_edge_dy_sign = inf_edge.dy().is_sign_positive();

            // Get the clipping mask line where the inf_vertex intersects
            let intersection_lines: Vec<Line<T>> = clipping_mask
                .exterior()
                .lines()
                .map(|x| trim_line_to_intersection(&inf_edge, &x))
                .filter(|x| x.is_some())
                .flatten()
                // A line can intersect and be outside of the bounding box
                // filter those out
                .filter(|x| {
                    x.end.x >= clipping_min.x
                        && x.end.x <= clipping_max.x
                        && x.end.y >= clipping_min.y
                        && x.end.y <= clipping_max.y
                })
                // Get the lines going in the same direction as the inf_edge
                .filter(|x| {
                    x.dx().is_sign_positive() == inf_edge_dx_sign
                        && x.dy().is_sign_positive() == inf_edge_dy_sign
                })
                .collect();

            if intersection_lines.len() > 1 {
                eprintln!("Warning: multiple intersection lines with clipping mask found. Using the first intersection");
            }

            let intersection_line = intersection_lines
                .get(0)
                .ok_or(VoronoiDiagramError::InvalidClippingMaskNoIntersections)?;

            inf_lines.push(*intersection_line);
        }
    }
    Ok(inf_lines)
}

#[derive(Debug, PartialEq, Eq)]
pub enum VoronoiDiagramError {
    DelaunayError(DelaunayTriangulationError),
    CannotConvertBetweenGeoGenerics,
    CannotDetermineBoundsFromClipppingMask,
    CannotComputeExpectedInfinityVertex,
    InvalidClippingMaskMultipleIntersections,
    InvalidClippingMaskNoIntersections,
    UnexpectedDirectionForInfinityVertex,
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
            VoronoiDiagramError::InvalidClippingMaskMultipleIntersections => {
                write!(
                    f,
                    "An edge to infinity intersects with multiple lines in the clipping mask"
                )
            }
            VoronoiDiagramError::InvalidClippingMaskNoIntersections => {
                write!(
                    f,
                    "An edge to infinity does not intersect with the clipping mask"
                )
            }
            VoronoiDiagramError::UnexpectedDirectionForInfinityVertex => {
                write!(f, "The direction of the infinity vertex is unexpected")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use geo_types::polygon;

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

        let shared = find_shared_edges(&triangles);

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

        let shared = find_shared_edges(&triangles);

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

        let bounds = Rect::new(coord! {x: -2., y: -2.}, coord! {x: 2., y: 2.});

        let circumcenter = tri.get_circumcircle_center().unwrap();
        let mut shared_edges = vec![
            Line::new(coord! {x: 0., y: 0.}, coord! {x: 0., y: 1. }),
            Line::new(coord! {x: 0., y: 1.}, coord! {x: 1., y: 1. }),
            Line::new(coord! {x: 0., y: 1.}, coord! {x: -1., y: 2. }),
        ];
        let perpendicular_line =
            define_edge_to_infinity(&tri, &circumcenter, &mut shared_edges, &bounds).unwrap();
        assert_eq!(
            perpendicular_line.unwrap(),
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: 8.5, y: -7.5},)
        );

        let circumcenter = tri2.get_circumcircle_center().unwrap();
        let perpendicular_line =
            define_edge_to_infinity(&tri2, &circumcenter, &mut shared_edges, &bounds).unwrap();
        assert_eq!(
            perpendicular_line.unwrap(),
            Line::new(coord! {x: -1.5, y: 0.5}, coord! {x: -9.5, y: -3.5},)
        );

        let circumcenter = tri3.get_circumcircle_center().unwrap();
        let perpendicular_line =
            define_edge_to_infinity(&tri3, &circumcenter, &mut shared_edges, &bounds).unwrap();
        assert_eq!(
            perpendicular_line.unwrap(),
            Line::new(coord! {x: 0.5, y: 2.5}, coord! {x: 16.5, y: 34.5},)
        );
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

    fn compare_voronoi(voronoi_vertices: Vec<Coord>, voronoi_lines: Vec<Line>) {
        let expected_lines = vec![
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: -1.5, y: 0.5}),
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: 0.5, y: 2.5}),
            Line::new(coord! {x: -1.5, y: 0.5}, coord! {x: 0.5, y: 2.5}),
            Line::new(coord! {x: 0.5, y: 0.5}, coord! {x: 39., y: -38.}),
            Line::new(coord! {x: -1.5, y: 0.5}, coord! {x: -41., y: -19.25}),
            Line::new(coord! {x: 0.5, y: 2.5}, coord! {x: 19.25, y: 40.}),
        ];

        let expected_vertices = vec![
            coord! { x: 0.5, y: 0.5},
            coord! {x: -1.5, y: 0.5},
            coord! {x: 0.5, y: 2.5},
        ];

        assert_eq!(expected_vertices.len(), voronoi_vertices.len());
        assert_eq!(expected_lines.len(), voronoi_lines.len());

        for vertex in expected_vertices.iter() {
            assert!(expected_vertices.contains(vertex));
        }

        for line in expected_lines.iter() {
            assert!(voronoi_lines.contains(line));
        }
    }

    #[test]
    fn test_compute_voronoi_from_delaunay() {
        let triangles: Vec<Triangle<_>> = vec![
            Triangle::new(
                coord! {x: 0., y: 0.},
                coord! {x: 0., y: 1.},
                coord! {x: 1., y: 1.},
            ),
            Triangle::new(
                coord! {x: 0., y: 0.},
                coord! {x: 0., y: 1.},
                coord! {x: -1., y: 2.},
            ),
            Triangle::new(
                coord! {x: 0., y: 1.},
                coord! {x: -1., y: 2.},
                coord! {x: 1., y: 1.},
            ),
        ];

        let voronoi = compute_voronoi_components_from_delaunay(&triangles, None).unwrap();

        compare_voronoi(voronoi.vertices, voronoi.lines);
    }

    #[test]
    fn test_compute_voronoi_twin_inf_edges() {
        let triangles: Vec<Triangle<_>> = vec![
            Triangle::new(
                coord! {x: 0., y: 0.},
                coord! {x: 1., y: 1.},
                coord! {x: 2., y: 0.},
            ),
            Triangle::new(
                coord! {x: 1., y: 1.},
                coord! {x: 2., y: 0.},
                coord! {x: 3., y: 1.},
            ),
            Triangle::new(
                coord! {x: 2., y: 0.},
                coord! {x: 3., y: 1.},
                coord! {x: 3., y: 0.},
            ),
        ];

        let voronoi = compute_voronoi_components_from_delaunay(&triangles, None).unwrap();

        assert_eq!(
            voronoi.vertices,
            vec![
                coord! {x: 1.0, y: 0.0},
                coord! {x: 2.0, y: 1.0},
                coord! {x: 2.5, y: 0.5}
            ]
        );
        assert_eq!(
            voronoi.lines,
            vec![
                Line::new(coord! {x: 1.0, y: 0.0}, coord! {x: 2.0, y: 1.0}),
                Line::new(coord! {x: 2.0, y: 1.0}, coord! {x: 2.5, y: 0.5}),
                Line::new(coord! {x: 1.0, y: 0.0}, coord! {x: -19., y: 20.0}),
                Line::new(coord! {x: 1.0, y: 0.0}, coord! {x: 1., y: -20.0}),
                Line::new(coord! {x: 2.0, y: 1.0}, coord! {x: 2., y: 20.0}),
                Line::new(coord! {x: 2.5, y: 0.5}, coord! {x: 60., y: 0.5}),
                Line::new(coord! {x: 2.5, y: 0.5}, coord! {x: 2.5, y: -20.}),
            ]
        );
    }

    #[test]
    fn test_voronoi_from_polygon() {
        let poly = polygon![(x: 0., y: 0.), (x: 0., y: 1.), (x: 1., y: 1.), (x: -1., y: 2.)];

        let voronoi = poly.compute_voronoi_components(None).unwrap();
        compare_voronoi(voronoi.vertices, voronoi.lines);
    }

    // https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/VoronoiTest.cpp#L154
    #[test]
    fn test_single_point() {
        let poly = polygon![(x: 150., y: 200.)];
        let voronoi = poly.compute_voronoi_components(None).unwrap();

        assert!(voronoi.vertices.is_empty());
        assert!(voronoi.lines.is_empty());
    }

    // https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/VoronoiTest.cpp#L164
    #[test]
    fn test_simple() {
        let points = polygon![(x: 150., y: 200.), (x: 180., y: 270.), (x: 275., y: 163.)];

        let voronoi = points.compute_voronoi_components(None).unwrap();

        let expected_vertex = coord! {x: 211.205, y: 210.911};

        approx::assert_relative_eq!(
            voronoi.vertices[0],
            expected_vertex,
            max_relative = 0.3 // epsilon = 1e-3
        );

        let expected_lines = vec![
            Line::new(expected_vertex, coord! {x: -2350.0, y: 1312.857}),
            Line::new(expected_vertex, coord! {x: -426.416, y: -1977.0}),
            Line::new(expected_vertex, coord! {x: 2577.558, y: 2303.0}),
        ];

        for (line, expected) in voronoi.lines.iter().zip(expected_lines) {
            approx::assert_relative_eq!(*line, expected, max_relative = 0.3);
        }
    }

    // https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/VoronoiTest.cpp#L174
    #[test]
    fn test_four_points() {
        let points: MultiPoint<_> =
            vec![(280., 300.), (420., 330.), (380., 230.), (320., 160.)].into();

        let voronoi = points.compute_voronoi_components(None).unwrap();

        let expected_vertices = [
            coord! {x: 353.516, y: 298.594},
            coord! {x: 306.875, y: 231.964},
        ];

        for (vertex, expected) in voronoi.vertices.iter().zip(expected_vertices) {
            approx::assert_relative_eq!(
                *vertex,
                expected,
                max_relative = 0.3 // epsilon = 1e-3
            );
        }

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

        for (line, expected) in voronoi.lines.iter().zip(expected_lines) {
            approx::assert_relative_eq!(
                *line,
                expected,
                max_relative = 0.3 // epsilon = 1e-3
            );
        }
    }

    //https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/VoronoiTest.cpp#L174
    #[test]
    fn test_five_points() {
        let points: MultiPoint<_> =
            vec![(280., 300.), (420., 330.), (380., 230.), (320., 160.)].into();

        let voronoi = points.compute_voronoi_components(None).unwrap();

        let expected_vertices = [
            coord! {x: 353.516, y: 298.594},
            coord! {x: 306.875, y: 231.964},
        ];

        for (vertex, expected) in voronoi.vertices.iter().zip(expected_vertices) {
            approx::assert_relative_eq!(
                *vertex,
                expected,
                max_relative = 0.3 // epsilon = 1e-3
            );
        }

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

        for (line, expected) in voronoi.lines.iter().zip(expected_lines) {
            approx::assert_relative_eq!(
                *line,
                expected,
                max_relative = 0.3 // epsilon = 1e-3
            );
        }
    }
}
