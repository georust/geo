//! Solver for the linearly separable subproblem (Amato 1994, Section 4).
//!
//! Input: two polygonal chains P and Q separated by a line l; vertices and
//! edges of either chain may lie on l. Output: σ(P, Q), the minimum distance
//! between the chains. The full algorithm runs in O(|P| + |Q|) time:
//!
//! 1. Prune vertices that are not perpendicularly visible from l, and
//!    vertices with visible angles below 90 degrees (Lemmas 2 and 3). The
//!    surviving vertices are monotone with respect to l.
//! 2. Construct visible wedges and feasible regions for the surviving
//!    vertices (closest visible vertex, CVV) and for partitioned edges
//!    (closest vertex-edge, CVE).
//! 3. Search the resulting totally monotone matrix for its minimum entry.
//!
//! σ(P, Q) = min(cvv(P, Q), cve(P, Q), cve(Q, P)).
//!
//! Status: `separation` currently delegates to the O(|P| · |Q|) brute force;
//! the steps above land incrementally, each property-tested against it.

use crate::algorithm::line_intersection::line_intersection;
use crate::algorithm::line_measures::{Distance, Euclidean};
use crate::{GeoFloat, GeoNum, Kernel, Line, LineString, Orientation};
use geo_types::Coord;

/// Two polygonal chains separated by a line.
///
/// `p` lies on or to the left of the directed separator line, `q` on or to
/// its right.
#[derive(Debug)]
pub(super) struct SeparatedChains<'a, T: GeoFloat> {
    p: &'a LineString<T>,
    q: &'a LineString<T>,
    // Read once pruning lands (work.md step 3).
    #[allow(dead_code)]
    separator: Line<T>,
}

impl<'a, T: GeoFloat> SeparatedChains<'a, T> {
    /// Validate that `separator` separates the chains: no vertex of `p` may
    /// lie strictly to the right of the directed line, no vertex of `q`
    /// strictly to its left; vertices on the line are permitted. Returns
    /// `None` when the separator is degenerate, a chain has fewer than two
    /// vertices, or a vertex lies on the wrong side.
    pub(super) fn new(
        p: &'a LineString<T>,
        q: &'a LineString<T>,
        separator: Line<T>,
    ) -> Option<Self> {
        if separator.start == separator.end || p.0.len() < 2 || q.0.len() < 2 {
            return None;
        }
        let never = |chain: &LineString<T>, forbidden: Orientation| {
            chain.coords().all(|c| {
                <T as GeoNum>::Ker::orient2d(separator.start, separator.end, *c) != forbidden
            })
        };
        (never(p, Orientation::Clockwise) && never(q, Orientation::CounterClockwise))
            .then_some(Self { p, q, separator })
    }

    /// The separation σ(P, Q) between the two chains.
    pub(super) fn separation(&self) -> T {
        // Delegates to the brute force until the LinSep solver replaces it.
        separation_brute_force(self.p, self.q)
    }
}

/// O(|P| · |Q|) reference: minimum distance over all boundary segment pairs.
/// This is the test oracle for the solver and the benchmark baseline.
///
/// The fold over segment pairs is deliberate: the LineString-LineString
/// `Euclidean.distance` takes a project-and-prune fast path that, as of
/// PR #1560, can return an overestimate (fix on the branch
/// `fix-separable-fast-path-prefix-prune`). An oracle must not share code
/// with an optimised path.
pub(super) fn separation_brute_force<T: GeoFloat>(p: &LineString<T>, q: &LineString<T>) -> T {
    p.lines()
        .flat_map(|lp| q.lines().map(move |lq| Euclidean.distance(&lp, &lq)))
        .fold(T::infinity(), T::min)
}

/// The foot of the perpendicular from `v` to the infinite line through
/// `separator`.
fn perpendicular_foot<T: GeoFloat>(separator: &Line<T>, v: Coord<T>) -> Coord<T> {
    let d = separator.delta();
    let len2 = d.x * d.x + d.y * d.y;
    let t = ((v.x - separator.start.x) * d.x + (v.y - separator.start.y) * d.y) / len2;
    Coord {
        x: separator.start.x + d.x * t,
        y: separator.start.y + d.y * t,
    }
}

/// The visible wedge of a candidate vertex: the cone of directions from the
/// apex bounded by the tangent rays to the candidate vertices above and
/// below it, heights measured along the separator direction. This is the
/// operational form of the paper's W(p): the successive-convex-hull
/// construction of its Section 4.3 computes exactly these tangents. The
/// chain must lie on or to the left of the directed separator; compute
/// wedges for the right-hand chain with the separator reversed.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // read by the solver steps as they land (work.md steps 4-6)
pub(super) struct VisibleWedge<T: GeoFloat> {
    /// Direction of the upper boundary ray; parallel to the separator when
    /// no candidate vertex lies above the apex.
    upper: Coord<T>,
    /// Direction of the lower boundary ray; anti-parallel to the separator
    /// when no candidate vertex lies below the apex.
    lower: Coord<T>,
    /// True when another candidate sits at the apex height on the
    /// separator side: the apex is hidden and can never be a candidate.
    degenerate: bool,
}

impl<T: GeoFloat> VisibleWedge<T> {
    /// Whether the visible angle α is at least 90 degrees (Lemma 3 keeps
    /// only such vertices). The angle is measured counterclockwise from the
    /// lower to the upper boundary through the toward-separator direction,
    /// capped at 180 degrees; it falls below 90 exactly when the two
    /// boundary directions span less than a quarter turn.
    #[allow(dead_code)] // called by the solver steps as they land (work.md steps 4-6)
    pub(super) fn alpha_at_least_90(&self) -> bool {
        if self.degenerate {
            return false;
        }
        let cross = self.lower.x * self.upper.y - self.lower.y * self.upper.x;
        let dot = self.lower.x * self.upper.x + self.lower.y * self.upper.y;
        !(cross >= T::zero() && dot > T::zero())
    }
}

/// Compute the visible wedge of `candidates[idx]` against the other
/// candidate vertices. O(n) per vertex; the paper's successive convex
/// hulls achieve O(n) for a whole monotone chain.
#[allow(dead_code)] // called by the solver steps as they land (work.md steps 4-6)
pub(super) fn visible_wedge<T: GeoFloat>(
    candidates: &[Coord<T>],
    idx: usize,
    separator: &Line<T>,
) -> VisibleWedge<T> {
    let apex = candidates[idx];
    let d = separator.delta();
    // Right normal of the separator direction: points from the chain's
    // side toward the line.
    let toward = Coord { x: d.y, y: -d.x };

    let cross = |a: Coord<T>, b: Coord<T>| a.x * b.y - a.y * b.x;
    let dot = |a: Coord<T>, b: Coord<T>| a.x * b.x + a.y * b.y;

    // Upper tangent: the direction with the smallest counterclockwise
    // angle from `toward` among vertices above the apex. Lower tangent:
    // the largest (least negative) clockwise angle among vertices below.
    let mut upper: Option<Coord<T>> = None;
    let mut lower: Option<Coord<T>> = None;
    let mut degenerate = false;

    for (j, &c) in candidates.iter().enumerate() {
        if j == idx || c == apex {
            continue;
        }
        let dir = c - apex;
        // Height of the candidate relative to the apex, measured along the
        // separator direction. Computed from the difference vector: an
        // absolute height (c - separator.start) · d absorbs coordinates
        // that are small relative to the separator's start point.
        let h = dot(dir, d);
        if h > T::zero() {
            upper = Some(match upper {
                // A more clockwise direction bounds the wedge more tightly.
                Some(best) if cross(dir, best) <= T::zero() => best,
                _ => dir,
            });
        } else if h < T::zero() {
            lower = Some(match lower {
                // A more counterclockwise direction bounds more tightly.
                Some(best) if cross(dir, best) >= T::zero() => best,
                _ => dir,
            });
        } else if dot(dir, toward) > T::zero() {
            // A candidate at the apex height between the apex and the
            // separator hides the apex completely.
            degenerate = true;
        }
    }

    VisibleWedge {
        upper: upper.unwrap_or(d),
        lower: lower.unwrap_or(Coord::zero() - d),
        degenerate,
    }
}

/// Indices of the vertices of `chain` that are perpendicularly visible from
/// the separator line: the segment from the vertex to its perpendicular foot
/// on the line does not properly intersect the chain (Step 1 of
/// LinSep-CVV). Only proper crossings block: endpoint contact, grazing a
/// vertex, and collinear overlap all leave the vertex visible, matching the
/// paper's relaxed visibility (a chain running along the sight segment does
/// not hide it). A vertex on the line is trivially visible.
///
/// O(n²): each sight segment is tested against every chain edge. The paper
/// achieves this step in O(n) with a linear scan; optimise once the solver
/// is complete.
#[allow(dead_code)] // called by the solver steps as they land (work.md steps 4-6)
pub(super) fn perpendicularly_visible<T: GeoFloat>(
    chain: &LineString<T>,
    separator: &Line<T>,
) -> Vec<usize> {
    (0..chain.0.len())
        .filter(|&i| {
            let v = chain.0[i];
            let foot = perpendicular_foot(separator, v);
            if foot == v {
                return true;
            }
            let sight = Line::new(v, foot);
            // An edge that shares an endpoint with the sight segment cannot
            // properly cross it; skipping such edges also sidesteps the
            // robust intersector's endpoint snapping, which can report a
            // proper intersection at very small coordinate scales.
            chain
                .lines()
                .filter(|edge| edge.start != v && edge.end != v)
                .all(|edge| !line_intersection(edge, sight).is_some_and(|i| i.is_proper()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wkt;
    use approx::assert_relative_eq;
    use hegel::generators;

    fn vertical_separator() -> Line<f64> {
        Line::new((0.0, 0.0), (0.0, 1.0))
    }

    #[test]
    fn accepts_separated_chains_and_finds_vertex_edge_minimum() {
        // Closest approach is from the vertex (2, 1) of q to the interior
        // of p's only edge, at (-1, 1).
        let p = wkt! { LINESTRING(-1. 0.,-1. 2.) };
        let q = wkt! { LINESTRING(2. 1.,4. 1.) };
        let chains = SeparatedChains::new(&p, &q, vertical_separator()).unwrap();
        assert_relative_eq!(chains.separation(), 3.0);
    }

    #[test]
    fn accepts_chains_touching_the_separator() {
        let p = wkt! { LINESTRING(0. 5.,-2. 5.) };
        let q = wkt! { LINESTRING(0. 0.,2. 0.) };
        let chains = SeparatedChains::new(&p, &q, vertical_separator()).unwrap();
        assert_relative_eq!(chains.separation(), 5.0);
    }

    #[test]
    fn rejects_chain_crossing_the_separator() {
        let p = wkt! { LINESTRING(-1. 0.,1. 0.) };
        let q = wkt! { LINESTRING(2. 0.,3. 0.) };
        assert!(SeparatedChains::new(&p, &q, vertical_separator()).is_none());
    }

    #[test]
    fn rejects_degenerate_separator() {
        let p = wkt! { LINESTRING(-1. 0.,-1. 1.) };
        let q = wkt! { LINESTRING(1. 0.,1. 1.) };
        let separator = Line::new((0.0, 0.0), (0.0, 0.0));
        assert!(SeparatedChains::new(&p, &q, separator).is_none());
    }

    #[test]
    fn accepts_slanted_separator() {
        // Separator along y = x; p above-left, q below-right.
        let p = wkt! { LINESTRING(0. 4.,2. 5.) };
        let q = wkt! { LINESTRING(4. 0.,5. 2.) };
        let separator = Line::new((0.0, 0.0), (1.0, 1.0));
        let chains = SeparatedChains::new(&p, &q, separator).unwrap();
        assert_relative_eq!(chains.separation(), separation_brute_force(&p, &q));
    }

    /// A chain with all x-coordinates in `[x_min, x_max]`. Coordinate
    /// magnitudes are bounded so that relative-tolerance comparisons stay
    /// meaningful once the fast solver replaces the brute force; bounded
    /// float generators exclude NaN and infinity by default.
    #[hegel::composite]
    fn side_chain(tc: &hegel::TestCase, x_min: f64, x_max: f64) -> LineString<f64> {
        let n = tc.draw(generators::integers::<usize>().min_value(2).max_value(16));
        let coords: Vec<(f64, f64)> = (0..n)
            .map(|_| {
                let x = tc.draw(
                    generators::floats::<f64>()
                        .min_value(x_min)
                        .max_value(x_max),
                );
                let y = tc.draw(generators::floats::<f64>().min_value(-1e6).max_value(1e6));
                (x, y)
            })
            .collect();
        LineString::from(coords)
    }

    /// Whether the vertices `a` of `p` and `b` of `q` see each other: the
    /// sight segment does not properly intersect either chain, per the
    /// paper's (relaxed) visibility definition.
    fn visible(a: Coord<f64>, b: Coord<f64>, p: &LineString<f64>, q: &LineString<f64>) -> bool {
        if a == b {
            return true;
        }
        let sight = Line::new(a, b);
        // Edges sharing an endpoint with the sight cannot properly cross
        // it; skipping them sidesteps endpoint snapping in the robust
        // intersector at very small coordinate scales.
        let clear = |chain: &LineString<f64>| {
            chain
                .lines()
                .filter(|edge| edge.start != a && edge.end != a && edge.start != b && edge.end != b)
                .all(|edge| !line_intersection(edge, sight).is_some_and(|i| i.is_proper()))
        };
        clear(p) && clear(q)
    }

    /// Minimum distance over mutually visible vertex pairs drawn from the
    /// given index subsets (infinity when no pair is visible).
    fn brute_cvv(
        p: &LineString<f64>,
        p_idx: &[usize],
        q: &LineString<f64>,
        q_idx: &[usize],
    ) -> f64 {
        let mut min = f64::INFINITY;
        for &i in p_idx {
            for &j in q_idx {
                let (a, b) = (p.0[i], q.0[j]);
                if visible(a, b, p, q) {
                    min =
                        min.min(Euclidean.distance(&crate::Point::from(a), &crate::Point::from(b)));
                }
            }
        }
        min
    }

    #[test]
    fn pruning_removes_pocket_vertex() {
        // The last vertex (-3, 1) hides behind the chain's first edge,
        // which runs along x = -1: the horizontal sight segment from
        // (-3, 1) to its foot (0, 1) crosses that edge at (-1, 1).
        let p = wkt! { LINESTRING(-1. 0.,-1. 2.,-3. 1.) };
        let visible_idx = perpendicularly_visible(&p, &vertical_separator());
        assert_eq!(visible_idx, vec![0, 1]);
    }

    #[test]
    fn vertex_on_separator_is_visible() {
        let p = wkt! { LINESTRING(0. 0.,-2. 1.) };
        let visible_idx = perpendicularly_visible(&p, &vertical_separator());
        assert_eq!(visible_idx, vec![0, 1]);
    }

    /// Candidate indices of `chain` after both pruning steps: perpendicular
    /// visibility, then the alpha >= 90 degrees elimination computed over
    /// the survivors. `separator` must have the chain on its left.
    fn pruned_candidates(chain: &LineString<f64>, separator: &Line<f64>) -> Vec<usize> {
        let vis = perpendicularly_visible(chain, separator);
        let coords: Vec<Coord<f64>> = vis.iter().map(|&i| chain.0[i]).collect();
        vis.iter()
            .enumerate()
            .filter(|&(k, _)| visible_wedge(&coords, k, separator).alpha_at_least_90())
            .map(|(_, &i)| i)
            .collect()
    }

    #[test]
    fn alpha_elimination_removes_deep_pocket_vertex() {
        // The middle vertex sits at the bottom of a deep pocket: its wedge
        // is bounded by the rays to its neighbours, spanning well under 90
        // degrees, so Lemma 3 eliminates it.
        let p = wkt! { LINESTRING(-0.1 0.,-2. 1.,-0.1 2.) };
        assert_eq!(pruned_candidates(&p, &vertical_separator()), vec![0, 2]);

        // A shallow pocket spans more than 90 degrees and survives.
        let p = wkt! { LINESTRING(-0.1 0.,-1. 1.,-0.1 2.) };
        assert_eq!(pruned_candidates(&p, &vertical_separator()), vec![0, 1, 2]);
    }

    #[test]
    fn spike_toward_separator_survives_alpha_elimination() {
        // A vertex protruding toward the separator sees it broadly: the
        // wedge is reflex and is treated as 180 degrees.
        let p = wkt! { LINESTRING(-1. 0.,-0.1 1.,-1. 2.) };
        assert_eq!(pruned_candidates(&p, &vertical_separator()), vec![0, 1, 2]);
    }

    #[hegel::test]
    fn alpha_elimination_preserves_closest_visible_vertex_pair(tc: hegel::TestCase) {
        let p = tc.draw(side_chain(-1e6, 0.0));
        let q = tc.draw(side_chain(0.0, 1e6));
        let sep = vertical_separator();
        // q lies right of the separator; reverse it so q is on the left.
        let sep_reversed = Line::new(sep.end, sep.start);

        let all_p: Vec<usize> = (0..p.0.len()).collect();
        let all_q: Vec<usize> = (0..q.0.len()).collect();
        let cand_p = pruned_candidates(&p, &sep);
        let cand_q = pruned_candidates(&q, &sep_reversed);

        let full = brute_cvv(&p, &all_p, &q, &all_q);
        let pruned = brute_cvv(&p, &cand_p, &q, &cand_q);

        assert_eq!(full.total_cmp(&pruned), std::cmp::Ordering::Equal);
    }

    #[hegel::test]
    fn pruning_preserves_closest_visible_vertex_pair(tc: hegel::TestCase) {
        let p = tc.draw(side_chain(-1e6, 0.0));
        let q = tc.draw(side_chain(0.0, 1e6));
        let sep = vertical_separator();

        let all_p: Vec<usize> = (0..p.0.len()).collect();
        let all_q: Vec<usize> = (0..q.0.len()).collect();
        let vis_p = perpendicularly_visible(&p, &sep);
        let vis_q = perpendicularly_visible(&q, &sep);

        let full = brute_cvv(&p, &all_p, &q, &all_q);
        let pruned = brute_cvv(&p, &vis_p, &q, &vis_q);

        assert_eq!(full.total_cmp(&pruned), std::cmp::Ordering::Equal);
    }

    #[hegel::test]
    fn separation_matches_segment_pair_minimum(tc: hegel::TestCase) {
        let p = tc.draw(side_chain(-1e6, 0.0));
        let q = tc.draw(side_chain(0.0, 1e6));
        let chains = SeparatedChains::new(&p, &q, vertical_separator())
            .expect("chains are separated by construction");

        let expected = p
            .lines()
            .flat_map(|lp| q.lines().map(move |lq| Euclidean.distance(&lp, &lq)))
            .fold(f64::INFINITY, f64::min);

        assert_eq!(
            chains.separation().total_cmp(&expected),
            std::cmp::Ordering::Equal
        );
    }
}
