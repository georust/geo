/-
Findings §3.1 ("a separator can miss one polygon entirely") and §3.2 ("the
orientation rule contradicts the path-order conventions") of
`paper_corrections.typ`, on the instance

    P: POLYGON((0 2, 0 -2, 2 0, 0 2))
    Q: POLYGON((-6 -2, -6 -6, -2 -5, -6 -2))

Modelling choices, stated once:

* `P`, `Q` are the closed triangles; `H = CH(P ∪ Q)`.
* The channel `R` is `interior H \ (P ∪ Q)`. For two disjoint convex polygons in
  the non-containing case this is exactly the open polygon bounded by the two
  facing arcs and the two bridges.
* The geodesic is shortest among polygonal paths in `closure R`.
* `ExtEnd` (in `Basic`) formalises the prolongation of a path segment: the
  segment is extended while it stays in the open channel, up to the first
  point outside it.
-/
import Minsep.Basic

open Set

set_option linter.unusedSimpArgs false

noncomputable section

namespace Minsep.Bridge

def p1 : Pt := (0, 2)
def p2 : Pt := (0, -2)
def p3 : Pt := (2, 0)
def q1 : Pt := (-6, -2)
def q2 : Pt := (-6, -6)
def q3 : Pt := (-2, -5)

def P : Set Pt := convexHull ℝ {p1, p2, p3}
def Q : Set Pt := convexHull ℝ {q1, q2, q3}
def H : Set Pt := convexHull ℝ (P ∪ Q)

/-- The open channel between `P` and `Q`. -/
def R : Set Pt := interior H \ (P ∪ Q)

/-- Where the first geodesic segment, prolonged past `p2`, leaves the channel. -/
def w0 : Pt := (0, -5 / 2)

/-- Where the second geodesic segment, prolonged back past `p2`, leaves the channel. -/
def b1 : Pt := (-18 / 7, -32 / 7)

/-! ### Membership -/

theorem mem_P {z : Pt} : z ∈ P ↔ 0 ≤ z.1 ∧ z.1 - 2 ≤ z.2 ∧ z.1 + z.2 ≤ 2 := by
  rw [P, tri_eq (by norm_num [cross, p1, p2, p3])]
  simp only [triH, hp, mem_inter_iff, mem_ofPred_eq, cross, p1, p2, p3]
  constructor
  · rintro ⟨⟨h1, h2⟩, h3⟩; refine ⟨?_, ?_, ?_⟩ <;> linarith
  · rintro ⟨h1, h2, h3⟩; refine ⟨⟨?_, ?_⟩, ?_⟩ <;> linarith

theorem mem_Q {z : Pt} :
    z ∈ Q ↔ -6 ≤ z.1 ∧ 0 ≤ 4 * z.2 - z.1 + 18 ∧ 4 * z.2 + 3 * z.1 + 26 ≤ 0 := by
  rw [Q, tri_eq (by norm_num [cross, q1, q2, q3])]
  simp only [triH, hp, mem_inter_iff, mem_ofPred_eq, cross, q1, q2, q3]
  constructor
  · rintro ⟨⟨h1, h2⟩, h3⟩; refine ⟨?_, ?_, ?_⟩ <;> linarith
  · rintro ⟨h1, h2, h3⟩; refine ⟨⟨?_, ?_⟩, ?_⟩ <;> linarith

theorem P_sub_H : P ⊆ H := fun _ h => subset_convexHull ℝ _ (Or.inl h)
theorem Q_sub_H : Q ⊆ H := fun _ h => subset_convexHull ℝ _ (Or.inr h)

theorem H_sub_hp {a b : Pt} (hP : ∀ x ∈ ({p1, p2, p3} : Set Pt), 0 ≤ cross a b x)
    (hQ : ∀ x ∈ ({q1, q2, q3} : Set Pt), 0 ≤ cross a b x) : H ⊆ hp a b :=
  convexHull_subset_hp fun _ hx =>
    hx.elim (fun h => convexHull_subset_hp hP h) (fun h => convexHull_subset_hp hQ h)

/-- The hull edge behind `P`: `x + y ≤ 2`, strictly on the interior. -/
theorem interior_H_back {z : Pt} (hz : z ∈ interior H) : z.1 + z.2 < 2 := by
  have hsub : H ⊆ hp p3 p1 := H_sub_hp
    (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
        norm_num [cross, p1, p2, p3])
    (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
        norm_num [cross, p3, p1, q1, q2, q3])
  have := interior_hp_subset (by simp [p3, p1]) (interior_mono hsub hz)
  simp only [hpS, mem_ofPred_eq, cross, p3, p1] at this
  linarith

/-- The bottom bridge `q3 → p3` is a hull edge. -/
theorem H_sub_bottom : H ⊆ hp q3 p3 := H_sub_hp
  (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
      norm_num [cross, p1, p2, p3, q3])
  (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
      norm_num [cross, p3, q1, q2, q3])

/-- The top bridge `p1 → q1` is a hull edge. -/
theorem H_sub_top : H ⊆ hp p1 q1 := H_sub_hp
  (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
      norm_num [cross, p1, p2, p3, q1])
  (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
      norm_num [cross, p1, q1, q2, q3])

theorem interior_H_bottom {z : Pt} (hz : z ∈ interior H) : 0 < 4 * z.2 - 5 * z.1 + 10 := by
  have := interior_hp_subset (by simp [q3, p3]) (interior_mono H_sub_bottom hz)
  simp only [hpS, mem_ofPred_eq, cross, q3, p3] at this
  linarith

theorem b1_mem_Q : b1 ∈ Q := by
  rw [mem_Q]; norm_num [b1]

theorem p1_mem_P : p1 ∈ P := subset_convexHull ℝ _ (by simp)
theorem p2_mem_P : p2 ∈ P := subset_convexHull ℝ _ (by simp)
theorem p3_mem_P : p3 ∈ P := subset_convexHull ℝ _ (by simp)
theorem q1_mem_Q : q1 ∈ Q := subset_convexHull ℝ _ (by simp)
theorem q3_mem_Q : q3 ∈ Q := subset_convexHull ℝ _ (by simp)

/-- A counter-clockwise triangle with vertices in `P ∪ Q` has its open interior in the
interior of `H`. -/
theorem triS_sub_interior_H {a b c : Pt} (h : 0 < cross a b c) (ha : a ∈ P ∪ Q)
    (hb : b ∈ P ∪ Q) (hc : c ∈ P ∪ Q) : triS a b c ⊆ interior H := by
  refine (triS_subset_interior h).trans (interior_mono (convexHull_mono ?_))
  intro x hx
  simp only [mem_insert_iff, mem_singleton_iff] at hx
  rcases hx with rfl | rfl | rfl <;> assumption

/-- `K4 = (p3, q1, q3)` spans the lower part of the channel. -/
theorem mem_interior_H_of_K4 {z : Pt} (h1 : 0 < -8 * z.2 + 2 * z.1 - 4)
    (h2 : 0 < 4 * z.2 + 3 * z.1 + 26) (h3 : 0 < 4 * z.2 - 5 * z.1 + 10) : z ∈ interior H := by
  refine triS_sub_interior_H (by norm_num [cross, p3, q1, q3]) (Or.inl p3_mem_P)
    (Or.inr q1_mem_Q) (Or.inr q3_mem_Q) ?_
  simp only [triS, hpS, mem_inter_iff, mem_ofPred_eq, cross, p3, q1, q3]
  refine ⟨⟨?_, ?_⟩, ?_⟩ <;> linarith

/-! ### The geodesic `p1 → p2 → p3` -/

/-- The closed region left of `p1`'s facing edge, `x ≤ 0`. -/
def A : Set Pt := hp p2 p1
/-- The closed region below `P`'s other facing edge, `y ≤ x - 2`. -/
def B : Set Pt := hp p3 p2

theorem R_sub_AB : R ⊆ A ∪ B := by
  rintro z ⟨hzH, hzPQ⟩
  have hback := interior_H_back hzH
  by_contra hcon
  simp only [A, B, hp, mem_union, mem_ofPred_eq, cross, p1, p2, p3, not_or, not_le] at hcon
  exact hzPQ (Or.inl (mem_P.2 ⟨by linarith, by linarith, by linarith⟩))

/-- The open triangle `(p1, q1, p2)`, left of the first geodesic segment, is in `R`. -/
theorem K1_closure : convexHull ℝ ({p1, q1, p2} : Set Pt) ⊆ closure R := by
  refine convexHull3_subset_closure (by norm_num [cross, p1, q1, p2]) ?_
  intro z hz
  have hint := triS_sub_interior_H (by norm_num [cross, p1, q1, p2]) (Or.inl p1_mem_P)
    (Or.inr q1_mem_Q) (Or.inl p2_mem_P) hz
  simp only [triS, hpS, mem_inter_iff, mem_ofPred_eq, cross, p1, q1, p2] at hz
  obtain ⟨⟨h1, h2⟩, h3⟩ := hz
  refine ⟨hint, ?_⟩
  rintro (hP | hQ)
  · have := (mem_P.1 hP).1; linarith
  · obtain ⟨k1, k2, k3⟩ := mem_Q.1 hQ; linarith

/-- The open triangle `(p3, b1, q3)`, below the second geodesic segment, is in `R`. -/
theorem K2_closure : convexHull ℝ ({p3, b1, q3} : Set Pt) ⊆ closure R := by
  refine convexHull3_subset_closure (by norm_num [cross, p3, b1, q3]) ?_
  intro z hz
  have hint := triS_sub_interior_H (by norm_num [cross, p3, b1, q3]) (Or.inl p3_mem_P)
    (Or.inr b1_mem_Q) (Or.inr q3_mem_Q) hz
  simp only [triS, hpS, mem_inter_iff, mem_ofPred_eq, cross, p3, b1, q3] at hz
  obtain ⟨⟨h1, h2⟩, h3⟩ := hz
  refine ⟨hint, ?_⟩
  rintro (hP | hQ)
  · have := (mem_P.1 hP).2.1; norm_num at this; linarith
  · have := (mem_Q.1 hQ).2.2; linarith

theorem p1_mem_K1 : p1 ∈ convexHull ℝ ({p1, q1, p2} : Set Pt) := subset_convexHull ℝ _ (by simp)
theorem p2_mem_K1 : p2 ∈ convexHull ℝ ({p1, q1, p2} : Set Pt) := subset_convexHull ℝ _ (by simp)
theorem p3_mem_K2 : p3 ∈ convexHull ℝ ({p3, b1, q3} : Set Pt) := subset_convexHull ℝ _ (by simp)
theorem b1_mem_K2 : b1 ∈ convexHull ℝ ({p3, b1, q3} : Set Pt) := subset_convexHull ℝ _ (by simp)

theorem p2_mem_K2 : p2 ∈ convexHull ℝ ({p3, b1, q3} : Set Pt) := by
  rw [tri_eq (by norm_num [cross, p3, b1, q3])]
  simp only [triH, hp, mem_inter_iff, mem_ofPred_eq]
  norm_num [cross, p3, b1, q3, p2]

/-- The geodesic lies in the closed channel. -/
theorem geodesic_in : PathIn (closure R) [p1, p2, p3] :=
  ⟨(convex_convexHull ℝ _).segment_subset p1_mem_K1 p2_mem_K1 |>.trans K1_closure,
   (convex_convexHull ℝ _).segment_subset p2_mem_K2 p3_mem_K2 |>.trans K2_closure,
   trivial⟩

/-- **The geodesic.** No polygonal path in the closed channel from `p1` to `p3` is
shorter than `p1 → p2 → p3`. -/
theorem geodesic_shortest (L : List Pt) (hL : PathIn (closure R) (p1 :: L))
    (hlast : (p1 :: L).getLast (List.cons_ne_nil p1 L) = p3) :
    len [p1, p2, p3] ≤ len (p1 :: L) := by
  refine single_turn (isClosed_hp _ _) (isClosed_hp _ _)
    (closure_minimal R_sub_AB ((isClosed_hp _ _).union (isClosed_hp _ _)))
    ?_ ?_ (K1_closure p1_mem_K1) ?_ L hL hlast
  · simp only [A, hp, mem_ofPred_eq]; norm_num [cross, p1, p2]
  · simp only [B, hp, mem_ofPred_eq]; norm_num [cross, p3, p2]
  · rintro z ⟨_, hA, hB⟩
    simp only [A, B, hp, mem_ofPred_eq, cross, p1, p2, p3] at hA hB
    simp only [dot, Prod.fst_sub, Prod.snd_sub, p1, p2, p3]
    constructor <;> nlinarith

/-! ### The prolongations of the two geodesic segments -/

theorem not_mem_R_of_back {z : Pt} (h : 2 ≤ z.1 + z.2) : z ∉ R :=
  fun hz => absurd (interior_H_back hz.1) (not_lt.2 h)

/-- The first segment cannot be prolonged backwards past `p1`. -/
theorem ext_l0_bottom : ExtEnd R p1 (p1 - p2) p1 := by
  refine ⟨0, le_rfl, by simp, fun μ h1 h2 => (lt_irrefl _ (h1.trans h2)).elim, ?_, ?_⟩
  · intro _
    refine ⟨1, one_pos, fun μ hμ _ => not_mem_R_of_back ?_⟩
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p1, p2]
    linarith
  · intro h; exact (lt_irrefl _ h).elim

/-- The first segment, prolonged past `p2`, leaves the channel at `w0`. -/
theorem ext_l0_top : ExtEnd R p2 (p2 - p1) w0 := by
  refine ⟨1 / 8, by norm_num, ?_, ?_, fun h => by norm_num at h, ?_⟩
  · ext <;> norm_num [w0, p1, p2]
  · intro μ h1 h2
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p1, p2]
    refine ⟨mem_interior_H_of_K4 (by norm_num; linarith) (by norm_num; linarith)
      (by norm_num; linarith), ?_⟩
    rintro (hP | hQ)
    · have := (mem_P.1 hP).2.1; norm_num at this; linarith
    · have := (mem_Q.1 hQ).2.2; norm_num at this; linarith
  · intro _ hR
    have := interior_H_bottom hR.1
    norm_num [w0] at this

/-- The second segment, prolonged back past `p2`, leaves the channel at `b1 ∈ Q`. -/
theorem ext_l1_bottom : ExtEnd R p2 (p2 - p3) b1 := by
  refine ⟨9 / 7, by norm_num, ?_, ?_, fun h => by norm_num at h, ?_⟩
  · ext <;> norm_num [b1, p2, p3]
  · intro μ h1 h2
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p2, p3]
    refine ⟨mem_interior_H_of_K4 (by norm_num; linarith) (by norm_num; linarith)
      (by norm_num; linarith), ?_⟩
    rintro (hP | hQ)
    · have := (mem_P.1 hP).1; norm_num at this; linarith
    · have := (mem_Q.1 hQ).2.2; norm_num at this; linarith
  · intro _ hR
    exact hR.2 (Or.inr b1_mem_Q)

/-- The second segment cannot be prolonged forwards past `p3`. -/
theorem ext_l1_top : ExtEnd R p3 (p3 - p2) p3 := by
  refine ⟨0, le_rfl, by simp, fun μ h1 h2 => (lt_irrefl _ (h1.trans h2)).elim, ?_, ?_⟩
  · intro _
    refine ⟨1, one_pos, fun μ hμ _ => not_mem_R_of_back ?_⟩
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p2, p3]
    linarith
  · intro h; exact (lt_irrefl _ h).elim

/-! ### Finding §3.1: the separator `l0 = [p1, w0]` misses `Q` -/

theorem w0_on_bottom_bridge : w0 ∈ segment ℝ q3 p3 := by
  rw [segment_eq_image']
  exact ⟨1 / 2, ⟨by norm_num, by norm_num⟩, by ext <;> norm_num [w0, q3, p3]⟩

theorem fst_eq_zero_of_mem_l0 {z : Pt} (hz : z ∈ segment ℝ p1 w0) : z.1 = 0 := by
  rw [segment_eq_image'] at hz
  obtain ⟨θ, _, rfl⟩ := hz
  simp [p1, w0]

/-- **§3.1.** The first separator is disjoint from `Q`, so the cut `q0⁺` does not exist,
contradicting the claim (Amato 1994, §3) that every separator intersects both polygons. -/
theorem l0_disjoint_Q : Disjoint (segment ℝ p1 w0) Q := by
  rw [Set.disjoint_left]
  intro z hz hzQ
  have h0 := fst_eq_zero_of_mem_l0 hz
  obtain ⟨_, k2, k3⟩ := mem_Q.1 hzQ
  linarith

/-- **§3.1, assembled.** `p1 → p2 → p3` is a shortest path in the closed channel; the
prolongation of its first segment is `[p1, w0]`; `w0` lies on the bottom bridge; and the
whole separator misses `Q`. -/
theorem finding_bridge :
    PathIn (closure R) [p1, p2, p3] ∧
    (∀ L : List Pt, PathIn (closure R) (p1 :: L) →
      (p1 :: L).getLast (List.cons_ne_nil p1 L) = p3 → len [p1, p2, p3] ≤ len (p1 :: L)) ∧
    ExtEnd R p1 (p1 - p2) p1 ∧ ExtEnd R p2 (p2 - p1) w0 ∧
    w0 ∈ segment ℝ q3 p3 ∧ H ⊆ hp q3 p3 ∧
    Disjoint (segment ℝ p1 w0) Q :=
  ⟨geodesic_in, geodesic_shortest, ext_l0_bottom, ext_l0_top, w0_on_bottom_bridge,
    H_sub_bottom, l0_disjoint_Q⟩

/-! ### Finding §3.2: closest-endpoint orientation disagrees with path order -/

/-- **§3.2.** The second separator is `[b1, p3]`, with `b1` on the side of `p2` (path
order: `b1` is its bottom). Yet `p3` is strictly closer to `l0` than `b1` is, so the
rule "the bottom endpoint of `l1` is the endpoint closest to `l0`" picks `p3`. -/
theorem finding_orientation :
    ExtEnd R p2 (p2 - p3) b1 ∧ ExtEnd R p3 (p3 - p2) p3 ∧
    ∃ y ∈ segment ℝ p1 w0, ∀ y' ∈ segment ℝ p1 w0, eu p3 y < eu b1 y' := by
  refine ⟨ext_l1_bottom, ext_l1_top, (0, 0), ?_, fun y' hy' => ?_⟩
  · rw [segment_eq_image']
    exact ⟨4 / 9, ⟨by norm_num, by norm_num⟩, by ext <;> norm_num [p1, w0]⟩
  · have h0 := fst_eq_zero_of_mem_l0 hy'
    have e1 : eu p3 (0, 0) ≤ 2 := eu_le_of_sq (by norm_num) (by norm_num [p3])
    have e2 : 18 / 7 ≤ eu b1 y' := le_eu_of_sq (by norm_num) (by
      rw [h0]; norm_num [b1]; positivity)
    linarith

end Minsep.Bridge

end
