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

use crate::algorithm::line_intersection::{LineIntersection, line_intersection};
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

/// Indices of the vertices of `chain` that are perpendicularly visible from
/// the separator line: the segment from the vertex to its perpendicular foot
/// on the line meets the chain only at the vertex itself (Step 1 of
/// LinSep-CVV). Proper crossings and interior touches block the sight
/// segment; contact at either of its endpoints does not. A vertex on the
/// line is trivially visible.
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
            chain
                .lines()
                .all(|edge| match line_intersection(edge, sight) {
                    None => true,
                    Some(LineIntersection::Collinear { .. }) => false,
                    Some(LineIntersection::SinglePoint {
                        intersection,
                        is_proper,
                    }) => !is_proper && (intersection == v || intersection == foot),
                })
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
    /// sight segment meets each chain at most at its own endpoints, per the
    /// paper's visibility definition (no proper intersection with P or Q).
    fn visible(a: Coord<f64>, b: Coord<f64>, p: &LineString<f64>, q: &LineString<f64>) -> bool {
        if a == b {
            return true;
        }
        let sight = Line::new(a, b);
        let clear = |chain: &LineString<f64>| {
            chain
                .lines()
                .all(|edge| match line_intersection(edge, sight) {
                    None => true,
                    Some(LineIntersection::Collinear { .. }) => false,
                    Some(LineIntersection::SinglePoint {
                        intersection,
                        is_proper,
                    }) => !is_proper && (intersection == a || intersection == b),
                })
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
