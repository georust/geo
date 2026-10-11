/-
Finding §3.5 ("the single-contact (pinched) channel") of `paper_corrections.typ`,
on the instance

    P: POLYGON((3 2, 0 -4, 4 2, 3 2))
    Q: POLYGON((0 8, 2 3, 8 4, 0 8))

Amato (1994), Step 1(a), takes the two common supporting segments to have distinct
contacts `p_b ≠ p_t` on `P`. Here `P` meets the boundary of `CH(P ∪ Q)` in a single
point, so every supporting segment touches `P` at that point: `p_b = p_t`.
-/
import Minsep.Basic

open Set

noncomputable section

namespace Minsep.Pinch

def p1 : Pt := (3, 2)
def p2 : Pt := (0, -4)
def p3 : Pt := (4, 2)
def q1 : Pt := (0, 8)
def q2 : Pt := (2, 3)
def q3 : Pt := (8, 4)

def P : Set Pt := convexHull ℝ {p1, p2, p3}
def Q : Set Pt := convexHull ℝ {q1, q2, q3}
def H : Set Pt := convexHull ℝ (P ∪ Q)

theorem mem_P {z : Pt} :
    z ∈ P ↔ 0 ≤ 6 * z.1 - 3 * z.2 - 12 ∧ 0 ≤ 4 * z.2 - 6 * z.1 + 16 ∧ z.2 ≤ 2 := by
  rw [P, tri_eq (by norm_num [cross, p1, p2, p3])]
  simp only [triH, hp, mem_inter_iff, mem_ofPred_eq, cross, p1, p2, p3]
  constructor
  · rintro ⟨⟨h1, h2⟩, h3⟩; refine ⟨?_, ?_, ?_⟩ <;> linarith
  · rintro ⟨h1, h2, h3⟩; refine ⟨⟨?_, ?_⟩, ?_⟩ <;> linarith

/-- The hull contains the open triangle `(p2, q3, q1)`. -/
theorem triS_sub_interior_H : triS p2 q3 q1 ⊆ interior H := by
  refine (triS_subset_interior (by norm_num [cross, p2, q3, q1])).trans
    (interior_mono (convexHull_mono ?_))
  intro x hx
  simp only [mem_insert_iff, mem_singleton_iff] at hx
  rcases hx with rfl | rfl | rfl
  · exact Or.inl (subset_convexHull ℝ _ (by simp))
  · exact Or.inr (subset_convexHull ℝ _ (by simp))
  · exact Or.inr (subset_convexHull ℝ _ (by simp))

/-- The hull lies right of the line `x = 0`. -/
theorem H_sub_left : H ⊆ hp q1 p2 :=
  convexHull_subset_hp fun _ hx => hx.elim
    (fun h => convexHull_subset_hp
      (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
          norm_num [cross, p1, p2, p3, q1]) h)
    (fun h => convexHull_subset_hp
      (by simp only [mem_insert_iff, mem_singleton_iff, forall_eq_or_imp, forall_eq];
          norm_num [cross, p2, q1, q2, q3]) h)

/-- Every point of `P` except `p2` is interior to `CH(P ∪ Q)`. -/
theorem mem_interior_H {z : Pt} (hz : z ∈ P) (hne : z ≠ p2) : z ∈ interior H := by
  obtain ⟨h1, h2, h3⟩ := mem_P.1 hz
  have hx : 0 < z.1 := by
    by_contra hle
    apply hne
    ext <;> simp only [p2] <;> linarith
  apply triS_sub_interior_H
  simp only [triS, hpS, mem_inter_iff, mem_ofPred_eq, cross, p2, q3, q1]
  refine ⟨⟨?_, ?_⟩, ?_⟩ <;> linarith

theorem p2_not_mem_interior_H : p2 ∉ interior H := by
  intro h
  have := interior_hp_subset (by norm_num [q1, p2, Prod.ext_iff]) (interior_mono H_sub_left h)
  norm_num [hpS, cross, q1, p2] at this

/-- **§3.5.** `P` meets the boundary of `CH(P ∪ Q)` only at `p2 = (0, -4)`. Both common
supporting segments lie on that boundary, so both contact `P` at `p2`: `p_b = p_t`, and
the channel is pinched there. -/
theorem finding_pinch : {z ∈ P | z ∉ interior H} = {p2} := by
  ext z
  simp only [mem_ofPred_eq, mem_singleton_iff]
  constructor
  · rintro ⟨hz, hint⟩
    by_contra hne
    exact hint (mem_interior_H hz hne)
  · rintro rfl
    exact ⟨subset_convexHull ℝ _ (by simp), p2_not_mem_interior_H⟩

end Minsep.Pinch

end
