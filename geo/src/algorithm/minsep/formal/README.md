# Lean formalisation of the counterexamples

Machine-checked versions of the counterexamples in `../paper_corrections.typ`.
Each finding is stated against exact rational coordinates, and the geodesic
underlying each construction is *proved* shortest rather than assumed.

| File | Report section | Main theorem |
| --- | --- | --- |
| `Minsep/Basic.lean` | — | `single_turn` (shortest-path certificate), `tri_eq`, `ExtEnd.unique` |
| `Minsep/Bridge.lean` | §3.1 bridge landing, §3.2 orientation | `finding_bridge`, `finding_orientation` |
| `Minsep/Wrap.lean` | §3.4 descending cuts | `finding_wrap` |
| `Minsep/Pinch.lean` | §3.5 pinched channel | `finding_pinch` |

§3.3 (the `i = m − 1` truncation branch) is not yet formalised. Its geodesic
turns at a vertex of the *opposite* polygon, which needs a different covering
argument from the one used here.

## What is modelled, and how

- Points are `ℝ × ℝ`, and `eu` is the Euclidean distance. Mathlib's own metric
  on a product is the sup metric, so it is not used.
- `P` and `Q` are the closed polygonal regions, written as convex hulls of
  their vertices. `H` is `convexHull ℝ (P ∪ Q)`.
- The channel is `R := interior H \ (P ∪ Q)`. For the instances here, where
  both polygons are convex and in the non-containing case, this is the open
  polygon bounded by the two facing arcs and the two bridges.
- A path is a list of vertices. `PathIn S L` says every edge lies in `S`, and
  `len L` is the Euclidean length. "Shortest" means among polygonal paths in
  `closure R`. The shortest path in a polygonal domain is polygonal, but that
  fact is not formalised here.
- `ExtEnd R v e t`: prolong the ray from `v` in direction `e` while it stays
  in the open channel `R`. `t` is the first point outside `R`, or `v` itself
  if the ray leaves `R` at once. `ExtEnd.unique` shows `t` is determined.

### The shortest-path certificate

`single_turn` proves that `a → v → b` is shortest. It needs closed sets
`A ∋ a` and `B ∋ b` that cover the region, such that every point of
`A ∩ B` in the region lies beyond `v` as seen from both `a` and `b`. Any path
from `a` to `b` must cross `A ∩ B`, by connectedness of each edge, and the
crossing point is at least as far from `a` and from `b` as `v` is. For a
geodesic that wraps a vertex of `P`, `A` and `B` are the outer half-planes of
`P`'s two edges at that vertex.

## Building

```sh
lake build
```

`lean-toolchain` and the Mathlib revision in `lakefile.toml` are pinned
together. With network access to the Mathlib cache, run `lake exe cache get`
first to skip compiling Mathlib.
