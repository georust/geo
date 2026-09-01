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
   - ~~polygon-level σ property (star-polygon generator, hegel) as the
     standing regression guard; passes trivially while the pipeline ends
     in the brute-force fallback, and keeps DECOMPOSE honest as each
     placeholder is replaced.~~ Done 2026-09-01. Generator lesson: a
     star polygon is simple only while every angular gap is < 180°;
     draw gaps from [0.6, 1.0] before normalising (a dominant gap made
     the chord leave its wedge and produced self-intersecting "stars",
     found when an R-property failure shrank to one).
   - ~~common supporting segments~~ Done 2026-09-01. No tangent
     computation needed: they are the mixed edges of CH(P ∪ Q) (one
     endpoint from each polygon; membership by coordinate equality,
     sound because disjoint boundaries share no coords), with contacts
     shrunk along the line to the extreme on-line vertices of each
     polygon (hull construction elides collinear vertices, so the mixed
     edge can overshoot the tangency).
   - ~~facing portions + polygon R (non-containing)~~ Done 2026-09-01.
     R (CCW) = p→q bridge + Q's ring walked CW between its contacts +
     q→p bridge + P's ring walked CW back; the bridge directions from
     the CCW hull walk are what make the CW polygon walks select the
     facing arcs. Single-vertex hull contact ⇒ that polygon's facing
     arc is its FULL ring and R is pinched (weakly simple, vertex
     appears twice) — the analogue of the containing case's doubled cut
     vertex; the shortest path must then route around the pinched-in
     polygon (both path endpoints are the pinch vertex). Only one side
     can pinch (both ⇒ hull degenerates to a segment).
   - ~~shortest path~~ Done 2026-09-01, correctness-first: visibility
     graph over R's ring vertices + scan Dijkstra, O(|R|³); endpoints
     and visibility are INDEX-based so pinched rings route around the
     pinched-in polygon (the two pinch instances never see each other).
     Earcut + funnel replaces it in the perf pass.
   - ~~segment extension + redundant-segment removal~~ Done 2026-09-01.
     Extension = linear ray scan per direction with the midpoint-decides
     rule at path vertices (no extension past convex corners), EXCLUDING
     the through-vertex's incident edges from the scan (a rounded t just
     above 1 on an incident edge otherwise masks the true hit through a
     reflex vertex, leaving a coverage gap — σ harness find). Extensions
     carry exact provenance (path window + hit edge index).
   - ~~subproblem construction (Step 2)~~ Done 2026-09-01, the hardest
     part; the σ property harness found six distinct coverage bugs
     before it held at 300k cases. Deviations from the paper, each
     forced by a shrunk counterexample:
     - separator b/t orientation is PATH order, not "endpoint closest
       to l_{i−1}" (they disagree when a backward extension overshoots);
     - cuts are combinatorial (since the coverage proof, 2026-09-01): the
       highest point of l_i on an arc is the first of t_i, the run's
       upper vertex, its lower vertex, b_i that lies on that arc
       (Appendix A, Lemma A6), located by ring index; Step 2 performs
       no intersection tests. (The earlier candidate-list version had
       float trouble: exact intersection tests missed touches by an
       ulp, and a distance-based side heuristic once inserted a
       mid-bridge point into a subchain — the only underestimate ever
       seen, since chains must contain only boundary points.)
     - a cut truncates ONLY when the corresponding separator endpoint
       lands on that polygon (top endpoint for the end cut, previous
       separator's bottom endpoint for the start cut); the paper's
       "otherwise → neighbour cut" and "i = m−1 → own cut" branches are
       both unsound (bridge landings break the l_i-intersects-both
       invariant; the last separator can be anchored on one polygon at
       both ends and merely graze the other), and widen to the facing
       arc sentinel instead;
     - subchains are facing-arc slices between the cuts; the whole-ring
       override once used when an anchor cut fell outside the slice was
       removed after Appendix A showed it never affects coverage;
     - pinched channels take full-ring subchains for every separator
       (topologically the containing case: cyclic separator sequence).
     Coverage of every visible pair by these amended rules is proved
     in paper_corrections.typ, Appendix A (2026-09-01); the proof
     shows the wrap override is never needed for coverage and that
     property (ii) of the 1992 report holds for the greedy output.
     The subchain brute-force fallback (when a separator fails
     SeparatedChains validation in both orientations) is retained by
     design: over-large subchains cannot undercut σ, and the σ property
     guards completeness. Aggregate subchain size is not yet O(n); the
     perf pass revisits with the paper's exact machinery.
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

## Complexity implication of the coverage proof (2026-09-01)

With the amended Step 2 rules, at each end of every subproblem one
subchain runs to its sentinel (Appendix A, Lemma A4: the geodesic
segment inside a separator ends on one wall, so only the other wall's
subchain is truncated there). Whenever the geodesic alternates walls
Θ(n) times the aggregate subproblem size is Θ(n²), and sentinel chains
are generally not linearly separable, so the LinSep fallback to the
subchain brute force makes the current pipeline O(n³) worst case —
slower than the O(n²) oracle. Property (v) of the 1992 report is what
bounds the sentinel side by a later cut; its two-path construction is
therefore the first item of the perf pass, not a refinement. Until
then the decomposition is a correctness reference, not a speed-up.

## Perf-pass plan (agreed 2026-09-01)

Order matters: item 0 is small and removes the quadratic behaviour for
the common case; items 1–2 are the research-grade work; 3–5 are the
known linear-time internals.

0. **Route hull-disjoint inputs around DECOMPOSE.** The 1992 report
   invokes the decomposition only when CH(P) ∩ CH(Q) ≠ ∅; with disjoint
   hulls the whole boundaries are linearly separable and the LinSep
   solver applies to them directly. The 1994 paper sends those inputs
   through DECOMPOSE as well, and that is where the bridge-landing,
   orientation, wrap and pinch counterexamples live: of the report's
   fixtures only D3 (grazing) has intersecting hulls (checked
   2026-09-01 with `ch_p.intersects(&ch_q)`). Implementation: after
   `compute_hulls`, test CH(P) ∩ CH(Q) = ∅ (linear by rotating
   calipers; the hull-vertex brute force is acceptable to start), find
   a separating line (an inner common tangent of the two hulls, or the
   perpendicular bisector of the hull closest pair), and return
   `SeparatedChains::new(p.exterior(), q.exterior(), line).separation()`;
   fall through to DECOMPOSE if validation fails (it should not).
   Re-run the σ property and the fixtures; expect D1, D4, D5 to take
   the new branch.
   Report follow-up: the four hull-disjoint findings (bridge landing,
   orientation, wrap, pinch) are then artefacts of the 1994 paper
   widening DECOMPOSE's remit beyond the 1992 report's, which only
   decomposes when the hulls intersect. §3.1 half says this ("in the
   @amato94 setting, where DECOMPOSE also serves hull-disjoint
   polygons, they occur"); when item 0 lands, say it outright in §2 or
   the closing remarks, classify each finding by hull regime, and
   present the routing as the first remedy. Only the grazing finding
   (D3) and the complexity result then bear on the 1992 construction.
1. **Two-path construction for the hull-intersecting non-containing
   case** ([A92] Theorem 3). Two geodesics in R: L_p between the
   P-contacts (exists) and L_q between the Q-contacts. Their common
   subpath ρ* (nonempty when the hulls intersect, per the report; its
   first and last vertices are the second and next-to-last vertices of
   L_p and/or L_q). Separators: the first edge of L_p, the edges of ρ*,
   the last edge of L_p. Each is extended ONE-SIDEDLY, from its top
   shared vertex until it meets ∂R; the landing wall is read from the
   wall of that shared vertex (our Lemma A4), never searched for. Then
   the greedy of Step 1(d) for property (ii). Obligations to settle
   before trusting it, using Lemmas A2–A5 of the appendix (they hold
   for any geodesic in R, L_q included): (a) ρ* is nonempty and the
   two paths agree on it; (b) no one-sided extension from a vertex of
   ρ* lands on a bridge (property (iv)); (c) the landing is the
   highest point of the opposite wall visible from the separator
   (property (v)) — D3 is the acid test, since its grazing separator
   comes from a one-path turn at a Q vertex that L_q may not share.
   If (b) or (c) fails, we have the sixth configuration and the
   sentinel rules stay for those separators.
2. **Subchain rules under (v).** With (iv) and (v) established, use
   the [A92] rules P_1(i), P_2(i) (aggregate at most five appearances
   per vertex, hence Θ(n)); keep the amended sentinel rules as the
   fallback wherever (v) is not established (a bridge landing, if any
   survive item 1). Re-prove coverage for the two-path sequence: the
   appendix's Lemmas A7–A10 go through unchanged given (i)–(iii); the
   (v)-dependent end bound replaces the sentinel.
3. **Linear-time geodesics**: earcut triangulation of R, dual-tree
   walk, funnel algorithm; replaces the O(|R|³) visibility-graph
   Dijkstra for both L_p and L_q.
4. **Linear-time LinSep**: feasible regions R(), u⁺/u⁻, successive
   hulls for edge partitioning, totally monotone row minima (SMAWK;
   `smawk` crate vs vendoring, decide with maintainers). Needed for
   item 0 to be linear as well.
5. **Containing case** (annulus R with the doubled cut vertex, cyclic
   separator sequence, one path from p to p as in both papers) and the
   pinched channel, which is its non-containing shadow.
6. Benchmarks against the oracle only after 0–4; until then the
   decomposition is a correctness reference.

## Report follow-ups (pending, do not send before)

- Classify the five findings by hull regime once item 0 of the perf
  pass lands (see there): D1, D2, D4, D5 hull-disjoint, D3 hull-
  intersecting. State that the 1992 report never decomposes
  hull-disjoint inputs, so four findings concern the 1994 paper's
  wider remit for DECOMPOSE, and present the 1992 routing as the
  first remedy.
- If item 1 settles obligations (b) and (c), report whether the
  two-path construction has properties (iv) and (v); if not, add the
  sixth configuration.
- Then revisit the complexity paragraph of §4 and the appendix's
  complexity remark: they describe the amended one-path rules and
  should say what the two-path construction changes.

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
- Nancy M. Amato, "Computing the Minimum Visible Vertex Distance Between
  Two Nonintersecting Simple Polygons", TR UILU-ENG-92-2206, CSL,
  University of Illinois, 1992 (IDEALS item 100137). Journal version:
  Algorithmica 14, 183–201, doi:10.1007/BF01293668. Contains the full
  separator machinery the 1994 paper compresses (axiomatic S(P,Q)
  properties, two-path construction, containing-case detail); the perf
  pass and the containing-case chunk should follow it. Deviations from
  the papers: paper_corrections.typ (bookmark: minsep-paper-corrections).
- Totally monotone matrix search: Aggarwal et al. 1986 (SMAWK); sequential
  row minima Atallah–Kosaraju 1991.
