use geo_types::Coord;

use crate::{
    Line, LineString, Point, Polygon,
    algorithm::{
        Distance, closest_point::ClosestPoint, contains::Contains, convex_hull::ConvexHull,
        intersects::Intersects, line_measures::metric_spaces::Euclidean, relate::Relate,
    },
};

/// Main function to compute the separation distance between two simple polygons
pub fn compute_polygon_separation(p: &Polygon<f64>, q: &Polygon<f64>) -> f64 {
    // Phase 1: Check for intersection
    if polygons_intersect(p, q) {
        return 0.0;
    }

    // Phase 2: Classify case and construct R
    let (ch_p, ch_q, ch_union) = compute_hulls(p, q);
    let case = classify_case(&ch_p, &ch_q, &ch_union, p, q);
    let polygon_r = construct_polygon_r(&case, p, q);

    // Phase 3: Find shortest path (simplified for skeleton)
    let (start, end) = get_path_endpoints(&case);
    let shortest_path = shortest_path_in_polygon(&polygon_r, start, end);

    // Phase 4: Construct separators
    let extended_segments = extend_segments_to_boundary(&shortest_path, &polygon_r);
    let separators = remove_redundant_segments(extended_segments);

    // Phase 5: Solve subproblems
    let subproblems = construct_subproblems(&separators, p, q);
    let distances: Vec<f64> = subproblems
        .iter()
        .map(solve_linearly_separable_subproblem)
        .collect();

    // Phase 6: Return minimum
    distances
        .into_iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(f64::INFINITY)
}

/// Check if two polygons intersect using Bentley-Ottmann approach
fn polygons_intersect(p: &Polygon<f64>, q: &Polygon<f64>) -> bool {
    // Extract all edges from both polygons
    let p_lines: Vec<Line<f64>> = p.exterior().lines().collect();
    let q_lines: Vec<Line<f64>> = q.exterior().lines().collect();

    // Check for any intersections between edges
    for p_line in &p_lines {
        for q_line in &q_lines {
            if p_line.intersects(q_line) {
                return true;
            }
        }
    }

    // Also check if one polygon is completely inside the other
    if p.contains(q) || q.contains(p) {
        return true;
    }

    false
}

/// Compute convex hulls
fn compute_hulls(p: &Polygon<f64>, q: &Polygon<f64>) -> (Polygon<f64>, Polygon<f64>, Polygon<f64>) {
    let ch_p = p.convex_hull();
    let ch_q = q.convex_hull();

    // Combine vertices and compute union hull
    let mut combined_points: Vec<Coord<f64>> = Vec::new();
    combined_points.extend(p.exterior().coords());
    combined_points.extend(q.exterior().coords());

    let combined_polygon = Polygon::new(LineString::from(combined_points), vec![]);
    let ch_union = combined_polygon.convex_hull();

    (ch_p, ch_q, ch_union)
}

/// Check polygon equality using topological relationship
fn polygons_equal(p1: &Polygon<f64>, p2: &Polygon<f64>) -> bool {
    p1.relate(p2).is_equal_topo()
}

/// Classification of separation cases
#[derive(Debug, Clone)]
enum SeparationCase {
    NonContaining {
        supporting_lines: (Line<f64>, Line<f64>),
        p_portion: LineString<f64>,
        q_portion: LineString<f64>,
    },
    Containing {
        _container: ContainerType,
        visible_segment: Line<f64>,
    },
}

#[derive(Debug, Clone)]
enum ContainerType {
    PContainsQ,
    QContainsP,
}

/// Classify the relationship between polygons
fn classify_case(
    ch_p: &Polygon<f64>,
    ch_q: &Polygon<f64>,
    ch_union: &Polygon<f64>,
    p: &Polygon<f64>,
    q: &Polygon<f64>,
) -> SeparationCase {
    if polygons_equal(ch_union, ch_p) {
        // P contains Q case
        SeparationCase::Containing {
            _container: ContainerType::PContainsQ,
            visible_segment: find_visible_segment(p, q),
        }
    } else if polygons_equal(ch_union, ch_q) {
        // Q contains P case
        SeparationCase::Containing {
            _container: ContainerType::QContainsP,
            visible_segment: find_visible_segment(p, q),
        }
    } else {
        // Non-containing case - simplified implementation
        let (lt, lb) = find_common_supporting_lines_simplified(ch_p, ch_q);
        let p_portion = extract_facing_portion(p, &lt, &lb);
        let q_portion = extract_facing_portion(q, &lt, &lb);

        SeparationCase::NonContaining {
            supporting_lines: (lt, lb),
            p_portion,
            q_portion,
        }
    }
}

/// Simplified supporting lines finding (placeholder implementation)
fn find_common_supporting_lines_simplified(
    ch_p: &Polygon<f64>,
    ch_q: &Polygon<f64>,
) -> (Line<f64>, Line<f64>) {
    // Simplified: use bounding box approach as placeholder
    let p_coords: Vec<_> = ch_p.exterior().coords().collect();
    let q_coords: Vec<_> = ch_q.exterior().coords().collect();

    let p_top = p_coords
        .iter()
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
        .unwrap();
    let p_bottom = p_coords
        .iter()
        .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
        .unwrap();
    let q_top = q_coords
        .iter()
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
        .unwrap();
    let q_bottom = q_coords
        .iter()
        .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
        .unwrap();

    let upper_line = Line::new(Point::from(**p_top), Point::from(**q_top));
    let lower_line = Line::new(Point::from(**p_bottom), Point::from(**q_bottom));

    (upper_line, lower_line)
}

/// Extract the portion of polygon between two lines
fn extract_facing_portion(
    polygon: &Polygon<f64>,
    upper_line: &Line<f64>,
    lower_line: &Line<f64>,
) -> LineString<f64> {
    let coords: Vec<Coord<f64>> = polygon
        .exterior()
        .coords()
        .filter(|coord| {
            let point = Point::from(**coord);
            point_between_lines(&point, upper_line, lower_line)
        })
        .cloned()
        .collect();

    LineString::from(coords)
}

/// Check if point lies between two lines (simplified)
fn point_between_lines(point: &Point<f64>, upper_line: &Line<f64>, lower_line: &Line<f64>) -> bool {
    // Simplified: use distance-based approximation
    let upper_dist = Euclidean.distance(upper_line, point);
    let lower_dist = Euclidean.distance(lower_line, point);

    // This is a simplified check - in full implementation we'd need proper signed distance
    upper_dist + lower_dist < 1000.0 // Arbitrary threshold for skeleton
}

/// Find a visible segment between polygons (simplified)
fn find_visible_segment(p: &Polygon<f64>, q: &Polygon<f64>) -> Line<f64> {
    let (inner_polygon, outer_polygon) = if is_inside(p, q) { (p, q) } else { (q, p) };

    // Find highest vertex of inner polygon
    let highest_point = inner_polygon
        .exterior()
        .coords()
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
        .unwrap();
    let highest_point = Point::from(*highest_point);

    // Find closest point on outer polygon
    let visible_point = find_closest_visible_point(&highest_point, outer_polygon);

    Line::new(highest_point, visible_point)
}

fn is_inside(inner: &Polygon<f64>, outer: &Polygon<f64>) -> bool {
    outer.contains(inner)
}

fn find_closest_visible_point(point: &Point<f64>, polygon: &Polygon<f64>) -> Point<f64> {
    // Simplified: just use closest point on boundary
    match polygon.closest_point(point) {
        crate::Closest::Intersection(point) => point,
        crate::Closest::SinglePoint(point) => point,
        crate::Closest::Indeterminate => {
            // Fallback: return first vertex
            Point::from(*polygon.exterior().coords().next().unwrap())
        }
    }
}

/// Construct polygon R (simplified)
fn construct_polygon_r(
    case: &SeparationCase,
    _p: &Polygon<f64>,
    _q: &Polygon<f64>,
) -> Polygon<f64> {
    match case {
        SeparationCase::NonContaining {
            supporting_lines,
            p_portion,
            q_portion,
        } => {
            let mut coords = Vec::new();

            // Add P portion Coords
            coords.extend(p_portion.coords().cloned());

            // Add upper supporting line endpoints
            coords.push(supporting_lines.0.start);
            coords.push(supporting_lines.0.end);

            // Add Q portion Coords (reversed for proper winding)
            let q_coords: Vec<_> = q_portion.coords().collect();
            coords.extend(q_coords.into_iter().rev().cloned());

            // Add lower supporting line endpoints (reversed)
            coords.push(supporting_lines.1.end);
            coords.push(supporting_lines.1.start);

            // Ensure closure
            if let Some(first) = coords.first() {
                coords.push(*first);
            }

            Polygon::new(LineString::from(coords), vec![])
        }
        SeparationCase::Containing {
            visible_segment, ..
        } => {
            // Simplified containing case
            construct_containing_polygon_r_simplified(visible_segment)
        }
    }
}

fn construct_containing_polygon_r_simplified(segment: &Line<f64>) -> Polygon<f64> {
    // Very simplified: create a small rectangle around the segment
    let dx = 1.0;
    let dy = 1.0;

    let coords = vec![
        Coord {
            x: segment.start.x - dx,
            y: segment.start.y - dy,
        },
        Coord {
            x: segment.end.x + dx,
            y: segment.start.y - dy,
        },
        Coord {
            x: segment.end.x + dx,
            y: segment.end.y + dy,
        },
        Coord {
            x: segment.start.x - dx,
            y: segment.end.y + dy,
        },
        Coord {
            x: segment.start.x - dx,
            y: segment.start.y - dy,
        }, // Close
    ];

    Polygon::new(LineString::from(coords), vec![])
}

fn get_path_endpoints(case: &SeparationCase) -> (Point<f64>, Point<f64>) {
    match case {
        SeparationCase::NonContaining {
            supporting_lines, ..
        } => (
            supporting_lines.1.start.into(),
            supporting_lines.0.start.into(),
        ),
        SeparationCase::Containing {
            visible_segment, ..
        } => (visible_segment.start.into(), visible_segment.end.into()),
    }
}

/// Simplified shortest path (just return the direct line for skeleton)
fn shortest_path_in_polygon(
    _polygon: &Polygon<f64>,
    start: Point<f64>,
    end: Point<f64>,
) -> Vec<Point<f64>> {
    vec![start, end]
}

fn extend_segments_to_boundary(path: &[Point<f64>], _polygon: &Polygon<f64>) -> Vec<Line<f64>> {
    path.windows(2)
        .map(|window| Line::new(window[0], window[1]))
        .collect()
}

fn remove_redundant_segments(segments: Vec<Line<f64>>) -> Vec<Line<f64>> {
    // Simplified: just return first and last segments
    if segments.len() <= 2 {
        segments
    } else {
        vec![segments[0], segments[segments.len() - 1]]
    }
}

#[derive(Debug, Clone)]
struct Subproblem {
    p_chain: LineString<f64>,
    q_chain: LineString<f64>,
    _separator: Line<f64>,
}

fn construct_subproblems(
    separators: &[Line<f64>],
    p: &Polygon<f64>,
    q: &Polygon<f64>,
) -> Vec<Subproblem> {
    separators
        .iter()
        .map(|separator| Subproblem {
            p_chain: p.exterior().clone(),
            q_chain: q.exterior().clone(),
            _separator: *separator,
        })
        .collect()
}

/// Simplified linearly separable solver
fn solve_linearly_separable_subproblem(subproblem: &Subproblem) -> f64 {
    // Very simplified: just compute minimum distance between all point pairs
    let mut min_distance = f64::INFINITY;

    for p_coord in subproblem.p_chain.coords() {
        for q_coord in subproblem.q_chain.coords() {
            let p_point = Point::from(*p_coord);
            let q_point = Point::from(*q_coord);
            let distance = Euclidean.distance(&p_point, &q_point);

            if distance < min_distance {
                min_distance = distance;
            }
        }
    }

    min_distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_separation() {
        // Create two simple non-intersecting squares
        let p = Polygon::new(
            LineString::from(vec![
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (0.0, 1.0),
                (0.0, 0.0),
            ]),
            vec![],
        );

        let q = Polygon::new(
            LineString::from(vec![
                (2.0, 0.0),
                (3.0, 0.0),
                (3.0, 1.0),
                (2.0, 1.0),
                (2.0, 0.0),
            ]),
            vec![],
        );

        let separation = compute_polygon_separation(&p, &q);
        assert!(separation > 0.0);
        println!("Separation distance: {}", separation);
    }

    #[test]
    fn test_intersecting_polygons() {
        // Create two overlapping squares
        let p = Polygon::new(
            LineString::from(vec![
                (0.0, 0.0),
                (2.0, 0.0),
                (2.0, 2.0),
                (0.0, 2.0),
                (0.0, 0.0),
            ]),
            vec![],
        );

        let q = Polygon::new(
            LineString::from(vec![
                (1.0, 1.0),
                (3.0, 1.0),
                (3.0, 3.0),
                (1.0, 3.0),
                (1.0, 1.0),
            ]),
            vec![],
        );

        let separation = compute_polygon_separation(&p, &q);
        assert_eq!(separation, 0.0);
    }
}
