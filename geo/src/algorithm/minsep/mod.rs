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
//! Status: work in progress. The linearly separable subproblem solver
//! (`linsep`) is complete; of DECOMPOSE, case classification, the common
//! supporting segments, and the polygon R construction for the
//! non-containing case are real, while the shortest path, segment
//! extension, redundancy removal, subproblem extraction, and the
//! containing case remain placeholders backed by a brute-force fallback.
//! See `work.md` for the implementation plan.

use geo_types::Coord;

use crate::{
    GeoFloat, GeoNum, Kernel, Line, LineString, Orientation, Point, Polygon,
    algorithm::{
        convex_hull::ConvexHull,
        intersects::Intersects,
        line_intersection::line_intersection,
        line_measures::{Distance, Euclidean},
        relate::Relate,
        winding_order::Winding,
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
    match &case {
        SeparationCase::NonContaining {
            bridge_pq,
            bridge_qp,
        } => {
            // Phases 2b-6: R and its facing arcs, the shortest path,
            // the separator sequence S(P, Q), and one linearly
            // separable subproblem per separator.
            let channel = construct_polygon_r_noncontaining(bridge_pq, bridge_qp, p, q);
            let ring = channel.r.exterior();
            let path = shortest_path_in_ring(ring, channel.path_start, channel.path_end);
            let extended = extend_segments_to_boundary(&path, ring);
            let separators = remove_redundant_segments(&extended);
            let subproblems = construct_subproblems(&separators, &path, &channel);
            let sigma = subproblems
                .iter()
                .map(solve_linearly_separable_subproblem)
                .fold(T::infinity(), T::min);
            if sigma.is_finite() {
                sigma
            } else {
                // A degenerate decomposition (no separators, or every
                // subproblem empty) must not report infinity.
                linsep::separation_brute_force(p.exterior(), q.exterior())
            }
        }
        SeparationCase::Containing => {
            // Placeholder until the containing case (annulus R with the
            // doubled cut vertex) is implemented: the segment-pair
            // brute force is correct for any disjoint boundaries,
            // nested ones included.
            linsep::separation_brute_force(p.exterior(), q.exterior())
        }
    }
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
    Containing,
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
        Some(SeparationCase::Containing)
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
/// Returns R together with the ring indices of the two shortest-path
/// endpoints (the paper's p_b and p_t, the P contacts of the bridges).
/// Indices, not coordinates: in a pinched R the endpoints are the two
/// instances of the pinch vertex, which share a coordinate.
fn construct_polygon_r_noncontaining<T: GeoFloat>(
    bridge_pq: &Line<T>,
    bridge_qp: &Line<T>,
    p: &Polygon<T>,
    q: &Polygon<T>,
) -> Channel<T> {
    let mut p_ring = p.exterior().clone();
    let mut q_ring = q.exterior().clone();
    p_ring.make_ccw_winding();
    q_ring.make_ccw_winding();

    let arc_q = ring_arc_cw(&q_ring, bridge_pq.end, bridge_qp.start);
    // Built descending (p_t down to p_b); stored ascending like arc_q.
    let mut arc_p = ring_arc_cw(&p_ring, bridge_qp.end, bridge_pq.start);

    let mut coords = vec![bridge_pq.start];
    coords.extend(arc_q.iter().copied());
    // The first coordinate of the P arc is bridge_qp's P contact.
    let path_end = coords.len();
    coords.extend(arc_p.iter().copied());

    arc_p.reverse();
    Channel {
        r: Polygon::new(LineString::from(coords), vec![]),
        arc_p,
        arc_q,
        p_ring,
        q_ring,
        bridge_pq: *bridge_pq,
        bridge_qp: *bridge_qp,
        path_start: 0,
        path_end,
    }
}

/// The channel between P and Q in the non-containing case: the polygon R
/// with its structural pieces. The facing arcs are stored ascending,
/// from the bottom bridge contact (`bridge_pq`) to the top
/// (`bridge_qp`); `arc_p` runs p_b to p_t, `arc_q` runs q_b to q_t.
struct Channel<T: GeoFloat> {
    r: Polygon<T>,
    arc_p: Vec<Coord<T>>,
    arc_q: Vec<Coord<T>>,
    p_ring: LineString<T>,
    q_ring: LineString<T>,
    bridge_pq: Line<T>,
    bridge_qp: Line<T>,
    path_start: usize,
    path_end: usize,
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
/// The Euclidean shortest path (geodesic) between two ring vertices of
/// the closed, possibly pinched, ring of R (DECOMPOSE Step 1(b)),
/// returned as the sequence of ring vertices it turns at, endpoints
/// included.
///
/// Correctness-first construction, O(|R|³) overall: the geodesic in a
/// simple polygon turns only at ring vertices, so it is a shortest route
/// in the visibility graph over the ring vertices, found here with a
/// scan-based Dijkstra. The linear-time replacement (triangulate,
/// walk the dual tree, funnel algorithm) comes with the performance
/// pass.
///
/// Endpoints are ring indices, not coordinates: in a pinched R the same
/// coordinate appears twice and the path between the two instances must
/// wrap around the pinched-in polygon.
fn shortest_path_in_ring<T: GeoFloat>(
    ring: &LineString<T>,
    start: usize,
    end: usize,
) -> Vec<usize> {
    let coords = &ring.0;
    let m = coords.len() - 1;
    if start == end || m < 2 {
        return vec![start];
    }

    let mut dist = vec![T::infinity(); m];
    let mut prev = vec![usize::MAX; m];
    let mut done = vec![false; m];
    dist[start] = T::zero();
    while let Some(u) = (0..m)
        .filter(|&v| !done[v] && dist[v].is_finite())
        .min_by(|&a, &b| {
            dist[a]
                .partial_cmp(&dist[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        if u == end {
            break;
        }
        done[u] = true;
        for v in 0..m {
            if !done[v] && ring_vertices_visible(ring, u, v) {
                let step = Euclidean.distance(&Point::from(coords[u]), &Point::from(coords[v]));
                let next = dist[u] + step;
                if next < dist[v] {
                    dist[v] = next;
                    prev[v] = u;
                }
            }
        }
    }

    let mut path = vec![end];
    let mut v = end;
    while v != start {
        v = prev[v];
        if v == usize::MAX {
            // Unreachable: the ring edges alone connect every vertex.
            return vec![start, end];
        }
        path.push(v);
    }
    path.reverse();
    path
}

/// Whether ring vertices `i` and `j` see each other within the ring: the
/// open segment between them stays inside. Consecutive ring vertices are
/// always visible. A segment that touches ANY non-incident edge —
/// properly, at a grazed vertex, or along a collinear overlap — is
/// rejected; this loses no geodesic, because a grazing shot decomposes
/// into visible sub-segments through the grazed ring vertices, with the
/// same total length. A touch-free segment is either entirely inside or
/// entirely outside the ring; its midpoint decides which.
///
/// Incidence is by index, not coordinate: in a pinched ring the two
/// instances of the pinch coordinate are distinct vertices, so a shot
/// through the pinch touches the other instance's edges and is
/// rejected, and the pinch instances never see each other directly —
/// paths are forced around the pinched-in polygon.
fn ring_vertices_visible<T: GeoFloat>(ring: &LineString<T>, i: usize, j: usize) -> bool {
    let coords = &ring.0;
    let m = coords.len() - 1;
    if (i + 1) % m == j || (j + 1) % m == i {
        return true;
    }
    if coords[i] == coords[j] {
        // The two instances of a pinch vertex.
        return false;
    }
    let seg = Line::new(coords[i], coords[j]);
    let clear = (0..m).all(|e| {
        let e_next = (e + 1) % m;
        e == i
            || e == j
            || e_next == i
            || e_next == j
            || line_intersection(Line::new(coords[e], coords[e_next]), seg).is_none()
    });
    if !clear {
        return false;
    }
    let two = T::one() + T::one();
    let mid = Coord {
        x: (coords[i].x + coords[j].x) / two,
        y: (coords[i].y + coords[j].y) / two,
    };
    coord_inside_ring(ring, mid)
}

/// Even-odd point-in-ring test by horizontal ray casting. Works on
/// weakly simple (pinched) rings; points exactly on the boundary are
/// classified arbitrarily, which visibility testing tolerates because
/// boundary-touching segments are rejected before the midpoint test.
fn coord_inside_ring<T: GeoFloat>(ring: &LineString<T>, c: Coord<T>) -> bool {
    let coords = &ring.0;
    let m = coords.len() - 1;
    let mut inside = false;
    for e in 0..m {
        let a = coords[e];
        let b = coords[(e + 1) % m];
        if (a.y > c.y) != (b.y > c.y) {
            let x = a.x + (c.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if c.x < x {
                inside = !inside;
            }
        }
    }
    inside
}

/// Extend each geodesic segment along its own line to the boundary of R,
/// in both directions (DECOMPOSE Step 1(c)), by linear scan over the ring
/// edges. A path vertex is extended only when the ray beyond it continues
/// through R's interior (the vertex is reflex with respect to the ray):
/// between the vertex and the first ray hit the line crosses no boundary,
/// so the midpoint of that span decides inside or outside, as in the
/// visibility test. Beyond a convex corner there is no extension.
///
/// The extended segments keep the path's orientation: `start` lies on the
/// side of the path's beginning, which Step 2 uses as the paper's b_i/t_i
/// vertical orientation (b_i nearest l_{i-1}, t_i nearest l_{i+1}).
fn extend_segments_to_boundary<T: GeoFloat>(
    path: &[usize],
    ring: &LineString<T>,
) -> Vec<ExtendedSegment<T>> {
    path.windows(2)
        .enumerate()
        .map(|(window, w)| {
            let (a, b) = (ring.0[w[0]], ring.0[w[1]]);
            let backward = extend_ray(ring, b, a - b, w[0]);
            let forward = extend_ray(ring, a, b - a, w[1]);
            ExtendedSegment {
                line: Line::new(
                    backward.map_or(a, |(c, _)| c),
                    forward.map_or(b, |(c, _)| c),
                ),
                window,
                back_edge: backward.map(|(_, e)| e),
                fwd_edge: forward.map(|(_, e)| e),
            }
        })
        .collect()
}

/// One extended geodesic segment with the provenance the subproblem
/// construction needs: the path window it extends and, per direction,
/// the ring edge the extension hit (`None` when that endpoint is the
/// path vertex itself). Provenance is exact where the coordinates are
/// not: extension endpoints are computed hits that sit on their edge
/// only within rounding.
#[derive(Clone, Copy, Debug)]
struct ExtendedSegment<T: GeoFloat> {
    line: Line<T>,
    window: usize,
    back_edge: Option<usize>,
    fwd_edge: Option<usize>,
}

/// The first boundary hit of the ray `origin + t * dir` with t > 1 and
/// the ring edge it lies on, provided the span from `origin + dir` to
/// the hit runs through R's interior; `None` when the ray leaves R
/// immediately (a convex corner) or hits nothing beyond the segment
/// end.
///
/// `through` is the ring index of the vertex the ray passes at t = 1;
/// its incident edges are excluded from the scan. They end at that
/// vertex, so they can never be the true first hit beyond it, but
/// rounding can report one at t just above 1 — and such a spurious hit,
/// being minimal, masks the real extension through a reflex vertex,
/// leaving a gap in the separator sequence that nearby pairs slip
/// through (found by the σ property harness).
fn extend_ray<T: GeoFloat>(
    ring: &LineString<T>,
    origin: Coord<T>,
    dir: Coord<T>,
    through: usize,
) -> Option<(Coord<T>, usize)> {
    let coords = &ring.0;
    let m = coords.len() - 1;
    let cross = |a: Coord<T>, b: Coord<T>| a.x * b.y - a.y * b.x;

    let mut best: Option<(T, usize)> = None;
    for e in 0..m {
        if e == through || (e + 1) % m == through {
            continue;
        }
        let (ea, eb) = (coords[e], coords[(e + 1) % m]);
        let edge_delta = eb - ea;
        let denom = cross(dir, edge_delta);
        if denom == T::zero() {
            continue;
        }
        let t = cross(ea - origin, edge_delta) / denom;
        let s = cross(ea - origin, dir) / denom;
        if t > T::one() && s >= T::zero() && s <= T::one() && best.is_none_or(|(bt, _)| t < bt) {
            best = Some((t, e));
        }
    }
    let (t, edge) = best?;
    let hit = origin + dir * t;
    let two = T::one() + T::one();
    let segment_end = origin + dir;
    let mid = Coord {
        x: (segment_end.x + hit.x) / two,
        y: (segment_end.y + hit.y) / two,
    };
    coord_inside_ring(ring, mid).then_some((hit, edge))
}

/// Remove redundant segments from the extended shortest path (DECOMPOSE
/// Step 1(d)): keep l_0, then repeatedly keep the maximal-indexed segment
/// that intersects the previously kept one. Progress is guaranteed by
/// falling back to the next segment: consecutive extended segments
/// contain the original path segments, which share a path vertex — in
/// exact arithmetic they always intersect, but the computed extension
/// endpoints round, and two near-collinear extensions can end up
/// disjoint by an ulp.
fn remove_redundant_segments<T: GeoFloat>(
    segments: &[ExtendedSegment<T>],
) -> Vec<ExtendedSegment<T>> {
    let Some(&first) = segments.first() else {
        return Vec::new();
    };
    let mut kept = vec![first];
    let mut current = 0;
    while current + 1 < segments.len() {
        let last = kept.last().expect("kept starts non-empty").line;
        let next = (current + 1..segments.len())
            .rev()
            .find(|&j| segments[j].line.intersects(&last))
            .unwrap_or(current + 1);
        kept.push(segments[next]);
        current = next;
    }
    kept
}

#[derive(Debug, Clone)]
struct Subproblem<T: GeoFloat> {
    p_chain: LineString<T>,
    q_chain: LineString<T>,
    separator: Line<T>,
}

/// A located cut: its lexicographic position along a facing arc and its
/// coordinate.
type ArcCut<T> = Option<((usize, T), Coord<T>)>;

/// Which piece of R's boundary a separator endpoint lies on.
#[derive(PartialEq, Clone, Copy)]
enum RSide {
    P,
    Q,
    Bridge,
}

/// DECOMPOSE Step 2: one linearly separable subproblem per separator.
///
/// Every separator is a chord of R, so its intersections with each
/// polygon lie on that polygon's facing arc; each subchain is the slice
/// of the facing arc between two cut positions. The paper phrases the
/// slice as a fixed-direction scan of the whole polygon, which is the
/// same thing when consecutive cuts ascend, but a later separator can
/// cut a polygon BELOW an earlier one (the extension of a geodesic
/// segment can land anywhere on the opposite wall), and the facing-arc
/// slice between positions is what covers Lemma 1's pair assignment in
/// both situations.
///
/// The end cut keeps the paper's rule only where its shielding argument
/// is airtight: separator i's subchain of X ends at i's own cut when
/// the separator's top endpoint t_i lies on X (the separator terminates
/// into that wall and seals it) or i is the last index. The paper's
/// "otherwise" branch ends at separator i+1's cut instead; that
/// truncation is unsound when an extension lands on a bridge — the
/// separator then misses one polygon entirely, its invariant "each l_i
/// intersects both P and Q" fails, and a neighbour cut can exclude the
/// realising vertex (found by the σ property harness on two triangles
/// with a diagonal channel axis). This implementation widens the
/// otherwise-branch to the arc's top sentinel: always enlarging, which
/// cannot undercut σ and only widens the covered pair set.
///
/// Separator orientation is path order (start nearest the path's
/// beginning), not the paper's "endpoint closest to l_{i-1}": the two
/// disagree when a backward extension overshoots, and path order is
/// what makes the last separator's cut land at the arc top, as the
/// paper's sentinel definitions assume.
fn construct_subproblems<T: GeoFloat>(
    separators: &[ExtendedSegment<T>],
    path: &[usize],
    channel: &Channel<T>,
) -> Vec<Subproblem<T>> {
    let m = separators.len();
    let ring = &channel.r.exterior().0;

    // A pinched channel is topologically the containing case: the path
    // wraps around the pinched-in polygon and the separator sequence is
    // cyclic (the paper switches to modulo-m arithmetic), so the linear
    // bottom-to-top subchain rules below do not apply — in particular
    // the last separator's cut does not seal the arc top, which wraps
    // back to the bottom bridge. Until the containing-case machinery
    // lands, use full-ring subchains: always a sound superset, at the
    // cost of the subchain brute force when validation fails.
    let pinched = channel.bridge_pq.start == channel.bridge_qp.end
        || channel.bridge_pq.end == channel.bridge_qp.start;
    if pinched {
        return separators
            .iter()
            .map(|s| Subproblem {
                p_chain: channel.p_ring.clone(),
                q_chain: channel.q_ring.clone(),
                separator: s.line,
            })
            .collect();
    }

    // Exact side classification from the ring layout: edge 0 is the
    // bottom bridge, edges 1..path_end-1 are the Q facing arc, edge
    // path_end-1 is the top bridge, and edges from path_end on are the
    // P facing arc; a bare ring vertex belongs to P at 0 and beyond
    // path_end (the bridge contacts are arc endpoints of P and Q
    // respectively), to Q in between.
    let side_of_edge = |e: usize| -> RSide {
        if e == 0 || e + 1 == channel.path_end {
            RSide::Bridge
        } else if e < channel.path_end {
            RSide::Q
        } else {
            RSide::P
        }
    };
    let side_of_vertex = |v: usize| -> RSide {
        if v == 0 || v >= channel.path_end {
            RSide::P
        } else {
            RSide::Q
        }
    };
    let bottoms: Vec<RSide> = separators
        .iter()
        .map(|s| {
            s.back_edge
                .map(side_of_edge)
                .unwrap_or_else(|| side_of_vertex(path[s.window]))
        })
        .collect();
    let tops: Vec<RSide> = separators
        .iter()
        .map(|s| {
            s.fwd_edge
                .map(side_of_edge)
                .unwrap_or_else(|| side_of_vertex(path[s.window + 1]))
        })
        .collect();

    // Cut candidates beyond the exact segment intersections: a
    // separator's endpoints were computed as hits ON the ring, and it
    // passes through its window's two path vertices — but only in exact
    // arithmetic, so the robust segment-segment test can miss these
    // touches by an ulp (found by the σ property harness: a missed
    // endpoint cut made the resolution fall through to a much lower
    // neighbouring cut and truncate the realising pair away). These
    // coordinates are known exactly, so they join the candidate set
    // directly, filtered by which side of R they lie on.
    // Cut candidates beyond the exact segment intersections, assigned
    // by exact provenance (found necessary twice by the σ property
    // harness: computed extension endpoints miss the exact intersection
    // test by an ulp, and a distance-based side heuristic once assigned
    // a mid-bridge endpoint to an arc, inserting a fake off-boundary
    // vertex into a subchain — an underestimate, the one failure class
    // the design must exclude). A separator endpoint joins an arc's
    // candidates when its hit edge lies on that arc; a path vertex when
    // its ring index does. A path vertex that doubles as a bridge
    // contact belongs to BOTH arcs' endpoints, so vertex classification
    // maps the contacts to their arcs, never to the bridge.
    let cuts = |arc: &[Coord<T>], own_side: RSide| -> Vec<ArcCut<T>> {
        separators
            .iter()
            .map(|s| {
                let mut extras: Vec<Coord<T>> = Vec::new();
                if s.back_edge.map(side_of_edge) == Some(own_side) {
                    extras.push(s.line.start);
                }
                if s.fwd_edge.map(side_of_edge) == Some(own_side) {
                    extras.push(s.line.end);
                }
                for &v in &[path[s.window], path[s.window + 1]] {
                    let vertex_side = side_of_vertex(v);
                    // Bridge contacts sit on both arcs.
                    let on_arc = vertex_side == own_side
                        || v == 0
                        || v == 1
                        || v + 1 == channel.path_end
                        || v == channel.path_end;
                    if on_arc && arc.contains(&ring[v]) {
                        extras.push(ring[v]);
                    }
                }
                highest_intersection(&s.line, arc, &extras).map(|c| (arc_position(arc, c), c))
            })
            .collect()
    };
    let p_cuts = cuts(&channel.arc_p, RSide::P);
    let q_cuts = cuts(&channel.arc_q, RSide::Q);

    let subchain = |arc: &[Coord<T>],
                    arc_cuts: &[ArcCut<T>],
                    full_ring: &LineString<T>,
                    own_side: RSide,
                    i: usize|
     -> LineString<T> {
        let sentinel_start = ((0, T::zero()), arc[0]);
        let sentinel_end = ((arc.len().saturating_sub(2), T::one()), arc[arc.len() - 1]);
        // A cut truncates a subchain only where the shielding is
        // airtight: the end stops at separator i's own cut exactly when
        // the separator's top endpoint lands on this polygon (it
        // terminates into that wall and seals it above the cut); the
        // start begins at separator i-1's cut exactly when i-1's bottom
        // endpoint lands on this polygon. In every other situation the
        // sentinel (the whole facing arc on that side) is the sound
        // choice: a separator anchored elsewhere — on the other
        // polygon, a bridge, or (its last incarnation) on this side's
        // own arc while merely grazing the other's vertex mid-flight —
        // shields nothing here. The paper's "i = m-1 ends at its own
        // cut" shortcut is unsound in that grazing configuration and is
        // deliberately not implemented (found by the σ property
        // harness). When the endpoint condition holds, the cut provably
        // exists: the endpoint itself is one of its candidates.
        let start = if i > 0 && bottoms[i - 1] == own_side {
            arc_cuts[i - 1].unwrap_or(sentinel_start)
        } else {
            sentinel_start
        };
        let end = if tops[i] == own_side {
            arc_cuts[i].unwrap_or(sentinel_end)
        } else {
            sentinel_end
        };
        // Wrap detection: the paper's subchain is a fixed-direction scan
        // of the WHOLE ring, which can wrap around the polygon's back
        // when a later separator cuts below an earlier one. An arc slice
        // cannot represent a wrap; the signal is one of the paper's
        // immediate anchor cuts (i-1 or i) falling outside the computed
        // slice. The full ring is the always-sound superset there.
        let (lo, hi) = if pos_le(start.0, end.0) {
            (start.0, end.0)
        } else {
            (end.0, start.0)
        };
        let wraps = [i.checked_sub(1), Some(i)]
            .into_iter()
            .flatten()
            .filter_map(|j| arc_cuts[j])
            .any(|(pos, _)| !pos_le(lo, pos) || !pos_le(pos, hi));
        if wraps {
            return full_ring.clone();
        }
        arc_slice(arc, start, end)
    };

    (0..m)
        .map(|i| Subproblem {
            p_chain: subchain(&channel.arc_p, &p_cuts, &channel.p_ring, RSide::P, i),
            q_chain: subchain(&channel.arc_q, &q_cuts, &channel.q_ring, RSide::Q, i),
            separator: separators[i].line,
        })
        .collect()
}

/// Lexicographic order on arc positions.
fn pos_le<T: GeoFloat>(a: (usize, T), b: (usize, T)) -> bool {
    a.0 < b.0 || (a.0 == b.0 && a.1 <= b.1)
}

/// The intersection point of `separator` with the open polyline that is
/// furthest along the separator's own orientation (the paper's p_i+ and
/// q_i+, with b_i at `separator.start`). A collinear overlap contributes
/// both overlap endpoints. `None` when they do not touch.
fn highest_intersection<T: GeoFloat>(
    separator: &Line<T>,
    arc: &[Coord<T>],
    extras: &[Coord<T>],
) -> Option<Coord<T>> {
    use crate::algorithm::line_intersection::LineIntersection;
    let d = separator.delta();
    let mut best: Option<(T, Coord<T>)> = None;
    let mut consider = |c: Coord<T>| {
        let h = (c.x - separator.start.x) * d.x + (c.y - separator.start.y) * d.y;
        if best.is_none_or(|(bh, _)| h > bh) {
            best = Some((h, c));
        }
    };
    for &c in extras {
        consider(c);
    }
    for w in arc.windows(2) {
        match line_intersection(*separator, Line::new(w[0], w[1])) {
            Some(LineIntersection::SinglePoint { intersection, .. }) => consider(intersection),
            Some(LineIntersection::Collinear { intersection }) => {
                consider(intersection.start);
                consider(intersection.end);
            }
            None => {}
        }
    }
    best.map(|(_, c)| c)
}

/// The position of the point of `arc` nearest to `c`: segment index and
/// clamped parameter within that segment, ordered lexicographically
/// along the arc. Cut coordinates are computed intersections, so they
/// sit on their segment only within rounding; nearest-segment location
/// absorbs that.
fn arc_position<T: GeoFloat>(arc: &[Coord<T>], c: Coord<T>) -> (usize, T) {
    let mut best = (0, T::zero());
    let mut best_d = T::infinity();
    for (e, w) in arc.windows(2).enumerate() {
        let d = w[1] - w[0];
        let len2 = d.x * d.x + d.y * d.y;
        let t = if len2 == T::zero() {
            T::zero()
        } else {
            (((c.x - w[0].x) * d.x + (c.y - w[0].y) * d.y) / len2)
                .max(T::zero())
                .min(T::one())
        };
        let proj = Coord {
            x: w[0].x + d.x * t,
            y: w[0].y + d.y * t,
        };
        let dist = Euclidean.distance(&Point::from(c), &Point::from(proj));
        if dist < best_d {
            best_d = dist;
            best = (e, t);
        }
    }
    best
}

/// The sub-polyline of `arc` between two located cuts, endpoints
/// included, running from the earlier position to the later one (the
/// orientation of a subchain does not matter to the subproblem solver).
fn arc_slice<T: GeoFloat>(
    arc: &[Coord<T>],
    a: ((usize, T), Coord<T>),
    b: ((usize, T), Coord<T>),
) -> LineString<T> {
    let (first, second) = if pos_le(a.0, b.0) { (a, b) } else { (b, a) };
    let ((e1, _), c1) = first;
    let ((e2, _), c2) = second;

    let mut coords = vec![c1];
    coords.extend_from_slice(&arc[(e1 + 1)..=e2]);
    coords.push(c2);
    coords.dedup();
    LineString::from(coords)
}

/// Solve one subproblem with the LinSep solver, trying the separator in
/// both orientations (the extended segments carry the path's direction,
/// not sidedness). When neither validates — a knife-edge cut can land a
/// hair beyond its separator's line — fall back to the brute force on
/// the SUBCHAINS: it cannot undercut σ (all values are boundary-point
/// distances) and preserves the subproblem's assigned pair coverage.
fn solve_linearly_separable_subproblem<T: GeoFloat>(subproblem: &Subproblem<T>) -> T {
    let separator = subproblem.separator;
    let reversed = Line::new(separator.end, separator.start);
    for candidate in [separator, reversed] {
        if let Some(chains) =
            linsep::SeparatedChains::new(&subproblem.p_chain, &subproblem.q_chain, candidate)
        {
            return chains.separation();
        }
    }
    linsep::separation_brute_force(&subproblem.p_chain, &subproblem.q_chain)
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
        let r = &construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q).r;
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
        let r = &construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q).r;
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

        let r = &construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q).r;
        assert_eq!(r.exterior().0, vec![a2, b3, b2, b1, a2, a1, a3, a2]);
    }

    #[test]
    fn shortest_path_direct_in_convex_channel() {
        let p = wkt! { POLYGON((0. 0.,1. 0.,1. 1.,0. 1.,0. 0.)) };
        let q = wkt! { POLYGON((2. 0.,3. 0.,3. 1.,2. 1.,2. 0.)) };
        let (_, _, ch_union) = compute_hulls(&p, &q);
        let (bridge_pq, bridge_qp) = common_supporting_segments(&ch_union, &p, &q).unwrap();
        let channel = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        let (r, start, end) = (&channel.r, channel.path_start, channel.path_end);
        let path: Vec<Coord<f64>> = shortest_path_in_ring(r.exterior(), start, end)
            .into_iter()
            .map(|i| r.exterior().0[i])
            .collect();
        assert_eq!(
            path,
            vec![Coord { x: 1.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 }]
        );
    }

    #[test]
    fn shortest_path_bends_around_spike() {
        // P carries a spike whose apex (1.8, 1) pokes into the channel:
        // the straight route from (1, 0) to (1, 2) is blocked (it grazes
        // the spike's base vertices and runs through P's interior), so
        // the geodesic bends around the apex.
        let p = wkt! { POLYGON((0. 0.,1. 0.,1. 0.4,1.8 1.,1. 1.6,1. 2.,0. 2.,0. 0.)) };
        let q = wkt! { POLYGON((2. 0.,3. 0.,3. 2.,2. 2.,2. 0.)) };
        let (_, _, ch_union) = compute_hulls(&p, &q);
        let (bridge_pq, bridge_qp) = common_supporting_segments(&ch_union, &p, &q).unwrap();
        let channel = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        let (r, start, end) = (&channel.r, channel.path_start, channel.path_end);
        let path: Vec<Coord<f64>> = shortest_path_in_ring(r.exterior(), start, end)
            .into_iter()
            .map(|i| r.exterior().0[i])
            .collect();
        assert_eq!(
            path,
            vec![
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.8, y: 1.0 },
                Coord { x: 1.0, y: 2.0 },
            ]
        );
    }

    #[test]
    fn shortest_path_wraps_pinched_polygon() {
        // Same configuration as polygon_r_wraps_single_contact_polygon:
        // both path endpoints are the two instances of the pinch vertex,
        // and the geodesic must wrap around the pinched-in triangle p —
        // for a convex p, hugging its boundary exactly.
        let p = wkt! { POLYGON((0.5000000000000002 0.8660254037844387,0.49999999999999956 -0.8660254037844384,1.125 -0.00000000000000003061616997868383,0.5000000000000002 0.8660254037844387)) };
        let q = wkt! { POLYGON((0.5000000000000002 1.8660254037844388,0.7499999999999998 0.5669872981077808,2.0 0.9999999999999998,0.5000000000000002 1.8660254037844388)) };
        let (a1, a2, a3) = (p.exterior().0[0], p.exterior().0[1], p.exterior().0[2]);
        let (_, _, ch_union) = compute_hulls(&p, &q);
        let (bridge_pq, bridge_qp) = common_supporting_segments(&ch_union, &p, &q).unwrap();
        let channel = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        let (r, start, end) = (&channel.r, channel.path_start, channel.path_end);
        assert_eq!(r.exterior().0[start], r.exterior().0[end]);
        let path: Vec<Coord<f64>> = shortest_path_in_ring(r.exterior(), start, end)
            .into_iter()
            .map(|i| r.exterior().0[i])
            .collect();
        assert_eq!(path, vec![a2, a3, a1, a2]);
    }

    #[hegel::test]
    fn shortest_path_stays_inside_r_and_beats_boundary_walks(tc: hegel::TestCase) {
        let p = draw_star(&tc);
        let q = draw_star(&tc);
        let (bridge_pq, bridge_qp) = assume_noncontaining_bridges(&tc, &p, &q);
        let channel = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        let (r, start, end) = (&channel.r, channel.path_start, channel.path_end);
        let ring = r.exterior();
        let path_idx = shortest_path_in_ring(ring, start, end);
        let path: Vec<Coord<f64>> = path_idx.iter().map(|&i| ring.0[i]).collect();

        assert_eq!(path.first(), Some(&ring.0[start]));
        assert_eq!(path.last(), Some(&ring.0[end]));

        // No path segment may properly cross the ring.
        for w in path.windows(2) {
            let seg = Line::new(w[0], w[1]);
            for edge in ring.lines() {
                assert!(!line_intersection(edge, seg).is_some_and(|i| i.is_proper()));
            }
        }

        // Optimality upper bound: the geodesic is no longer than either
        // walk along the ring between the endpoints.
        let length = |coords: &[Coord<f64>]| {
            coords
                .windows(2)
                .map(|w| Euclidean.distance(&Point::from(w[0]), &Point::from(w[1])))
                .fold(0.0, |acc, d| acc + d)
        };
        let m = ring.0.len() - 1;
        let mut forward = vec![];
        let mut i = start;
        loop {
            forward.push(ring.0[i]);
            if i == end {
                break;
            }
            i = (i + 1) % m;
        }
        let mut backward = vec![];
        let mut i = start;
        loop {
            backward.push(ring.0[i]);
            if i == end {
                break;
            }
            i = if i == 0 { m - 1 } else { i - 1 };
        }
        let path_len = length(&path);
        let bound = length(&forward).min(length(&backward));
        assert!(path_len <= bound * (1.0 + 1e-12));
    }

    #[test]
    fn extension_stops_at_convex_corners() {
        // The single path segment runs along R's left edge between two
        // convex corners of R: no extension in either direction.
        let p = wkt! { POLYGON((0. 0.,1. 0.,1. 1.,0. 1.,0. 0.)) };
        let q = wkt! { POLYGON((2. 0.,3. 0.,3. 1.,2. 1.,2. 0.)) };
        let (_, _, ch_union) = compute_hulls(&p, &q);
        let (bridge_pq, bridge_qp) = common_supporting_segments(&ch_union, &p, &q).unwrap();
        let channel = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        let (r, start, end) = (&channel.r, channel.path_start, channel.path_end);
        let path = shortest_path_in_ring(r.exterior(), start, end);
        let extended = extend_segments_to_boundary(&path, r.exterior());
        assert_eq!(extended.len(), 1);
        assert_eq!(extended[0].line, Line::new((1.0, 0.0), (1.0, 1.0)));
        let separators = remove_redundant_segments(&extended);
        assert_eq!(separators.len(), 1);
        assert_eq!(separators[0].line, Line::new((1.0, 0.0), (1.0, 1.0)));
    }

    #[test]
    fn extension_continues_through_reflex_apex() {
        // The geodesic bends at the spike apex (1.8, 1), which is reflex
        // in R: each segment extends through it to Q's facing edge at
        // x = 2, and the two extended separators cross, so both survive
        // redundancy removal.
        let p = wkt! { POLYGON((0. 0.,1. 0.,1. 0.4,1.8 1.,1. 1.6,1. 2.,0. 2.,0. 0.)) };
        let q = wkt! { POLYGON((2. 0.,3. 0.,3. 2.,2. 2.,2. 0.)) };
        let (_, _, ch_union) = compute_hulls(&p, &q);
        let (bridge_pq, bridge_qp) = common_supporting_segments(&ch_union, &p, &q).unwrap();
        let channel = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        let (r, start, end) = (&channel.r, channel.path_start, channel.path_end);
        let path = shortest_path_in_ring(r.exterior(), start, end);
        let extended = extend_segments_to_boundary(&path, r.exterior());
        let lines: Vec<Line<f64>> = extended.iter().map(|s| s.line).collect();
        assert_eq!(
            lines,
            vec![
                Line::new((1.0, 0.0), (2.0, 1.25)),
                Line::new((2.0, 0.75), (1.0, 2.0)),
            ]
        );
        let separators = remove_redundant_segments(&extended);
        assert_eq!(separators.len(), 2);
    }

    #[hegel::test]
    fn extended_separators_are_chords_of_r(tc: hegel::TestCase) {
        let p = draw_star(&tc);
        let q = draw_star(&tc);
        let (bridge_pq, bridge_qp) = assume_noncontaining_bridges(&tc, &p, &q);
        let channel = construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q);
        let (r, start, end) = (&channel.r, channel.path_start, channel.path_end);
        let ring = r.exterior();
        let path = shortest_path_in_ring(ring, start, end);
        let extended = extend_segments_to_boundary(&path, ring);
        let separators = remove_redundant_segments(&extended);

        for (seg, w) in extended.iter().zip(path.windows(2)) {
            let segment = seg.line;
            // Extension contains the original segment, in orientation.
            let along = |c: Coord<f64>| {
                (c.x - segment.start.x) * (segment.end.x - segment.start.x)
                    + (c.y - segment.start.y) * (segment.end.y - segment.start.y)
            };
            assert!(along(ring.0[w[0]]) <= along(ring.0[w[1]]));
            // Endpoints lie on R's boundary. The hit point inherits the
            // ray-edge intersection's conditioning (near-parallel hits
            // amplify rounding), so this is a smoke bound: a genuinely
            // wrong hit is off by a feature-sized amount, not 1e-9.
            for endpoint in [segment.start, segment.end] {
                let d = Euclidean.distance(&Point::from(endpoint), ring);
                assert!(d <= 1e-10 * 16.0, "endpoint off-ring by {d:e}");
            }
        }

        // The kept separators form an intersecting chain — up to
        // rounding: consecutive extended segments share a path vertex in
        // exact arithmetic, but the computed extension endpoints round,
        // so near-collinear neighbours may only touch within an ulp.
        for pair in separators.windows(2) {
            let gap = Euclidean.distance(&pair[0].line, &pair[1].line);
            assert!(
                gap <= 16.0 * f64::EPSILON * 16.0,
                "kept separators disconnected by {gap:e}"
            );
        }
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
        let r = &construct_polygon_r_noncontaining(&bridge_pq, &bridge_qp, &p, &q).r;
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
