# Polygonal coverage support: implementation plan

Port of the JTS `org.locationtech.jts.coverage` package (also present in GEOS as
`geos::coverage`, which is a direct port of the JTS code) to the `geo` crate.
JTS source surveyed at `/Users/sth/dev/jts` (1.20.x, package
`modules/core/src/main/java/org/locationtech/jts/coverage/`).

A polygonal coverage is a set of polygons that may share boundaries but do not
overlap, with shared boundaries represented by identical vertex sequences in
both polygons. The JTS package provides four user-facing operations plus a
repair tool:

| Operation | JTS entry point | What it does |
|---|---|---|
| Validate | `CoverageValidator` / `CoveragePolygonValidator` | Reports boundary linework where adjacent polygons overlap, are misaligned, or (optionally) leave narrow gaps |
| Union | `CoverageUnion` | Fast union that exploits coverage topology: shared edges cancel, no noding or intersection computation |
| Find gaps | `CoverageGapFinder` | Finds fully-enclosed narrow gaps (holes in the union narrower than a given width) |
| Simplify | `CoverageSimplifier` (TPVW) | Simplifies boundaries while preserving coverage topology: shared edges simplify identically, no new crossings |
| Clean | `CoverageCleaner` (JTS 1.20) | Repairs an invalid coverage by snapping, dissolving, polygonizing and merging |

## Scope

In scope: validate, union, find gaps, simplify. Out of scope for this effort:
`CoverageCleaner` – it requires a snapping noder, a line dissolver, a
polygonizer and RelateNG, none of which exist in geo; it is a separate, much
larger project and is deferred.

## Existing geo machinery to reuse

The core finding from surveying `geo/src` is that the validator and simplifier
need very little that geo does not already have. The reuse map, JTS dependency
to geo equivalent:

| JTS dependency | geo equivalent |
|---|---|
| `STRtree` (bulk-loaded) | `rstar::RTree::bulk_load` – already the established pattern (`GeometryGraph::new`, `RStarEdgeSetIntersector`) |
| `RobustLineIntersector` | `line_intersection()` (`algorithm/line_intersection.rs`) – documented as producing identical results to JTS `RobustLineIntersector`; `LineIntersection::SinglePoint { is_proper }` and `Collinear` cover the cases `InvalidSegmentDetector` distinguishes. A small helper is needed to classify "interior intersection" (single intersection point that is not an endpoint of one segment) |
| `Orientation.isCCW` | `Winding` trait (RobustKernel-backed) |
| `Kernel` predicates | `GeoNum::Ker` / `RobustKernel::orient2d` (Shewchuk predicates via the `robust` crate) |
| `MCIndexSegmentSetMutualIntersector` | `rstar::RTree` over target segments with payload (ring id, segment index), envelopes expanded by the gap-width tolerance, queried per adjacent segment. `monotone_chain` primitives exist if profiling later justifies a chain-level index |
| `IndexedPointInAreaLocator` | `MonotonicPolygons` / `MonoPoly` (`algorithm/monotone/`) – implements `CoordinatePosition`, so strict-interior tests (`Inside`, excluding `OnBoundary`) are available; build lazily per adjacent polygon exactly as JTS's `CoveragePolygon` does. `coord_pos_relative_to_ring` as the simple fallback |
| `Triangle.area` / `Triangle.intersects` | `Triangle::unsigned_area` / `Intersects<Coord>` |
| `LinkedLine` (doubly-linked vertex list) | Same structure as `simplify_vw.rs`'s `adjacent: Vec<(i32, i32)>` index-linked list – reimplement locally (about 60 lines), do not depend on `simplify_vw` internals |
| `VertexSequencePackedRtree` (with `remove(index)`) | `rstar::RTree` mutated incrementally – the exact pattern already used by `visvalingam_preserve_indices` (`tree.remove` / `tree.insert` per accepted removal) |
| `PriorityQueue<Corner>` | `BinaryHeap` with reversed `Ord` via `total_cmp`, stale-entry invalidation – same design as `simplify_vw.rs`'s `VScore` min-heap |
| `MaximumInscribedCircle.isRadiusWithin` | Negative buffer: a hole is a gap iff `hole_polygon.buffer(-gap_width / 2)` is empty (erosion by r is empty iff no inscribed disc of radius r exists). `Buffer` already supports negative distances. Avoids implementing maximum inscribed circle |
| `overlayng.CoverageUnion` (`BoundaryChainNoder` + `OverlayNG`) | New boundary-chain implementation (see below); `unary_union` as the correctness oracle in tests |
| `CoordinateArrays.removeRepeatedPoints` etc. | `RemoveRepeatedPoints`, `LineString` utilities |
| `HashMap` keyed on `Coordinate` / normalized `LineSegment` | Hash/BTree keys built on float bit patterns / `GeoNum::total_cmp`; precedent: the private `TotalOrdCoord` in `validation/polygon.rs` (promote a shared version into the coverage module or a crate-internal util) |
| Union-find (gap/connectivity grouping, if needed) | Private `UnionFind` in `validation/polygon.rs` – promote to a crate-internal util if required |

Considered and rejected:

- Adapting `SimplifyVwPreserve` directly for the coverage simplifier. It has no
  concept of per-edge tolerance, pinned node endpoints, or a cross-edge index
  shared between separately-simplified edges, and its removal semantics (demote
  the preceding point on intersection) differ from TPVW's (skip the corner).
  The TPVW kernel is ported fresh, borrowing `simplify_vw`'s data-structure
  patterns (min-heap of scored corners, incremental R-tree) rather than its code.
- Reusing `stitch.rs` for union. Its kernel (odd-occurrence edge cancellation,
  ring chaining, containment-based nesting) is the right shape, but it is
  deprecated, triangle-gated at the API, and O(n²) in three places. The union
  implementation below uses the same ideas with hashing and an R-tree.
- Building on the `relate` `GeometryGraph`. It computes edge intersections but
  never materialises split edges and has no ring builder, so it provides
  nothing the coverage decomposition needs; coverage algorithms deliberately
  avoid noding altogether.

## Proposed API

New module `geo/src/algorithm/coverage/` (directory with `mod.rs`), registered
in `algorithm/mod.rs`. Export the module itself; do not glob re-export its
functions into the prelude, to avoid collisions (`union` vs `BooleanOps::union`).
Access is `geo::coverage::{...}`.

All operations take `&[Polygon<T>]` where `T: GeoFloat`, matching JTS's
`Geometry[]` with elements restricted to simple polygons in v1. (JTS allows
MultiPolygon elements; support can be added later behind the same functions via
a sealed trait if there is demand.) Free functions with options structs where
parameters exceed two, per crate convention:

```rust
// validate.rs
/// Per-element invalid boundary linework, parallel to the input; None = valid.
pub struct CoverageValidation<T: GeoFloat> { /* Vec<Option<MultiLineString<T>>> */ }
impl<T: GeoFloat> CoverageValidation<T> {
    pub fn is_valid(&self) -> bool;
    pub fn invalid_boundaries(&self) -> impl Iterator<Item = (usize, &MultiLineString<T>)>;
}
pub fn validate<T: GeoFloat>(coverage: &[Polygon<T>]) -> CoverageValidation<T>;
pub fn validate_with_gap_width<T: GeoFloat>(coverage: &[Polygon<T>], gap_width: T) -> CoverageValidation<T>;

/// Validate one polygon against its adjacent neighbours (JTS CoveragePolygonValidator).
pub fn validate_polygon<T: GeoFloat>(target: &Polygon<T>, adjacent: &[Polygon<T>]) -> Option<MultiLineString<T>>;

// union.rs
#[derive(Debug, thiserror::Error)]
pub enum CoverageUnionError {
    EmptyInput,
    InvalidCoverage,   // boundary chains do not close into rings
}
pub fn union<T: GeoFloat>(coverage: &[Polygon<T>]) -> Result<MultiPolygon<T>, CoverageUnionError>;

// gaps.rs
pub fn find_gaps<T: GeoFloat>(coverage: &[Polygon<T>], gap_width: T)
    -> Result<MultiPolygon<T>, CoverageUnionError>;

// simplify.rs
pub struct CoverageSimplifyOptions<T: GeoFloat> {
    pub tolerance_inner: T,
    pub tolerance_outer: T,
    pub smooth_weight: T,               // [0, 1], default 0
    pub removable_ring_size_factor: T,  // default 1.0; 0 disables ring removal
}
pub fn simplify<T: GeoFloat>(coverage: &[Polygon<T>], tolerance: T) -> Vec<Polygon<T>>;
pub fn simplify_with_options<T: GeoFloat>(coverage: &[Polygon<T>], options: CoverageSimplifyOptions<T>) -> Vec<Polygon<T>>;
pub fn simplify_per_element<T: GeoFloat>(coverage: &[Polygon<T>], tolerances: &[T]) -> Result<Vec<Polygon<T>>, /* length-mismatch error */>;
```

Return-type conventions follow CLAUDE.md: errors/None for degenerate input, no
third-party types in signatures, most precise geometry type available (the
validator returns `MultiLineString` because invalid runs are genuinely
multi-segment, not guaranteed two-point).

## Internal design, per component

### 1. Topology decomposition: `CoverageRingEdges` (internal)

The shared foundation for the simplifier (and reusable by a future cleaner).
Decomposes a coverage into unique node-to-node `CoverageEdge`s and remembers,
per ring, the ordered edge list so polygons can be rebuilt after edge mutation.
Pure hashing – no spatial index needed:

- Nodes are the union of three sources (port faithfully from JTS):
  vertices in >= 3 rings (`VertexRingCounter`), boundary vertices with > 2
  incident boundary segments, and per-ring boundary/inner status flips.
  Boundary segments are those occurring an odd number of times
  (`CoverageBoundarySegmentFinder`).
- Edge dedup uses JTS's canonical `LineSegment` keys (lowest endpoint plus
  first distinct inward point; whole-ring edges use the extremal-vertex rule).
  Replicate the key construction exactly, including its quirks, since the
  test fixtures pin the resulting edge ordering.
- Rebuild logic (`buildCoverage`): edge direction inference, removed-ring
  handling (empty coordinate array as the removal signal), primary-ring
  guarantee (the largest polygon's shell per element is never removable).

Exact float equality (bit-level, via `total_cmp`-keyed maps) is correct here by
design: coverage semantics require exactly matching vertices on shared edges.

### 2. Union: boundary-chain cancellation

JTS routes through OverlayNG with a custom `BoundaryChainNoder`; geo has no
overlay graph, but for polygonal coverages the boundary chains are already a
complete noded description of the result boundary, so we skip the overlay:

1. Toggle-insert every normalized segment into a hash set: present = remove,
   absent = add. Surviving segments (odd occurrence count) are the coverage
   boundary; shared interior edges cancel. O(n) with hashing.
2. Mark surviving segments per source ring; extract maximal runs as chains.
   Split chains at node points (chain endpoints plus interior vertices shared
   by more than one chain) to handle self-touching topology (touching holes).
3. Assemble rings by chaining endpoint-matched chains (hash map from start
   coordinate to chains, walking with leftmost-turn selection at multi-way
   nodes), then determine nesting: R-tree over rings, signed area/winding for
   shell vs hole classification, `coord_pos_relative_to_ring` for parent
   assignment. Parity of containment depth decides exterior vs interior –
   the same scheme as `stitch.rs` but indexed instead of O(r²).
4. Any failure to close a ring, or inconsistent nesting, returns
   `CoverageUnionError::InvalidCoverage`. Note: this detects less than
   OverlayNG's `TopologyException` (which JTS tests rely on); the docs will
   state that `union` on an invalid coverage returns an error or garbage, and
   that `validate` should be run first when input validity is unknown.
   Tests compare output against `unary_union` as an oracle.

### 3. Validator

Faithful port of `CoveragePolygonValidator`'s four phases:

1. Segment matching: double-normalized segment keys (coverage orientation,
   then comparison order) in a hash map; same-slot collisions mark both
   segments invalid (duplicate/overlapping edges).
2. Interacting segments: R-tree over target segments (envelopes expanded by
   `gap_width`); per candidate pair, `InvalidSegmentDetector` logic – collinear
   overlap, proper or interior intersection, endpoint-node interior test via a
   port of `PolygonNodeTopology::isInteriorSegment` (small, orientation
   predicates only – part of the core port), and the nearly-parallel projection
   heuristic when `gap_width > 0`.
3. Interior segments: sectioned ring walk (stride 1000) against lazily-built
   `MonotonicPolygons` locators for adjacent polygons; strict-interior hits
   mark the segment and its predecessor.
4. Invalid-line extraction: wrap-around runs of invalid segments per ring.

`validate` (whole coverage) is an R-tree over element bounding rects; each
element is validated against neighbours within `gap_width`. Neighbour lookup
excludes the target by index, not by geometric equality (JTS uses identity
removal, which a value-comparison port would get wrong for duplicate
geometries). Two known JTS quirks to resolve deliberately, with comments:
the `isKnown()` whole-ring short-circuit uses AND where OR was intended (dead
code in practice – we implement the OR version), and large gap widths can
mask narrow gaps (documented JTS behaviour; keep and document).

### 4. Gap finder

`union(coverage)`, extract holes, wrap each hole as a polygon, keep it if
`buffer(-gap_width / 2)` is empty. Same documented limitations as JTS: only
fully-enclosed gaps are found; gaps that split the coverage, and gores, are
invisible.

### 5. Simplifier

Two layers, ported fresh (see rejection of `SimplifyVwPreserve` above):

- TPVW kernel (internal, testable standalone on `MultiLineString` inputs,
  mirroring `TPVWSimplifierTest`): per edge, a min-heap of corners scored by
  smoothing-weighted triangle area (`CornerArea`: JTS 1.20 semantics including
  the angle-based weight); an index-linked vertex list; a shared R-tree of
  edge envelopes plus per-edge vertex R-trees for the corner-triangle
  emptiness test; endpoint pinning for non-free-ring edges (the node
  preservation invariant); ring minimum of 4 vertices, open-edge minimum of 2;
  the 2-point-baseline-edge guard; small-ring removal via
  `removable_ring_size_factor`. Determinism: heap ties broken by corner index,
  edges processed in array order (order-dependent by design, as in JTS).
- `coverage::simplify*`: decompose via `CoverageRingEdges`, assign per-edge
  tolerance (minimum of adjacent elements' tolerances; inner/outer split via
  edge ring-count; tolerance 0 = untouched), run the kernel, write back,
  rebuild.

## Implementation phases

Each phase is a reviewable unit (jj commit series, stacked PRs if desired,
using manual base-ref chaining per the stacked-prs setup for this repo).
Phases 2–4 depend on 1 only where noted; validator (phase 4) is independent of
2 and 3.

1. Module skeleton and topology decomposition. `algorithm/coverage/mod.rs`
   with module-level prose docs (what a coverage is, references to the JTS
   package and Ramsey's FOSS4G material); internal `CoverageEdge`,
   `CoverageRingEdges`, node detection, rebuild; shared internals
   (total-ordered coord key, canonical segment key). Tests: port
   `CoverageRingEdgesTest` verbatim (names preserved, `wkt!` fixtures,
   source noted).
2. Union. Boundary-chain cancellation, chain extraction and noding at
   touch points, ring assembly and nesting; `CoverageUnionError`. Tests:
   port `CoverageUnionTest` (both the coverage-package and overlayng-level
   suites), plus oracle comparisons against `unary_union` using
   `relate(...).is_equal_topo()`.
3. Gap finder. Thin layer over union + negative buffer. Tests: port
   `CoverageGapFinderTest`.
4. Validator. `CoverageRing` (internal), segment matching,
   `InvalidSegmentDetector`, `PolygonNodeTopology::is_interior_segment`
   helper, interior-segment phase, invalid-line extraction; whole-coverage
   wrapper with R-tree neighbour query. Tests: port
   `CoveragePolygonValidatorTest` and `CoverageValidatorTest` (the richest
   suites; include the empty-polygon and duplicate-point edge cases).
5. Simplifier. TPVW kernel with standalone tests ported from
   `TPVWSimplifierTest`; then `CoverageSimplifyOptions` and the public
   functions; tests ported from `CoverageSimplifierTest` (noop, inner/outer,
   per-element tolerances, ring removal, smooth weight).
6. Docs and integration. `lib.rs` doc listing (new `## Coverage` subsection
   under Algorithms), `CHANGES.md` entry, rustdoc examples for each public
   function, benches in `geo-benches` (union vs `unary_union` is the headline
   comparison).

No changes to `jts-test-runner` are needed: the JTS coverage package has no
XML tests – its suites are JUnit with inline WKT, which transliterate directly
into Rust unit tests.

## Risks and open questions

- Exact-equality semantics. All coverage algorithms assume shared boundaries
  have bitwise-identical vertices. This is inherent to the model (JTS is the
  same) but must be prominent in the module docs, with `validate` as the
  advertised way to check inputs and a pointer to `unary_union` for
  non-coverage inputs.
- Union error detection is weaker than JTS's (no OverlayNG consistency check).
  Mitigated by documentation and the validate-first recommendation; if this
  proves insufficient, an area-sum sanity check (result area vs sum of input
  areas within a tolerance) is cheap to add.
- `f32` support comes with `GeoFloat`; robust predicates already promote to
  `f64` internally. Test fixtures are `f64`; add a smoke test for `f32`.
- Parallelism: `validate` is trivially parallel per element; a rayon path
  under the existing `multithreading` feature is a possible follow-up, not in
  v1.
- MultiPolygon coverage elements (JTS parity) deferred; the internal
  decomposition already models per-element polygon lists, so the public
  surface can grow without rework.
