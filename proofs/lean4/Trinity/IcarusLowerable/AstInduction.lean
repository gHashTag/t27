/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 496
  Induction principles for the simplified t27 AST.

  The `Expr` type is a nested inductive (it contains `List Expr` and
  `List (String × Expr)`), so the default `induction` tactic refuses to work.
  This file provides a custom recursor that delivers an induction hypothesis
  for every sub-expression that appears inside a list.

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

import Trinity.IcarusLowerable.Ast
import Mathlib

namespace Trinity.IcarusLowerable

/-- Custom induction principle for `Expr`.  The automatically generated
    `Expr.rec` handles the nested list subterms, so we package it as a
    recursor that gives an induction hypothesis for every sub-expression
    appearing in a list position. -/
theorem Expr.induction_on_lists {P : Expr → Prop}
    (boolLit : ∀ b, P (Expr.boolLit b))
    (intLit : ∀ n, P (Expr.intLit n))
    (f32Lit : ∀ s, P (Expr.f32Lit s))
    (stringLit : ∀ s, P (Expr.stringLit s))
    (identifier : ∀ n, P (Expr.identifier n))
    (binop : ∀ op l r, P l → P r → P (Expr.binop op l r))
    (unop : ∀ op e, P e → P (Expr.unop op e))
    (fieldAccess : ∀ base f, P base → P (Expr.fieldAccess base f))
    (index : ∀ base idx, P base → P idx → P (Expr.index base idx))
    (call : ∀ name args, (∀ e ∈ args, P e) → P (Expr.call name args))
    (structLit : ∀ name fields, (∀ p ∈ fields, P p.2) → P (Expr.structLit name fields))
    (arrayLit : ∀ ty elems, (∀ e ∈ elems, P e) → P (Expr.arrayLit ty elems))
    (enumVal : ∀ e v, P (Expr.enumVal e v))
    (len : ∀ base, P base → P (Expr.len base))
    (contains : ∀ base item, P base → P item → P (Expr.contains base item))
    (switch : ∀ disc cases default,
        P disc → (∀ p ∈ cases, P p.1 ∧ P p.2) → P default → P (Expr.switch disc cases default))
    (unsupportedIcarus : ∀ r, P (Expr.unsupportedIcarus r))
    : ∀ e, P e := by
  intro e
  apply Expr.rec
    (motive_1 := P)
    (motive_2 := fun args => ∀ e ∈ args, P e)
    (motive_3 := fun fields => ∀ p ∈ fields, P p.2)
    (motive_4 := fun cases => ∀ p ∈ cases, P p.1 ∧ P p.2)
    (motive_5 := fun p : String × Expr => P p.2)
    (motive_6 := fun p : Expr × Expr => P p.1 ∧ P p.2)
    boolLit intLit f32Lit stringLit identifier binop unop fieldAccess index
    (fun _ _ ih => call _ _ ih)
    (fun _ _ ih => structLit _ _ ih)
    (fun _ _ ih => arrayLit _ _ ih)
    enumVal len contains
    (fun _ _ _ ih1 ih2 ih3 => switch _ _ _ ih1 ih2 ih3)
    unsupportedIcarus
    (by simp)
    (fun head tail ih1 ih2 e he => by
      rcases List.mem_cons.mp he with (rfl | he')
      · exact ih1
      · exact ih2 e he')
    (by simp)
    (fun head tail ih1 ih2 p hp => by
      rcases List.mem_cons.mp hp with (rfl | hp')
      · exact ih1
      · exact ih2 p hp')
    (by simp)
    (fun head tail ih1 ih2 p hp => by
      rcases List.mem_cons.mp hp with (rfl | hp')
      · exact ih1
      · exact ih2 p hp')
    (fun _ _ ih => ih)
    (fun _ _ ih1 ih2 => ⟨ih1, ih2⟩)
    e

/-- `List.all` is equivalent to the universal quantifier over list membership. -/
theorem List.all_iff {α : Type} (p : α → Bool) (xs : List α) :
    xs.all p ↔ ∀ x ∈ xs, p x := by
  induction xs with
  | nil => simp
  | cons x xs ih => simp [ih]

/-- Membership from a successful `List.find?`. -/
theorem List.find?_mem {α : Type} {p : α → Bool} {xs : List α} {x : α}
    (h : xs.find? p = some x) : x ∈ xs := by
  induction xs with
  | nil => simp at h
  | cons y ys ih =>
    simp at h
    rcases h with (⟨hp, heq⟩ | ⟨hp, hfind⟩)
    · simp [heq]
    · simp [ih hfind]

/-- The element returned by a successful `List.find?` satisfies the predicate. -/
theorem List.find?_some {α : Type} {p : α → Bool} {xs : List α} {x : α}
    (h : xs.find? p = some x) : p x = true := by
  induction xs with
  | nil => simp at h
  | cons y ys ih =>
    simp at h
    rcases h with (⟨hp, heq⟩ | ⟨hp, hfind⟩)
    · simp [heq] at hp
      exact hp
    · exact ih hfind

end Trinity.IcarusLowerable

namespace List

/-- If `x` is in `xs` and satisfies the predicate `p`, then `find? p xs` returns
    `some x` (or some other satisfying element earlier in the list). -/
theorem find?_mem_eq {α : Type} {p : α → Bool} {xs : List α} {x : α}
    (hmem : x ∈ xs) (hp : p x = true) :
    ∃ y, xs.find? p = some y := by
  induction xs with
  | nil => simp at hmem
  | cons z zs ih =>
      have h : x = z ∨ x ∈ zs := List.mem_cons.mp hmem
      cases h with
      | inl heq =>
          rw [heq] at hp
          exact ⟨z, by simp [hp]⟩
      | inr hmem =>
          cases hz : p z with
          | true => exact ⟨z, by simp [hz]⟩
          | false =>
              have : ∃ y, zs.find? p = some y := ih hmem
              rcases this with ⟨y, hy⟩
              exact ⟨y, by simp [hz, hy]⟩

end List
