# RelateNG follow-ups

Work identified while simplifying the RelateNG stack (#1627: #1587 – #1591,
#1599) that does not belong in that stack. Findings that were measured and
rejected are listed so that they are not repeated.

## Open

### Check the point-on-segment bounding box first

`Intersects<Coord> for Line` in `geo/src/algorithm/intersects/line.rs` runs
`orient2d` before `point_in_rect`. RelateNG reaches it per segment through
`RelatePointLocator::locate_on_line` and `AdjacentEdgeLocator`.

Swap the operands, so that most segments are rejected without the orientation
test. The `&&` operands have no side effects, so the answer does not change. The
change affects every caller, so measure it on the `geo-benches` suite. Not
measured yet.

### Use the short-circuiting macro for `MultiPolygon` containment

`geo/src/algorithm/contains/polygon.rs` has eight hand-written
`rhs.relate(self).is_within()` impls for `MultiPolygon`. They compute the full
matrix. `impl_contains_from_relate!`, already used for `Polygon` in the same
file, short-circuits, and would remove about 60 lines.

This needs #1627, which provides the short-circuiting predicates. Not measured
yet.

### Reduce the `winding_order` cost of ring extraction

`RelateGeometry::extract_segment_strings` takes 57% of `poly_poly_matrix`, a
20,000-vertex polygon against a 2,000-vertex polygon, where the boundaries do not
interact. The cost is in orientation, not in copying:

| Operation on a 20,000-vertex ring | Time (µs) |
|---|---|
| `clone` | 6.3 |
| `winding_order` | 58.7 |
| `clone_to_winding_order`, already oriented | 63.7 |
| `clone_to_winding_order`, reversed | 68.9 |

Extraction takes under 4% of every other case measured, so first confirm that
inputs of this shape matter.

### Route the remaining named predicates through public traits

`disjoint`, `within`, `covered_by`, `crosses`, `equals_topo`, `overlaps` and
`touches` in `relateng/relate_predicate.rs` have no non-test caller. They are
`#[cfg(test)]`, and `PredicateKind` carries `#[cfg_attr(not(test),
expect(dead_code))]` because the tests alone construct six of its variants.

Routing them through public traits is an API decision. A blanket impl over
`Contains` already prevents a `Within` impl for `PreparedGeometry`. When a
predicate is routed, remove its `#[cfg(test)]`. The `expect` then reports itself
when the last variant is in use.

### Decide how `relate` handles NaN

`relate` panics in debug builds on a one-coordinate line with a NaN ordinate, on
`main` and on #1627, because `NodeKey` debug-asserts against NaN. Release builds
return a matrix.

The hegel rules in `CLAUDE.md` ask for a full-domain property that asserts no
panic, and `relate` fails it for this input. Choose one of these, and record the
choice in the known-defects list in `CLAUDE.md`:

- Accept NaN and return a defined result.
- Reject NaN at the `relate` entry point.

Results on `main` and #1627 for NaN inputs:

| Input | `main`, debug | `main`, release | #1627, debug | #1627, release |
|---|---|---|---|---|
| One-coordinate NaN line, relate point | Panic | `FF0FFF0F2` | Panic | `FF1FFF0F2` |
| One-coordinate NaN line, relate line | Panic | `FF0FFF102` | Panic | `FF1FFF102` |
| Ring with a NaN first vertex, relate line | Panic | `102FF1102` | `FF2FF1102` | `FF2FF1102` |
| Ring with a NaN interior vertex, relate line | `1F2001102` | `1F2001102` | `1F2001102` | `1F2001102` |

`NodeSection` keeps `Option` vertices for this reason. A NaN ring fails
`is_closed`, so its vertices can be absent. Required vertices would add a panic.

### Port `is_interior_segment` with the coverage validator

`PolygonNodeTopology::is_interior_segment` is not in #1627, because nothing there
calls it. `COVERAGE_PLAN.md` names it, so port it with `InvalidSegmentDetector`,
together with its helper `is_between` and the JTS tests `testInteriorSegment` and
`testExteriorSegment`.

## Measured and rejected

Timed against the tip of #1627 with seven interleaved rounds, on an idle machine,
taking the minimum per case. Spread across rounds was at most 1.08x.

| Change | Best result | Reason |
|---|---|---|
| Skip `is_crossing` once the interiors are known to cross | 2.5% faster, on a star polygon built to favour it | No gain on the other 14 cases |
| `node_map` as a sorted `Vec` | 3% faster on `poly_self_matrix_shared_edges` | Too small; unchanged on most cases |
| `Cow` coordinates in `RelateSegmentString` | At most 6.3 of 63.7 µs per ring | `winding_order` dominates; lines take 0.1% in extraction |

## Not pursued

These have no measurement, because the reason does not depend on one.

- **Prepared envelope method on `Relate`:** it adds to a public trait for a small
  saving.
- **Removing the A-first swap in `EdgeSegmentIntersector`:** it diverges from JTS.
- **`intersect_all` self-join through
  `intersection_candidates_with_other_tree`:** it changes the traversal order for
  little gain.
- **`IMPatternMatcher::value_im` through `matches`:** it needs the pattern string,
  which was removed.
- **`add_line_segment` taking a `Line`, and the nested loops in
  `jts-test-runner/src/runner.rs`:** these are minor.
