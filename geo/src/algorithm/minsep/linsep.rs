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

use crate::algorithm::line_measures::{Distance, Euclidean};
use crate::{GeoFloat, GeoNum, Kernel, Line, LineString, Orientation};

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
