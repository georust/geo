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
    GeoFloat, GeoNum, Kernel, Line, LineString, Orientation, Point, Polygon,
    algorithm::{
        closest_point::ClosestPoint, contains::Contains, convex_hull::ConvexHull,
        intersects::Intersects, relate::Relate, winding_order::Winding,
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
    let Some(case) = classify_case(&ch_p, &ch_q, &ch_union, p, q) else {
        // Escape hatch for hull degeneracies the classifier does not
        // recognise; correct, but without the linear-time decomposition.
        return linsep::separation_brute_force(p.exterior(), q.exterior());
    };
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
/// the non-containing case otherwise. In the non-containing case the two
/// bridges are the common supporting segments in CH(P ∪ Q) winding
/// order: `bridge_pq` runs from its P contact to its Q contact,
/// `bridge_qp` the other way.
#[derive(Debug, Clone)]
enum SeparationCase<T: GeoFloat> {
    NonContaining {
        bridge_pq: Line<T>,
        bridge_qp: Line<T>,
    },
    Containing {
        visible_segment: Line<T>,
    },
}

/// Classify the relationship between polygons. `None` when the
/// non-containing bridge construction fails (a hull degeneracy).
fn classify_case<T: GeoFloat>(
    ch_p: &Polygon<T>,
    ch_q: &Polygon<T>,
    ch_union: &Polygon<T>,
    p: &Polygon<T>,
    q: &Polygon<T>,
) -> Option<SeparationCase<T>> {
    if polygons_equal(ch_union, ch_p) || polygons_equal(ch_union, ch_q) {
        Some(SeparationCase::Containing {
            visible_segment: find_visible_segment(p, q),
        })
    } else {
        let (bridge_pq, bridge_qp) = common_supporting_segments(ch_union, p, q)?;
        Some(SeparationCase::NonContaining {
            bridge_pq,
            bridge_qp,
        })
    }
}

/// The two common supporting segments ("bridges") of P and Q in the
/// non-containing case, read off CH(P ∪ Q): they are the hull edges with
/// one endpoint from each polygon. Contact points are then adjusted along
/// the supporting line: when vertices of both polygons are collinear on
/// it, hull construction elides the inner collinear vertices and the hull
/// edge overshoots the tangency, so each contact is the extreme on-line
/// vertex of its own polygon toward the other polygon.
///
/// Returned in CH(P ∪ Q) winding order (counterclockwise, both polygons
/// on the left of each directed bridge): the first bridge runs from its
/// P contact to its Q contact, the second from Q to P. `None` when the
/// hull does not have exactly one mixed edge in each direction, which
/// does not occur for simple polygons with disjoint boundaries in the
/// non-containing case.
fn common_supporting_segments<T: GeoFloat>(
    ch_union: &Polygon<T>,
    p: &Polygon<T>,
    q: &Polygon<T>,
) -> Option<(Line<T>, Line<T>)> {
    // Disjoint boundaries share no coordinates, so ring membership
    // classifies every hull vertex unambiguously.
    let in_p = |c: Coord<T>| p.exterior().0.contains(&c);

    let mut bridge_pq = None;
    let mut bridge_qp = None;
    for edge in ch_union.exterior().lines() {
        let slot = match (in_p(edge.start), in_p(edge.end)) {
            (true, false) => &mut bridge_pq,
            (false, true) => &mut bridge_qp,
            _ => continue,
        };
        if slot.replace(edge).is_some() {
            return None;
        }
    }
    let bridge_pq = bridge_pq?;
    let bridge_qp = bridge_qp?;
    Some((
        contact_extremes(bridge_pq, p.exterior(), q.exterior()),
        contact_extremes(bridge_qp, q.exterior(), p.exterior()),
    ))
}

/// Shrink a mixed hull edge to the actual tangency contacts: the extreme
/// vertex of the start polygon's ring on the supporting line toward the
/// edge's end, and the extreme vertex of the end polygon's ring toward
/// the edge's start. The contact intervals of the two rings on the line
/// are disjoint (the boundaries are disjoint), so the result spans the
/// gap between the polygons.
fn contact_extremes<T: GeoFloat>(
    edge: Line<T>,
    start_ring: &LineString<T>,
    end_ring: &LineString<T>,
) -> Line<T> {
    let d = edge.delta();
    let along = |v: &Coord<T>| (v.x - edge.start.x) * d.x + (v.y - edge.start.y) * d.y;
    let on_line = |v: &&Coord<T>| {
        <T as GeoNum>::Ker::orient2d(edge.start, edge.end, **v) == Orientation::Collinear
    };
    let start_contact = start_ring
        .0
        .iter()
        .filter(on_line)
        .max_by(|a, b| along(a).total_cmp(&along(b)))
        .copied()
        .unwrap_or(edge.start);
    let end_contact = end_ring
        .0
        .iter()
        .filter(on_line)
        .min_by(|a, b| along(a).total_cmp(&along(b)))
        .copied()
        .unwrap_or(edge.end);
    Line::new(start_contact, end_contact)
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

/// Construction of the polygon R that lies between P and Q (DECOMPOSE
/// Step 1(a)). The containing arm is still a placeholder.
fn construct_polygon_r<T: GeoFloat>(
    case: &SeparationCase<T>,
    p: &Polygon<T>,
    q: &Polygon<T>,
) -> Polygon<T> {
    match case {
        SeparationCase::NonContaining {
            bridge_pq,
            bridge_qp,
        } => construct_polygon_r_noncontaining(bridge_pq, bridge_qp, p, q),
        SeparationCase::Containing {
            visible_segment, ..
        } => construct_containing_polygon_r_simplified(visible_segment),
    }
}

/// The simple polygon R between P and Q in the non-containing case,
/// wound counterclockwise: the p→q bridge, then Q's facing portion Q′
/// (Q's ring walked clockwise between its two contacts, so Q's interior
/// lies outside R), then the q→p bridge, then P's facing portion P′
/// back to the start. The bridge directions come from the
/// counterclockwise walk of CH(P ∪ Q), which is what makes the clockwise
/// polygon walks pick the facing arcs rather than the outer ones.
///
/// When a polygon touches the union hull at a single vertex (its two
/// bridge contacts coincide), its facing arc is its full ring and R is
/// pinched at that vertex: a weakly simple ring in which the vertex
/// appears twice, analogous to the containing case's doubled cut vertex.
/// The two instances also coincide with both shortest-path endpoints, so
/// the path step must route around the polygon rather than return the
/// empty path. Only one side can pinch: were both contacts single
/// vertices, CH(P ∪ Q) would degenerate to a segment.
fn construct_polygon_r_noncontaining<T: GeoFloat>(
    bridge_pq: &Line<T>,
    bridge_qp: &Line<T>,
    p: &Polygon<T>,
    q: &Polygon<T>,
) -> Polygon<T> {
    let mut p_ring = p.exterior().clone();
    let mut q_ring = q.exterior().clone();
    p_ring.make_ccw_winding();
    q_ring.make_ccw_winding();

    let mut coords = vec![bridge_pq.start];
    coords.extend(ring_arc_cw(&q_ring, bridge_pq.end, bridge_qp.start));
    coords.extend(ring_arc_cw(&p_ring, bridge_qp.end, bridge_pq.start));
    Polygon::new(LineString::from(coords), vec![])
}

/// The ring walked clockwise (interior kept on the right) from `from` to
/// `to`, both of which must be vertices of the closed counterclockwise
/// ring. Inclusive of both endpoints. When `from == to` the walk covers
/// the FULL ring: coinciding contacts mean the polygon touches the union
/// hull at that single vertex and hangs entirely inside the channel, so
/// its whole boundary faces the other polygon.
fn ring_arc_cw<T: GeoFloat>(
    ring_ccw: &LineString<T>,
    from: Coord<T>,
    to: Coord<T>,
) -> Vec<Coord<T>> {
    let coords = &ring_ccw.0;
    let m = coords.len() - 1; // closed ring: the last coordinate repeats the first
    let index_of = |c: Coord<T>| coords[..m].iter().position(|&v| v == c);
    let (Some(start), Some(end)) = (index_of(from), index_of(to)) else {
        // Unreachable for contacts drawn from the ring itself.
        return vec![from, to];
    };
    let mut arc = vec![coords[start]];
    let mut i = start;
    loop {
        i = if i == 0 { m - 1 } else { i - 1 };
        arc.push(coords[i]);
        if i == end {
            break;
        }
    }
    arc
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

/// The endpoints of the shortest path through R (DECOMPOSE Step 1(b)):
/// the two P contacts of the bridges in the non-containing case (the
/// paper's p_b and p_t), the two instances of the cut vertex in the
/// containing case.
fn get_path_endpoints<T: GeoFloat>(case: &SeparationCase<T>) -> (Point<T>, Point<T>) {
    match case {
        SeparationCase::NonContaining {
            bridge_pq,
            bridge_qp,
        } => (bridge_pq.start.into(), bridge_qp.end.into()),
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
    /// increasing angles with radii in [r_max / 10, r_max]. Every edge
    /// stays inside the convex angular wedge between its endpoints, so
    /// distinct edges occupy disjoint wedges and the ring is simple —
    /// but only while every angular gap is under 180 degrees. Gaps are
    /// drawn from [0.6, 1.0], so after normalisation the largest share
    /// is 1 / (1 + 2 · 0.6) of a turn, about 164 degrees. (An earlier
    /// version drew from [0.1, 1.0]; a dominant gap could then exceed
    /// 180 degrees and the chord across it left the wedge, producing
    /// self-intersecting "stars" — found when a property failure shrank
    /// to one.)
    #[hegel::composite]
    fn star_polygon(tc: &hegel::TestCase, cx: f64, cy: f64, r_max: f64) -> Polygon<f64> {
        let n = tc.draw(generators::integers::<usize>().min_value(3).max_value(12));
        let gaps: Vec<f64> = (0..n)
            .map(|_| tc.draw(generators::floats::<f64>().min_value(0.6).max_value(1.0)))
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

    /// Assume the pair into the non-containing case and return its two
    /// bridges.
    fn assume_noncontaining_bridges(
        tc: &hegel::TestCase,
        p: &Polygon<f64>,
        q: &Polygon<f64>,
    ) -> (Line<f64>, Line<f64>) {
        tc.assume(!p.exterior().intersects(q.exterior()));
        let (ch_p, ch_q, ch_union) = compute_hulls(p, q);
        tc.assume(!polygons_equal(&ch_union, &ch_p));
        tc.assume(!polygons_equal(&ch_union, &ch_q));
        common_supporting_segments(&ch_union, p, q)
            .expect("the non-containing case must yield exactly two bridges")
    }

    #[test]
    fn supporting_segments_shrink_collinear_hull_edges() {
        // The squares' top and bottom edges are collinear with the
        // supporting lines, so the mixed hull edges run corner to corner;
        // the contacts must shrink to span only the gap.
        let p = wkt! { POLYGON((0. 0.,1. 0.,1. 1.,0. 1.,0. 0.)) };
        let q = wkt! { POLYGON((2. 0.,3. 0.,3. 1.,2. 1.,2. 0.)) };
        let (_, _, ch_union) = compute_hulls(&p, &q);
        let (bridge_pq, bridge_qp) = common_supporting_segments(&ch_union, &p, &q).unwrap();
        assert_eq!(bridge_pq, Line::new((1.0, 0.0), (2.0, 0.0)));
        assert_eq!(bridge_qp, Line::new((2.0, 1.0), (1.0, 1.0)));
    }

    #[test]
    fn polygon_r_for_separated_squares() {
        let p = wkt! { POLYGON((0. 0.,1. 0.,1. 1.,0. 1.,0. 0.)) };
        let q = wkt! { POLYGON((2. 0.,3. 0.,3. 1.,2. 1.,2. 0.)) };
        let (_, _, ch_union) = compute_hulls(&p, &q);
        let (bridge_pq, bridge_qp) = common_supporting_segments(&ch_union, &p, &q).unwrap();
        let r = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        let expected = wkt! { POLYGON((1. 0.,2. 0.,2. 1.,1. 1.,1. 0.)) };
        assert!(r.relate(&expected).is_equal_topo());
    }

    #[test]
    fn polygon_r_includes_full_facing_arc() {
        // P is a triangle whose apex points at Q: its whole boundary from
        // (0, 1) through the apex to (0, 0) faces Q and belongs to R,
        // even though the apex lies strictly between the supporting
        // lines' contact points.
        let p = wkt! { POLYGON((0. 0.,1. 0.5,0. 1.,0. 0.)) };
        let q = wkt! { POLYGON((2. 0.,3. 0.,3. 1.,2. 1.,2. 0.)) };
        let (_, _, ch_union) = compute_hulls(&p, &q);
        let (bridge_pq, bridge_qp) = common_supporting_segments(&ch_union, &p, &q).unwrap();
        assert_eq!(bridge_pq, Line::new((0.0, 0.0), (2.0, 0.0)));
        assert_eq!(bridge_qp, Line::new((2.0, 1.0), (0.0, 1.0)));
        let r = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        let expected = wkt! { POLYGON((0. 0.,2. 0.,2. 1.,0. 1.,1. 0.5,0. 0.)) };
        assert!(r.relate(&expected).is_equal_topo());
    }

    #[test]
    fn polygon_r_wraps_single_contact_polygon() {
        // Shrunk from the R property harness: p touches CH(P ∪ Q) only
        // at its lowest vertex, so both bridges
        // share that P contact, p's facing arc is its full ring, and R is
        // pinched there — the channel wraps around p, with p's ring
        // traversed clockwise as the inner boundary.
        let p = wkt! { POLYGON((0.5000000000000002 0.8660254037844387,0.49999999999999956 -0.8660254037844384,1.125 -0.00000000000000003061616997868383,0.5000000000000002 0.8660254037844387)) };
        let q = wkt! { POLYGON((0.5000000000000002 1.8660254037844388,0.7499999999999998 0.5669872981077808,2.0 0.9999999999999998,0.5000000000000002 1.8660254037844388)) };
        let (a1, a2, a3) = (p.exterior().0[0], p.exterior().0[1], p.exterior().0[2]);
        let (b1, b2, b3) = (q.exterior().0[0], q.exterior().0[1], q.exterior().0[2]);

        let (_, _, ch_union) = compute_hulls(&p, &q);
        let (bridge_pq, bridge_qp) = common_supporting_segments(&ch_union, &p, &q).unwrap();
        assert_eq!(bridge_pq, Line::new(a2, b3));
        assert_eq!(bridge_qp, Line::new(b1, a2));

        let r = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        assert_eq!(r.exterior().0, vec![a2, b3, b2, b1, a2, a1, a3, a2]);
    }

    #[hegel::test]
    fn supporting_segments_support_both_polygons(tc: hegel::TestCase) {
        let p = draw_star(&tc);
        let q = draw_star(&tc);
        let (bridge_pq, bridge_qp) = assume_noncontaining_bridges(&tc, &p, &q);

        // Contacts belong to their polygons.
        assert!(p.exterior().0.contains(&bridge_pq.start));
        assert!(q.exterior().0.contains(&bridge_pq.end));
        assert!(q.exterior().0.contains(&bridge_qp.start));
        assert!(p.exterior().0.contains(&bridge_qp.end));

        // Supporting property: the bridges inherit the counterclockwise
        // hull winding, so every vertex of both polygons lies on or to
        // the left of each directed bridge.
        for bridge in [bridge_pq, bridge_qp] {
            for v in p.exterior().coords().chain(q.exterior().coords()) {
                assert_ne!(
                    <f64 as GeoNum>::Ker::orient2d(bridge.start, bridge.end, *v),
                    Orientation::Clockwise
                );
            }
        }
    }

    #[hegel::test]
    fn polygon_r_lies_between_the_polygons(tc: hegel::TestCase) {
        use crate::algorithm::Validation;
        use crate::algorithm::coordinate_position::CoordPos;
        use crate::algorithm::dimensions::Dimensions;

        let p = draw_star(&tc);
        let q = draw_star(&tc);
        let (bridge_pq, bridge_qp) = assume_noncontaining_bridges(&tc, &p, &q);
        let r = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        // A pinched R (single-vertex hull contact on one side) is weakly
        // simple: it fails OGC validity by construction and `relate` is
        // not defined for it, so restrict this property to unpinched
        // rings. The pinch structure has its own fixture:
        // `polygon_r_wraps_single_contact_polygon`.
        let interior = &r.exterior().0[..r.exterior().0.len() - 1];
        tc.assume(
            interior.len() >= 3
                && (0..interior.len()).all(|i| !interior[i + 1..].contains(&interior[i])),
        );

        assert!(r.is_valid());
        // R shares its facing arcs with the polygon boundaries but its
        // interior must not overlap either polygon's interior.
        for polygon in [&p, &q] {
            let im = r.relate(polygon);
            assert_eq!(
                im.get(CoordPos::Inside, CoordPos::Inside),
                Dimensions::Empty
            );
        }
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
