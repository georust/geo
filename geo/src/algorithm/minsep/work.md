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
2. ~~`linsep.rs` scaffold: `SeparatedChains` (validated with orient2d;
   vertices may lie on the line), brute-force solver, property-test
   harness.~~ Done. The harness immediately caught a real bug in geo's
   LineString-LineString fast path (see "Found bug" below).
3. ~~Pruning, Step 1 of LinSep-CVV: perpendicular visibility.~~ Done,
   O(n²) for now.
4. ~~Visible wedges and α ≥ 90° elimination (Lemma 3).~~ Done, O(n²)
   tangent scan per vertex for now (the successive-hull O(n) version
   comes with the perf pass). Hard-won lessons, enforced by hegel:
   - visibility must block only on PROPER intersections (the paper's
     §4.3 relaxed definition); collinear overlap does not hide;
   - exclude edges incident to a sight-segment endpoint exactly — they
     cannot properly cross it, and the robust intersector's endpoint
     snapping otherwise produces false positives at ~1e-17 scales;
   - measure wedge heights relative to the apex, never against the
     separator start (absolute heights absorb small coordinates).
5. ~~Candidate search (CVV): direct min over candidate pairs.~~ Done via
   mutual visible-wedge membership, without the feasible regions R():
   reading the paper closely, the u⁺/u⁻ points exist only to give the
   candidate matrix M its totally monotone structure — for correctness,
   mutual W() membership suffices (Lemma 4 puts the realising pair in both
   R() ⊂ W(); conversely a mutually-wedge-contained candidate pair is
   visible, since a crossing edge would either put a vertex inside a wedge
   or span one and block its apex's perpendicular sight). R()/u⁺/u⁻
   therefore move to the perf pass, alongside the totally monotone matrix
   row-minima search (SMAWK / Atallah–Kosaraju). Note: the `smawk` crate
   exists and is maintained; decide dep-vs-vendor with maintainers (~100
   lines to vendor).
   - Wedges for membership are computed against the full chain (the
   visibility argument needs W() vertex-free w.r.t. every chain vertex),
   while α-elimination keeps its candidate-based wedges.
   - Caveat found by hegel: with chain geometry ON the separator line, a
   properly-blocked pair can be admitted (blocking edge through the
   apex's perpendicular foot — an endpoint contact under relaxed
   visibility). Sound for σ: admitted pairs are vertex-vertex distances,
   so cvv never undercuts σ, and the visible realising pair is never
   missed. The cvv property is therefore a bracket
   (σ ≤ cvv ≤ closest-visible-pair), not equality; the full σ equality
   test lands with CVE in step 6.

6. ~~LinSep-CVE (vertex–edge case, Algorithm 4) and the separation()
   switch to min(cvv, cve(P,Q), cve(Q,P)).~~ Done 2026-09-01. Same
   deferral pattern as step 5: the direct cve search runs candidate
   vertices against every opposite edge, restricted to pairs whose
   nearest edge point lies in the vertex's wedge (Lemma 6 gives
   q_{p,e} ∈ R(p) ⊂ W(p) for the realising pair); the edge partitioning
   via successive convex hulls (Fig. 8; sequential O(n) per §4.3), the
   edge wedges W(e′) with the C_e circle elimination, and the R()
   regions all only structure the candidate matrix, so they move to the
   perf pass. cve distances use the same Point–Line primitive as the
   oracle so realising values agree bit-for-bit.
   - The σ property (solver vs segment-pair oracle, 500k cases) is
   equality within a scale-aware tolerance, not exact: pruning decides
   with exact predicates while distances round, so at knife-edge inputs
   solver and oracle can land a few ulps apart, on either side of the
   true σ (see the test comment for two shrunk examples). Tolerance:
   relative 8ε, absolute floor 32ε × max coordinate magnitude — never a
   fixed constant, so tiny-coordinate regressions stay visible.
   - Oracle hardening forced by the harness (see "Robust-kernel and
   primitive limits" below): no `intersects` short-circuit, and
   endpoint-pair distances taken alongside the four point-segment
   distances.

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

## Robust-kernel and primitive limits (found 2026-09-01)

Two float-limit behaviours in geo's primitives, both found by the σ
property harness while testing the LinSep solver. Neither is a bug to
fix upstream so much as a documented limit; the linsep test oracle works
around both (see `separation_brute_force`):

1. `RobustKernel::orient2d` loses exactness when an orientation product
   underflows (magnitude below ~1e-323): disjoint segments can be
   misclassified as collinear, after which `Intersects<Line> for Line`
   degenerates to a bounding-rect check and reports a false
   intersection, so `Euclidean.distance` returns a false zero.
   Counterexample: `LINESTRING(0 0, 0 1.3e-113)` vs
   `LINESTRING(0 1, 9.5e-212 0)` — true distance 9.5e-212.
2. `line_segment_distance` divides by the segment's squared length,
   which underflows to zero for segments shorter than ~1e-162; the
   result is NaN, which `Float::min` folds silently away (an all-NaN
   reduction leaves the initial infinity).

## Found bug: geo fast-path overestimate (2026-08-31)

The linsep property harness found that `Euclidean.distance` for
LineString pairs can overestimate: the prefix binary search added to
`separable_geometry_distance_fast` in PR #1560 prunes vertex pairs by a
bound that is sound for vertex-vertex distances but not for the
adjacent-segment distances the search evaluates. Counterexample:
`LINESTRING(0 0,0 4,-1 -1)` vs `LINESTRING(1 3,1 -3)` returns ~1.1767
instead of 1.0. Fixed on the branch `fix-separable-fast-path-prefix-prune`
(two commits: failing regression tests, then a fix widening the prefix
threshold by each geometry's maximum projected edge span; costs 3.6% on
the #1560 benchmark against 97% for removing the skip). The fix was merged on 2026-09-01

## Expected performance vs existing paths (noted 2026-09-01)

- vs the #1560 fast path (bbox-separated inputs): both near-linear in
  practice; the fast path is two O(n log n) sorts plus a heuristic prune
  with O(n·m) worst case. Amato is worst-case Θ(n) (modulo earcut in
  DECOMPOSE). Expect a constant-to-log-factor win at scale, decisive only
  on inputs where prefix pruning degrades.
- vs the overlapping-bbox fallback (R-tree nearest neighbour,
  O((n+m) log n) plus tree-build): the clearer win.
- Nested/overlapping polygons: `Distance` returns 0 there; σ is new
  capability, not a faster path. Whether 0 is the expected distance (what do JTS, GEOS, PostGIS return) is an open question, however

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
