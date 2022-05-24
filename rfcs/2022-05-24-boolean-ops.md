- Feature Name: `boolean-ops`
- Start Date: 2022-05-24

# Summary

Boolean Ops. refer to constructive set-theoretic operations such
as union, intersection, xor applied on geometries. This RFC
implements boolean-ops on 2-D geometry based on the algorithm of
[Martinez-Rueda-Feito].

# Implementation Break-down

The algorithm another classic algorithm: the [Bentley-Ottman]
plane-sweep which efficiently computes all intersections of a set
of line-segments.

[Martinez-Rueda-Feito]: //dx.doi.org/10.1016/j.cageo.2008.08.009
[Bentley-Ottman]: //en.wikipedia.org/wiki/Bentley%E2%80%93Ottmann_algorithm
