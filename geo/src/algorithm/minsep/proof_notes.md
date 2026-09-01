# Coverage proof for the amended Step 2: working notes

STATUS 2026-09-01: DONE. Appendix A of paper_corrections.typ holds
the proof (Lemmas A1–A10, Theorem A, remarks); §4, §2, §3.4, the
abstract and the closing remarks cite it. Remaining housekeeping
before the squash into the report commit: remove `mod proof_trace`
(the harness), the `CutOrder` enum and the `wrap_override` parameter
of `construct_subproblems_with` (both exist only for the harness),
Done afterwards (same day): harness removed; wrap override removed;
Step 2 rewritten combinatorially from Lemmas A4/A6 (landing side and
cut from provenance, no intersection tests); permanent hegel property
`realising_pair_lies_in_its_separators_subproblem` (Theorem A);
report §4 and Appendix A remarks updated with the complexity
implication: one sentinel per end per subproblem, Θ(n²) aggregate
typical and tight, sentinel subproblems not linearly separable, so the
current pipeline is O(n³) worst case; the two-path construction is
required for any gain. The session logs below are the working record.

Task (original): prove that the amended subchain construction (report §4, "An
amended construction") covers every visible pair, i.e. for every
visible pair (p, q), p ∈ ∂P, q ∈ ∂Q, there exists i with p ∈ P_i and
q ∈ Q_i. Scope: the non-containing, non-pinched channel (the pinch
override makes every subchain a whole boundary, so coverage there is
trivial). Deliverable: an appendix to paper_corrections.typ in the
same lemma/proof style, plus an update to the §4 sentence "We have
validated coverage empirically ... but do not prove it here" to point
at the appendix. The report should not be sent before this is done:
if the proof closes it upgrades the report; if it fails we find the
sixth configuration ourselves.

## Where things stand

- The report is paper_corrections.typ (bookmark:
  minsep-paper-corrections, pushed to origin with the adversarial
  review applied). The complexity paragraph in its §4 preamble and
  the "Complexity obligations" section below were added after the
  push; check `jj st` for uncommitted state before assuming either
  is in the pushed commit.
- The implementation chain head is the commit "regression fixtures
  for DECOMPOSE counterexamples; cite the 1992 CVV report"
  (yqrywxnr-ish; check `jj log`). mod.rs holds
  decompose_regressions_match_oracle with 11 pairs: 4 simplified +
  7 shrunk originals.
- Both source papers: [A94] = the 1994 IJCGA paper (Zotero S5BWK5Q3,
  PDF in Zotero storage K5XFZMWX); [A92] = the 1992 CSL tech report
  (Zotero RGVH8UWH, PDF was fetched to the old session scratchpad —
  re-fetch from IDEALS item 100137 if needed; also
  ~/Downloads/Amato-1992-...UILU-ENG-92-2206.pdf).

## Exact definitions (as implemented; mod.rs is authoritative)

Channel construction (`construct_polygon_r_noncontaining`): rings
normalised CCW; bridges from the mixed edges of CH(P ∪ Q) with
contacts shrunk to extreme on-line vertices; facing arcs stored
ascending: arc_p = p_b … p_t, arc_q = q_b … q_t; R's ring =
[p_b] + arc_q + reversed arc_p; path endpoints are RING INDICES
(pinch ⇒ same coord, distinct indices).

Path (`shortest_path_in_ring`): geodesic between the two P-contacts,
vertices are ring indices. Grazing shots are decomposed through
grazed vertices (visibility rejects any touch of a non-incident
edge), so consecutive path segments may be collinear and path
vertices may lie on arc_q (that is what "grazing" separators are).

Separators (`extend_segments_to_boundary`, `ExtendedSegment`): per
path window, one ray per direction; extension only if the span
beyond the vertex is interior to R (midpoint test); incident edges
of the through-vertex excluded from the ray scan. Records
back_edge/fwd_edge = ring edge hit (None ⇒ endpoint is the path
vertex itself). Orientation is path order: line.start on the path's
start side (= b_i), line.end = t_i. Redundancy removal
(`remove_redundant_segments`): keep l_0, then repeatedly the
maximal-indexed segment intersecting the last kept one, with a
fallback to the next segment (consecutive extended segments share a
path vertex in exact arithmetic).

Side classification (exact, from the ring layout): edge 0 = bottom
bridge, edges 1..path_end−1 = Q arc, edge path_end−1 = top bridge,
edges ≥ path_end = P arc; bare vertex 0 or ≥ path_end ⇒ P, else Q
(contacts belong to their arcs).

Cuts (`construct_subproblems`): p_i⁺ = HIGHEST point of l_i ∩ arc_p
along the b_i→t_i orientation, candidates = exact segment
intersections plus (float-level) the separator endpoints and path
vertices with matching provenance. NB the height order here is the
suspect part — see "candidate simplification" below.

Amended rules (per chain X ∈ {P, Q}, arc positions along the facing
arc):
- end e_i = x_i⁺ if t_i lands on X (fwd_edge on X's arc, or the
  unextended endpoint is a path vertex classified X), else the
  sentinel x_t;
- start s_i = x_{i−1}⁺ if i > 0 and b_{i−1} lands on X, else x_b;
- wrap override: if x_{i−1}⁺ or x_i⁺ exists and lies outside the
  slice [s_i, e_i] in arc order, X_i = the whole ring ∂X;
- pinch override: pinched channel ⇒ whole rings throughout.
σ = min_i σ(P_i, Q_i).

## Proof skeleton (worked out so far)

Theorem (target). Non-containing, non-pinched channel, S(P, Q) and
subchains as above: every visible pair (p, q) lies in P_i × Q_i for
some i.

Step 1 — Confinement lemma. Every visible pair has p ∈ P′, q ∈ Q′,
and its sight segment lies in cl(R). Argument: the sight lies in
CH(P ∪ Q) (convexity); the hull interior decomposes into P, Q, R and
"pockets" bounded by a polygon's back arc and its hull arc; a pocket
of P is sealed by ∂P and hull boundary, so a segment from a back-arc
point to Q must cross ∂P. Care: pockets adjacent to bridges are part
of R; contacts (p_b etc.) lie on both an arc and a bridge.

Step 2 — Separation lemma. The kept separators form a connected set
(consecutive kept separators intersect: each contains its unextended
path segment, and the greedy keeps a chain where consecutive members
intersect — in exact arithmetic the fallback case still shares a
path vertex). l_0 contains p_b, l_{m−1} contains p_t, both on ∂R, so
the union of kept separators ∪ its anchor points separates R; hence
every P′-to-Q′ sight meets some kept l_i. Define i* = minimal such
index. Do NOT assume property (ii) of [A92] (non-consecutive
separators may intersect; the greedy does not obviously establish
(ii) and the proof must not lean on it).

Step 3 — Shielding lemmas (the delicate part; argue in BOUNDARY
ORDER throughout, this is where [A94] slipped):
- (end) If t_{i*} lands on Q, then q precedes-or-equals q_{i*}⁺ in
  the q_b→q_t arc order, OR the pair is covered by another
  subproblem (see Step 4). The pure form is FALSE as stated —
  counterexample D4 below has t_0 ∈ Q and the realising q beyond
  q_0⁺ in arc order (though below in height). So the lemma must be:
  either arc-order-below the cut, or the wrap/adjacent-sentinel
  machinery catches it.
- (start) mirror for b_{i*−1} landing on Q.
The Jordan-style core: a separator with an endpoint ON Q′ at c,
together with a subarc of ∂R, bounds a region; a sight avoiding the
separator cannot connect across it. Where it breaks: l_i can
intersect Q′ several times; "highest along the separator" need not
be "last along the arc"; the region argument bounds arc positions
only when the relevant crossing is the arc-order-extreme one.

Step 4 — Witness transfer (KEY, discovered late). Coverage need not
come from the lemma-assigned subproblem. Concrete: in D4 the pair
touches l_0, t_0 lands on Q, sub 0 ends at q_0⁺ and misses the
realising q; our implementation covers the pair via sub 1, whose
start/end conditions BOTH fail (b_0 lands on P, t_1 lands on P), so
Q_1 = the full facing arc, and A2 ∈ P_1 because P_1 starts at p_0⁺ =
A2 (b_0 lands on P ⇒ start condition holds for the P side).
Generalising: when the end shielding fails on the Q side for i, show
the pair is picked up by i+1 (or i−1 for start failures): the same
endpoint-landing facts that break shielding for i force sentinels
for the neighbour on that side, while the P side stays anchored.
This pairing argument (both coordinates in the SAME neighbouring
subproblem) is the crux of the whole proof. Enumerate the four
(t_i side, b_{i−1} side) cases and chase each.

Step 5 — The wrap override. Under the current height-order cut
definition the override exists to catch anchors outside the slice.
If Step 4 closes without ever needing it, the override may be
provably dead code for coverage (it would remain harmless). If Step
4 needs it, state exactly where.

## Candidate simplification to evaluate FIRST

Redefine x_i⁺ as the LAST point of l_i ∩ X′ in arc order (not
highest along the separator). Conjecture: with arc-order cuts the
end-shielding lemma holds directly (the Jordan region argument and
the subchain now use the same order), Step 4 shrinks or disappears,
and the wrap override becomes unnecessary. Check against D4 first:
arc-order q_0⁺ would be the LAST crossing of l_0 with arc_q — l_0 is
a segment ending ON arc_q, single crossing, so the cut is unchanged
there and the D4 pair still needs witness transfer (its q is beyond
the cut in arc order but the sight only TOUCHES l_0 at A2 ∈ P — the
sight does not cross into the shielded region; re-examine whether
the shielding region argument actually excludes this pair or only
height does). If the redefinition helps, it is an implementation
change in `highest_intersection` (order along the arc instead of
along the separator) — then rerun: full minsep suite, σ property at
≥100k (HEGEL_TEST_CASES env var), and the T1/T2 toggle checks below
against all 11 regression pairs.

## Proof test-cases (the counterexamples the proof must survive)

Each is in mod.rs::decompose_regressions_match_oracle and the
report. Trace the finished proof through every one.

- D1 bridge landing: P (0 2, 0 −2, 2 0), Q (−6 −2, −6 −6, −2 −5).
  l_0 = (0,2)→(0,−2.5) ends mid-bridge; no q_0⁺. σ = 3.6.
- D3 graze: P (2 −4, −1 3, 7 12, −4 8, −4 −4), Q (12 4, 6 8, 4 4,
  8 0). Last separator (1.4667,−2.7556)→(7,12) anchored P–P, passes
  through Q vertex (4,4); realising vertex (6,8) beyond it.
- D4 wrap/slippage: P (0 0, −1 −2, 2 −1), Q (−8 −4, −8 −8, 4 −6).
  l_0 = (0,0)→(−2.4615,−4.9231) (t_0 on Q's arc), l_1 =
  (−7.3333,−4.1111)→(2,−1). Realising pair: P vertex A2 = (−1,−2)
  to the foot ≈ (−1.51,−5.08) on Q's top edge. Heights along l_0
  (dot with direction (−2.4615,−4.9231)): h(foot) ≈ 28.7 <
  h(q_0⁺) ≈ 30.25 — below in height, beyond in arc order. Sight
  touches l_0 and l_1 exactly at A2 (shared path vertex). Covered
  by sub 1 (double sentinel on Q, start-anchored on P).
- D5 pinch: P (3 2, 0 −4, 4 2), Q (0 8, 2 3, 8 4) — trivial branch,
  include for completeness.
- Also: grazing means path vertices can BE Q vertices; sights can
  touch a separator at a single point that is a polygon vertex of
  either polygon; separators can be collinear with polygon edges
  (D1's l_0 is collinear with P's left edge — l_0 ∩ P is a segment,
  "the cut" is an endpoint of an overlap).

## Toggle patches (for revalidation if the implementation changes)

T1 (paper rules; expect D1, D3, D4 to fail): in
construct_subproblems replace the start/end rules with
start = arc_cuts[i−1] if i > 0 else sentinel_start;
end = scan-forward own cut if (tops[i] == own || i == m−1) else
scan from i+1, both falling back to sentinel_end.
T2 (expect D5 to fail): `let pinched = false;`.
Debug harness pattern: a temporary test printing bridges, arcs,
path indices, separators with provenance, per-subproblem chains and
σ vs `linsep::separation_brute_force` — reconstruct from the shape
of `compute_polygon_separation`.

## Writing the appendix

Same file, after §6 (Closing remarks currently ends the body; put
the appendix before the bibliography and renumber nothing — use
`#heading(numbering: none)` or an "Appendix A" heading with manual
label). Style: Lemma/Proof blocks matching the report's register;
UK spelling; en dashes; cite [A92]/[A94] where an argument parallels
theirs (the confinement lemma parallels [A94] Lemma 1's setup; the
shielding lemmas parallel [A92] Lemma 9). State the theorem for the
non-containing case and note the pinch reduction in one line. After
it lands, change §4's "but do not prove it here" to reference the
appendix, and revisit the closing-remarks question list (the
two-orders question stays — it is about THEIR construction).

## Complexity obligations

Three layers, keep them distinct:

1. [A94] claims Θ(n) overall, via aggregate subchain size ≤ 4n
   (≤ 5n in [A92]) and linear-time subproblem solving.
2. The amended rules, even with all the performance machinery in
   place, give only O(n²) worst case: a sentinel-extended subchain
   runs to its facing-arc end, so sentinel-heavy configurations give
   aggregate Θ(m · n); the overrides are the same order. When every
   truncation condition applies the chains span consecutive cuts and
   the linear aggregate returns. The report (§4 preamble) now states
   exactly this; keep the two documents consistent.
3. The current implementation is separately and deliberately slower
   (visibility-graph geodesic O(|R|³), quadratic LinSep searches);
   that is the perf pass's business, catalogued in work.md, and is
   NOT what the report discusses.

Proof-adjacent tasks:
- Track, per lemma, what it implies for subchain sizes. If arc-order
  cuts make truncation sound whenever the cut exists, sentinels fire
  only for missing cuts (separator misses the polygon: bridge
  landings, opposite-anchored separators). Then bound how many
  separators can miss a polygon: D1's was l_0 and D3's was l_(m−1);
  conjecture that interior separators of the common-subpath region
  terminate into the opposite wall, which would confine sentinel
  blowup to O(1) subproblems and restore the linear aggregate
  without the two-path construction. Prove or refute.
- Construct an adversarial family realising Ω(n²) aggregate for the
  current rules (spiral or comb channel with Θ(n) P-anchored
  separators), or improve the bound. The report currently claims
  only the O(n²) upper bound and says tightness is open — resolve in
  whichever direction and update it.

## Open questions

- Does the arc-order cut redefinition make end-shielding
  unconditional? (Evaluate before writing anything.)
- Is property (ii) ever needed, or does connectedness of the kept
  chain suffice for Step 2?
- Can the witness-transfer argument cascade (shielding fails for i
  and the neighbour's OTHER side also truncates)? Construct or
  refute; the σ harness at high counts is the fastest oracle —
  30 s per 100k pairs.
- Confinement when the sight lies partly ALONG a bridge (endpoints
  can touch bridges at contacts).

## Session log (2026-09-01, proof session 1)

Facts established so far, to build on; nothing below is written up
yet. The trace harness is `mod proof_trace` at the end of mod.rs
(ignored test; run with `--run-ignored all --no-capture`), to be
removed before the final squash.

### Structure of the channel (no general-position assumption needed)

- Hull P-chain = the P–P edges of CH(P ∪ Q) from p_t to p_b (CCW along
  the hull); P′ = the CCW arc of ∂P from p_b to p_t; P″ the other arc.
  Cyclic-order argument (Jordan curve inside a convex disk meets the
  boundary in matching cyclic order): P′ ∩ ∂H = {p_b, p_t}, all other
  hull contacts of P lie on P″. P′ itself cannot be a hull edge
  (orientation: P would lie outside H). Symmetric for Q.
- No point of Q lies on the hull P-chain: a Q point in the open P–P
  hull edge [u, u′] would put Q inside the pocket bounded by [u, u′]
  and the P″-arc between u and u′ (Q is connected and avoids ∂P), but
  q_b lies on the bridge, outside that pocket. Symmetric for P.
- Hence H = K_P ∪ R ∪ K_Q with disjoint interiors, where K_P is bounded
  by the hull P-chain and P′ (two disjoint chords P′, Q′ of the disk H
  with non-interleaved endpoints), and ∂Q ∩ K_P = ∅, ∂P ∩ K_Q = ∅.
- Confinement then follows: p ∈ int(K_P) or on the hull P-chain forces
  the sight to exit K_P through P′ or through p_b/p_t (visibility
  violated), since near a hull-chain point not in {p_b, p_t} the sets
  K_P and H coincide. Same argument keeps the open sight out of
  int(K_P), int(K_Q). If the open sight touches an open bridge, the
  whole sight lies along the bridge (segment in a convex set touching
  the boundary at an interior point), so the pair IS a bridge:
  (p_b, q_b) or (p_t, q_t).
- p_b, p_t, q_b, q_t are hull vertices, hence convex in R: b_0 = p_b
  and t_{m−1} = p_t are unextended. Sub 0 has start sentinels on both
  sides so covers (p_b, q_b); sub m−1 ends at p_{m−1}⁺ = p_t on P
  (t_{m−1} = p_t classified P) and at the sentinel q_t on Q, so covers
  (p_t, q_t). The bridge pairs are done.

### Separation

- Chord-separation principle: a segment l ⊂ cl(R) with endpoints
  b, t ∈ ∂R separates the two open arcs of ∂R \ {b, t} within cl(R)
  (theta-curve: a path joining them would separate b from t, but l
  joins b and t avoiding the path). Touch points of l with ∂R in its
  interior do not matter for this statement.
- Applied to the geodesic π itself (a simple arc in cl(R) from p_b to
  p_t): EVERY sight P′ → Q′ meets π, not merely the separator union.
  Step 2 of the skeleton needs neither property (ii) nor the greedy
  structure — only that π ⊂ ∪ l_i.
- l_i ∩ ∂R ⊆ {b_i, v_w, v_{w+1}, t_i} ∪ (s_w when the path segment is
  a ring edge): extension spans are open chords of R.

### Cautions for the shielding step

- The implementation classifies a hit landing exactly on a contact
  vertex as Bridge (the bridge edge has the lower index in the scan),
  i.e. sentinel. Sentinels enlarge slices, BUT the wrap override is
  not monotone under enlargement (a cut moving inside the slice turns
  the whole-ring override off). So either prove the exact rule, or
  first prove the override never fires (conjectured for arc-order
  cuts) and then use monotonicity freely.
- [A94] Lemma 1 hypothesis is "pq ∩ l_i ≠ ∅ and pq ∩ l_{i−1} = ∅", not
  "i minimal". [A92] Lemma 9 splits l_i at q⁺(i) into a lower part
  l(l_i) = l_i⁻ q⁺(i) and upper part u(l_i) = q⁺(i) p⁺(i), argues the
  lower part gives q⁺(i−1) < q < q⁺(i) and the upper part gives
  q < q⁺(i+1) via property (v); orders are vertex-index orders.
- Whether path vertices on P′ (resp. Q′) appear in ascending arc
  order is NOT yet established; a local tautness analysis at a turn
  was inconclusive. Try to avoid needing it.

### Traces (from the harness)

- D1: kept l_0 win 0, fwd edge 2 (top bridge); l_1 win 1, back edge 1
  (Q arc), t_1 = p_t unextended. Sub 0 = (full P′, full Q′); sub 1 =
  ([(0,−2),(2,0)], full Q′). σ = 3.6 from both.
- D3: kept l_0 = (2,−4)→(5.609,10.435), fwd edge 5 = P′ edge
  (t_0 ∈ P); l_1 = (1.467,−2.756)→(7,12), back edge 6 = P′, t_1 = p_t.
  Sub 0 = (P′ up to (5.609,10.435), full Q′), σ = 1.910 = oracle;
  sub 1 = ([(5.609,10.435),(7,12)], full Q′), 2.466.
- D4: kept l_0 = (0,0)→(−2.4615,−4.9231), fwd edge 1 = Q′ (t_0 ∈ Q);
  l_1 = (−7.333,−4.111)→(2,−1), back edge 1 = Q′, t_1 = p_t. Sub 0 =
  (full P′, [q_b, q_0⁺]), 3.268; sub 1 = ([A2,(2,−1)], full Q′),
  3.1236 = oracle. Confirms the witness-transfer picture in Step 4.

## Session log 2: empirical shape of the theorem

Harness `coverage_hypotheses` (ignored hegel test in `mod proof_trace`;
`HEGEL_TEST_CASES=50000 cargo nextest r -p geo coverage_hypotheses
--run-ignored all --no-capture`). For each random non-pinched pair it
takes the realising pair (p, q) by brute force and checks which rule
assigns it to a covering subproblem. Findings over 50k cases:

- Coverage always holds, under height-order AND arc-order cuts.
- Arc-order cuts change the subchains in about 10% of cases but change
  NO assignment outcome: the candidate simplification is a wash for
  the proof and is not worth an implementation change. Dropped.
- Touching must be tested with a tolerance: computed extension
  endpoints round, so `intersects` misses exact path-vertex touches.
- Let i* = min{i : l_i ∩ [p, q] ≠ ∅} (closed sight, tolerant). Then
  EITHER sub i* covers, OR sub i*+1 covers — no third case in 50k.
- In every i*-failure the sight meets l_{i*} only at an endpoint
  v ∈ {p, q}, and v is a path vertex (the one "meets open" report was
  a near-collinear sight, angle ~1e-5 rad, ill-conditioned
  intersection point; exact arithmetic puts it in the endpoint case).
  Note v need not be shared with the KEPT l_{i*+1}: in one case the
  next extended segment through v was skipped by the greedy, and
  sub i*+1 still covered because P_{i*+1} starts at p_{i*}⁺ = v
  (b_{i*} ∈ P) and Q_{i*+1} took both sentinels.
- The sight never touches non-consecutive kept separators.
- "Open" i° (min separator meeting the OPEN sight) is the wrong
  notion: a sight from p_b crosses l_1 in its interior but touches l_0
  at p_b, and only sub 0 covers. The paper's closed-intersection
  hypothesis is the right one.
- Wrap override fires ~1 in 10k, in exact configurations (a later
  separator's backward extension lands on Q′ below an earlier one's
  top landing). In the inspected case the pair was already covered by
  subs i*−1 and i*+1; whether the override is ever NEEDED is being
  measured (200k run with the override disabled; see hyp200k.txt in
  the session scratchpad, or rerun).

Target theorem (to prove):
  For a visible pair with i* as above: (p, q) ∈ P_{i*} × Q_{i*}, or
  [p, q] ∩ l_{i*} = {v} with v ∈ {p, q} a path vertex and
  (p, q) ∈ P_{i*+1} × Q_{i*+1}.
Mechanism of the second case (D4 pattern): say v = p ∈ P′. Then
b_{i*} … the sight leaves v into the wedge ABOVE l_{i*}'s top arm, so
q lies beyond q_{i*}⁺ when t_{i*} ∈ Q′. P_{i*+1} starts at p_{i*}⁺,
which equals v because l_{i*}'s arm beyond v is an open chord of R
(no further P′ point) — this needs b_{i*} ∈ P′, to be shown in this
configuration; Q_{i*+1} starts at the sentinel q_b (b_{i*} ∉ Q′) and
its end is q_{i*+1}⁺ or q_t, above q in either case (to be shown).

200k run: 0 exceptions to the i*/i*+1 rule; wrap override fired 15
times and in every one coverage held WITHOUT the override (both by
some sub, and by i* or i*+1). The override is dead code for coverage;
keep it only as belt-and-braces or remove after the proof lands.

## Session log 3: structural lemmas (proved, to be written up)

Notation: run_i = l_i ∩ π, the maximal collinear run of path segments
through s_{w(i)}, from u_i⁻ (bottom vertex) to u_i⁺ (top vertex);
lower arm [b_i, u_i⁻), upper arm (u_i⁺, t_i]. "Facing up the channel"
P′ is on the left and Q′ on the right (R is CCW with P′ traversed
descending). The path turns LEFT at P′ vertices and RIGHT at Q′
vertices.

- Lemma S (straightness, = [A92] Thm 3 remark "no extended l_i can
  intersect any unextended l_j except at its endpoints"): l_j ∩ π =
  run_j. Proof: a point y ∈ l_j ∩ π gives a straight segment in cl(R)
  from y to the run, which is a shortest path; geodesics in a simple
  polygon are unique, so π between them IS that segment. Corollary:
  two kept separators meet at a point that is either their shared
  path vertex or lies on arms of both, never elsewhere on π.
- Sight ∩ π is connected (same uniqueness argument: a sight is the
  geodesic between its endpoints). So every sight has a P-part in one
  P-pocket and a Q-part in one Q-pocket, meeting π at one point z (or
  along a collinear piece).
- Lemma O (ordering): the P′ vertices of π occur in increasing arc
  order; likewise Q′. Purely topological (holds for any simple arc
  from p_b to p_t in cl(R)). Proof: take a violating pair v_a, v_c
  (a < c in path order, v_c below v_a) with the arc interval [v_c,
  v_a] containing no other P′ vertex of π (exists by finiteness).
  J = π[v_a, v_c] ∪ P′[v_c, v_a] is a simple closed curve bounding
  D_A; σ₁ = π[p_b, v_a] and σ₂ = π[v_c, p_t] avoid int(D_A) (they
  would have to exit through J). σ₂ leaves v_c into the wedge next to
  the DOWNWARD P′ edge at v_c, so a point y of P′ just below v_c and
  p_t are joined in cl(R) \ (σ₁ ∪ π[v_a, v_c]); but σ₁ ∪ π[v_a, v_c]
  is connected, contains p_b and v_c, and y, p_t lie on different arcs
  of ∂R \ {p_b, v_c}. Contradiction (refined theta principle: a path
  in cl(R) \ K, K connected, joins boundary points only within one
  component of ∂R \ K).
- Pockets: cl(R) \ π = P-pockets (between consecutive P′ vertices of
  π, bounded by that P′ arc and the path between; the intermediate
  path vertices are Q′ vertices) and Q-pockets (symmetric; the bottom
  and top Q-pockets also contain the bridges; if π never touches Q′
  there is a single Q-pocket). p_b, p_t are P′ vertices of π, so no
  P-pocket contains a bridge.
- Lemma A (arm sides / landing dichotomy): at a P′ turning vertex both
  arms (forward arm of the incoming segment, backward arm of the
  outgoing one) enter the adjacent Q-pocket and land on its Q′ arc or
  a bridge; at a Q′ vertex both arms enter the adjacent P-pocket and
  land on P′ (never a bridge). Consequently:
    t_i ∈ P′  ⇔  u_i⁺ ∈ Q′, or t_i = p_t (i = m−1);
    t_i ∈ Q′ ∪ bridge  ⇔  u_i⁺ ∈ P′ \ {p_t};
    b_i ∈ P′  ⇔  u_i⁻ ∈ Q′, or b_i = p_b (i = 0);   etc.
  Bridge landings occur only from P′ vertices whose adjacent Q-pocket
  is the bottom or top one. The truncation conditions of the amended
  rules are therefore conditions on which WALL the run's end vertex
  sits on.
- Property (ii) HOLDS for the greedy output: any j > w(i+1) has
  l_j ∩ l_{w(i)} = ∅ by maximality of w(i+1). So kept non-consecutive
  separators are disjoint; the spine Σ = [p_b,x_0] ∪ [x_0,x_1] ∪ … ∪
  [x_{m−2},p_t] (x_i ∈ l_i ∩ l_{i+1}) is a simple arc.
- Skips stay inside one pocket: if w(i+1) > w(i)+1 then u_i⁺ and
  u_{i+1}⁻ are same-wall vertices on one pocket's boundary path, the
  skipped segments have both arms in that pocket, and the region T_i
  bounded by the skipped chain, [u_i⁺, x_i] and [x_i, u_{i+1}⁻] is a
  convex polygon (the chain turns away from the pocket; checked all
  turn signs).
- Cut identification: if t_i ∈ P′ then p_i⁺ = t_i (the top of l_i).
  If t_i ∉ P′ then p_i⁺ is the top P′ point of l_i, namely u_i⁺ (∈ P′
  in that case) — the only P′ points of l_i are among b_i, u_i⁻, u_i⁺,
  t_i and an edge the run lies along.
- Refined theta principle (used throughout): for connected K ⊂ cl(R)
  and a path in cl(R) \ K between boundary points y, y′, the points y
  and y′ lie in the same component of ∂R \ K. With K = U_{<i} =
  l_0 ∪ … ∪ l_{i−1} (connected, ∋ p_b): for i = i*, p and q lie in the
  same component of ∂R \ U_{<i}.

### Start-side lemma (proved for i = i*; P side, Q symmetric)

Claim: if b_{i−1} ∈ P′ then p is above p_{i−1}⁺ in arc order.
b_{i−1} ∈ P′ means u_{i−1}⁻ ∈ Q′ and the lower arm of l_{i−1} lies in
the P-pocket Π′ whose boundary path contains u_{i−1}⁻.
(a) p below b_{i−1}: the component of ∂R \ U_{<i} containing p is a
    sub-arc of P′(p_b, b_{i−1}) (both endpoints in U_{<i}), which has
    no Q′ point; q cannot share it. Contradiction with i = i*.
(b) b_{i−1} < p < p_{i−1}⁺. Case u_{i−1}⁺ ∈ P′: then u_{i−1}⁺ is the top
    P′ vertex of Π′ and p_{i−1}⁺ = u_{i−1}⁺; the region D′ bounded by
    P′[b_{i−1}, u_{i−1}⁺] and the chord [b_{i−1}, u_{i−1}⁻] ∪
    [u_{i−1}⁻, u_{i−1}⁺] ⊂ l_{i−1} contains the open sight near p and
    meets Q′ only at u_{i−1}⁻ ∈ l_{i−1}; the sight must exit D′
    through P′ (visibility) or l_{i−1} (i*). Case u_{i−1}⁺ ∈ Q′: then
    t_{i−1} ∈ P′ = p_{i−1}⁺ and l_{i−1} is a chord of Π′ from b_{i−1}
    to t_{i−1} through the run; same D′ argument with P′[b_{i−1},
    t_{i−1}].
Hence p ≥ p_{i−1}⁺ (strictly, as the sight avoids l_{i−1}). ∎

### End side: in progress

If t_i ∉ P′ the P end is the sentinel: nothing to show. If t_i ∈ P′
(u_i⁺ ∈ Q′ or i = m−1): need p ≤ t_i, or the exceptional case. The
upper arm [u_i⁺, t_i] is a chord of the P-pocket Π″ containing u_i⁺,
splitting it into a lower part (boundary path from the pocket's
bottom P′ vertex up to u_i⁺; P′ from that vertex to t_i) and an upper
part (boundary path from u_i⁺ to the pocket's top P′ vertex; P′ from
t_i up to it). A sight with p ∈ P′ above t_i starts in the upper
part. l_i ∩ cl(Π″) = the upper arm plus the run (on the boundary path
below u_i⁺) plus, if u_i⁻ ∈ Q′, the lower arm too. The D4 exception
is the mirror image on the Q side: u_i⁺ ∈ P′ (= A2), t_i ∈ Q′, the
sight starts AT p = u_i⁺ and leaves into the upper part of the
Q-pocket, touching l_i only at p. Next: finish the case analysis of
where the sight can meet l_i from the upper part (cross the arm into
the lower part; touch at u_i⁺; or leave the pocket through the
boundary path above u_i⁺ and meet l_i's lower arm on the other side),
and show each is either impossible for i = i* or is the exceptional
case covered by i*+1.

### End-side lemma (proved for i = i*; stated for the P side)

Assume t_i ∈ P′ with i < m−1 (so u_i⁺ ∈ Q′; for i = m−1 the end is
p_t and there is nothing to show). Let Π″ = Π(v_a, v_c) be the
P-pocket whose boundary path contains u_i⁺; the upper arm A⁺ =
(u_i⁺, t_i] is a chord of Π″ landing on P′(v_a, v_c). It splits Π″
into D_hi (bounded by P′[t_i, v_c], A⁺, π[u_i⁺, v_c]) and the part
below A⁺ (which, if u_i⁻ ∈ Q′, is further split by the lower arm into
D_lo and D_mid, the latter bounded by P′[b_i, t_i] and the whole
chord l_i). Suppose p > t_i.

Case I, p ∈ P′(t_i, v_c): the P-part of the sight starts in D_hi and
stays in cl(Π″) until it meets π (sight ∩ π is connected).
 (a) It cannot cross A⁺: the region on the far side has boundary
     P′ ∪ l_i only (its path boundary is the run ⊂ l_i, or the chord),
     so the sight would have to meet the line of l_i twice.
 (b) It cannot touch A⁺ without crossing (that forces collinearity
     and a wall point in its interior).
 (c) If it leaves D_hi through π(u_i⁺, v_c) it enters a Q-pocket whose
     boundary path lies at or above u_i⁺, while the lower arm of l_i
     lies in the Q-pocket whose path ENDS at u_i⁺ (if u_i⁻ ∈ P′) or
     inside Π″ below A⁺ (if u_i⁻ ∈ Q′); the run lies below u_i⁺ on π.
     So it never meets l_i: contradiction with i = i*.
 (d) Remaining: the sight ends at q = u_i⁺ and touches l_i only there.
     THE EXCEPTIONAL CASE. Then sub i+1 covers:
     - P_{i+1} starts at p_i⁺ = t_i (if b_i ∈ P′) or p_b: both < p.
     - P_{i+1} ends at p_t unless t_{i+1} ∈ P′ (u_{i+1}⁺ ∈ Q′), in which
       case l_{i+1}'s upper arm is a chord of D_hi from u_{i+1}⁺ to
       t_{i+1} ∈ P′(t_i, v_c). No skip: the sight approaches q from the
       corner of D_hi at q; if p were above t_{i+1} it would have to
       cross that arm, but the sight already meets the line of l_{i+1}
       at q. Skip (u_{i+1}⁻ > u_i⁺, both Q′ on Π″'s path, l_{i+1}'s lower
       arm crossing A⁺ at x_i): the sight must enter the convex region
       T_i through (x_i, u_{i+1}⁻) ⊂ l_{i+1}, so it cannot also cross
       the upper arm; hence p ≤ t_{i+1}.
     - Q_{i+1} starts at q_b (b_i ∉ Q′) or at q_i⁺ = u_i⁺ = q (b_i ∈ Q′,
       the only Q′ points of l_i being b_i and u_i⁺, with b_i below u_i⁺
       in arc order because it lands in the pocket whose top is u_i⁺).
     - Q_{i+1} ends at q_t or at t_{i+1} ∈ Q′, which lands in the
       Q-pocket adjacent to u_{i+1}⁺ ∈ P′, whose bottom Q′ vertex is
       ≥ u_i⁺ = q in path order, hence in arc order (Lemma O).
Case II, p ≥ v_c: the P-part lies in a pocket at or above v_c and the
Q-part in a Q-pocket with path ≥ v_c; no piece of l_i is reachable
except along π, and a sight containing a path vertex in its interior
violates visibility. The one survivor is p = v_c with the sight equal
to the path segment [u_i⁺, v_c] (so q = u_i⁺): again the exceptional
case, and sub i+1 covers by the same four checks (no skip is possible
because v_c is on the other wall from u_i⁺).

Touch at the BOTTOM run vertex (sight meets l_i only at u_i⁻ = p,
possible only after a skip, when u_i⁻ ∉ l_{i−1}): the sight leaves
u_i⁻ into the skip pocket X on the side of l_i away from the convex
region T_{i−1} (entering T_{i−1} forces a meeting with l_{i−1} or a
crossing of π), i.e. into the region between the chord l_i and
Q′(b_i, t_i); so q ∈ Q′(b_i, t_i) and, staying on the upper side of
l_{i−1}'s chord of X, q > q_{i−1}⁺. Sub i covers. So the exceptional
case is exactly a touch at u_{i*}⁺ — matching the harness (i*+1 always
covers, never needed for a u⁻ touch).

Collinear sights (sight ⊂ l_i): a sight cannot contain a run vertex in
its interior, so it is one of [b_i, u_i⁻], the run, [u_i⁺, t_i] with
endpoints on different walls; each is handled by the same lemmas
(the run case has i* = i−1 when u_i⁻ is the shared vertex).

Bridge pairs (p_b, q_b) and (p_t, q_t): sub 0 and sub m−1 directly.

### Theorem (final form)

Non-containing, non-pinched. For a visible pair (p, q), let
i* = min{i : l_i ∩ [p, q] ≠ ∅}. Then (p, q) ∈ P_{i*} × Q_{i*}, except
when [p, q] ∩ l_{i*} = {u_{i*}⁺} with u_{i*}⁺ ∈ {p, q}, in which case
(p, q) ∈ P_{i*+1} × Q_{i*+1}. The wrap override is never invoked by
this argument (dead for coverage; confirmed by 200k cases with it
disabled). Property (ii) holds for the greedy output and IS used (the
spine is simple; U_{<i} connected). Arc-order cuts: not needed; the
proof uses height-order cuts as implemented, identified via Lemma A
as t_i, u_i⁺, or the top wall point of l_i.

Complexity remark for the report: sentinels fire exactly when the
run's end vertex is on the SAME wall as the subchain (Lemma A), i.e.
when l_i does not terminate into that wall. Nothing here bounds the
number of such separators; the O(n²) statement stands.
