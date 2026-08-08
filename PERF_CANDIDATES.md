# Performance optimisation candidates

Ranked candidates for performance work in geo, identified by a profiling
campaign over realistic workloads. This document records evidence only; no
optimisations have been implemented.

Method: 24 profiling scenarios (see `geo-benches/src/scenarios.rs` and
`geo-benches/PROFILING.md`) were run for 12 s each under samply (1000 Hz CPU
sampling) using the `profiling` build profile (release optimisation plus debug
symbols). Allocation counts come from a counting global allocator in the
`profiling` binary; allocator CPU share and its callers come from samples whose
leaf frames are in `libsystem_malloc`. Environment: Apple M2 Pro, macOS 15.6.1,
rustc 1.97.1. Percentages are self-time shares of the scenario's samples unless
stated otherwise. Reproduce with:

```sh
cargo build --profile profiling -p geo-benches --bin profiling
samply record ./target/profiling/profiling <scenario> --seconds 12
```

A deferred second phase (valgrind/gungraun on Linux) can confirm these with
deterministic instruction counts before and after any fix.

Each candidate has a tracking card on the
[geo performance project board](https://github.com/orgs/georust/projects/4);
the candidate names below link to the corresponding card.

## Summary ranking

| # | Candidate | Key evidence | Effort | Confidence |
|---|-----------|--------------|--------|------------|
| 1 | [`hypot` in distance inner loops (Hausdorff, Fréchet, concave hull)](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497533) | 86% / 83% / 13% of samples in libm `hypot` | S | High |
| 2 | [Relate/geomgraph allocation churn](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497542) | 29% of relate samples in the allocator | M–L | High |
| 3 | [Constrained Delaunay interior filter](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497549) | 53% of CDT time in per-triangle point-in-polygon | M | High |
| 4 | [Buffer superlinear scaling on MultiPolygon](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497557) | 8.6 s and 810 MiB per buffer of a 27k-coord fixture | M (investigation) | Medium |
| 5 | [`SimplifyVwPreserve` R-tree drain churn](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497563) | 42% in rstar `DrainIterator`, 11% allocator | M | High |
| 6 | [geo→i_overlay conversion overhead in boolean ops](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497573) | 12–21% self time in geo's wrapper layer | S–M | High |
| 7 | [`coord_pos_relative_to_ring` as the universal point-in-polygon kernel](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497587) | 99.7% of unindexed contains; hot in three other scenarios | M | Medium |
| 8 | [Concave hull inner loop (`line_segment_distance` + `intersects`)](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497601) | 24% + 13% self time, plus `hypot` | S–M | High |
| 9 | [Sweep-line `line_intersection` kernel cost](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497610) | 93% of sweep time in the intersection kernel | S–M | Medium |
| 10 | [Polygon–polygon Euclidean distance: sort dominates fast path](https://github.com/orgs/georust/projects/4?pane=issue&itemId=225497616) | ~51% of samples in `ProjectedVertex` sorting | M | Low–Medium |

## Candidates

### 1. `hypot` in distance inner loops

- Workload: `hausdorff-norway-louisiana` (8.9k x 2.4k coords, 41.7 ms/iter),
  `frechet-vw` (592 µs/iter), `concave-hull-norway` (20 ms/iter).
- Evidence: Hausdorff spends 68.3% of samples in `libsystem_m` plus 17.6% in
  the `hypot` dyld stub – roughly 86% of the algorithm is `hypot` calls.
  Fréchet: 77.4% + 5.8%. Concave hull: 9.4% + 3.6%.
- Hypothesis: `HausdorffDistance` (geo/src/algorithm/hausdorff_distance.rs:33)
  is a naive O(n·m) fold calling `Euclidean.distance` per coordinate pair, and
  that distance is `delta.x.hypot(delta.y)`
  (geo/src/algorithm/line_measures/metric_spaces/euclidean/distance.rs:34).
  `hypot` is an order of magnitude slower than `sqrt(dx*dx + dy*dy)` and is
  called where only comparisons matter: min/max folds are monotonic under
  squaring, so squared distances suffice until the final result.
- Proposed fix: compare squared distances in the inner loops and take one
  square root at the end (Hausdorff, Fréchet's `DiscreteFrechetCalculator`,
  concave hull's distance comparisons). Separately, Hausdorff's O(n·m) could
  later be reduced with spatial pruning, which is a larger change.
- Expected gain: 3–8x on Hausdorff and Fréchet; 10–20% on concave hull.
- Effort: S. Confidence: high.

### 2. Relate/geomgraph allocation churn

- Workload: `relate-nl-plots` (adjacent nl_plots polygon pairs, 1.76 ms/iter,
  18k allocations/iter), also visible in `validation-nl-zones` (19 µs/iter,
  173 allocations/iter, 22.2% allocator share) and `relate-jts`
  (729k allocations/iter, 250 MiB/iter).
- Evidence: 28.7% of `relate-nl-plots` samples are allocator work. Top
  callers: `RawVecInner::finish_grow` 7.0% (unreserved `Vec` growth), rstar
  bulk-load per relate call 2.9%, `RelateOperation` internals 2.0%,
  `BTreeMap` entry churn in `EdgeEndBundle` 1.3% + 1.1%,
  `GeometryGraph::new` 1.2%, `Rc<RTree>` drops 1.0%. A further 16.7% + 4.0%
  of self time is rstar `IntersectionIterator` traversal.
- Hypothesis: every `relate` call builds a full `GeometryGraph`
  (geo/src/algorithm/relate/geomgraph/) with per-edge `Rc<RefCell<Edge>>`,
  BTreeMaps for node/edge-end bundles, and a freshly bulk-loaded rstar tree,
  then drops it all. For small inputs the graph construction dominates the
  actual topology computation.
- Proposed fix, in increasing ambition: pre-reserve vectors sized from input
  coordinate counts; reuse a scratch `GeometryGraph` allocation across calls
  where the API permits (e.g. inside `is_valid`'s repeated relates); replace
  per-edge `Rc<RefCell<...>>` and BTreeMaps with index-based arenas
  (`Vec<Edge>` + integer handles). The last of these is a large refactor but
  removes both allocation and pointer-chasing costs.
- Expected gain: 15–30% on relate of small-to-medium geometries; validation
  and prepared-geometry construction benefit too.
- Effort: M–L. Confidence: high (allocator share is directly measured).

### 3. Constrained Delaunay interior filter

- Workload: `triangulate-cdt-nl-zones` (15.1 ms/iter).
- Evidence: 52.5% of samples in `coord_pos_relative_to_ring`; only ~10% in
  spade's actual triangulation.
- Hypothesis: `constrained_triangulation`
  (geo/src/algorithm/triangulate_delaunay.rs:327) filters the outer
  triangulation by running a full point-in-polygon test on every triangle
  centroid; each test scans the entire ring, giving
  O(triangles x ring length).
- Proposed fix: classify triangles by connectivity instead – flood-fill from
  a face known to be outside, crossing only non-constraint edges (the
  constraint edges are exactly the polygon boundary), which is O(triangles).
  Alternatively index the polygon once (monotone chains) and query that.
- Expected gain: around 2x on constrained triangulation of realistic
  polygons.
- Effort: M. Confidence: high.

### 4. Buffer superlinear scaling on MultiPolygon

- Workload: `buffer-nl-zones` (27k coords across many polygons, distances
  0.001 and 0.01): 8.6 s and 2.1M allocations / 810 MiB per iteration.
  Contrast `buffer-norway` (single 4k-coord LineString): 20 ms per iteration.
- Evidence: 91% of samples inside i_overlay's segment splitter
  (`CrossSolver::cross` 52.1%, `SplitSolver::tree_split` 26.7%,
  `SplitSolver::cross` 11.8%). geo-side frames are negligible.
- Hypothesis: `Buffer for MultiPolygon`
  (geo/src/algorithm/buffer.rs:402) converts all rings and hands them to a
  single i_overlay `outline` call. A 6.6x increase in input size over the
  norway scenario costs roughly 430x more time, so something in the offset +
  split pipeline behaves superlinearly on many-ring inputs. It is unclear how
  much is inherent (offset rings overlap heavily at distance 0.01) versus
  avoidable.
- Proposed fix: first characterise – buffer each polygon separately and
  `unary_union` the results, and sweep the buffer distance, comparing against
  the single-call approach; reduced arc resolution in `BufferStyle` should
  also be checked. If per-polygon + union wins at scale, adopt it (or make it
  a documented strategy); if not, file the evidence upstream against
  i_overlay.
- Expected gain: potentially an order of magnitude on large multi-part
  inputs, unconfirmed.
- Effort: M (investigation first). Confidence: medium.

### 5. `SimplifyVwPreserve` R-tree drain churn

- Workload: `simplify-vw-norway` (8.9k coords, 12.5 ms/iter, 29k
  allocations/iter).
- Evidence: 42.1% of samples in rstar `DrainIterator` removal (24.4% + 17.7%),
  9.5% allocator, 7.4% binary-heap pop, 4.3% `choose_subtree` (re-insertion).
  The non-index variant `visvalingam_preserve_indices` itself is only 20%.
- Hypothesis: the topology-preserving variant maintains an R-tree of segments
  and, per simplification step, drains and re-inserts entries; rstar's drain
  is expensive and allocates.
- Proposed fix: batch updates to the tree, or replace the R-tree with a
  simpler structure for this access pattern (the recently added `_idx`
  variants already changed part of this pipeline – measure those first).
- Expected gain: 20–40% on `simplify_vw_preserve`.
- Effort: M. Confidence: high.

### 6. geo→i_overlay conversion overhead in boolean ops

- Workload: `boolean-ops-nl-zones` (13.4 ms/iter), `boolean-ops-asia`
  (5.7 ms/iter).
- Evidence: 11.6% self time in `boolean_op_with_fill_rule` on nl-zones; 21.4%
  self time in `unary_union` on asia – i.e. a fifth of the operation is
  geo's wrapper, not the overlay engine.
- Hypothesis: every operation copies every ring via `ring_to_shape_path`
  (geo/src/algorithm/bool_ops/i_overlay_integration.rs:68) into fresh
  `Vec<BoolOpsCoord>`s. In `unary_union` (geo/src/algorithm/bool_ops/mod.rs:259)
  there is additionally an intermediate per-geometry
  `collect::<Vec<_>>()` that exists only to satisfy the `winding_order`
  closure borrow.
- Proposed fix: restructure `unary_union` to determine the winding order
  first and build the subject with a single pre-reserved collection;
  `BoolOpsCoord` is a newtype over `Coord`
  (i_overlay_integration.rs:14) – making it `#[repr(transparent)]` would
  allow ring slices to be reinterpreted without copying (behind a safe
  wrapper), removing the per-op copy entirely.
- Expected gain: 5–15% on boolean operations; more on many-geometry unions.
- Effort: S–M. Confidence: high.

### 7. `coord_pos_relative_to_ring` as the universal point-in-polygon kernel

- Workload: `contains-grid-nl-zones` (99.7% of samples), `knn-hull-norway`
  (19.4%), `triangulate-cdt-nl-zones` (52.5%, see #3), validation paths.
- Evidence: this single function
  (geo/src/algorithm/coordinate_position.rs) is the top or near-top frame in
  four scenarios.
- Hypothesis: unindexed point-in-polygon is a linear ring scan per query;
  algorithms and users repeatedly pay it. Indexed alternatives exist
  (`PreparedGeometry`, `IntervalTreeMultiPolygon`, monotone chains) but hot
  internal call sites (see #3) and default code paths do not use them.
- Proposed fix: audit internal callers that query many points against the
  same ring and route them through an index; micro-optimise the scan itself
  (branch-light crossing count over coordinate pairs) since any residual gain
  multiplies across the crate.
- Expected gain: workload-dependent; 10–30% for point-in-polygon-heavy uses.
- Effort: M. Confidence: medium.

### 8. Concave hull inner loop

- Workload: `concave-hull-norway` (20 ms/iter, 33k allocations/iter).
- Evidence: 23.8% in `geo_types::private_utils::line_segment_distance`, 13.2%
  in `Line::intersects`, 13% in libm `hypot` (see #1), 5% + 4.2% + 3.0% in
  rstar selection/nearest-neighbour/insert, 5.7% allocator.
- Hypothesis: per candidate edge the algorithm computes segment distances
  (which call `hypot` twice per invocation,
  geo-types/src/private_utils.rs:86,125) and runs intersection tests against
  the working hull, with R-tree insert/remove churn per accepted point.
- Proposed fix: squared-distance comparisons in `line_segment_distance`
  call sites where only ordering matters; reduce per-step R-tree mutation
  (batch or defer removals).
- Expected gain: 20–40% on concave hull.
- Effort: S–M. Confidence: high.

### 9. Sweep-line `line_intersection` kernel cost

- Workload: `sweep-crossings-1k/4k/16k` (7.5 ms / 119 ms / 1.93 s per iter).
- Evidence: 93% of samples in `geo::algorithm::line_intersection::line_intersection`
  at 16k lines; the sweep machinery itself is 7%. Timing scales ~16x per 4x
  lines, consistent with the quadratic growth of the true crossing count for
  long random lines (output-bound), not with a defect in the sweep.
- Hypothesis: the per-pair cost of the full robust intersection kernel
  (geo/src/algorithm/line_intersection.rs) dominates; a cheaper
  bbox/orientation rejection before the robust computation, and returning
  early for the common proper-intersection case, would cut per-pair cost.
- Proposed fix: profile the kernel in isolation (Linux/callgrind would give
  exact per-call instruction counts); add a fast-path rejection and only
  fall back to robust predicates when signs are inconclusive – mirroring the
  existing kernel pattern elsewhere in the crate.
- Expected gain: up to 2x on intersection-heavy sweeps, uncertain.
- Effort: S–M. Confidence: medium.

### 10. Polygon–polygon Euclidean distance: sort dominates fast path

- Workload: `euclidean-distance-norway-louisiana` (144 µs/iter).
- Evidence: ~51% of samples in sorting `ProjectedVertex` (quicksort +
  smallsort frames); 14.9% in the `Distance` impl; 10.4% in
  `has_disjoint_bboxes`; 3.9% in `separable_geometry_distance_fast` itself.
- Hypothesis: the project-and-sort fast path for disjoint geometries
  (geo/src/algorithm/line_measures/metric_spaces/euclidean/distance.rs:586)
  fully sorts both projected vertex sets although the subsequent pruned
  search typically consumes only a prefix. `has_disjoint_bboxes` at 10% for
  a guard function also merits a look.
- Proposed fix: lazy ordering (binary-heap or incremental selection) instead
  of full sorts; verify the guard is not recomputing bounding rects.
- Expected gain: unclear – possibly 20–30% of this fast path; it is already
  fast in absolute terms (recently optimised in #1560).
- Effort: M. Confidence: low–medium.

## Not candidates (checked, fine)

- `centroid` / `bounding_rect` on nl_zones: ~47 µs, single tight
  compute-bound frame, no allocation. Nothing to do.
- `triangulate-earcut`: 84% inside the external `earcut` crate; geo's wrapper
  overhead is ~3%.
- `boolean-ops` core: dominated by i_overlay internals (upstream), except for
  the conversion layer (#6).
- `make-valid`: dominated by spade CDT insertion and rstar nearest-neighbour
  (external); geo's `repair_polygon` frames are ~1.5%. The checkerboard case
  runs at 441 µs/iter.
- `minimum-rotated-rect`: 27% in `quick_hull` plus rotating calipers; already
  fast (334 µs on nl_zones) after #1464's optimisation.

## Appendix A: scenario timings and allocations (12 s runs)

| Scenario | Time/iter | Allocations/iter | MiB/iter |
|---|---|---|---|
| boolean-ops-nl-zones | 13.4 ms | 837 | 9.6 |
| boolean-ops-asia | 5.7 ms | 3,379 | 9.7 |
| relate-jts | 317 ms | 728,548 | 249.7 |
| relate-nl-plots | 1.8 ms | 18,064 | 4.5 |
| buffer-nl-zones | 8.6 s | 2,145,981 | 809.8 |
| buffer-norway | 20.2 ms | 2,425 | 24.8 |
| make-valid-checkerboard | 441 µs | 876 | 0.4 |
| make-valid-norway-zigzag | 15.0 ms | 27,912 | 10.6 |
| triangulate-earcut-nl-zones | 2.8 ms | 1,276 | 6.9 |
| triangulate-cdt-nl-zones | 15.1 ms | 10,661 | 6.3 |
| concave-hull-norway | 20.0 ms | 33,388 | 8.0 |
| knn-hull-norway | 6.9 ms | 10,387 | 4.9 |
| sweep-crossings-1k | 7.5 ms | 10 | 0.1 |
| sweep-crossings-4k | 119 ms | 12 | 0.3 |
| sweep-crossings-16k | 1.93 s | 14 | 1.1 |
| contains-grid-nl-zones | 147 ms | 0 | 0.0 |
| euclidean-distance-norway-louisiana | 144 µs | 2 | 0.2 |
| hausdorff-norway-louisiana | 41.7 ms | 0 | 0.0 |
| frechet-vw | 592 µs | 1 | 0.0 |
| validation-nl-zones | 19 µs | 173 | 0.0 |
| simplify-vw-norway | 12.5 ms | 28,852 | 16.3 |
| centroid-nl-zones | 47 µs | 0 | 0.0 |
| bounding-rect-nl-zones | 48 µs | 0 | 0.0 |
| minimum-rotated-rect-nl-zones | 334 µs | 16 | 1.2 |

## Appendix B: notes on the existing criterion suite

Observations made while selecting workloads; the criterion suite was not the
basis for candidate identification.

- `relate.rs`'s "overlapping 50-point polygons" is too small to expose the
  allocation-dominated behaviour visible at nl_plots scale; its JTS-suite
  bench is a good end-to-end workload but mixes overlay and relate costs
  (i_overlay frames dominate its profile).
- The sweep benches use random synthetic lines; results are output-bound
  (crossing count), so scaling comparisons must normalise by the number of
  intersections reported.
- `Buffer`, `MakeValid`, `HausdorffDistance`, `KNearestConcaveHull`,
  `centroid`, and `bounding_rect` had no benchmarks at all; scenarios now
  cover them, and the hottest (`Buffer`, Hausdorff) produced top-ranked
  candidates.

## Appendix C: deferred Linux phase

A second pass on a Linux machine with valgrind-backed gungraun would add deterministic
instruction counts, cache simulation, and DHAT heap profiles; useful to confirm #2, #6, and #9 and as
before/after evidence when fixes are attempted. The scenario functions were
written to be wrapped directly by a gungraun bench target.
