# Polygon Separation Algorithm Implementation Notes

## Background

We compute σ(P, Q), the minimum distance between the boundaries of two simple
polygons, using Amato's decomposition method (see reference below). The
sequential version runs in Θ(n) time, n = |P| + |Q|.

## Key facts established 2026-08-31

1. **σ is boundary separation, not interior distance.** A polygon strictly
   inside another has σ > 0. This differs from geo's `Distance` for polygons
   (zero on any overlap or containment). The earlier skeleton short-circuited
   containment to 0, which also made the containing branch of `classify_case`
   unreachable; both fixed.

2. **A correctness oracle already exists.** For non-intersecting boundaries,
   σ(P, Q) equals `Euclidean.distance(p.exterior(), q.exterior())` — geo's
   brute-force O(|P|·|Q|) LineString distance. Amato's contribution is purely
   complexity. Consequences:
   - every component can be property-tested against the oracle (use hegel);
   - the eventual PR is justified by benchmarks against the oracle.

3. **Resolved open TASKs from the earlier notes:**
   - *Signed distance / half-plane tests*: `Euclidean.distance` does not
     suffice (unsigned). Use the robust kernel predicate `Kernel::orient2d`
     (`T::Ker::orient2d(a, b, c)`) for all side-of-line tests.
   - *Shortest path in R*: no Dijkstra needed. R is a simple polygon without
     holes, so its earcut triangulation dual is a tree; walk the dual path
     between the triangles containing the endpoints and run the funnel
     algorithm. Linear time after triangulation.
   - *Polygon-polygon intersection*: `p.exterior().intersects(q.exterior())`
     is the σ = 0 test. Done.

## Implementation plan (agreed 2026-08-31)

Attack Section 4 (linearly separable subproblem solver) first, in
`linsep.rs`, before fixing DECOMPOSE: it is self-contained, testable in
isolation against the oracle, and defines what a subproblem must look like.

1. ~~Housekeeping: boundary-separation semantics, generic `T: GeoFloat`,
   honest placeholder comments, `total_cmp`, wkt! fixtures.~~ Done.
2. `linsep.rs` scaffold: `SeparatedChains` (two chains + separating line,
   validated with orient2d; vertices may lie on the line), brute-force
   solver, property-test harness vs an independent segment-pair loop.
3. Pruning, Step 1 of LinSep-CVV: remove vertices not perpendicularly
   visible from l. Property: surviving candidates still contain a pair
   realising cvv. Start O(n²) (test each perpendicular segment against all
   chain edges via `line_intersection`); optimise later.
4. Visible wedges via successive convex hulls on the pruned (monotone)
   chain; eliminate α < 90° vertices (Lemma 3).
5. Feasible regions R() (u⁺/u⁻ points) and candidate search. Correctness
   first: direct min over candidate pairs. Then the totally monotone matrix
   row-minima search (SMAWK / Atallah–Kosaraju) as a perf pass. Note: the
   `smawk` crate exists and is maintained; decide dep-vs-vendor with
   maintainers (~100 lines to vendor).
6. LinSep-CVE (vertex–edge case, Algorithm 4): edge partitioning so h()/l()
   are constant per sub-segment, via successive convex hulls (Fig. 8;
   sequential O(n) per §4.3). σ_subproblem = min(cvv, cve(P,Q), cve(Q,P)).
7. Return to DECOMPOSE with correct components:
   - common supporting lines from the CH(P)/CH(Q) merge;
   - facing portions via orient2d;
   - polygon R construction with proper winding (`Winding` trait);
   - shortest path (earcut + funnel);
   - segment extension by ray shooting (linear scan is fine sequentially);
   - redundant-segment removal: keep l_0, then greedily keep the
     maximal-indexed segment intersecting the previously kept one;
   - subchain extraction per Step 2 (p⁺/q⁺ intersection points).
8. API shaping for the PR: decide trait name and signature (likely
   `Option<T>` or a result carrying the realising pair, per geo
   conventions on degenerate inputs), document the σ vs `Distance`
   distinction, no third-party types in the public API.

## Preconditions and open questions

- Inputs must be simple polygons without interior rings (paper assumption).
  Decide at API-shaping time: reject, ignore holes, or document.
- Degenerate inputs (rings with < 3 distinct vertices) currently
  unhandled; return `Option`/error enum at API-shaping time.
- The containing case where CH(P ∪ Q) = CH(Q) but P ⊄ Q needs the extra
  CH(Q) edge in R (paper §3.1 parenthetical).

## Reference

- Nancy M. Amato, "Determining the Separation of Simple Polygons", IJCGA
  4(4), 1994. doi:10.1142/S0218195994000240. (Zotero key S5BWK5Q3; the
  Springer link in earlier notes was the WADS'93 preliminary version.)
- Totally monotone matrix search: Aggarwal et al. 1986 (SMAWK); sequential
  row minima Atallah–Kosaraju 1991.
