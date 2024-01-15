- Feature Name: `boolean-ops`
- Start Date: 2022-05-24
- [Feature PR]

# Summary

Boolean Ops. refer to constructive set-theoretic operations applied on
geometries; for eg., union, intersection. The implementation supports
boolean-ops on 2D geometries based on the algorithm of
[Martinez-Rueda-Feito] as well as extensions to clip a 1D geometry by a 2D
geometry.

# Implementation Break-down

The algorithm consists of:

1. idenfifying border of the output region: this uses the [Bentley-Ottman]
   planar sweep algorithm.
1. stitching: construct the output polygon(s) by forming exterior and
   interior rings from the line-segments.

## Planar Sweep

This is a classical comp. geom. algorithm and may be of independent use. In
fact, the boolean op. implementation uses the sweep twice: once for
identifying the border regions, and a second time to identify line-segments
to pair-up together to form the rings.

The current implementation uses the vanilla `BTreeSet` but a tailored
version of `BTreeSet` should improve performance here. Pls. check out
[rmanoka/sweep-tree] for a working poc that gives some decent speed-up
(requires a nightly compiler).

The implementation is not tied specifically to the boolean op. algorithm.
However, we use the idea of merging overlapping line-segments from the work
of [Martinez-Rueda-Feito]. This may be different from typical
implementation of this algorithm, but provides the same functionality.

Pls. refer `geo/src/algorithm/sweep` for the implementation.

## Identifying Borders

This is the crux of the boolean op. logic. The sweep splits the input
segments into segments that do not intersect in interiors. The sweep also
provides a list of "active segments" which are ordered list of segments
intersecting the current sweep line. This ordering allows us to infer the
region above/below each segment, and thus the segments belong in the output
goemetry.

Pls. refer `geo/src/algorithm/bool_ops/op.rs`.

## Operation Specification

The initial implementation focused on operation between two 2-d
geometries. However, the core logic to use the sweep to split segments, and
calculate regions based on segment arrangement lends itself to perform
boolean operations in multiple flavours.

The general spec. of a boolean operation is extracted as the `Spec` trait.
This trait captures the calculation of regions from the segments and their
ordering, as well as the construction of the final output.

Specifically, for operations between two 2D geometries, the outputs
segments are whose either side have different parity with the output region
(i.e. one side of the segment belongs to the output, and the other does
not). These are collected and used to assemble the final geometry. This is
captured in the `BoolOp` struct that implements the above trait.

We also support clipping of a 1D geometry by a 2D geometry. This is the
same as intersection but between geometries of different dimensions. In
this case, we calculate region solely from the 2D geometry, and only output
segments from the 1D geometries. These are then assembled to output a
`MultiLineString` in this case.  This is captured in the `ClipOp` struct.

Pls. ref `geo/src/algorithm/bool_ops/spec.rs`.

## Output Construction

Here, we construct a final geometry by assembling the segments obtained
from the previous section. For `BoolOp`, these segments are guaranteed to
represent a bounded region, and thus can be decomposed into a set of cycles
(the eulerian graph condition). The only constraint is to ensure the
output satisfies the validity constraints of the OGC SFS.

For `ClipOp`, we simply assemble segments that belonged to the same
original `LineString` via a greedy algorithm to assemble a final list of 1D
geometries.

Pls. ref `geo/src/algorithm/bool_ops/assembly.rs`.

# Ideas for Future Work

1. Custom b-tree to optimize sweep ops.
1. Robust sweep to handle all fixed-precision issues.

[Martinez-Rueda-Feito]: //dx.doi.org/10.1016/j.cageo.2008.08.009
[Bentley-Ottman]: //en.wikipedia.org/wiki/Bentley%E2%80%93Ottmann_algorithm
[Feature PR]: //github.com/georust/geo/pull/835
[Laminar Set]: //en.wikipedia.org/wiki/Laminar_set_family
[rmanoka/sweep-tree]: //github.com/rmanoka/sweep-tree
