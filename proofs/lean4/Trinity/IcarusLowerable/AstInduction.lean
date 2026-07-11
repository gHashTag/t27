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
    (unsupportedIcarus : ∀ r, P (Expr.unsupportedIcarus r))
    : ∀ e, P e := by
  intro e
  apply Expr.rec
    (motive_1 := P)
    (motive_2 := fun args => ∀ e ∈ args, P e)
    (motive_3 := fun fields => ∀ p ∈ fields, P p.2)
    (motive_4 := fun p => P p.2)
    boolLit intLit f32Lit stringLit identifier binop unop fieldAccess index
    (fun _ _ ih => call _ _ ih)
    (fun _ _ ih => structLit _ _ ih)
    (fun _ _ ih => arrayLit _ _ ih)
    enumVal len contains unsupportedIcarus
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
    (fun _ _ ih => ih)
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

end Trinity.IcarusLowerable
