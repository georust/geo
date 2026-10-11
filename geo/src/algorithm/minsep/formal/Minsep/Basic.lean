/-
Generic plane geometry for the minsep counterexamples.

Points are `ℝ × ℝ` with the product (usual) topology. Mathlib's metric on a
product is the sup metric, so Euclidean length is defined here as `eu`.

The main results are

* `single_turn`: a certificate that a two-segment path `a → v → b` is shortest
  among polygonal paths in a set `S`;
* `tri_eq`: a triangle is the intersection of three half-planes;
* `triH_subset_closure_triS`: a closed triangle lies in the closure of its
  open interior;
* `ExtEnd`: the endpoint of a segment's prolongation inside an open region,
  and its uniqueness.
-/
import Mathlib.Analysis.Convex.PathConnected
import Mathlib.Analysis.Convex.Segment
import Mathlib.Analysis.Convex.Hull
import Mathlib.Analysis.Real.Sqrt
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Positivity
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.NormNum
import Mathlib.Tactic.Ring
import Mathlib.Tactic.FunProp
import Mathlib.Tactic.Abel

open Set Filter Topology

set_option linter.unusedSimpArgs false

namespace Minsep

/-- Points of the plane. -/
abbrev Pt := ℝ × ℝ

/-! ### Euclidean distance -/

/-- Euclidean distance. -/
noncomputable def eu (p q : Pt) : ℝ := √((p.1 - q.1) ^ 2 + (p.2 - q.2) ^ 2)

theorem eu_nonneg (p q : Pt) : 0 ≤ eu p q := Real.sqrt_nonneg _

@[simp] theorem eu_self (p : Pt) : eu p p = 0 := by simp [eu]

theorem eu_comm (p q : Pt) : eu p q = eu q p := by
  unfold eu; congr 1; ring

theorem eu_triangle (p q r : Pt) : eu p r ≤ eu p q + eu q r := by
  unfold eu
  have hA0 : 0 ≤ (p.1 - q.1) ^ 2 + (p.2 - q.2) ^ 2 := by positivity
  have hB0 : 0 ≤ (q.1 - r.1) ^ 2 + (q.2 - r.2) ^ 2 := by positivity
  have hcs : (p.1 - q.1) * (q.1 - r.1) + (p.2 - q.2) * (q.2 - r.2) ≤
      √((p.1 - q.1) ^ 2 + (p.2 - q.2) ^ 2) * √((q.1 - r.1) ^ 2 + (q.2 - r.2) ^ 2) := by
    rw [← Real.sqrt_mul hA0]
    refine le_trans (le_abs_self _) (Real.abs_le_sqrt ?_)
    nlinarith [sq_nonneg ((p.1 - q.1) * (q.2 - r.2) - (p.2 - q.2) * (q.1 - r.1))]
  have hsA := Real.sq_sqrt hA0
  have hsB := Real.sq_sqrt hB0
  calc √((p.1 - r.1) ^ 2 + (p.2 - r.2) ^ 2)
      ≤ √((√((p.1 - q.1) ^ 2 + (p.2 - q.2) ^ 2) + √((q.1 - r.1) ^ 2 + (q.2 - r.2) ^ 2)) ^ 2) :=
        Real.sqrt_le_sqrt (by nlinarith)
    _ = _ := Real.sqrt_sq (by positivity)

theorem eu_le_of_sq {p q : Pt} {r : ℝ} (hr : 0 ≤ r)
    (h : (p.1 - q.1) ^ 2 + (p.2 - q.2) ^ 2 ≤ r ^ 2) : eu p q ≤ r := by
  unfold eu
  calc _ ≤ √(r ^ 2) := Real.sqrt_le_sqrt h
    _ = r := Real.sqrt_sq hr

theorem le_eu_of_sq {p q : Pt} {r : ℝ} (hr : 0 ≤ r)
    (h : r ^ 2 ≤ (p.1 - q.1) ^ 2 + (p.2 - q.2) ^ 2) : r ≤ eu p q := by
  unfold eu
  calc r = √(r ^ 2) := (Real.sqrt_sq hr).symm
    _ ≤ _ := Real.sqrt_le_sqrt h

/-- Inner product, written out. -/
def dot (u w : Pt) : ℝ := u.1 * w.1 + u.2 * w.2

/-- If `z` lies beyond `v` as seen from `a` (non-negative projection of `z - v` on
`v - a`), then `z` is at least as far from `a` as `v` is. -/
theorem eu_le_of_dot {a v z : Pt} (h : 0 ≤ dot (v - a) (z - v)) : eu a v ≤ eu a z := by
  unfold eu
  apply Real.sqrt_le_sqrt
  simp only [dot, Prod.fst_sub, Prod.snd_sub] at h
  nlinarith [sq_nonneg (z.1 - v.1), sq_nonneg (z.2 - v.2)]

/-- A point on a segment splits its length. -/
theorem eu_segment {p q z : Pt} (hz : z ∈ segment ℝ p q) : eu p z + eu z q = eu p q := by
  rw [segment_eq_image'] at hz
  obtain ⟨t, ⟨ht0, ht1⟩, rfl⟩ := hz
  have h1 : eu p (p + t • (q - p)) = t * eu p q := by
    unfold eu
    rw [show (p.1 - (p + t • (q - p)).1) ^ 2 + (p.2 - (p + t • (q - p)).2) ^ 2 =
        t ^ 2 * ((p.1 - q.1) ^ 2 + (p.2 - q.2) ^ 2) by simp; ring,
      Real.sqrt_mul (sq_nonneg t), Real.sqrt_sq ht0]
  have h2 : eu (p + t • (q - p)) q = (1 - t) * eu p q := by
    unfold eu
    rw [show ((p + t • (q - p)).1 - q.1) ^ 2 + ((p + t • (q - p)).2 - q.2) ^ 2 =
        (1 - t) ^ 2 * ((p.1 - q.1) ^ 2 + (p.2 - q.2) ^ 2) by simp; ring,
      Real.sqrt_mul (sq_nonneg _), Real.sqrt_sq (by linarith)]
  rw [h1, h2]; ring

/-! ### Polygonal paths -/

/-- Length of the polygonal path through the listed vertices. -/
noncomputable def len : List Pt → ℝ
  | p :: q :: rest => eu p q + len (q :: rest)
  | _ => 0

/-- Every edge of the polygonal path lies in `S`. -/
def PathIn (S : Set Pt) : List Pt → Prop
  | p :: q :: rest => segment ℝ p q ⊆ S ∧ PathIn S (q :: rest)
  | _ => True

theorem eu_le_len : ∀ (a : Pt) (L : List Pt),
    eu a ((a :: L).getLast (List.cons_ne_nil a L)) ≤ len (a :: L)
  | a, [] => by simp [len]
  | a, q :: L => by
    have ih := eu_le_len q L
    simp only [List.getLast_cons_cons, len]
    exact (eu_triangle _ _ _).trans (by linarith)

/-- A polygonal path in `S ⊆ A ∪ B` from a point of `A` to a point of `B` meets
`A ∩ B` at some `z`, and is at least as long as the broken line through `z`. -/
theorem crossing {S A B : Set Pt} (hA : IsClosed A) (hB : IsClosed B) (hS : S ⊆ A ∪ B) :
    ∀ (a : Pt) (L : List Pt), a ∈ S → PathIn S (a :: L) → a ∈ A →
      (a :: L).getLast (List.cons_ne_nil a L) ∈ B →
      ∃ z ∈ S ∩ (A ∩ B), eu a z + eu z ((a :: L).getLast (List.cons_ne_nil a L)) ≤ len (a :: L)
  | a, [], haS, _, haA, hb =>
    ⟨a, ⟨haS, haA, by simpa using hb⟩, by simp [len]⟩
  | a, q :: L, haS, hP, haA, hb => by
    obtain ⟨hseg, hrest⟩ := hP
    simp only [List.getLast_cons_cons] at hb ⊢
    have hqS : q ∈ S := hseg (right_mem_segment ℝ a q)
    by_cases hqA : q ∈ A
    · obtain ⟨z, hz, hlen⟩ := crossing hA hB hS q L hqS hrest hqA hb
      refine ⟨z, hz, ?_⟩
      have := eu_triangle a q z
      simp only [len]; linarith
    · have hqB : q ∈ B := (hS hqS).resolve_left hqA
      have hpre : IsPreconnected (segment ℝ a q) := (convex_segment a q).isPreconnected
      obtain ⟨z, hzseg, hzA, hzB⟩ := isPreconnected_closed_iff.1 hpre A B hA hB (hseg.trans hS)
        ⟨a, left_mem_segment ℝ a q, haA⟩ ⟨q, right_mem_segment ℝ a q, hqB⟩
      refine ⟨z, ⟨hseg hzseg, hzA, hzB⟩, ?_⟩
      have h1 := eu_segment hzseg
      have h2 := eu_triangle z q ((q :: L).getLast (List.cons_ne_nil q L))
      have h3 := eu_le_len q L
      simp only [len]; linarith

/-- **Shortest-path certificate for a path with one turn.** Suppose `S` is covered by
closed sets `A ∋ a` and `B ∋ b`, and every point of `S ∩ A ∩ B` lies beyond `v` as
seen from both `a` and `b`. Then no polygonal path in `S` from `a` to `b` is shorter
than `a → v → b`. -/
theorem single_turn {S A B : Set Pt} {a v b : Pt} (hA : IsClosed A) (hB : IsClosed B)
    (hS : S ⊆ A ∪ B) (haA : a ∈ A) (hbB : b ∈ B) (haS : a ∈ S)
    (hD : ∀ z ∈ S ∩ (A ∩ B), 0 ≤ dot (v - a) (z - v) ∧ 0 ≤ dot (v - b) (z - v))
    (L : List Pt) (hL : PathIn S (a :: L))
    (hlast : (a :: L).getLast (List.cons_ne_nil a L) = b) :
    len [a, v, b] ≤ len (a :: L) := by
  obtain ⟨z, hz, hlen⟩ := crossing hA hB hS a L haS hL haA (hlast ▸ hbB)
  obtain ⟨h1, h2⟩ := hD z hz
  have e1 := eu_le_of_dot h1
  have e2 := eu_le_of_dot h2
  rw [hlast] at hlen
  have c1 := eu_comm v b
  have c2 := eu_comm z b
  simp only [len]
  linarith

/-! ### Half-planes and triangles -/

/-- Twice the signed area of `a b p`; positive iff `p` is left of the directed line `a → b`. -/
def cross (a b p : Pt) : ℝ := (b.1 - a.1) * (p.2 - a.2) - (b.2 - a.2) * (p.1 - a.1)

/-- Closed half-plane left of `a → b`. -/
def hp (a b : Pt) : Set Pt := {p | 0 ≤ cross a b p}

/-- Open half-plane left of `a → b`. -/
def hpS (a b : Pt) : Set Pt := {p | 0 < cross a b p}

theorem continuous_cross (a b : Pt) : Continuous (cross a b) := by
  unfold cross; fun_prop

theorem isClosed_hp (a b : Pt) : IsClosed (hp a b) :=
  isClosed_le continuous_const (continuous_cross a b)

theorem isOpen_hpS (a b : Pt) : IsOpen (hpS a b) :=
  isOpen_lt continuous_const (continuous_cross a b)

/-- `cross` is affine in its last argument. -/
theorem cross_combo (a b x y : Pt) (s : ℝ) :
    cross a b (x + s • (y - x)) = (1 - s) * cross a b x + s * cross a b y := by
  simp only [cross, Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
    Prod.snd_sub, smul_eq_mul]
  ring

theorem convex_hp (a b : Pt) : Convex ℝ (hp a b) := by
  intro x hx y hy s t hs ht hst
  have ht' : t = 1 - s := by linarith
  subst ht'
  have := cross_combo a b x y (1 - s)
  simp only [hp, mem_ofPred_eq] at *
  rw [show s • x + (1 - s) • y = x + (1 - s) • (y - x) by ext <;> simp <;> ring]
  rw [this]
  nlinarith [mul_nonneg hs hx, mul_nonneg ht hy]

theorem convexHull_subset_hp {s : Set Pt} {a b : Pt} (h : ∀ x ∈ s, 0 ≤ cross a b x) :
    convexHull ℝ s ⊆ hp a b :=
  convexHull_min h (convex_hp a b)

/-- The interior of a closed half-plane lies in the open half-plane. -/
theorem interior_hp_subset {a b : Pt} (hab : a ≠ b) : interior (hp a b) ⊆ hpS a b := by
  intro p hp'
  rw [mem_interior_iff_mem_nhds] at hp'
  -- move from `p` against the gradient of `cross a b`
  set n : Pt := (-(b.2 - a.2), b.1 - a.1)
  have hc : Continuous fun t : ℝ => p - t • n := by fun_prop
  have ht : Tendsto (fun t : ℝ => p - t • n) (𝓝[>] 0) (𝓝 p) := by
    have := hc.tendsto 0
    simp only [zero_smul, sub_zero] at this
    exact this.mono_left nhdsWithin_le_nhds
  obtain ⟨t, hmem, ht0⟩ := ((ht.eventually hp').and self_mem_nhdsWithin).exists
  have hpos : 0 < (b.1 - a.1) ^ 2 + (b.2 - a.2) ^ 2 := by
    by_cases h : b.1 - a.1 = 0
    · have h2 : b.2 - a.2 ≠ 0 := fun h2 => hab (Prod.ext (by linarith) (by linarith))
      positivity
    · positivity
  simp only [hp, hpS, mem_ofPred_eq, mem_Ioi] at hmem ht0 ⊢
  by_contra hcon
  have key : cross a b (p - t • n) = cross a b p - t * ((b.1 - a.1) ^ 2 + (b.2 - a.2) ^ 2) := by
    simp only [cross, n, Prod.fst_sub, Prod.snd_sub, Prod.smul_fst, Prod.smul_snd, smul_eq_mul]
    ring
  nlinarith [mul_pos ht0 hpos]

/-- Closed triangle as an intersection of half-planes. -/
def triH (a b c : Pt) : Set Pt := hp a b ∩ hp b c ∩ hp c a

/-- Open triangle. -/
def triS (a b c : Pt) : Set Pt := hpS a b ∩ hpS b c ∩ hpS c a

theorem isOpen_triS (a b c : Pt) : IsOpen (triS a b c) :=
  ((isOpen_hpS _ _).inter (isOpen_hpS _ _)).inter (isOpen_hpS _ _)

theorem mem_convexHull3 {a b c : Pt} {u v w : ℝ} (hu : 0 ≤ u) (hv : 0 ≤ v) (hw : 0 ≤ w)
    (hs : u + v + w = 1) : u • a + v • b + w • c ∈ convexHull ℝ ({a, b, c} : Set Pt) := by
  have hconv := convex_convexHull ℝ ({a, b, c} : Set Pt)
  have ha : a ∈ convexHull ℝ ({a, b, c} : Set Pt) := subset_convexHull ℝ _ (by simp)
  have hb : b ∈ convexHull ℝ ({a, b, c} : Set Pt) := subset_convexHull ℝ _ (by simp)
  have hc : c ∈ convexHull ℝ ({a, b, c} : Set Pt) := subset_convexHull ℝ _ (by simp)
  by_cases huv : u + v = 0
  · have hu0 : u = 0 := by linarith
    have hv0 : v = 0 := by linarith
    have hw1 : w = 1 := by linarith
    subst hu0 hv0 hw1
    simpa using hc
  · have hpos : 0 < u + v := lt_of_le_of_ne (by linarith) (Ne.symm huv)
    have hab : (u / (u + v)) • a + (v / (u + v)) • b ∈ convexHull ℝ ({a, b, c} : Set Pt) :=
      hconv ha hb (by positivity) (by positivity) (by field_simp)
    have := hconv hab hc hpos.le hw (by linarith)
    convert this using 1
    ext <;> simp <;> field_simp

/-- A counter-clockwise triangle is the intersection of its three left half-planes. -/
theorem tri_eq {a b c : Pt} (h : 0 < cross a b c) :
    convexHull ℝ ({a, b, c} : Set Pt) = triH a b c := by
  apply Subset.antisymm
  · refine convexHull_min ?_ (((convex_hp a b).inter (convex_hp b c)).inter (convex_hp c a))
    intro x hx
    simp only [mem_insert_iff, mem_singleton_iff] at hx
    simp only [triH, hp, mem_inter_iff, mem_ofPred_eq]
    rcases hx with rfl | rfl | rfl <;> simp only [cross] at h ⊢ <;>
      refine ⟨⟨?_, ?_⟩, ?_⟩ <;> nlinarith
  · intro p hp'
    simp only [triH, hp, mem_inter_iff, mem_ofPred_eq] at hp'
    obtain ⟨⟨h1, h2⟩, h3⟩ := hp'
    set D := cross a b c
    have hsum : cross b c p + cross c a p + cross a b p = D := by
      simp only [D, cross]; ring
    have hmem := mem_convexHull3 (a := a) (b := b) (c := c)
      (div_nonneg h2 h.le) (div_nonneg h3 h.le) (div_nonneg h1 h.le)
      (by field_simp; linarith)
    convert hmem using 1
    ext <;> simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, smul_eq_mul] <;>
      field_simp <;> simp only [D, cross] <;> ring

/-- The open triangle lies in the interior of the closed one. -/
theorem triS_subset_interior {a b c : Pt} (h : 0 < cross a b c) :
    triS a b c ⊆ interior (convexHull ℝ ({a, b, c} : Set Pt)) := by
  rw [tri_eq h]
  refine interior_maximal ?_ (isOpen_triS a b c)
  intro p hp'
  simp only [triS, triH, hp, hpS, mem_inter_iff, mem_ofPred_eq] at hp' ⊢
  exact ⟨⟨hp'.1.1.le, hp'.1.2.le⟩, hp'.2.le⟩

/-- A point `w` with `w + t (c - w) ∈ U` for every `t ∈ (0, 1]` is in the closure of `U`. -/
theorem mem_closure_of_ray {U : Set Pt} {w c : Pt}
    (h : ∀ t : ℝ, 0 < t → t ≤ 1 → w + t • (c - w) ∈ U) : w ∈ closure U := by
  have hc : Continuous fun t : ℝ => w + t • (c - w) := by fun_prop
  have ht : Tendsto (fun t : ℝ => w + t • (c - w)) (𝓝[>] 0) (𝓝 w) := by
    have := hc.tendsto 0
    simp only [zero_smul, add_zero] at this
    exact this.mono_left nhdsWithin_le_nhds
  apply mem_closure_of_tendsto ht
  filter_upwards [Ioc_mem_nhdsGT (show (0 : ℝ) < 1 by norm_num)] with t ht
  exact h t ht.1 ht.2

/-- A counter-clockwise closed triangle lies in the closure of the open one. -/
theorem triH_subset_closure_triS {a b c : Pt} (h : 0 < cross a b c) :
    triH a b c ⊆ closure (triS a b c) := by
  intro w hw
  -- the centroid is strictly inside
  set g : Pt := (1 / 3 : ℝ) • (a + b + c)
  have hg1 : cross a b g = cross a b c / 3 := by
    simp only [g, cross, Prod.smul_fst, Prod.smul_snd, Prod.fst_add, Prod.snd_add, smul_eq_mul]
    ring
  have hg2 : cross b c g = cross a b c / 3 := by
    simp only [g, cross, Prod.smul_fst, Prod.smul_snd, Prod.fst_add, Prod.snd_add, smul_eq_mul]
    ring
  have hg3 : cross c a g = cross a b c / 3 := by
    simp only [g, cross, Prod.smul_fst, Prod.smul_snd, Prod.fst_add, Prod.snd_add, smul_eq_mul]
    ring
  simp only [triH, hp, mem_inter_iff, mem_ofPred_eq] at hw
  obtain ⟨⟨h1, h2⟩, h3⟩ := hw
  apply mem_closure_of_ray (c := g)
  intro t ht0 ht1
  simp only [triS, hpS, mem_inter_iff, mem_ofPred_eq, cross_combo]
  rw [hg1, hg2, hg3]
  have hD : 0 < t * (cross a b c / 3) := by positivity
  refine ⟨⟨?_, ?_⟩, ?_⟩ <;> nlinarith [mul_nonneg (sub_nonneg.2 ht1) h1,
    mul_nonneg (sub_nonneg.2 ht1) h2, mul_nonneg (sub_nonneg.2 ht1) h3]

/-- A closed counter-clockwise triangle whose open interior lies in `R` lies in the
closure of `R`. -/
theorem convexHull3_subset_closure {a b c : Pt} {R : Set Pt} (h : 0 < cross a b c)
    (hR : triS a b c ⊆ R) : convexHull ℝ ({a, b, c} : Set Pt) ⊆ closure R := by
  rw [tri_eq h]
  exact (triH_subset_closure_triS h).trans (closure_mono hR)

/-! ### Prolongation of a segment inside a region -/

/-- `t` is where the ray from `v` in direction `e`, prolonged while it stays in the open
region `R`, first meets the complement of `R`. When `t = v` (`l = 0`), the ray leaves `R`
at once. -/
def ExtEnd (R : Set Pt) (v e t : Pt) : Prop :=
  ∃ l : ℝ, 0 ≤ l ∧ t = v + l • e ∧ (∀ μ, 0 < μ → μ < l → v + μ • e ∈ R) ∧
    (l = 0 → ∃ ε : ℝ, 0 < ε ∧ ∀ μ, 0 < μ → μ < ε → v + μ • e ∉ R) ∧ (0 < l → t ∉ R)

private theorem ExtEnd.aux {R : Set Pt} {v e : Pt} {l l' : ℝ} (hl : 0 ≤ l) (hll : l < l')
    (hin' : ∀ μ, 0 < μ → μ < l' → v + μ • e ∈ R)
    (h0 : l = 0 → ∃ ε : ℝ, 0 < ε ∧ ∀ μ, 0 < μ → μ < ε → v + μ • e ∉ R)
    (hout : 0 < l → v + l • e ∉ R) : False := by
  rcases hl.eq_or_lt with h | h
  · obtain ⟨ε, hε, hε'⟩ := h0 h.symm
    have hm : 0 < min ε l' / 2 := by
      have : 0 < min ε l' := lt_min hε (by linarith)
      positivity
    have hm1 : min ε l' / 2 < ε := by
      have := min_le_left ε l'; have : 0 < min ε l' := by linarith
      linarith
    have hm2 : min ε l' / 2 < l' := by
      have := min_le_right ε l'; have : 0 < min ε l' := by linarith
      linarith
    exact hε' _ hm hm1 (hin' _ hm hm2)
  · exact hout h (hin' l h hll)

/-- The prolongation endpoint is unique. -/
theorem ExtEnd.unique {R : Set Pt} {v e t t' : Pt} (h : ExtEnd R v e t) (h' : ExtEnd R v e t') :
    t = t' := by
  obtain ⟨l, hl, rfl, hin, h0, hout⟩ := h
  obtain ⟨l', hl', rfl, hin', h0', hout'⟩ := h'
  rcases lt_trichotomy l l' with hlt | heq | hgt
  · exact (ExtEnd.aux hl hlt hin' h0 hout).elim
  · rw [heq]
  · exact (ExtEnd.aux hl' hgt hin h0' hout').elim

end Minsep
