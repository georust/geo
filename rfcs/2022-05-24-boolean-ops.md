- Feature Name: `boolean-ops`
- Start Date: 2022-05-24
- [Feature PR]

# Summary

Boolean Ops. refer to constructive set-theoretic operations applied on
geometries; for eg., union, intersection. This RFC implements boolean-ops
on 2D geometries based on the algorithm of [Martinez-Rueda-Feito].

# Implementation Break-down

The algorithm consists of:

1. planar sweep: the classic algorithm of [Bentley-Ottman] which
   efficiently computes all intersections of a set of lines.
2. ring construction: compute the borders of the output region as disjoint
   set of rings.
3. stitching: construct the output polygons by detecting exterior rings,
   and assigning hole rings to its parent ext. ring.

## Planar Sweep

Here, we use ideas from [Martinez-Rueda-Feito] to combine overlapping
line-segments. This helps us simplify many edge-cases. The implementation
is in `geo/src/algorithm/sweep`.

## Ring Construction

This is essentially as in [Martinez-Rueda-Feito]. The region of each
intersection-point is inferred from the sweep data-structure, and this is
used to compute whether each segment starting at the point belongs to the
border of the region of interest (which is the output of the bool. op).
Pls. ref. `geo/src/algorithm/bool_ops`.

The border regions are then connected greedily into a set of rings. Ref.
`geo/src/algorithm/bool_ops/rings.rs`.

## Stitching Rings

The region enclosed by the rings form a [Laminar Set]. We re-use the
planar-sweep to again detect the inclusion relations of the rings, and
assemble as polygons.  Ref `geo/src/algorithm/bool_ops/laminar.rs`.

# Ideas for Future Work

1. JTS test suites for boolean ops.
1. Custom b-tree to optimize sweep ops.
1. Robust sweep to handle all fixed-precision issues.

[Martinez-Rueda-Feito]: //dx.doi.org/10.1016/j.cageo.2008.08.009
[Bentley-Ottman]: //en.wikipedia.org/wiki/Bentley%E2%80%93Ottmann_algorithm
[Feature PR]: //github.com/georust/geo/pull/835
[Laminar Set]: //en.wikipedia.org/wiki/Laminar_set_family
