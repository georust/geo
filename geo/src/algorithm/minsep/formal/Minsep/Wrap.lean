/-
Finding §3.4 ("fixed-direction subchain scans can wrap") of `paper_corrections.typ`,
on the instance

    P: POLYGON((0 0, -1 -2, 2 -1, 0 0))
    Q: POLYGON((-8 -4, -8 -8, 4 -6, -8 -4))

The geodesic from `p_b = (0, 0)` to `p_t = (2, -1)` is `p1 → p2 → p3`. Its two
prolonged segments cut `Q`'s facing arc, the edge from `q_b = (-8, -4)` to
`q_t = (4, -6)`, in *descending* order: `q₁⁺` comes before `q₀⁺`. A fixed-direction
scan of `Q` from `q₀⁺` to `q₁⁺` therefore wraps round the back of `Q`.

Modelling choices as in `Minsep.Bridge`.
-/
import Minsep.Basic

open Set

set_option linter.unusedSimpArgs false

noncomputable section

namespace Minsep.Wrap

def p1 : Pt := (0, 0)
def p2 : Pt := (-1, -2)
def p3 : Pt := (2, -1)
def q1 : Pt := (-8, -4)
def q2 : Pt := (-8, -8)
def q3 : Pt := (4, -6)

def P : Set Pt := convexHull ℝ {p1, p2, p3}
def Q : Set Pt := convexHull ℝ {q1, q2, q3}
def H : Set Pt := convexHull ℝ (P ∪ Q)

/-- The open channel between `P` and `Q`. -/
def R : Set Pt := interior H \ (P ∪ Q)

/-- Where the first geodesic segment, prolonged past `p2`, meets `Q`. -/
def c0 : Pt := (-32 / 13, -64 / 13)

/-- Where the second geodesic segment, prolonged back past `p2`, meets `Q`. -/
def c1 : Pt := (-22 / 3, -37 / 9)

/-! ### Membership -/

theorem mem_P {z : Pt} :
    z ∈ P ↔ z.2 ≤ 2 * z.1 ∧ z.1 ≤ 3 * z.2 + 5 ∧ z.1 + 2 * z.2 ≤ 0 := by
  rw [P, tri_eq (by norm_num [cross, p1, p2, p3])]
  simp only [triH, hp, mem_inter_iff, mem_ofPred_eq, cross, p1, p2, p3]
  constructor
  · rintro ⟨⟨h1, h2⟩, h3⟩; refine ⟨?_, ?_, ?_⟩ <;> linarith
  · rintro ⟨h1, h2, h3⟩; refine ⟨⟨?_, ?_⟩, ?_⟩ <;> linarith

theorem mem_Q {z : Pt} :
    z ∈ Q ↔ -8 ≤ z.1 ∧ 0 ≤ 6 * z.2 - z.1 + 40 ∧ 6 * z.2 + z.1 + 32 ≤ 0 := by
  rw [Q, tri_eq (by norm_num [cross, q1, q2, q3])]
  simp only [triH, hp, mem_inter_iff, mem_ofPred_eq, cross, q1, q2, q3]
  constructor
  · rintro ⟨⟨h1, h2⟩, h3⟩; refine ⟨?_, ?_, ?_⟩ <;> linarith
  · rintro ⟨h1, h2, h3⟩; refine ⟨⟨?_, ?_⟩, ?_⟩ <;> linarith

theorem p1_mem_P : p1 ∈ P := subset_convexHull ℝ _ (by simp)
theorem p2_mem_P : p2 ∈ P := subset_convexHull ℝ _ (by simp)
theorem p3_mem_P : p3 ∈ P := subset_convexHull ℝ _ (by simp)
theorem q1_mem_Q : q1 ∈ Q := subset_convexHull ℝ _ (by simp)
theorem q3_mem_Q : q3 ∈ Q := subset_convexHull ℝ _ (by simp)
theorem c0_mem_Q : c0 ∈ Q := by rw [mem_Q]; norm_num [c0]
theorem c1_mem_Q : c1 ∈ Q := by rw [mem_Q]; norm_num [c1]

theorem H_sub_hp {a b : Pt} (hP : ∀ x ∈ ({p1, p2, p3} : Set Pt), 0 ≤ cross a b x)
    (hQ : ∀ x ∈ ({q1, q2, q3} : Set Pt), 0 ≤ cross a b x) : H ⊆ hp a b :=
  convexHull_subset_hp fun _ hx =>
    hx.elim (fun h => convexHull_subset_hp hP h) (fun h => convexHull_subset_hp hQ h)

/-- The hull edge behind `P`, `x + 2y ≤ 0`, strictly on the interior. -/
theorem interior_H_back {z : Pt} (hz : z ∈ interior H) : z.1 + 2 * z.2 < 0 := by
  have hsub : H ⊆ hp p3 p1 := H_sub_hp
    (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
        norm_num [cross, p1, p2, p3])
    (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
        norm_num [cross, p3, p1, q1, q2, q3])
  have := interior_hp_subset (by simp [p3, p1]) (interior_mono hsub hz)
  simp only [hpS, mem_ofPred_eq, cross, p3, p1] at this
  linarith

/-- The two bridges are hull edges: `p1 → q1` (bottom, at the path start) and
`q3 → p3` (top). -/
theorem H_sub_bridges : H ⊆ hp p1 q1 ∩ hp q3 p3 := fun _ hz =>
  ⟨H_sub_hp (a := p1) (b := q1)
    (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
        norm_num [cross, p1, p2, p3, q1])
    (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
        norm_num [cross, p1, q1, q2, q3]) hz,
   H_sub_hp (a := q3) (b := p3)
    (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
        norm_num [cross, p1, p2, p3, q3])
    (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
        norm_num [cross, p3, q1, q2, q3]) hz⟩

theorem triS_sub_interior_H {a b c : Pt} (h : 0 < cross a b c) (ha : a ∈ P ∪ Q)
    (hb : b ∈ P ∪ Q) (hc : c ∈ P ∪ Q) : triS a b c ⊆ interior H := by
  refine (triS_subset_interior h).trans (interior_mono (convexHull_mono ?_))
  intro x hx
  simp only [mem_insert_iff, mem_singleton_iff] at hx
  rcases hx with rfl | rfl | rfl <;> assumption

/-- `K = (q1, q3, p1)` spans the channel. -/
theorem mem_interior_H_of_K {z : Pt} (h1 : 0 < 12 * z.2 + 2 * z.1 + 64)
    (h2 : 0 < -4 * z.2 - 6 * z.1) (h3 : 0 < -8 * z.2 + 4 * z.1) : z ∈ interior H := by
  refine triS_sub_interior_H (by norm_num [cross, q1, q3, p1]) (Or.inr q1_mem_Q)
    (Or.inr q3_mem_Q) (Or.inl p1_mem_P) ?_
  simp only [triS, hpS, mem_inter_iff, mem_ofPred_eq, cross, q1, q3, p1]
  refine ⟨⟨?_, ?_⟩, ?_⟩ <;> linarith

/-! ### The geodesic `p1 → p2 → p3` -/

def A : Set Pt := hp p2 p1
def B : Set Pt := hp p3 p2

theorem R_sub_AB : R ⊆ A ∪ B := by
  rintro z ⟨hzH, hzPQ⟩
  have hback := interior_H_back hzH
  by_contra hcon
  simp only [A, B, hp, mem_union, mem_ofPred_eq, cross, p1, p2, p3, not_or, not_le] at hcon
  exact hzPQ (Or.inl (mem_P.2 ⟨by linarith, by linarith, by linarith⟩))

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

theorem K2_closure : convexHull ℝ ({p3, c1, q3} : Set Pt) ⊆ closure R := by
  refine convexHull3_subset_closure (by norm_num [cross, p3, c1, q3]) ?_
  intro z hz
  have hint := triS_sub_interior_H (by norm_num [cross, p3, c1, q3]) (Or.inl p3_mem_P)
    (Or.inr c1_mem_Q) (Or.inr q3_mem_Q) hz
  simp only [triS, hpS, mem_inter_iff, mem_ofPred_eq, cross, p3, c1, q3] at hz
  obtain ⟨⟨h1, h2⟩, h3⟩ := hz
  refine ⟨hint, ?_⟩
  rintro (hP | hQ)
  · have := (mem_P.1 hP).2.1; linarith
  · have := (mem_Q.1 hQ).2.2; linarith

theorem p1_mem_K1 : p1 ∈ convexHull ℝ ({p1, q1, p2} : Set Pt) := subset_convexHull ℝ _ (by simp)
theorem p2_mem_K1 : p2 ∈ convexHull ℝ ({p1, q1, p2} : Set Pt) := subset_convexHull ℝ _ (by simp)
theorem p3_mem_K2 : p3 ∈ convexHull ℝ ({p3, c1, q3} : Set Pt) := subset_convexHull ℝ _ (by simp)

theorem p2_mem_K2 : p2 ∈ convexHull ℝ ({p3, c1, q3} : Set Pt) := by
  rw [tri_eq (by norm_num [cross, p3, c1, q3])]
  simp only [triH, hp, mem_inter_iff, mem_ofPred_eq]
  norm_num [cross, p3, c1, q3, p2]

theorem geodesic_in : PathIn (closure R) [p1, p2, p3] :=
  ⟨(convex_convexHull ℝ _).segment_subset p1_mem_K1 p2_mem_K1 |>.trans K1_closure,
   (convex_convexHull ℝ _).segment_subset p2_mem_K2 p3_mem_K2 |>.trans K2_closure,
   trivial⟩

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

/-! ### The prolongations -/

theorem not_mem_R_of_back {z : Pt} (h : 0 ≤ z.1 + 2 * z.2) : z ∉ R :=
  fun hz => absurd (interior_H_back hz.1) (not_lt.2 h)

theorem ext_l0_bottom : ExtEnd R p1 (p1 - p2) p1 := by
  refine ⟨0, le_rfl, by simp, fun μ h1 h2 => (lt_irrefl _ (h1.trans h2)).elim, ?_, ?_⟩
  · intro _
    refine ⟨1, one_pos, fun μ hμ _ => not_mem_R_of_back ?_⟩
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p1, p2]
    linarith
  · intro h; exact (lt_irrefl _ h).elim

theorem ext_l0_top : ExtEnd R p2 (p2 - p1) c0 := by
  refine ⟨19 / 13, by norm_num, ?_, ?_, fun h => by norm_num at h, ?_⟩
  · ext <;> norm_num [c0, p1, p2]
  · intro μ h1 h2
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p1, p2]
    refine ⟨mem_interior_H_of_K (by norm_num; linarith) (by norm_num; linarith)
      (by norm_num; linarith), ?_⟩
    rintro (hP | hQ)
    · have := (mem_P.1 hP).2.1; norm_num at this; linarith
    · have := (mem_Q.1 hQ).2.2; norm_num at this; linarith
  · intro _ hR
    exact hR.2 (Or.inr c0_mem_Q)

theorem ext_l1_bottom : ExtEnd R p2 (p2 - p3) c1 := by
  refine ⟨19 / 9, by norm_num, ?_, ?_, fun h => by norm_num at h, ?_⟩
  · ext <;> norm_num [c1, p2, p3]
  · intro μ h1 h2
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p2, p3]
    refine ⟨mem_interior_H_of_K (by norm_num; linarith) (by norm_num; linarith)
      (by norm_num; linarith), ?_⟩
    rintro (hP | hQ)
    · have := (mem_P.1 hP).1; norm_num at this; linarith
    · have := (mem_Q.1 hQ).2.2; norm_num at this; linarith
  · intro _ hR
    exact hR.2 (Or.inr c1_mem_Q)

theorem ext_l1_top : ExtEnd R p3 (p3 - p2) p3 := by
  refine ⟨0, le_rfl, by simp, fun μ h1 h2 => (lt_irrefl _ (h1.trans h2)).elim, ?_, ?_⟩
  · intro _
    refine ⟨1, one_pos, fun μ hμ _ => not_mem_R_of_back ?_⟩
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p2, p3]
    linarith
  · intro h; exact (lt_irrefl _ h).elim

/-! ### The cuts -/

/-- `l0 = [p1, c0]` meets `Q` only at `c0`, so `q₀⁺ = c0`. -/
theorem l0_inter_Q : segment ℝ p1 c0 ∩ Q = {c0} := by
  ext z
  simp only [mem_inter_iff, mem_singleton_iff]
  constructor
  · rintro ⟨hz, hzQ⟩
    rw [segment_eq_image'] at hz
    obtain ⟨θ, ⟨_, h1⟩, rfl⟩ := hz
    have := (mem_Q.1 hzQ).2.2
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p1, c0] at this
    have hθ : θ = 1 := by linarith
    subst hθ
    simp
  · rintro rfl
    exact ⟨right_mem_segment ℝ _ _, c0_mem_Q⟩

/-- `l1 = [c1, p3]` meets `Q` only at `c1`, so `q₁⁺ = c1`. -/
theorem l1_inter_Q : segment ℝ c1 p3 ∩ Q = {c1} := by
  ext z
  simp only [mem_inter_iff, mem_singleton_iff]
  constructor
  · rintro ⟨hz, hzQ⟩
    rw [segment_eq_image'] at hz
    obtain ⟨θ, ⟨h0, _⟩, rfl⟩ := hz
    have := (mem_Q.1 hzQ).2.2
    simp only [Prod.fst_add, Prod.snd_add, Prod.smul_fst, Prod.smul_snd, Prod.fst_sub,
      Prod.snd_sub, smul_eq_mul, p3, c1] at this
    have hθ : θ = 0 := by linarith
    subst hθ
    simp
  · rintro rfl
    exact ⟨left_mem_segment ℝ _ _, c1_mem_Q⟩

/-- **§3.4.** On the facing arc of `Q`, the edge from `q_b = q1` to `q_t = q3`, the cut of
the later separator `l1` comes *before* the cut of `l0`: `c1` is at arc parameter `1/18`,
`c0` at `6/13`. The cuts descend, so the scan from `q₀⁺` to `q₁⁺` wraps round `Q`. -/
theorem finding_wrap :
    PathIn (closure R) [p1, p2, p3] ∧
    (∀ L : List Pt, PathIn (closure R) (p1 :: L) →
      (p1 :: L).getLast (List.cons_ne_nil p1 L) = p3 → len [p1, p2, p3] ≤ len (p1 :: L)) ∧
    ExtEnd R p1 (p1 - p2) p1 ∧ ExtEnd R p2 (p2 - p1) c0 ∧
    ExtEnd R p2 (p2 - p3) c1 ∧ ExtEnd R p3 (p3 - p2) p3 ∧
    segment ℝ p1 c0 ∩ Q = {c0} ∧ segment ℝ c1 p3 ∩ Q = {c1} ∧
    H ⊆ hp p1 q1 ∩ hp q3 p3 ∧
    c0 = q1 + (6 / 13 : ℝ) • (q3 - q1) ∧ c1 = q1 + (1 / 18 : ℝ) • (q3 - q1) ∧
    (1 / 18 : ℝ) < 6 / 13 := by
  refine ⟨geodesic_in, geodesic_shortest, ext_l0_bottom, ext_l0_top, ext_l1_bottom,
    ext_l1_top, l0_inter_Q, l1_inter_Q, H_sub_bridges, ?_, ?_, by norm_num⟩
  · ext <;> norm_num [c0, q1, q3]
  · ext <;> norm_num [c1, q1, q3]

end Minsep.Wrap

end
