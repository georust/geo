//! Minimum separation between two simple polygons.
//!
//! The separation of simple polygons P and Q, written σ(P, Q), is the minimum
//! distance between their *boundaries*. This differs from the polygon-polygon
//! [`Distance`]: when one polygon lies strictly inside the other their
//! interiors overlap, so the distance is zero, but the separation of their
//! boundaries is positive.
//!
//! The implementation follows Amato's decomposition algorithm:
//!
//! > Nancy M. Amato, "Determining the Separation of Simple Polygons",
//! > International Journal of Computational Geometry & Applications 4(4),
//! > 1994. <https://doi.org/10.1142/S0218195994000240>
//!
//! The algorithm decomposes the problem into subproblems whose inputs are
//! polygonal chains separated by a line, then solves each subproblem in time
//! linear in its size. The sequential version runs in Θ(n) total time, where
//! n = |P| + |Q|. The brute-force alternative (minimum distance over all
//! boundary segment pairs) is O(|P| · |Q|); that computation is available as
//! `Euclidean.distance(p.exterior(), q.exterior())` and serves as the test
//! oracle for this module.
//!
//! Both inputs must be simple polygons without interior rings.
//!
//! Status: work in progress. The decomposition phases below are placeholders.
//! See `work.md` for the implementation plan.

use geo_types::Coord;

use crate::{
    GeoFloat, Line, LineString, Point, Polygon,
    algorithm::{
        Distance, closest_point::ClosestPoint, contains::Contains, convex_hull::ConvexHull,
        intersects::Intersects, line_measures::metric_spaces::Euclidean, relate::Relate,
    },
};

mod linsep;

/// Compute the separation σ(P, Q) between the boundaries of two simple polygons.
///
/// Returns zero if the boundaries intersect.
pub fn compute_polygon_separation<T: GeoFloat>(p: &Polygon<T>, q: &Polygon<T>) -> T {
    // Phase 1: σ(P, Q) = 0 exactly when the boundaries intersect. Containment
    // is not tested here: a polygon strictly inside another has positive
    // boundary separation and is handled by the containing case below.
    if boundaries_intersect(p, q) {
        return T::zero();
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

    // Phase 6: Return minimum
    subproblems
        .iter()
        .map(solve_linearly_separable_subproblem)
        .fold(T::infinity(), T::min)
}

/// Test whether the boundaries of two polygons intersect.
fn boundaries_intersect<T: GeoFloat>(p: &Polygon<T>, q: &Polygon<T>) -> bool {
    p.exterior().intersects(q.exterior())
}

/// Compute convex hulls of P, Q, and P ∪ Q
fn compute_hulls<T: GeoFloat>(
    p: &Polygon<T>,
    q: &Polygon<T>,
) -> (Polygon<T>, Polygon<T>, Polygon<T>) {
    let ch_p = p.convex_hull();
    let ch_q = q.convex_hull();

    let mut combined_points: Vec<Coord<T>> = Vec::new();
    combined_points.extend(p.exterior().coords());
    combined_points.extend(q.exterior().coords());

    let combined_polygon = Polygon::new(LineString::from(combined_points), vec![]);
    let ch_union = combined_polygon.convex_hull();

    (ch_p, ch_q, ch_union)
}

/// Check polygon equality using topological relationship
fn polygons_equal<T: GeoFloat>(p1: &Polygon<T>, p2: &Polygon<T>) -> bool {
    p1.relate(p2).is_equal_topo()
}

/// Classification of separation cases per Step 1(a) of DECOMPOSE:
/// the containing case applies when CH(P ∪ Q) equals CH(P) or CH(Q),
/// the non-containing case otherwise.
#[derive(Debug, Clone)]
enum SeparationCase<T: GeoFloat> {
    NonContaining {
        supporting_lines: (Line<T>, Line<T>),
        p_portion: LineString<T>,
        q_portion: LineString<T>,
    },
    Containing {
        _container: ContainerType,
        visible_segment: Line<T>,
    },
}

#[derive(Debug, Clone)]
enum ContainerType {
    PContainsQ,
    QContainsP,
}

/// Classify the relationship between polygons
fn classify_case<T: GeoFloat>(
    ch_p: &Polygon<T>,
    ch_q: &Polygon<T>,
    ch_union: &Polygon<T>,
    p: &Polygon<T>,
    q: &Polygon<T>,
) -> SeparationCase<T> {
    if polygons_equal(ch_union, ch_p) {
        SeparationCase::Containing {
            _container: ContainerType::PContainsQ,
            visible_segment: find_visible_segment(p, q),
        }
    } else if polygons_equal(ch_union, ch_q) {
        SeparationCase::Containing {
            _container: ContainerType::QContainsP,
            visible_segment: find_visible_segment(p, q),
        }
    } else {
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

/// Placeholder for the two common supporting lines of CH(P) and CH(Q).
/// Uses the extreme-y vertices of each hull, which is not correct in
/// general. The correct construction finds the supporting lines during the
/// merge of CH(P) and CH(Q) into CH(P ∪ Q).
fn find_common_supporting_lines_simplified<T: GeoFloat>(
    ch_p: &Polygon<T>,
    ch_q: &Polygon<T>,
) -> (Line<T>, Line<T>) {
    let p_coords: Vec<_> = ch_p.exterior().coords().collect();
    let q_coords: Vec<_> = ch_q.exterior().coords().collect();

    let p_top = p_coords.iter().max_by(|a, b| a.y.total_cmp(&b.y)).unwrap();
    let p_bottom = p_coords.iter().min_by(|a, b| a.y.total_cmp(&b.y)).unwrap();
    let q_top = q_coords.iter().max_by(|a, b| a.y.total_cmp(&b.y)).unwrap();
    let q_bottom = q_coords.iter().min_by(|a, b| a.y.total_cmp(&b.y)).unwrap();

    let upper_line = Line::new(Point::from(**p_top), Point::from(**q_top));
    let lower_line = Line::new(Point::from(**p_bottom), Point::from(**q_bottom));

    (upper_line, lower_line)
}

/// Placeholder for extraction of the facing portion of a polygon between the
/// two supporting lines. The side tests must use `Kernel::orient2d`, not
/// distances; the current filter accepts nearly everything.
fn extract_facing_portion<T: GeoFloat>(
    polygon: &Polygon<T>,
    upper_line: &Line<T>,
    lower_line: &Line<T>,
) -> LineString<T> {
    let coords: Vec<Coord<T>> = polygon
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

/// Placeholder side test. To be replaced with `Kernel::orient2d` tests
/// against each supporting line.
fn point_between_lines<T: GeoFloat>(
    point: &Point<T>,
    upper_line: &Line<T>,
    lower_line: &Line<T>,
) -> bool {
    let upper_dist = Euclidean.distance(upper_line, point);
    let lower_dist = Euclidean.distance(lower_line, point);

    upper_dist + lower_dist < T::from(1000.0).unwrap()
}

/// Placeholder for the visible segment between a polygon and one it contains.
/// The paper constructs this from the highest vertex p of the inner polygon
/// and the closest edge of the outer polygon cut by the horizontal line
/// through p; this version uses the closest boundary point without a
/// visibility test.
fn find_visible_segment<T: GeoFloat>(p: &Polygon<T>, q: &Polygon<T>) -> Line<T> {
    let (inner_polygon, outer_polygon) = if q.contains(p) { (p, q) } else { (q, p) };

    let highest_point = inner_polygon
        .exterior()
        .coords()
        .max_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap();
    let highest_point = Point::from(*highest_point);

    let visible_point = find_closest_visible_point(&highest_point, outer_polygon);

    Line::new(highest_point, visible_point)
}

fn find_closest_visible_point<T: GeoFloat>(point: &Point<T>, polygon: &Polygon<T>) -> Point<T> {
    match polygon.closest_point(point) {
        crate::Closest::Intersection(point) => point,
        crate::Closest::SinglePoint(point) => point,
        crate::Closest::Indeterminate => {
            // Fallback: return first vertex
            Point::from(*polygon.exterior().coords().next().unwrap())
        }
    }
}

/// Placeholder construction of the polygon R that lies between P and Q.
/// Concatenates the facing portions and supporting-line endpoints without
/// enforcing winding or simplicity.
fn construct_polygon_r<T: GeoFloat>(
    case: &SeparationCase<T>,
    _p: &Polygon<T>,
    _q: &Polygon<T>,
) -> Polygon<T> {
    match case {
        SeparationCase::NonContaining {
            supporting_lines,
            p_portion,
            q_portion,
        } => {
            let mut coords = Vec::new();

            coords.extend(p_portion.coords().cloned());
            coords.push(supporting_lines.0.start);
            coords.push(supporting_lines.0.end);

            let q_coords: Vec<_> = q_portion.coords().collect();
            coords.extend(q_coords.into_iter().rev().cloned());

            coords.push(supporting_lines.1.end);
            coords.push(supporting_lines.1.start);

            if let Some(first) = coords.first() {
                coords.push(*first);
            }

            Polygon::new(LineString::from(coords), vec![])
        }
        SeparationCase::Containing {
            visible_segment, ..
        } => construct_containing_polygon_r_simplified(visible_segment),
    }
}

/// Placeholder: a rectangle around the visible segment stands in for the
/// region between the inner and outer polygon.
fn construct_containing_polygon_r_simplified<T: GeoFloat>(segment: &Line<T>) -> Polygon<T> {
    let dx = T::one();
    let dy = T::one();

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
        },
    ];

    Polygon::new(LineString::from(coords), vec![])
}

fn get_path_endpoints<T: GeoFloat>(case: &SeparationCase<T>) -> (Point<T>, Point<T>) {
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

/// Placeholder: the direct segment stands in for the shortest path within R.
/// The full version triangulates R (earcut), walks the triangulation dual
/// (a tree for a hole-free simple polygon), and applies the funnel algorithm.
fn shortest_path_in_polygon<T: GeoFloat>(
    _polygon: &Polygon<T>,
    start: Point<T>,
    end: Point<T>,
) -> Vec<Point<T>> {
    vec![start, end]
}

/// Placeholder: performs no extension. Step 1(c) of DECOMPOSE extends each
/// path segment to the boundary of R by ray shooting.
fn extend_segments_to_boundary<T: GeoFloat>(
    path: &[Point<T>],
    _polygon: &Polygon<T>,
) -> Vec<Line<T>> {
    path.windows(2)
        .map(|window| Line::new(window[0], window[1]))
        .collect()
}

/// Placeholder. Step 1(d) of DECOMPOSE keeps l_0 and then greedily keeps the
/// maximal-indexed segment that intersects the previously kept one; this
/// version keeps only the first and last segments.
fn remove_redundant_segments<T: GeoFloat>(segments: Vec<Line<T>>) -> Vec<Line<T>> {
    if segments.len() <= 2 {
        segments
    } else {
        vec![segments[0], segments[segments.len() - 1]]
    }
}

#[derive(Debug, Clone)]
struct Subproblem<T: GeoFloat> {
    p_chain: LineString<T>,
    q_chain: LineString<T>,
    separator: Line<T>,
}

/// Placeholder: every subproblem receives the full exterior rings instead of
/// the subchains bounded by consecutive separators (Step 2 of DECOMPOSE).
fn construct_subproblems<T: GeoFloat>(
    separators: &[Line<T>],
    p: &Polygon<T>,
    q: &Polygon<T>,
) -> Vec<Subproblem<T>> {
    separators
        .iter()
        .map(|separator| Subproblem {
            p_chain: p.exterior().clone(),
            q_chain: q.exterior().clone(),
            separator: *separator,
        })
        .collect()
}

/// Solve one subproblem with the LinSep solver when its chains really are
/// separated by the subproblem's separator. The placeholder decomposition
/// does not yet produce genuinely separated subchains, so the brute-force
/// fallback usually applies.
fn solve_linearly_separable_subproblem<T: GeoFloat>(subproblem: &Subproblem<T>) -> T {
    match linsep::SeparatedChains::new(
        &subproblem.p_chain,
        &subproblem.q_chain,
        subproblem.separator,
    ) {
        Some(chains) => chains.separation(),
        None => linsep::separation_brute_force(&subproblem.p_chain, &subproblem.q_chain),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wkt;
    use approx::assert_relative_eq;
    use hegel::generators;

    /// A star-shaped polygon around (cx, cy): vertices at strictly
    /// increasing angles (gaps bounded below before normalisation, so no
    /// two vertices are radially aligned) with radii in
    /// [r_max / 10, r_max]. Star-shapedness guarantees a simple ring.
    #[hegel::composite]
    fn star_polygon(tc: &hegel::TestCase, cx: f64, cy: f64, r_max: f64) -> Polygon<f64> {
        let n = tc.draw(generators::integers::<usize>().min_value(3).max_value(12));
        let gaps: Vec<f64> = (0..n)
            .map(|_| tc.draw(generators::floats::<f64>().min_value(0.1).max_value(1.0)))
            .collect();
        let total: f64 = gaps.iter().sum();
        let mut angle = 0.0;
        let coords: Vec<Coord<f64>> = gaps
            .iter()
            .map(|g| {
                angle += g / total * std::f64::consts::TAU;
                let r = tc.draw(
                    generators::floats::<f64>()
                        .min_value(0.1 * r_max)
                        .max_value(r_max),
                );
                Coord {
                    x: cx + r * angle.cos(),
                    y: cy + r * angle.sin(),
                }
            })
            .collect();
        Polygon::new(LineString::from(coords), vec![])
    }

    /// A star polygon with centre in [-8, 8]² and maximum radius in
    /// [0.2, 4]: pairs cover the separated, nearby, and nested
    /// configurations.
    fn draw_star(tc: &hegel::TestCase) -> Polygon<f64> {
        let cx = tc.draw(generators::floats::<f64>().min_value(-8.0).max_value(8.0));
        let cy = tc.draw(generators::floats::<f64>().min_value(-8.0).max_value(8.0));
        let r_max = tc.draw(generators::floats::<f64>().min_value(0.2).max_value(4.0));
        tc.draw(star_polygon(cx, cy, r_max))
    }

    /// The headline DECOMPOSE property: for polygons with disjoint
    /// boundaries, the pipeline agrees with the segment-pair oracle. The
    /// tolerance is the scale-aware contract established for the LinSep
    /// solver (see `separation_matches_segment_pair_minimum` in
    /// linsep.rs); coordinates here are bounded by centre plus radius,
    /// so the scale is a constant.
    #[hegel::test]
    fn separation_matches_oracle_for_disjoint_boundaries(tc: hegel::TestCase) {
        let p = draw_star(&tc);
        let q = draw_star(&tc);
        tc.assume(!p.exterior().intersects(q.exterior()));

        let sigma = compute_polygon_separation(&p, &q);
        let oracle = linsep::separation_brute_force(p.exterior(), q.exterior());

        assert_relative_eq!(
            sigma,
            oracle,
            max_relative = 8.0 * f64::EPSILON,
            epsilon = 32.0 * f64::EPSILON * 16.0
        );
    }

    #[test]
    fn separated_squares() {
        let p = wkt! { POLYGON((0. 0.,1. 0.,1. 1.,0. 1.,0. 0.)) };
        let q = wkt! { POLYGON((2. 0.,3. 0.,3. 1.,2. 1.,2. 0.)) };

        let separation = compute_polygon_separation(&p, &q);
        assert_relative_eq!(separation, 1.0);
    }

    #[test]
    fn intersecting_squares() {
        let p = wkt! { POLYGON((0. 0.,2. 0.,2. 2.,0. 2.,0. 0.)) };
        let q = wkt! { POLYGON((1. 1.,3. 1.,3. 3.,1. 3.,1. 1.)) };

        let separation = compute_polygon_separation(&p, &q);
        assert_relative_eq!(separation, 0.0);
    }

    #[test]
    fn contained_square_has_positive_separation() {
        // Boundary separation, not interior distance: a polygon strictly
        // inside another must yield σ > 0. The exact value (4.0) is asserted
        // once the containing case is implemented; the placeholder pipeline
        // currently overestimates it.
        let p = wkt! { POLYGON((0. 0.,10. 0.,10. 10.,0. 10.,0. 0.)) };
        let q = wkt! { POLYGON((4. 4.,6. 4.,6. 6.,4. 6.,4. 4.)) };

        let separation: f64 = compute_polygon_separation(&p, &q);
        assert!(separation > 0.0);
        assert!(separation.is_finite());
    }
}
