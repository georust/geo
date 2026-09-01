// Report on implementing the Amato polygon-separation algorithm.
// Source of truth for the corrections document; figures are drawn
// natively from the counterexample coordinates below.

#set document(
  title: [Notes and errata from an implementation of the Amato
    polygon-separation algorithm],
  author: "Stephan Hügel",
)
#set page(paper: "a4", margin: (x: 2.6cm, y: 3cm), numbering: "1")
#set text(size: 10.5pt, lang: "en", region: "GB")
#set par(justify: true, leading: 0.62em)
#set heading(numbering: "1.1")
#show heading: set block(above: 1.4em, below: 0.8em)
#show raw.where(block: true): set text(size: 9pt)
#show figure.caption: set text(size: 9pt)
#set figure(gap: 0.9em)
#show ref: set text(fill: rgb("#1a3c6e"))
#show cite: set text(fill: rgb("#1a3c6e"))
#show link: set text(fill: rgb("#1a3c6e"))

#let scene(p, q, annots: (), width: 210pt) = {
  let pts = p + q
  for a in annots {
    if a.kind == "seg" { pts = pts + (a.a, a.b) }
    if a.kind == "pt" or a.kind == "tag" { pts = pts + (a.at,) }
    if a.kind == "region" { pts = pts + a.pts }
  }
  let xs = pts.map(v => v.at(0))
  let ys = pts.map(v => v.at(1))
  let pad = 0.9
  let x0 = calc.min(..xs) - pad
  let x1 = calc.max(..xs) + pad
  let y0 = calc.min(..ys) - pad
  let y1 = calc.max(..ys) + pad
  let s = width / (x1 - x0)
  let tx = v => ((v.at(0) - x0) * s, (y1 - v.at(1)) * s)
  box(width: width, height: (y1 - y0) * s, {
    for a in annots {
      if a.kind == "region" {
        place(polygon(fill: a.fill, stroke: none, ..a.pts.map(tx)))
      }
    }
    place(polygon(fill: rgb("#dbe7f5"), stroke: rgb("#35577d") + 0.9pt, ..p.map(tx)))
    place(polygon(fill: rgb("#f7e3cf"), stroke: rgb("#8a5a2b") + 0.9pt, ..q.map(tx)))
    for a in annots {
      if a.kind == "seg" {
        let style = a.at("style", default: "acc")
        let stroke = if style == "dash" {
          (paint: rgb("#7a7a7a"), thickness: 0.7pt, dash: "dashed")
        } else if style == "dot" {
          (paint: rgb("#35577d"), thickness: 0.9pt, dash: "dotted")
        } else if style == "sight" {
          rgb("#2a7a3b") + 1.1pt
        } else if style == "skip" {
          rgb("#9a9a9a") + 0.8pt
        } else {
          rgb("#b3273a") + 1.2pt
        }
        place(line(start: tx(a.a), end: tx(a.b), stroke: stroke))
      } else if a.kind == "pt" {
        let c = tx(a.at)
        place(dx: c.at(0) - 2.2pt, dy: c.at(1) - 2.2pt, circle(
          radius: 2.2pt,
          fill: rgb("#b3273a"),
          stroke: white + 0.6pt,
        ))
      } else if a.kind == "tag" {
        let c = tx(a.at)
        place(dx: c.at(0) + a.at("dx", default: 4pt), dy: c.at(1) + a.at("dy", default: -12pt), text(size: 8pt, a.body))
      }
    }
  })
}

#let algo(title, body) = block(
  width: 100%,
  stroke: (top: 0.8pt + black, bottom: 0.8pt + black),
  inset: (y: 0.9em, x: 0.2em),
  breakable: false,
)[
  #text(weight: "bold")[#title]
  #v(0.5em)
  #body
]

#align(center)[
  #text(size: 15pt, weight: "bold")[
    Notes and errata from an implementation of the \
    Amato polygon-separation algorithm
  ]
  #v(0.8em)
  Stephan Hügel \
  #link("mailto:shugel@tcd.ie")[shugel\@tcd.ie] \
  #v(0.3em)
  September 2026
]

#v(1em)
#block(inset: (x: 2.2em))[
  #text(weight: "semibold")[Abstract.]
  We describe what is, to our knowledge, the first implementation of the
  $Theta(n)$ polygon-separation algorithm of Amato
  @amato94, undertaken as part of the ```rust geo``` computational-geometry
  library. The implementation is validated by property-based testing
  against a brute-force oracle over randomly generated
  simple polygons. This process surfaced five configurations in which
  the decomposition step, as described in @amato94, is underspecified
  or fails to cover the pair realising the separation; each is
  perturbation-stable and arises in exact arithmetic. For each we give
  a minimal integer-coordinate counterexample, an analysis, and the
  resolution adopted, which are then collected into an
  amended statement of the subchain-construction step, with a proof
  that it assigns every visible pair to some subproblem. Most of the
  missing detail is present in the
  earlier technical report @amato92, whose axiomatic treatment of the
  separator sequence the 1994 paper compresses; two findings appear to
  go beyond both papers. We would welcome corrections to our reading.
]
#v(1em)

= Introduction

The separation $sigma(P, Q)$ of two simple polygons -- the minimum
distance between their boundaries -- can be computed in optimal
$Theta(n)$ sequential time by the decomposition algorithm of
@amato94, which reduces the problem to linearly separable subproblems
using a sequence of separating segments derived from a shortest path,
and solves each subproblem with the closest-visible-vertex machinery
of the technical report @amato92 (published in revised form as
@amato95; we cite the report, whose section numbering we follow). We have implemented the sequential algorithm in Rust for
the ```rust geo``` library#footnote[https://github.com/georust/geo].

Nothing in these notes disputes the papers' central results. Our
findings concern the decomposition step (DECOMPOSE, Step 1(b--d) and Step 2
of @amato94), whose full treatment the 1994 paper compresses from
@amato92. Our experience is that this compression omits important
detail: an implementer working from @amato94 alone will, on certain
perturbation-stable inputs, compute an overestimate of
$sigma(P, Q)$. Most of the machinery needed to repair this is present
in @amato92; in two places we believe the configurations fall outside
both papers.

Each finding was discovered by a property-based test harness
comparing the implementation against a brute-force oracle on randomly
generated simple polygons (@sec-verification). Each is presented with
a minimal integer-coordinate counterexample, simplified from the
harness's shrunk original and verified to reproduce the defect by
reverting the corresponding repair. There is precedent for such
findings in this line of work: @amato92 itself demonstrates that the
two prior published algorithms for the closest-visible-vertex problem
are incorrect on some instances -- the subproblem decomposition of
@wangchan86 "can produce subproblems whose solutions may not always
be valid solutions for the original problem", and Lemma~2.2 of
@amss89 does not hold as stated.

= The separator sequence, as defined in the 1992 report

@amato92 (Section 4) defines $S(P, Q) = (l_0, dots.h, l_(m - 1))$ as
any sequence of segments satisfying:

+ $l_i inter l_(i + 1) eq.not nothing$ (consecutive separators
  intersect);
+ $l_i inter l_j = nothing$ for $j in.not { i - 1, i, i + 1 }$
  (non-consecutive ones do not);
+ $l_i$ does not intersect the interior of $P$ or $Q$;
+ *both endpoints of $l_i$ lie on the boundaries of $P$ and/or $Q$*;
+ *each $l_i$ is maximal*: if the top endpoint of $l_i$ is on $P$ and
  the top endpoint of $l_(i + 1)$ is on $Q$, no point of $Q$ at or
  above $l_(i + 1)$'s top is visible from $l_i$, and symmetrically.

@amato92 notes that (v) implies each $l_i$ intersects the boundaries
of both $P$ and $Q$. The subchain-truncation rules of Step 2 are
sound _given_ these properties; they are theorems about sequences
satisfying (i)--(v), not about arbitrary extended geodesics.
@amato94 restates the consequence ("it is a simple matter to verify
that each $l_i$ intersects both $P$ and $Q$") without the properties;
its one-path construction establishes (i)--(iii) but not (iv) or (v)
(@app-coverage). That is the root of the
findings in @sec-findings.

A second structural difference: the construction in @amato92
(Theorem 3) computes *two* shortest paths -- $L_p$ between the two
$P$-contacts and $L_q$ between the two $Q$-contacts -- takes their
common subpath, extends each segment *one-sidedly* beyond the shared
path vertex, and determines which polygon an extension endpoint lies
on *by provenance* ("by inspecting the (unextended) segments"), not
by geometric search. The monotonicity of the cut points ("if
$l_i^+ in P$ and $l_j^+ in P$, $i < j$, then $l_i^+ < l_j^+$") is
proved from the two-path structure. @amato94 uses a single path
between the $P$-contacts and extends in both directions, and none of
those guarantees carry over.

The repairs described below share one safety principle, valid for
the separation problem but *not* for the closest-visible-vertex
problem (@sec-cvv): enlarging a subchain is always sound, because
every value a subproblem produces is a distance between boundary
points and therefore at least $sigma$, while the pair realising
$sigma$ only gains coverage.

= Findings <sec-findings>

Each finding states the claim, a counterexample (as WKT, with a
figure drawn from the same coordinates), an analysis, and the
resolution adopted in our implementation. In the figures, supporting
segments (bridges) are dashed, the geodesic is dotted, and separators
are solid red.

== A separator can miss one polygon entirely <sec-bridge>

*Claim* (@amato94, Section 3): every separator intersects both
polygons; the cut points $p_i^+$ and $q_i^+$ always exist.

*Counterexample:*

```wkt
P: POLYGON((0 2, 0 -2, 2 0, 0 2))
Q: POLYGON((-6 -2, -6 -6, -2 -5, -6 -2))
```

#figure(
  scene(
    ((0, 2), (0, -2), (2, 0)),
    ((-6, -2), (-6, -6), (-2, -5)),
    annots: (
      (kind: "seg", a: (0, 2), b: (-6, -2), style: "dash"),
      (kind: "seg", a: (-2, -5), b: (2, 0), style: "dash"),
      (kind: "seg", a: (0, 2), b: (0, -2.5)),
      (kind: "pt", at: (0, -2.5)),
      (kind: "tag", at: (0.5, 0.4), body: [$P$]),
      (kind: "tag", at: (-5.3, -4.2), body: [$Q$]),
      (kind: "tag", at: (0, -1), body: [$l_0$], dx: 4pt, dy: 0pt),
    ),
  ),
  caption: [The separator $l_0$ is the extension of the first
    geodesic segment; it terminates on a bridge and does not intersect
    $Q$, so the cut $q_0^+$ does not exist.],
) <fig-bridge>

Two triangles whose channel axis is diagonal (@fig-bridge). The
geodesic in $R$ runs along $P$'s facing arc; the extension of its
first segment leaves $P$ and lands on a *bridge* (a supporting
segment of $R$'s boundary), touching $Q$ nowhere. In exact
arithmetic, $l_0 inter Q = nothing$: no $q_0^+$ exists. The
configuration is an open set -- perturbing the vertices preserves it.
The "otherwise" branch of Step 2 (end the subchain at the next
separator's cut) then truncates the realising vertex of $Q$ out of
every subchain, and the computed separation exceeds the true value.

*Analysis.* This violates property (iv) of @amato92, which the
one-path construction of @amato94 does not establish. $R$'s boundary
consists of the two facing arcs _and the two bridges_; nothing
prevents an extension from landing on a bridge. We have not
determined whether the two-path construction of @amato92 excludes
bridge landings in its own setting, where the machinery is invoked
only for polygons whose hulls intersect; in the @amato94 setting,
where DECOMPOSE also serves hull-disjoint polygons, they occur.

*Resolution.* The truncation branches that assume the missing cut
exists are replaced by the facing-arc sentinel (@sec-truncation),
and each extension records the boundary edge it hit, so a bridge
landing is recognised exactly rather than searched for
geometrically.

== The orientation rule contradicts the path-order conventions <sec-orientation>

*Claim* (@amato94): the bottom endpoint $b_i$ of $l_i$ is "the
endpoint closest to $l_(i - 1)$", the top $t_i$ the endpoint closest
to $l_(i + 1)$.

The sentinel conventions ($p_m^+ = p_t$, and the last separator's cut
landing at the top contact) presuppose that this orientation agrees
with the direction of the path.

*Counterexample:* the triangle pair of @fig-bridge. The backward
extension of the second geodesic segment overshoots so far that the
extended segment's far endpoint is _more distant_ from $l_0$ than
its top endpoint: the closest-endpoint rule labels the endpoints in
the opposite sense to path order, $p_1^+$ becomes the wrong
intersection, and the last subchain degenerates.

*Analysis.* @amato92 does not use a distance rule. It defines the
anchors as the _shared path vertices_ ($l_i^- = l_(i - 1) inter l_i$,
$l_i^+ = l_i inter l_(i + 1)$) and extends one-sidedly, so
orientation is path order by construction. The distance formulation
of @amato94 is an operationalisation that agrees with path order in
the generic figure and diverges when an extension overshoots.

*Resolution.* Orientation is taken from path order, matching the
definition in @amato92 rather than the operationalisation in
@amato94.

== The truncation branches require property (v) <sec-truncation>

*Claim* (@amato94, Step 2): the subchain of $X$ for separator $i$
ends at $i$'s own cut when $t_i in X$ *or $i = m - 1$*, and at
separator $i + 1$'s cut otherwise.

*Counterexamples:* @sec-bridge breaks the "otherwise" branch (the
next cut does not exist, or truncates the realising vertex). The
"$i = m - 1$" branch fails on:

```wkt
P: POLYGON((2 -4, -1 3, 7 12, -4 8, -4 -4, 2 -4))
Q: POLYGON((12 4, 6 8, 4 4, 8 0, 12 4))
```

#figure(
  scene(
    ((2, -4), (-1, 3), (7, 12), (-4, 8), (-4, -4)),
    ((12, 4), (6, 8), (4, 4), (8, 0)),
    annots: (
      (kind: "seg", a: (2, -4), b: (8, 0), style: "dash"),
      (kind: "seg", a: (12, 4), b: (7, 12), style: "dash"),
      (kind: "seg", a: (2, -4), b: (4, 4), style: "dot"),
      (kind: "seg", a: (4, 4), b: (7, 12), style: "dot"),
      (kind: "seg", a: (1.4666666666666668, -2.7555555555555564), b: (7, 12)),
      (kind: "pt", at: (4, 4)),
      (kind: "pt", at: (6, 8)),
      (kind: "tag", at: (-2.2, 5), body: [$P$]),
      (kind: "tag", at: (8.2, 3.6), body: [$Q$]),
      (kind: "tag", at: (4, 4), body: [grazed cut], dx: 5pt, dy: 1pt),
      (kind: "tag", at: (6, 8), body: [realising vertex], dx: 6pt, dy: -3pt),
      (kind: "tag", at: (2.4, -1), body: [$l_(m - 1)$], dx: 4pt, dy: 0pt),
    ),
  ),
  caption: [The last separator is anchored on $P$ at both ends
    and passes through a vertex of $Q$ without terminating into $Q$'s
    wall; the realising vertex lies beyond the grazed cut, which
    shields nothing.],
) <fig-graze>

The last separator is anchored on $P$ at both ends (@fig-graze) --
the geodesic turned at a vertex of $Q$, so the extended segment
_grazes_ $Q$ at that vertex without terminating into $Q$'s wall. Its
"own cut" on $Q$ is the grazed vertex; it shields nothing, and the
realising pair (assigned to this subproblem by the coverage lemma)
lies beyond it.

*Analysis.* In @amato92 both truncations are justified by property
(v), which the grazing configuration violates: the separator is not
maximal with respect to $Q$. Geodesics generically turn at reflex
vertices of the channel boundary, and the channel boundary includes
both polygons, so separators grazing the opposite polygon are not
exotic.

*Resolution.* A subchain end stops at separator $i$'s own cut
exactly when $i$'s top endpoint lands on that polygon (established
by hit-edge provenance); a subchain start begins at separator
$i - 1$'s cut exactly when $i - 1$'s bottom endpoint lands on that
polygon. In every other situation the subchain extends to the
facing-arc sentinel. These are precisely the situations in which the
@amato92 shielding arguments go through; when the endpoint condition
holds, the cut exists, since the endpoint itself lies in
$l_i inter P$.

== Fixed-direction subchain scans can wrap <sec-wrap>

*Claim* (@amato94): subchains are fixed-direction scans between
consecutive cut points; "no point of $P$ or $Q$ appears in more than
four subproblems", so the aggregate subproblem size is $O(n)$.
(@amato92 says five, not four, for its closely related
construction.)

*Counterexample:*

```wkt
P: POLYGON((0 0, -1 -2, 2 -1, 0 0))
Q: POLYGON((-8 -4, -8 -8, 4 -6, -8 -4))
```

#figure(
  scene(
    ((0, 0), (-1, -2), (2, -1)),
    ((-8, -4), (-8, -8), (4, -6)),
    annots: (
      (kind: "seg", a: (0, 0), b: (-8, -4), style: "dash"),
      (kind: "seg", a: (4, -6), b: (2, -1), style: "dash"),
      (kind: "seg", a: (0, 0), b: (-2.4615384615384617, -4.923076923076923)),
      (kind: "seg", a: (-7.333333333333334, -4.111111111111111), b: (2, -1)),
      (kind: "pt", at: (-2.4615384615384617, -4.923076923076923)),
      (kind: "pt", at: (-7.333333333333334, -4.111111111111111)),
      (kind: "tag", at: (0.4, -0.9), body: [$P$]),
      (kind: "tag", at: (-5, -6.3), body: [$Q$]),
      (kind: "tag", at: (-2.46, -4.92), body: [$q_0^+$], dx: 2pt, dy: 3pt),
      (kind: "tag", at: (-7.33, -4.11), body: [$q_1^+$], dx: -4pt, dy: 4pt),
      (kind: "tag", at: (-1.6, -2.9), body: [$l_0$], dx: 4pt, dy: 0pt),
      (kind: "tag", at: (-5.4, -3.5), body: [$l_1$], dx: 0pt, dy: -11pt),
    ),
  ),
  caption: [$l_1$'s backward extension lands before $l_0$'s cut in
    boundary order ($q_1^+$ precedes $q_0^+$), so the fixed-direction
    subchain scan between the cuts wraps around $Q$.],
) <fig-wrap>

A later separator cuts $Q$ _below_ an earlier one (@fig-wrap): the
backward extension of the second geodesic segment lands far down the
opposite wall, before the earlier separator's cut in boundary order.
The pair realising $sigma$ touches $l_0$, so the coverage lemma
asserts it lies in the first subproblem, whose $Q$-subchain ends at
$q_0^+$; but while the pair's $Q$-point lies below $q_0^+$ in the
height order along the separator, which the lemma's proof uses, it
lies beyond $q_0^+$ in the boundary order the subchain uses. The two
orders come apart, and only a scan continuing past $q_0^+$ --
wrapping around the polygon -- reaches the point.

*Analysis.* @amato92 proves the cut points ascend, but from the
two-path construction; under the one-path construction of @amato94,
descending cuts occur. When they do, either the subchain wraps (and
the four-per-vertex aggregate claim no longer follows) or coverage
is lost. The slippage is between two orders the construction treats
as interchangeable: position along a separator, which the coverage
lemma's proof uses, and position along the boundary, which the
subchains use. This appears to be a gap in the complexity argument
of @amato94 for configurations its construction can produce.

*Resolution.* An earlier version of the implementation detected an
anchor cut lying outside the computed slice of the facing arc and
substituted the full polygon ring for that subchain, a sound superset.
The coverage proof (@app-remarks) shows the override is never needed:
the pairs it concerns are covered by the neighbouring subproblems, and
the slice between the cuts is now kept as it is. The
$O(n)$ aggregate bound is deliberately given up in these cases;
restoring it is a matter for adopting the @amato92 two-path
construction, whose monotonicity argument rules the wraps out.

== The single-contact ("pinched") channel <sec-pinch>

*Claim* (@amato94, Step 1(a)): the non-containing case yields a
simple polygon $R$ bounded by two facing portions and two supporting
segments, with distinct path endpoints $p_b eq.not p_t$.

*Counterexample:*

```wkt
P: POLYGON((3 2, 0 -4, 4 2, 3 2))
Q: POLYGON((0 8, 2 3, 8 4, 0 8))
```

#figure(
  scene(
    ((3, 2), (0, -4), (4, 2)),
    ((0, 8), (2, 3), (8, 4)),
    annots: (
      (kind: "seg", a: (0, -4), b: (8, 4), style: "dash"),
      (kind: "seg", a: (0, 8), b: (0, -4), style: "dash"),
      (kind: "seg", a: (0, -4), b: (4, 2), style: "dot"),
      (kind: "seg", a: (4, 2), b: (3, 2), style: "dot"),
      (kind: "seg", a: (3, 2), b: (0, -4), style: "dot"),
      (kind: "pt", at: (0, -4)),
      (kind: "tag", at: (2.5, 0.6), body: [$P$]),
      (kind: "tag", at: (3, 5.2), body: [$Q$]),
      (kind: "tag", at: (0, -4), body: [pinch vertex], dx: 6pt, dy: 1pt),
    ),
  ),
  caption: [Both supporting segments (dashed) share $P$'s single hull
    contact: the channel is pinched there, and the shortest path
    (dotted) wraps around $P$ between the two instances of the pinch
    vertex.],
) <fig-pinch>

$P$ touches $"CH"(P union Q)$ at a single vertex (@fig-pinch). Both
supporting segments share that $P$-contact, $P$'s facing portion is
its entire boundary, $R$ is weakly simple (pinched at the shared
vertex, which appears twice in its ring), and $p_b = p_t$. The
shortest path runs between the two instances of the pinch vertex,
around $P$ -- structurally the containing case's doubled cut vertex,
inside the non-containing case's framing. The linear bottom-to-top
subchain rules do not apply: the separator sequence is cyclic, as in
the containing case's "all arithmetic is modulo $m$".

*Analysis.* Neither paper discusses the configuration. It is not
excluded by the general-position assumption of @amato92 (no three of
the counterexample's vertices are collinear).

*Resolution.* Pinched channels are detected exactly (shared bridge
contact) and take full-ring subchains for every separator -- sound,
with the containing-case machinery as the eventual proper treatment.
The shortest-path and visibility machinery is index-based, so the
two instances of the pinch vertex are distinct vertices, which is
what forces paths to route around the pinched-in polygon.

= An amended construction <sec-amended>

We collect the resolutions of @sec-findings into an amended statement
of the affected steps, in the style of Algorithm 2 of @amato94. Only
the diverging parts are restated; the pointers give the finding
motivating each change. The statement is correctness-first: it
preserves the coverage of every visible pair, but weakens the
complexity guarantee. A sentinel-extended subchain runs to the end of its facing
portion, and by Lemma A4 of @app-coverage the geodesic segment
inside each separator ends on one wall, so at each end of every
subproblem one of the two subchains runs to its sentinel. Whenever
the geodesic alternates between the walls $Theta(n)$ times, the
aggregate subproblem size is therefore $Theta(n^2)$, against
$Theta(n)$ in @amato94: the quadratic bound is tight, and it is the
typical behaviour rather than a worst case. A sentinel-extended
subchain need not lie on one side of its separator either, so such
subproblems are not linearly separable and an implementation must
solve them by other means. Bounding the sentinel side by a later cut
is exactly what property (v) supplies, and securing it is the role of
the two-path construction of @amato92 (@sec-wrap). Coverage of every visible pair by
the amended rules is proved in @app-coverage and confirmed
empirically (@sec-verification).

#algo(
  [Amended DECOMPOSE($P$, $Q$), Steps 1(c)--2, non-containing
    case],
  [
    *Step 1(c).* #emph[Extend the segments of the shortest path to
      $R$'s boundary.] One ray per direction; an endpoint whose ray
    leaves $R$ immediately is not extended. Record, for each extension,
    the feature of $R$'s boundary on which it lands: an edge of $P'$,
    an edge of $Q'$, or a bridge (@sec-bridge). Orient each extended
    segment by path order: $b_i$ is the endpoint on the side of the
    path's start, $t_i$ the other (@sec-orientation).

    *Step 1(d).* #emph[Remove redundant segments.] Unchanged.

    *Step 2.* #emph[Construct the subproblems.] For each
    $l_i in S(P, Q)$ let $p_i^+$ be the highest point of intersection
    of $l_i$ with $P$ along the orientation of Step 1(c), when one
    exists (@sec-bridge), and let the
    subchain run along the facing portion $P'$. Define
    $P_i = P_(s_i, e_i)$, where

    $
      e_i = cases(
        p_i^+ & "if" t_i in P,
        p_t & "otherwise",
      ), quad
      s_i = cases(
        p_(i - 1)^+ & "if" i > 0 "and" b_(i - 1) in P,
        p_b & "otherwise",
      )
    $

    except that if the channel is pinched, then $P_i = partial P$ for
    every $i$ (@sec-pinch).

    $Q_i$ is defined symmetrically. As in @amato94,
    $sigma(P, Q) = min_(0 lt.eq i < m) sigma(P_i, Q_i)$.
  ],
)

For comparison, Step 2 of @amato94 reads

$
  P_i = cases(
    P_(p_(i - 1)^+, p_i^+) & "if" t_i in P "or" i = m - 1,
    P_(p_(i - 1)^+, p_(i + 1)^+) & "otherwise",
  )
$

with the subchain taken as a fixed-direction scan of the whole
polygon. The amendments are: the removal of the "$i = m - 1$"
disjunct (@sec-truncation); the replacement of the neighbouring cut
$p_(i + 1)^+$ by the sentinel $p_t$ (@sec-bridge, @sec-truncation);
the added start condition with its sentinel $p_b$ (@sec-truncation);
and the restriction of subchains to the facing portions, with the
pinch override (@sec-pinch) replacing the wrapping scans; the
descending cuts of @sec-wrap need no rule of their own
(@app-remarks).

= Remarks

== General position

@amato92 assumes no three vertices are collinear "for ease of
exposition", stating the algorithms remain valid without alteration.
The counterexamples above satisfy general position, so the
assumption does not account for them.

== Separation versus closest visible vertices <sec-cvv>

@amato94 correctly observes that $sigma(P_i, Q_i) gt.eq sigma(P, Q)$
holds for arbitrary subchains, which is why the separation problem
tolerates enlarged subchains: this one-sided safety underpins the
sentinel and whole-boundary resolutions above. The closest-visible-vertex problem does *not*
enjoy it: a subproblem can contain a vertex pair that is closer than
$"CVV"(P, Q)$ but not visible, and returning it is an error.
@amato92 devotes Lemma 11 and Figure 9(b) to exactly this hazard and
prunes the affected subchains. Any adaptation of the repairs above
to CVV must add that machinery; enlarging subchains is not sound
there.

== Floating-point transfer

Three further failure classes arose in transferring the real-RAM
algorithm to IEEE-754 doubles; they are implementation matters, not
errata. Exact segment-intersection tests miss computed extension
endpoints by an ulp, repaired by carrying exact provenance instead
of re-deriving contacts geometrically, an approach anticipated by the
instruction in @amato92 to inspect the unextended segments. A spurious hit of the
extension ray on an edge incident to the vertex it passes through,
at parameter $1 + epsilon$, masks the true extension through a
reflex vertex, repaired by excluding those edges from the scan by
index. Finally, exactness of Shewchuk-style robust predicates fails
when orientation products underflow (magnitudes below roughly
$10^(-323)$), which affected our test oracle rather than the
algorithm.

= Verification methodology <sec-verification>

The implementation is validated by property-based testing. Input
pairs are star-shaped simple polygons with 3--12 vertices, generated
by drawing bounded angular gaps (normalised so that every gap is
below 180#sym.degree, which guarantees simplicity) and varying
radii; pairs whose boundaries intersect are discarded. The oracle is
the $O(n^2)$ minimum over all boundary point--segment distances. The
property asserts agreement of the implementation with the oracle
within a scale-aware tolerance (relative $8 epsilon$, with an
absolute floor of $32 epsilon$ times the coordinate magnitude, where
$epsilon = 2^(-52)$ is the binary64 machine epsilon: both
sides round at that scale, and at knife-edge inputs exact pruning
decisions interact with rounded distance evaluations on either side
of the true value). At the time of writing the property holds over
300,000 generated pairs, with the individual pipeline stages covered
by their own unit fixtures and properties.

The counterexamples in this document are integer-coordinate
simplifications of the harness's shrunk originals. Each was verified
to reproduce its defect by reverting the corresponding resolution
and observing the oracle mismatch, and to pass under the unmodified
implementation; all four are in general position. Both the
simplified and the original shrunk forms are retained as regression
fixtures in the implementation's test suite.

= Closing remarks

We hope these notes are useful as a record of what a
correctness-first implementation of the separation algorithm
encounters, three decades on. We would welcome corrections to our
reading of either paper -- in particular, whether the two-path
construction of @amato92 was intended to establish properties (iv)
and (v) in the hull-disjoint setting served by DECOMPOSE, and
whether the wrapping subchain scans of @sec-wrap were considered in
the aggregate-size analysis. @app-coverage proves that the amended
rules cover every visible pair; the linear aggregate bound is the
question that remains. The implementation, including the
property harness and all fixtures, is being prepared for inclusion
in the ```rust geo``` library.


#show figure.where(kind: "lemma"): it => block(
  width: 100%,
  above: 1.1em,
  below: 0.8em,
  breakable: true,
)[
  #set align(left)
  #text(weight: "bold")[Lemma #it.counter.display(it.numbering)]#it.body
]
#let lemma(title, body) = figure(
  kind: "lemma",
  supplement: [Lemma],
  numbering: n => "A" + str(n),
  caption: none,
)[#if title != none [ #text(weight: "bold")[(#title).]] else [.] #emph(body)]
#let proof(body) = block(above: 0.6em, below: 1.1em)[
  #emph[Proof.] #body #h(1fr) $square$
]

#counter(heading).update(0)
#set heading(numbering: (..nums) => {
  let n = nums.pos()
  if n.len() == 1 { "A" } else { "A." + str(n.at(1)) }
})
#show heading.where(level: 1): set heading(supplement: [Appendix])
#counter(figure.where(kind: image)).update(0)
#show figure.where(kind: image): set figure(numbering: n => "A." + str(n))

= Coverage of the amended construction <app-coverage>

We prove that the amended Step 2 of @sec-amended assigns every
visible pair to some subproblem, in the non-containing case. The
pinch override makes every subchain a whole boundary, so coverage
there is immediate; we assume throughout that the channel is not
pinched. The argument is elementary plane topology together with
one metric fact, the uniqueness of geodesics in a simple polygon. It
does not use properties (ii), (iv) or (v) of @amato92 as hypotheses;
property (ii) is shown to follow from Step 1(d), and the places
where (iv) and (v) fail are exactly the places where the sentinels
of @sec-amended take over.

== Setting and notation

$P$ and $Q$ are simple polygons with disjoint boundaries, in the
non-containing case, with bridges $p_b q_b$ (bottom) and $q_t p_t$
(top) meeting $P union Q$ only at their contacts. The facing arc
$P'$ is the arc of $partial P$ from $p_b$ to $p_t$ that bounds $R$;
$Q'$ is the arc of $partial Q$ from $q_b$ to $q_t$ that bounds $R$.
Walking $partial R$ with $R$ on the left visits $p_b$, the bottom
bridge, $Q'$ from $q_b$ to $q_t$, the top bridge, then $P'$ from
$p_t$ back to $p_b$. _Arc order_ on $P'$ runs from $p_b$ to $p_t$,
on $Q'$ from $q_b$ to $q_t$; "above" and "below" refer to arc order
on the arc in question, never to a direction in the plane. Facing up
the channel, $P'$ is the left wall and $Q'$ the right wall. We call
$P'$ and $Q'$ the two _walls_ and say that points of $P'$ and $Q'$
lie on _opposite_ walls.

$pi = (v_0 = p_b, v_1, dots.h, v_k = p_t)$ is the geodesic in
$"cl"(R)$ from $p_b$ to $p_t$, listed with every boundary vertex it
passes through, so that consecutive vertices may be collinear. Each
$v_j$ lies on a wall; every open segment $(v_j, v_(j + 1))$ lies in
the interior of $R$ or along an edge of $R$. $pi$ is a simple arc.
Its interior vertices are reflex vertices of $R$ or grazed vertices;
at a $P'$ vertex where it turns, it turns left, at a $Q'$ vertex
right (it wraps the polygon whose vertex it passes).

For a path segment $s_j = v_j v_(j + 1)$, the _extended segment_
$l_j$ is $s_j$ prolonged in both directions while the prolongation
stays in the interior of $R$, up to the first boundary point in each
direction; a direction in which the prolongation leaves $R$ at once
is not extended. $l_j$ is oriented by path order: $b_j$ is its
endpoint on the side of $v_j$, $t_j$ on the side of $v_(j + 1)$.
The kept sequence $S(P, Q) = (l_0, dots.h, l_(m - 1))$ is the output
of Step 1(d); $w(i)$ denotes the path window of $l_i$, so
$w(0) = 0 < w(1) < dots.h < w(m - 1) = k - 1$ and $l_i$ is the
extension of $s_(w(i))$.

The _run_ of $l_i$ is $l_i inter pi$; by @lem-straight below it is
the maximal collinear stretch of $pi$ through $s_(w(i))$, a segment
from a path vertex $u_i^-$ to a later path vertex $u_i^+$. The _lower
arm_ of $l_i$ is $[b_i, u_i^-)$ and the _upper arm_ is
$(u_i^+, t_i]$; either may be empty. $x_i$ denotes a point of
$l_i inter l_(i + 1)$.

For $X in {P, Q}$ we write $x_i^+$ for the highest point of
$l_i inter X'$ along the orientation of $l_i$, when the intersection
is non-empty. The subchains are those of @sec-amended: $X_i$ is the
arc of $X'$ from $s_i^X$ to $e_i^X$ where $e_i^X = x_i^+$ if
$t_i in X'$ and $e_i^X = x_t$ otherwise, and $s_i^X = x_(i - 1)^+$
if $i > 0$ and $b_(i - 1) in X'$, $s_i^X = x_b$ otherwise; the whole-boundary override once used for @sec-wrap is discussed in
@app-remarks and is not needed. Landing
on a contact vertex counts as landing on that wall.

A pair $(p, q)$, $p in partial P$, $q in partial Q$, is _visible_ if
the open segment $(p, q)$ meets neither boundary. Its _sight_ is the
closed segment $[p, q]$.

== Preliminaries

We use one topological tool repeatedly. Let $K subset "cl"(R)$ be
closed and connected, and let $gamma$ be a path in $"cl"(R)$
avoiding $K$ whose endpoints $y, y'$ lie on $partial R$. Then $y$
and $y'$ lie in the same component of $partial R without K$. (If
not, choose $k_1, k_2 in K inter partial R$ separating $y$ from $y'$
on the circle $partial R$; a simple sub-arc of $gamma$ is a cross-cut
of the disc $"cl"(R)$ that separates $k_1$ from $k_2$, contradicting
the connectedness of $K$.) We refer to this as the _separation
principle_. Two special cases: a segment in $"cl"(R)$ with endpoints
on $partial R$ separates the two open arcs it cuts $partial R$ into;
and a sight from $P'$ to $Q'$ meets every connected set that contains
$p_b$ and $p_t$, since $P' without {p_b, p_t}$ and $Q'$ lie in
different components of $partial R without {p_b, p_t}$.

#lemma([confinement])[
  For every visible pair, $p in P'$, $q in Q'$, and the sight lies in
  $"cl"(R)$. If the open sight meets a bridge, the pair is
  $(p_b, q_b)$ or $(p_t, q_t)$.
]
#proof[
  Let $H = "CH"(P union Q)$. Its boundary consists of the two bridges
  and two monochromatic chains, the hull $P$-chain from $p_t$ to
  $p_b$ and the hull $Q$-chain from $q_b$ to $q_t$. A Jordan curve
  inside a closed convex disc meets the boundary in the same cyclic
  order as the curve itself, so the hull vertices of $P$ occur on
  $partial P$ in hull order and $P' inter partial H = {p_b, p_t}$;
  $P'$ cannot itself be a hull edge, since $P$ and $H$ traverse it in
  opposite senses. No point of $Q$ lies on the hull $P$-chain: a
  point of $Q$ interior to a hull edge $u u'$ with $u, u' in P$
  would put all of $Q$ (connected, disjoint from $partial P$) inside
  the pocket bounded by $u u'$ and the arc of $partial P$ between
  $u$ and $u'$, but $q_b$ lies on a bridge, outside that pocket.
  Hence $P'$ and $Q'$ are disjoint chords of the disc $H$ with
  non-interleaved endpoints, and $H$ is the union of $"cl"(R)$ with
  the regions $K_P$ (bounded by the hull $P$-chain and $P'$) and
  $K_Q$, with disjoint interiors, $partial Q inter K_P = nothing$
  and $partial P inter K_Q = nothing$.

  The sight lies in $H$ by convexity. If $p in "int"(K_P)$, or $p$
  lies on the hull $P$-chain, the sight must leave $K_P$ to reach
  $q in.not K_P$; near a hull-chain point other than $p_b, p_t$ the
  sets $K_P$ and $H$ coincide, so the exit point lies on $P'$ or is
  $p_b$ or $p_t$, in the open sight, contradicting visibility. Hence
  $p in P'$, and symmetrically $q in Q'$. The same exit argument
  keeps the open sight out of $"int"(K_P)$ and $"int"(K_Q)$, so it
  lies in $"cl"(R)$. A segment inside a convex set that meets the
  boundary at a relative-interior point lies in a supporting line,
  so an open sight meeting a bridge lies along that bridge, and its
  endpoints are the bridge's contacts.
]

#lemma([straightness])[
  For every $j$, $l_j inter pi$ is the maximal collinear stretch of
  $pi$ containing $s_j$. In particular an arm of $l_j$ meets $pi$
  only at its own path vertex, and two extended segments meet either
  at a shared path vertex or at a point of an arm of each.
] <lem-straight>
#proof[
  Let $y in l_j inter pi$ and let $v$ be the endpoint of $s_j$ on the
  side of $y$. The segment $[v, y] subset l_j$ lies in $"cl"(R)$, so
  it is a shortest path from $v$ to $y$; the sub-path of $pi$
  between $v$ and $y$ is also a shortest path; geodesics in a simple
  polygon are unique, so the two coincide and $pi$ is straight from
  $v$ to $y$. This is the observation of @amato92 (proof of
  Theorem 3) that no extended segment meets an unextended one except
  at endpoints.
]

The same uniqueness argument shows that a sight, being the geodesic
between its endpoints, meets $pi$ in a connected set: a point, or a
collinear segment.

#lemma([ordering])[
  The $P'$ vertices of $pi$ occur in increasing arc order along $P'$;
  likewise the $Q'$ vertices along $Q'$.
] <lem-order>
#proof[
  Suppose not. Among pairs $v_a, v_c$ of $P'$ vertices of $pi$ with
  $a < c$ and $v_c$ below $v_a$, choose one whose arc interval
  $[v_c, v_a]$ contains no other $P'$ vertex of $pi$; one exists,
  since any third vertex inside the interval yields a violating pair
  with a smaller interval. Then $J = pi[v_a, v_c] union P'[v_c, v_a]$
  is a simple closed curve; let $D$ be the region it bounds. The
  sub-arcs $sigma_1 = pi[p_b, v_a]$ and $sigma_2 = pi[v_c, p_t]$
  avoid $"int"(D)$: to enter it they would have to cross $J$, which
  is impossible along $pi[v_a, v_c]$ (simplicity) and along
  $P'(v_c, v_a)$ (no $P'$ vertex of $pi$ there). At $v_c$ the
  interior of $R$ is split by the arriving segment of $pi$ into the
  wedge on the side of $D$ and the wedge adjacent to the $P'$ edge
  running downward from $v_c$; $sigma_2$ leaves into the latter.
  Hence a point $y in P'$ just below $v_c$ and $p_t$ are joined by a
  path avoiding $K = sigma_1 union pi[v_a, v_c]$. But $K$ is
  connected, contains $p_b$ and $v_c$, and $y$ and $p_t$ lie on
  different arcs of $partial R without {p_b, v_c}$, contradicting
  the separation principle.
]

The lemma is purely topological: it holds for any simple arc from
$p_b$ to $p_t$ in $"cl"(R)$.

_Pockets._ By @lem-order the components of $"cl"(R) without pi$
are of two kinds (@fig-pockets). A _$P$-pocket_ $Pi(v_a, v_c)$ is bounded by the
arc $P'[v_a, v_c]$ between two consecutive $P'$ vertices of $pi$ and
by $pi[v_a, v_c]$, whose interior vertices all lie on $Q'$. A
_$Q$-pocket_ is defined symmetrically, except that the bottom
$Q$-pocket is bounded by $pi[p_b, v]$, the bottom bridge and
$Q'[q_b, v]$, where $v$ is the first $Q'$ vertex of $pi$, and the
top $Q$-pocket symmetrically; if $pi$ never touches $Q'$ there is a
single $Q$-pocket. Since $p_b$ and $p_t$ are $P'$ vertices of $pi$,
no $P$-pocket contains a bridge. A sight consists of a _$P$-part_
from $p$ to the point where it meets $pi$, lying in one $P$-pocket,
and a _$Q$-part_ from there to $q$, lying in one $Q$-pocket; one
part is empty when $p$ or $q$ is a path vertex.


#figure(
  scene(
    ((0, 0), (4, 3), (0, 5), (0, 10), (-3, 9), (-3, 1)),
    ((8, 0), (11, 1), (11, 9), (8, 10), (1, 7)),
    width: 250pt,
    annots: (
      (kind: "region", pts: ((0, 0), (8, 0), (1, 7), (4, 3)), fill: rgb("#f3ece4")),
      (kind: "region", pts: ((1, 7), (8, 10), (0, 10)), fill: rgb("#f3ece4")),
      (kind: "region", pts: ((4, 3), (1, 7), (0, 10), (0, 5)), fill: rgb("#e4ebf3")),
      (kind: "region", pts: ((1, 7), (0, 8.333333333333334), (0, 10)), fill: rgb("#c5d3e6")),
      (kind: "seg", a: (0, 0), b: (8, 0), style: "dash"),
      (kind: "seg", a: (8, 10), b: (0, 10), style: "dash"),
      (kind: "seg", a: (0, 0), b: (4, 3), style: "dot"),
      (kind: "seg", a: (4, 3), b: (1, 7), style: "dot"),
      (kind: "seg", a: (1, 7), b: (0, 10), style: "dot"),
      (kind: "seg", a: (0, 0), b: (4.571428571428571, 3.4285714285714284)),
      (kind: "seg", a: (6.25, 0), b: (0, 8.333333333333334)),
      (kind: "seg", a: (2, 4), b: (0, 10)),
      (kind: "seg", a: (0, 9), b: (1, 7), style: "sight"),
      (kind: "pt", at: (4.571428571428571, 3.4285714285714284)),
      (kind: "pt", at: (6.25, 0)),
      (kind: "pt", at: (0, 8.333333333333334)),
      (kind: "pt", at: (2, 4)),
      (kind: "tag", at: (-2.2, 5), body: [$P$]),
      (kind: "tag", at: (9.2, 5), body: [$Q$]),
      (kind: "tag", at: (0, 0), body: [$p_b$], dx: -14pt, dy: -2pt),
      (kind: "tag", at: (0, 10), body: [$p_t$], dx: -14pt, dy: -9pt),
      (kind: "tag", at: (8, 0), body: [$q_b$], dx: 3pt, dy: -2pt),
      (kind: "tag", at: (8, 10), body: [$q_t$], dx: 3pt, dy: -9pt),
      (kind: "tag", at: (4, 3), body: [$u_0^+$], dx: -6pt, dy: 1pt),
      (kind: "tag", at: (1, 7), body: [$u_1^+$], dx: 4pt, dy: -4pt),
      (kind: "tag", at: (4.571428571428571, 3.4285714285714284), body: [$t_0$], dx: 4pt, dy: -8pt),
      (kind: "tag", at: (6.25, 0), body: [$b_1$], dx: 3pt, dy: -13pt),
      (kind: "tag", at: (0, 8.333333333333334), body: [$t_1$], dx: -14pt, dy: -3pt),
      (kind: "tag", at: (0, 9), body: [$p$], dx: -10pt, dy: -8pt),
      (kind: "tag", at: (2, 4), body: [$b_2$], dx: 3pt, dy: 0pt),
      (kind: "tag", at: (2.6, 1.5), body: [$l_0$], dx: 0pt, dy: 0pt),
      (kind: "tag", at: (3.0, 5.0), body: [$l_1$], dx: 0pt, dy: 0pt),
      (kind: "tag", at: (0.6, 5.7), body: [$l_2$], dx: 0pt, dy: 0pt),
      (kind: "tag", at: (0.28, 9.15), body: [$D$], dx: 0pt, dy: 0pt),
    ),
  ),
  caption: [Pockets and arms. The geodesic (dotted) turns left at the
    $P'$ vertex $u_0^+ = (4, 3)$ and right at the $Q'$ vertex
    $u_1^+ = (1, 7)$, cutting the channel into a $P$-pocket (blue) and
    two $Q$-pockets (sand). The two arms at $u_0^+$ enter the bottom
    $Q$-pocket and land on $Q'$ ($t_0$) and on the bottom bridge
    ($b_1$); the two arms at $u_1^+$ enter the $P$-pocket and land on
    $P'$ ($t_1$, $b_2$). The upper arm of $l_1$ cuts the region $D$
    (dark) off the $P$-pocket. The green sight from $p = (0, 9)$ to
    $q = u_1^+$ touches $l_1$ only at $q$: it lies above the cut $t_1$
    of subproblem 1 and is covered by subproblem 2, whose subchains
    both run to their sentinels.],
) <fig-pockets>

#lemma([arms and landings])[
  Let $u$ be a vertex of $pi$ at which a run ends, other than $p_b$
  or $p_t$. Both arms leaving $u$ -- the upper arm of the extended
  segment whose run ends at $u$ and the lower arm of the one whose
  run begins there -- lie in the pocket adjacent to $u$ on the wall
  opposite to $u$'s, and land on that pocket's arc of the opposite
  wall or, if $u in P'$ and the pocket is the bottom or top
  $Q$-pocket, on a bridge. Consequently, for every $i$:
  $t_i in P'$ if and only if $u_i^+ in Q'$ or $i = m - 1$;
  $t_i$ lies on $Q'$ or on a bridge if and only if
  $u_i^+ in P' without {p_t}$; and symmetrically for $b_i$ and
  $u_i^-$.
] <lem-arms>
#proof[
  A run ends where $pi$ turns; at an interior vertex $u$ of $pi$ the
  turn wraps the polygon owning $u$, so by tautness the angle between
  the incoming and outgoing segments measured through $R$ is at least
  $180 degree$, and both prolongations -- of the incoming segment
  beyond $u$ and of the outgoing segment backwards beyond $u$ -- enter
  the interior of $R$ on the side of $pi$ away from that polygon. An
  arm is an open segment in the interior of $R$ that avoids $pi$
  (@lem-straight), so it lies in a single component of
  $"cl"(R) without pi$, the pocket adjacent to $u$ on that side, and
  its endpoint lies on the part of $partial R$ in the closure of that
  pocket other than $pi$: the pocket's arc of the opposite wall, and
  the bridges for the two end $Q$-pockets (@fig-pockets). It cannot end at a path
  vertex, again by @lem-straight. The two equivalences follow, with
  $t_(m - 1) = p_t$ and $b_0 = p_b$ unextended because the contacts
  are hull vertices and hence convex in $R$.
]

Bridge landings, which violate property (iv) of @amato92
(@sec-bridge), therefore arise only from $P'$ vertices adjacent to
an end $Q$-pocket. The truncation conditions of @sec-amended are,
by @lem-arms, conditions on the wall of the run's end vertex: the
$P$-subchain of separator $i$ ends at its own cut exactly when the
run of $l_i$ ends on $Q'$ (or $i = m - 1$), and begins at the cut of
$i - 1$ exactly when the run of $l_(i - 1)$ begins on $Q'$.

#lemma([the kept sequence])[
  (a) $l_i inter l_(i + 1) eq.not nothing$, and $l_i inter l_j = nothing$
  for $j > i + 1$: properties (i) and (ii) of @amato92 hold.
  (b) Along $l_i$ the points $b_i, x_(i - 1), u_i^-, u_i^+, x_i, t_i$
  occur in this order (with $x_(-1) = p_b$ and $x_(m - 1) = p_t$).
  (c) If $w(i + 1) > w(i) + 1$, then $u_i^+$ and $u_(i + 1)^-$ lie on
  the same wall, the path between them is a stretch of the boundary
  path of a single pocket $X$ on the opposite side, $x_i$ lies on
  the upper arm of $l_i$ and the lower arm of $l_(i + 1)$, and the
  region $T_i$ bounded by $pi[u_i^+, u_(i + 1)^-]$, $[u_i^+, x_i]$ and
  $[x_i, u_(i + 1)^-]$ is a convex polygon whose interior lies in $X$
  and whose boundary meets the walls only at path vertices.
] <lem-kept>
#proof[
  (a) Step 1(d) keeps $l_(w(i + 1))$ as the largest-indexed extended
  segment meeting $l_(w(i))$; every kept index beyond $w(i + 1)$ is
  larger, so its segment misses $l_(w(i))$. Consecutive extended
  segments share a path vertex, so the sequence is well defined.

  (b), (c) If the runs of $l_i$ and $l_(i + 1)$ are adjacent,
  $x_i = u_i^+ = u_(i + 1)^-$ by @lem-straight. Otherwise $x_i$ lies
  on an arm of each. Suppose it lay on the lower arm of $l_i$, which
  sits in the pocket $Y$ adjacent to $u_i^-$; the arm of $l_(i + 1)$
  through $x_i$ then also sits in $Y$ and leaves from a vertex of
  $Y$'s boundary path later than $u_i^-$, hence after $u_i^+$ (at
  $u_i^+$ itself the runs would be adjacent).
  If $u_i^+$ is on the wall opposite $u_i^-$ it is the last vertex of
  $Y$'s boundary path and no such vertex exists. If $u_i^+$ is on the
  same wall, $l_i$ is a chord of $Y$ through its run and the arm of
  $l_(i + 1)$ starts in the part of $Y$ bounded by the upper arm of
  $l_i$, the path above $u_i^+$ and the wall arc above $t_i$; a
  straight segment leaving that part crosses the line of $l_i$ once,
  at the upper arm, and cannot reach the lower arm. So $x_i$ is on
  the upper arm of $l_i$, and by the mirror argument on the lower arm
  of $l_(i + 1)$; this is (b). Both arms lie in the pocket $X$
  adjacent to $u_i^+$ on the opposite side, and $u_(i + 1)^-$ is a
  vertex of $X$'s boundary path after $u_i^+$, so the skipped
  vertices between them are interior vertices of that boundary path
  and lie on the wall of $u_i^+$. The boundary of $T_i$, traversed
  from $u_i^+$ along the skipped chain to $u_(i + 1)^-$, then to $x_i$
  and back, turns towards the wall at every skipped vertex (the path
  wraps the polygon there) and turns the same way at $u_(i + 1)^-$,
  $x_i$ and $u_i^+$, because the two closing segments prolong the
  end segments of the chain; a closed polygonal curve with all turns
  in one sense and total turning $2 pi$ is convex (@fig-skip).
]


#figure(
  scene(
    ((0, 0), (2, 3), (2, 6), (0, 9), (-3, 8), (-3, 1)),
    ((8, 0), (11, 1), (11, 8), (8, 9)),
    width: 250pt,
    annots: (
      (kind: "region", pts: ((0, 0), (8, 0), (8, 9), (0, 9), (2, 6), (2, 3)), fill: rgb("#f3ece4")),
      (kind: "region", pts: ((2, 3), (2, 6), (3, 4.5)), fill: rgb("#c5d3e6")),
      (kind: "seg", a: (0, 0), b: (8, 0), style: "dash"),
      (kind: "seg", a: (8, 9), b: (0, 9), style: "dash"),
      (kind: "seg", a: (2, 0), b: (2, 9), style: "skip"),
      (kind: "seg", a: (0, 0), b: (2, 3), style: "dot"),
      (kind: "seg", a: (2, 3), b: (2, 6), style: "dot"),
      (kind: "seg", a: (2, 6), b: (0, 9), style: "dot"),
      (kind: "seg", a: (0, 0), b: (6, 9)),
      (kind: "seg", a: (6, 0), b: (0, 9)),
      (kind: "pt", at: (3, 4.5)),
      (kind: "tag", at: (-2.2, 4.5), body: [$P$]),
      (kind: "tag", at: (9.2, 4.5), body: [$Q$]),
      (kind: "tag", at: (2, 3), body: [$u_0^+$], dx: -24pt, dy: -2pt),
      (kind: "tag", at: (2, 6), body: [$u_1^-$], dx: -24pt, dy: -10pt),
      (kind: "tag", at: (3, 4.5), body: [$x_0$], dx: 4pt, dy: -6pt),
      (kind: "tag", at: (2.2, 4.4), body: [$T_0$], dx: 0pt, dy: 0pt),
      (kind: "tag", at: (4.6, 6.9), body: [$l_0$], dx: 0pt, dy: 0pt),
      (kind: "tag", at: (4.6, 2.1), body: [$l_1$], dx: 0pt, dy: 0pt),
      (kind: "tag", at: (2, 7.6), body: [skipped], dx: 3pt, dy: 0pt),
      (kind: "tag", at: (6, 9), body: [$t_0$], dx: 2pt, dy: -8pt),
      (kind: "tag", at: (6, 0), body: [$b_1$], dx: 2pt, dy: -2pt),
    ),
  ),
  caption: [A skip. The geodesic turns left twice on $P'$; the
    extension of its first segment ($l_0$) meets the extension of its
    third ($l_1$ in the kept sequence) at $x_0$, so Step 1(d) skips the
    middle segment's extension (grey). The skipped chain is the single
    $P'$ edge from $u_0^+$ to $u_1^-$, and the region $T_0$ (dark) it
    bounds with the two arm pieces is a convex triangle. Both kept
    separators land on bridges, so every subchain but $P_1$ runs to a
    sentinel; $P_1$ starts at $p_0^+ = u_0^+$.],
) <fig-skip>

The greedy of Step 1(d) therefore does establish property (ii);
what it does not establish is (iv) and (v). By (a) the pieces
$[p_b, x_0], [x_0, x_1], dots.h, [x_(m - 2), p_t]$ form a simple arc,
the _spine_ of $S(P, Q)$, and $U_(<i) = l_0 union dots.h union l_(i - 1)$
is connected and contains $p_b$.

#lemma([cuts])[
  $l_i inter P' eq.not nothing$ for every $i$, and
  $p_i^+ = t_i$ if $t_i in P'$, while $p_i^+ = u_i^+$ otherwise, in
  which case $u_i^+$ is a vertex of $P'$. If $t_i in P'$ and
  $b_i in Q'$, then $u_i^+$ is the only point of $l_i inter Q'$ other
  than $b_i$, and $b_i$ lies below $u_i^+$ in arc order on $Q'$. For $Q$: $q_i^+ = t_i$ if
  $t_i in Q'$; otherwise $q_i^+ = u_i^+$ if $u_i^+ in Q'$; otherwise
  $l_i inter Q'$ is contained in ${b_i, u_i^-}$ and may be empty
  (@sec-bridge), the top of $l_i$ then lying on $P'$ or a bridge.
] <lem-cuts>
#proof[
  The points of $l_i inter partial R$ are $b_i$, $t_i$, the run
  vertices and any edge the run lies along, since arms are interior.
  If $t_i in.not P'$ then $u_i^+ in P'$ by @lem-arms, and $u_i^+$ is
  the highest point of $P'$ on $l_i$ because the upper arm is
  interior. If $t_i in P'$ it is the highest point of $l_i$
  altogether. For the last claim, $b_i in Q'$ means $u_i^- in P'$,
  the lower arm lands in the $Q$-pocket adjacent to $u_i^-$, whose
  last $Q'$ vertex is $u_i^+$ (the run from $u_i^-$ reaches $Q'$ first at $u_i^+$), so $b_i$ lies
  on that pocket's arc, below $u_i^+$. The $Q$ statement differs from
  the $P$ one only because an arm leaving a $P'$ vertex may land on a
  bridge.
]

#lemma([first separator])[
  Every sight meets some $l_i$. Let $i^*$ be the least such index.
  Then $p$ and $q$ lie in the same component of
  $partial R without U_(<i^*)$.
] <lem-first>
#proof[
  $U = l_0 union dots.h union l_(m - 1)$ is connected by
  @lem-kept(a) and contains $p_b in l_0$ and $p_t in l_(m - 1)$; the
  separation principle gives the first claim, and applied to
  $K = U_(<i^*)$, which the sight avoids, the second.
]

== The coverage theorem

Throughout this section $(p, q)$ is a visible pair other than the
two bridge pairs, and $i = i^*$ is the index of @lem-first. We treat
the $P$ side; the $Q$ side is the mirror image.

#lemma([start side])[
  If $i > 0$ and $b_(i - 1) in P'$, then $p$ lies strictly above
  $p_(i - 1)^+$ in arc order.
] <lem-start>
#proof[
  By @lem-arms, $u_(i - 1)^- in Q'$, so the lower arm of $l_(i - 1)$
  lies in the $P$-pocket $Pi'$ whose boundary path contains
  $u_(i - 1)^-$, and $b_(i - 1)$ is on the arc of $Pi'$. The sight
  avoids $l_(i - 1)$, so $p eq.not b_(i - 1)$ and $p eq.not p_(i - 1)^+$.

  If $p$ is below $b_(i - 1)$: both $p_b$ and $b_(i - 1)$ belong to
  $U_(<i)$, so the component of $partial R without U_(<i)$ containing
  $p$ is a sub-arc of $P'(p_b, b_(i - 1))$ and contains no point of
  $Q'$, contradicting @lem-first.

  If $b_(i - 1) < p < p_(i - 1)^+$: by @lem-cuts, $p_(i - 1)^+$ is
  $u_(i - 1)^+$ (when that vertex is on $P'$; it is then the top
  vertex of $Pi'$, being the first $P'$ vertex after $u_(i - 1)^-$) or
  $t_(i - 1)$ (when $u_(i - 1)^+ in Q'$, so that the upper arm also
  lies in $Pi'$). In either case $l_(i - 1) inter "cl"(Pi')$ is a
  chord of $Pi'$ from $b_(i - 1)$ to $p_(i - 1)^+$ through the run,
  and the region $D'$ bounded by $P'[b_(i - 1), p_(i - 1)^+]$ and this
  chord meets $Q'$ only at run vertices, which lie on $l_(i - 1)$.
  The open sight starts inside $D'$, and $q in.not D'$ since a point
  of $Q'$ in $D'$ would lie on $l_(i - 1)$; so the sight leaves $D'$
  through $P'$ (impossible by visibility) or through the chord
  (impossible by the choice of $i$). In the mirror statement for $Q$,
  when $l_(i - 1)$ meets $Q'$ only at $b_(i - 1)$ the cut is
  $b_(i - 1)$ itself and this second case is empty.
]

#lemma([end side])[
  Suppose $t_i in P'$, so that $e_i^P = p_i^+ = t_i$. Then either
  $p$ lies at or below $t_i$ in arc order, or
  $[p, q] inter l_i = {u_i^+}$ and $q = u_i^+$.
] <lem-end>
#proof[
  If $i = m - 1$ then $t_i = p_t$ and there is nothing to prove, so
  let $i < m - 1$; by @lem-arms, $u_i^+ in Q'$. Let $Pi'' = Pi(v_a, v_c)$
  be the $P$-pocket whose boundary path contains $u_i^+$. The upper
  arm $A^+ = (u_i^+, t_i]$ is a chord of $Pi''$ landing on
  $P'(v_a, v_c)$; it cuts off the region $D$ bounded by
  $P'[t_i, v_c]$, $A^+$ and $pi[u_i^+, v_c]$. The rest of $Pi''$ has,
  as its boundary path, $pi[v_a, u_i^+]$, whose last segment is the
  run of $l_i$; if $u_i^- in Q'$ the lower arm of $l_i$ lies in it
  too and $l_i inter "cl"(Pi'')$ is the whole chord $[b_i, t_i]$.
  Suppose $p$ is above $t_i$.

  _Case $p in P'(t_i, v_c)$._ The $P$-part of the sight starts in
  $D$ and stays in $"cl"(Pi'')$ until it meets $pi$. It cannot cross
  $A^+$: the region beyond has boundary $P'$ together with pieces of
  $l_i$ only (the run, or the whole chord), and its points of $Q'$
  lie on $l_i$; having crossed the line of $l_i$ once the sight can
  meet it no more, so it could neither leave that region nor end in
  it. It cannot touch
  $A^+$ without crossing, since that forces collinearity with $l_i$
  and puts $t_i$ in the open sight. If it leaves $D$ through
  $pi(u_i^+, v_c)$ it enters a $Q$-pocket whose boundary path lies
  at or above $u_i^+$, whereas the remaining pieces of $l_i$ -- the
  run, below $u_i^+$ on $pi$, and the lower arm, which lies in
  $Pi''$ below $A^+$ or in the $Q$-pocket whose boundary path ends at
  $u_i^+$ -- are out of reach, contradicting $[p, q] inter l_i eq.not nothing$.
  What remains is that the sight ends at $q = u_i^+$ and meets $l_i$
  nowhere else (the green sight of @fig-pockets, with $i = 1$).

  _Case $p gt.eq v_c$._ The $P$-part lies in a pocket at or above
  $v_c$ and the $Q$-part in a $Q$-pocket whose boundary path lies at
  or above $v_c$, so no arm of $l_i$ is reachable, and a point of
  $l_i$ on $pi$ can be reached only along $pi$, the sight meeting
  $pi$ in a connected set; a sight containing a path vertex in its
  interior violates visibility, so the only possibility is
  $p = v_c$ with the sight equal to the path segment $[u_i^+, v_c]$,
  which is again $q = u_i^+$ and a single touch.
]

The exceptional alternative of @lem-end is the configuration of
@sec-wrap, seen from the other wall: the sight leaves the top vertex
of the run of $l_i$ into the region above $l_i$'s upper arm, and
the cut $t_i$, which lies below $p$ in arc order, shields nothing.
This is precisely the step "no point above $q_(i + 1)^+$ is visible
from $l_i$" in the proof of Lemma 1 of @amato94, whose justification
is property (v). The next lemma shows that the neighbouring
subproblem catches such a pair.

#lemma([witness transfer])[
  Suppose $t_i in P'$, $q = u_i^+$ and $[p, q] inter l_i = {q}$. Then
  $p in P_(i + 1)$ and $q in Q_(i + 1)$.
] <lem-transfer>
#proof[
  $q in Q_(i + 1)$: the start of $Q_(i + 1)$ is $q_b$ unless
  $b_i in Q'$, in which case it is $q_i^+ = u_i^+ = q$ by @lem-cuts.
  The end is $q_t$ unless $t_(i + 1) in Q'$; then $u_(i + 1)^+ in P'$
  and $t_(i + 1)$ lands on the arc of the $Q$-pocket adjacent to
  $u_(i + 1)^+$, whose first $Q'$ vertex is at or after $u_i^+$ in
  path order, hence at or above $q$ in arc order by @lem-order.

  $p in P_(i + 1)$: the start is $p_b$ or $p_i^+ = t_i$, both below
  $p$. The end is $p_t$ unless $t_(i + 1) in P'$, i.e.
  $u_(i + 1)^+ in Q'$. Then $u_(i + 1)^+$ is a vertex of the boundary
  path of $Pi''$ after $u_i^+$, and the upper arm of $l_(i + 1)$ is a
  chord of the region $D$ of @lem-end from $u_(i + 1)^+$ to
  $t_(i + 1) in P'(t_i, v_c)$, so $t_(i + 1) > t_i$. We show
  $p lt.eq t_(i + 1)$. If the runs of $l_i$ and $l_(i + 1)$ are
  adjacent, $l_(i + 1)$ passes through $q$; the sight ends at $q$ and
  so meets the line of $l_(i + 1)$ only there, hence cannot cross
  that arm, and $p$ lies on the near side of it. Otherwise
  (@lem-kept(c)) the sight approaches $q$ through the corner of the
  convex region $T_i$ at $q$, and since it does not cross $[q, x_i]
  subset l_i$ it must enter $T_i$ across $(x_i, u_(i + 1)^-) subset
  l_(i + 1)$; a second crossing of the line of $l_(i + 1)$ at the
  upper arm is impossible, so again $p lt.eq t_(i + 1)$. In the
  situation $p = v_c$ of @lem-end the runs are adjacent, since $v_c$
  and $u_i^+$ lie on different walls.
]

#block(above: 1.1em, below: 0.8em)[
  #text(weight: "bold")[Theorem A.] #emph[
    Let $P$ and $Q$ be simple polygons with disjoint boundaries in
    the non-containing case, with an unpinched channel, and let
    $S(P, Q)$ and the subchains be as in @sec-amended. For every
    visible pair $(p, q)$ there is an index $i$ with $p in P_i$ and
    $q in Q_i$. Specifically, with $i^*$ as in @lem-first, either
    $(p, q) in P_(i^*) times Q_(i^*)$, or $[p, q] inter l_(i^*) =
    {u_(i^*)^+}$ with $u_(i^*)^+ in {p, q}$ and
    $(p, q) in P_(i^* + 1) times Q_(i^* + 1)$.
  ]
]
#proof[
  The bridge pairs are covered by subproblems $0$ and $m - 1$: both
  subchains of subproblem $0$ start at their sentinels, and those of
  $m - 1$ end at $p_(m - 1)^+ = p_t$ and at the sentinel $q_t$
  ($t_(m - 1) = p_t in.not Q'$). For any other pair let $i = i^*$.
  On the $P$ side, $p$ is above $s_i^P$ by @lem-start (the sentinel
  case being trivial) and at or below $e_i^P$ by @lem-end unless the
  exceptional alternative holds; symmetrically on the $Q$ side. The
  two exceptional alternatives are exclusive, $u_i^+$ lying on one
  wall, and in either @lem-transfer or its mirror image places both
  $p$ and $q$ in subproblem $i + 1$; note $i < m - 1$ there, since
  $t_(m - 1) = p_t$ never has $p$ above it. Hence
  $sigma(P, Q) = min_i sigma(P_i, Q_i)$.
]

== Remarks <app-remarks>

_The wrap override._ The proof never invokes the whole-boundary
override of @sec-wrap: with the cuts identified by @lem-cuts and the
slices of @lem-start, @lem-end and @lem-transfer, the anchor cuts
$x_(i - 1)^+$ and $x_i^+$ always lie inside the slice whenever the
corresponding truncation applies, and the pair is covered without it.
The configuration that triggers it -- a later separator landing below
an earlier one's cut -- does occur (about one pair in ten thousand in
the harness of @sec-verification), but the pairs it concerns are
covered by the neighbouring subproblems. Our implementation no longer
applies it.

_Which cut._ The proof uses the paper's cut, the highest point of
$l_i inter X'$ along $l_i$. By @lem-cuts this is $t_i$ or the top
run vertex, and in the one place where two orders could disagree
(@lem-cuts, last claim) they agree. Redefining the cut as the last
point of $l_i inter X'$ in arc order changes the subchains in about a
tenth of random instances but no coverage outcome; we tested it and
did not adopt it.

_What fails in the original._ Every step of the proof of Lemma 1 of
@amato94 survives except the one that needs property (v): with the
minimal index in place of the hypothesis $p q inter l_(i - 1) = nothing$,
@lem-start is the claim that $p$ and $q$ are above the previous
cuts, and @lem-end is the claim that they are below the current
ones. The exceptional alternative is where a grazing or wrongly
landing separator leaves a visible pair above its own cut; the
paper's remedy is to end the subchain at the next separator's cut,
which is sound only under property (v), and the sentinel is the
replacement that needs no such property. The bridge landings of
@sec-bridge enter only through @lem-arms: a separator whose run ends
on $P'$ lands on $Q'$ or a bridge, and in both cases its
$Q$-subchain runs to the sentinel.

_Complexity._ By @lem-arms the run of $l_i$ ends on one wall, so at
the top end of subproblem $i$ exactly one subchain is truncated, the
one on the other wall, and the other runs to its sentinel; likewise at
the bottom end, according to the wall on which the run of $l_(i - 1)$
begins; a bridge landing leaves both at the sentinel. Every
subproblem therefore contains, at each end, the whole remainder of one
facing arc. A channel whose geodesic alternates between the walls
$Theta(n)$ times has $Theta(n)$ subproblems each containing a whole
facing arc, so the aggregate subproblem size is $Theta(n^2)$: the
bound stated in @sec-amended is tight, and it is the typical case.
Nor need a sentinel-extended subchain lie on one side of the line of
$l_i$ (both subchains of subproblem 2 in @fig-pockets are whole arcs),
so such subproblems are not linearly separable; an implementation
that solves them by a quadratic method runs in $O(n^3)$ overall,
slower than the brute force it set out to improve on. Bounding the
sentinel side by a later cut is exactly what property (v) supplies,
and the two-path construction of @amato92 remains the route to it.

#bibliography(
  bytes(
    "
@techreport{amato92,
  author = {Amato, Nancy M.},
  title = {Computing the Minimum Visible Vertex Distance Between Two
    Nonintersecting Simple Polygons},
  institution = {Coordinated Science Laboratory, University of
    Illinois at Urbana-Champaign},
  number = {UILU-ENG-92-2206, Coordinated Science Laboratory,
    University of Illinois at Urbana-Champaign},
  year = {1992},
  url = {https://www.ideals.illinois.edu/items/100137},
}
@article{amato95,
  author = {Amato, Nancy M.},
  title = {Finding a Closest Visible Vertex Pair Between Two Polygons},
  journal = {Algorithmica},
  volume = {14},
  pages = {183--201},
  year = {1995},
  doi = {10.1007/BF01293668},
}
@article{amato94,
  author = {Amato, Nancy M.},
  title = {Determining the Separation of Simple Polygons},
  journal = {International Journal of Computational Geometry and
    Applications},
  volume = {4},
  number = {4},
  year = {1994},
  doi = {10.1142/S0218195994000240},
}
@inproceedings{wangchan86,
  author = {Wang, Cao An and Chan, Edward P. F.},
  title = {Finding the Minimum Visible Vertex Distance Between Two
    Non-intersecting Simple Polygons},
  booktitle = {Proceedings of the 2nd Annual ACM Symposium on
    Computational Geometry},
  pages = {34--42},
  year = {1986},
}
@inproceedings{amss89,
  author = {Aggarwal, Alok and Moran, Shlomo and Shor, Peter and
    Suri, Subhash},
  title = {Computing the Minimum Visible Vertex Distance Between Two
    Polygons},
  booktitle = {Proceedings of the Workshop on Algorithms and Data
    Structures (WADS)},
  series = {Lecture Notes in Computer Science},
  volume = {382},
  pages = {115--134},
  year = {1989},
}
",
  ),
  style: "ieee",
)
