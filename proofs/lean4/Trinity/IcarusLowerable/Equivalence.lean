/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 498
  Generic structural equivalence proof for the Icarus-lowerable combinational
  subset.

  This file closes the remaining `sorry` in `Soundness.lean` by a combined
  fuel/AST forward-simulation argument.

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

import Trinity.IcarusLowerable.Ast
import Trinity.IcarusLowerable.Predicate
import Trinity.IcarusLowerable.Verilog
import Trinity.IcarusLowerable.Emitter
import Trinity.IcarusLowerable.Semantics
import Trinity.IcarusLowerable.SemanticsTotal
import Trinity.IcarusLowerable.AstInduction
import Std.Data.String.ToInt

set_option maxRecDepth 2048
set_option maxHeartbeats 1000000

namespace Trinity.IcarusLowerable

set_option linter.unusedSimpArgs false
set_option linter.unreachableTactic false
set_option linter.unusedTactic false
set_option linter.unusedVariables false
set_option linter.unnecessarySeqFocus false

/- φ² + φ⁻² = 3 | TRINITY -/

/-- Congruence for `Option.bind`. -/
theorem Option.bind_congr {α β : Type u} {oa ob : Option α} (h : oa = ob)
    (f : α → Option β) : oa.bind f = ob.bind f := by
  rw [h]

/-- Extensional congruence for `Option.bind`: the continuations agree on the
    carried value. -/
theorem Option.bind_congr_ext {α β} {oa ob : Option α} {f g : α → Option β}
    (ho : oa = ob) (hf : ∀ a, f a = g a) : oa.bind f = ob.bind g := by
  rw [ho]
  cases ob with
  | none => simp
  | some a => simp [hf a]

/-- Associativity of `Option.bind`. -/
theorem Option.bind_assoc {α β γ} (x : Option α) (f : α → Option β) (g : β → Option γ) :
    (x.bind f).bind g = x.bind (fun a => (f a).bind g) := by
  cases x with
  | none => simp
  | some a => simp

/-- The integer-literal string roundtrip used by the Verilog evaluator. -/
theorem Int.toInt?_toString (n : Int) : String.toInt? (toString n) = some n := by
  rw [Int.toString_eq_repr]
  exact Int.toInt?_repr n

/-- Evaluating a Verilog literal whose string comes from `toString n` always
    yields the original integer bit-vector. -/
theorem evalVExprTotal_lit_toString (fuel : Nat) (env : Env) (vm : VModule)
    (val : Valuation) (w : Nat) (n : Int) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.lit w (toString n)) =
    some ⟨w, BitVec.ofInt w n⟩ := by
  simp [evalVExprTotal, Int.toInt?_toString]

/-- Extensionality for runtime values. -/
theorem Value.ext {v1 v2 : Value} (hw : v1.width = v2.width)
    (hb : HEq v1.bits v2.bits) : v1 = v2 := by
  cases v1; cases v2; cases hw; simp at hb ⊢; exact hb

/-- Pointwise valuation update. -/
def Valuation.set (val : Valuation) (name : String) (v : Option Value) :
    Valuation :=
  fun x => if x == name then v else val x

/-- Updating equivalent valuations with equal values preserves equivalence. -/
theorem Valuation.equiv_set {val1 val2 : Valuation} {name : String}
    {v1 v2 : Option Value} (hequiv : Valuation.equiv val1 val2) (hv : v1 = v2) :
    Valuation.equiv (val1.set name v1) (val2.set name v2) := by
  intro x
  simp [Valuation.set, hequiv x]
  split <;> simp [hv]

/-- `Valuation.set` expands to the inline update used by the total evaluator. -/
theorem Valuation.set_eq (val : Valuation) (name : String) (v : Value) :
    val.set name (some v) = fun x => if x == name then some v else val x := by
  funext x
  simp [Valuation.set]

/-- Reflexivity of valuation equivalence. -/
theorem Valuation.equiv_refl (val : Valuation) : Valuation.equiv val val :=
  fun _ => rfl

/-- `Value.concatList` is congruent over equal lists of values. -/
theorem Value.concatList_congr {vs1 vs2 : List Value} (h : vs1 = vs2) :
    Value.concatList vs1 = Value.concatList vs2 := by
  rw [h]

/-- `widthOfType` ignores its fuel parameter, so widths are stable across the
    fuel induction. -/
theorem widthOfType_fuel_independent (fuel1 fuel2 : Nat) (env : Env) (ty : Ty) :
    widthOfType fuel1 env ty = widthOfType fuel2 env ty := by
  simp [widthOfType]

/-- For positive width, the low offset of a slice never exceeds the high offset. -/
lemma offset_le_add_sub_one (off w : Nat) (hw : w > 0) : off ≤ off + w - 1 := by
  have h1 : 1 ≤ w := by exact hw
  have h2 : off + w - 1 = off + (w - 1) := by rw [Nat.add_sub_assoc h1]
  rw [h2]
  apply Nat.le_add_right

/-- For positive width, slicing from `off` to `off + w - 1` yields width `w`. -/
lemma slice_width_eq (off w : Nat) (hw : w > 0) : (off + w - 1) - off + 1 = w := by
  have h1 : 1 ≤ w := by exact hw
  have h2 : off + w - 1 = off + (w - 1) := by rw [Nat.add_sub_assoc h1]
  rw [h2]
  have h3 : (off + (w - 1)) - off = w - 1 := Nat.add_sub_cancel_left off (w - 1)
  have h4 : (w - 1) + 1 = w := Nat.sub_add_cancel h1
  rw [h3, h4]

/-- The emitted `indexElemWidth` equals the width of the array element type. -/
theorem indexElemWidth_eq (fuel : Nat) (env : Env) (m : Module) (base : Expr)
    (hty : Expr.typeOf env m base = some (.array n elem)) :
    indexElemWidth fuel env m base = widthOfType fuel env elem := by
  simp [indexElemWidth, hty]

/-- `List.mapM` preserves equality when the elementwise mapping does. -/
theorem List.mapM_congr' {α β} {f g : α → Option β} {xs : List α}
    (h : ∀ a ∈ xs, f a = g a) : List.mapM f xs = List.mapM g xs := by
  induction xs with
  | nil =>
      simp [← List.mapM'_eq_mapM]
  | cons x xs ih =>
      have hx := h x (by simp)
      have hrest : ∀ a ∈ xs, f a = g a := fun a ha => h a (by simp [ha])
      simp only [← List.mapM'_eq_mapM, List.mapM'_cons, hx]
      apply Option.bind_congr_ext rfl
      intro y
      simp only [List.mapM'_eq_mapM, ih hrest]

/-- `List.mapM` commutes with `List.map`: applying a monadic function to a
    mapped list equals applying the composition to the original list. -/
theorem List.mapM_map {α β γ} (f : β → Option γ) (g : α → β) (xs : List α) :
    List.mapM f (xs.map g) = List.mapM (fun a => f (g a)) xs := by
  induction xs with
  | nil =>
      simp [← List.mapM'_eq_mapM]
  | cons x xs ih =>
      simp only [← List.mapM'_eq_mapM, List.mapM'_cons, List.map_cons]
      apply Option.bind_congr_ext rfl
      intro y
      simp only [List.mapM'_eq_mapM, ih]

/-- Empty arguments make the function-call initial valuation equal to the base
    valuation, so the parameter lookup is a no-op. -/
theorem evalFunctionTotal_empty_args (fuel : Nat) (env : Env) (m : Module)
    (fn : Function) (base : Valuation) :
    evalFunctionTotal fuel env m fn [] base =
    (evalStmtsTotal (fuel - 1) env m base fn.body).bind (fun final => final "__return") := by
  cases fuel with
  | zero =>
      simp [evalFunctionTotal, evalStmtsTotal]
  | succ n =>
      have hinit : (fun name =>
          (fn.params.zip []).find? (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name)) = base := by
        funext name
        simp
      simp [evalFunctionTotal, evalStmtsTotal, hinit]

theorem evalVFunctionTotal_empty_args (fuel : Nat) (env : Env) (vm : VModule)
    (fn : VFunction) (base : Valuation) :
    evalVFunctionTotal fuel env vm fn [] base =
    (evalVStmtsTotal (fuel - 1) env vm base fn.body).bind (fun final => final "__return") := by
  cases fuel with
  | zero =>
      simp [evalVFunctionTotal, evalVStmtsTotal]
  | succ n =>
      have hinit : (fun name =>
          (fn.params.zip []).find? (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name)) = base := by
        funext name
        simp
      simp [evalVFunctionTotal, evalVStmtsTotal, hinit]

/-- Membership from a successful `Module.findFunction`. -/
theorem Module.findFunction_mem {m : Module} {name : String} {fn : Function}
    (h : m.findFunction name = some fn) :
    fn ∈ m.functions := by
  have h1 : m.functions.find? (fun f => f.name == name) = some fn := by
    rwa [Module.findFunction] at h
  exact List.mem_of_find?_eq_some h1

/-- Name equality from a successful `Module.findFunction`. -/
theorem Module.findFunction_name {m : Module} {name : String} {fn : Function}
    (h : m.findFunction name = some fn) :
    fn.name = name := by
  have h1 : m.functions.find? (fun f => f.name == name) = some fn := by
    rwa [Module.findFunction] at h
  have hp := List.find?_some h1
  simpa using hp

/-- If `a` occurs in `l` and satisfies `p`, then `List.find? p l` returns some
    element satisfying `p` (the first such). -/
private theorem find?_some_of_mem {α} {p : α → Bool} {l : List α} {a : α}
    (ha : a ∈ l) (hp : p a = true) :
    ∃ b, l.find? p = some b ∧ p b = true := by
  induction l with
  | nil =>
      simp at ha
  | cons x xs ih =>
      by_cases hx : p x = true
      · use x; simp [hx]
      · have hne : a ≠ x := by
          intro hax
          rw [hax] at hp
          simp [hx] at hp
        have ha' : a ∈ xs := by
          simp [hne] at ha
          exact ha
        have hex := ih ha'
        rcases hex with ⟨b, hb, hpb⟩
        use b; simp [hx, hb]; exact hpb

/-- If a module has an emitted function named `name`, such a function exists in
    the emitted set. -/
theorem Module.hasEmittedFunctionNamed_exists {env : Env} {m : Module} {name : String}
    (h : Module.hasEmittedFunctionNamed env m name) :
    ∃ fn, fn ∈ Module.emittedFunctions env m ∧ fn.name = name := by
  simp only [Module.hasEmittedFunctionNamed, Module.emittedFunctions, List.any_eq_true] at h
  rcases h with ⟨fn, hmem, heq⟩
  exact ⟨fn, hmem, LawfulBEq.eq_of_beq heq⟩

/-- If a module has an emitted function named `name`, `Module.findFunction`
    returns it.  Emitted functions are always in `m.functions`, so we never need to
    search tests/benches. -/
theorem Module.findFunction_of_hasEmittedFunctionNamed {env : Env} {m : Module} {name : String}
    (h : Module.hasEmittedFunctionNamed env m name) :
    ∃ fn, m.findFunction name = some fn ∧ fn.name = name := by
  rcases Module.hasEmittedFunctionNamed_exists h with ⟨fn, hmem, heq⟩
  have hmem_fn : fn ∈ m.functions := List.mem_of_mem_filter hmem
  have hpred : (fn.name == name) = true := by
    simp [heq]
  have ⟨fn', hfind, hfn'_p⟩ := find?_some_of_mem (p := fun f => f.name == name) hmem_fn hpred
  use fn'
  constructor
  · exact hfind
  · exact LawfulBEq.eq_of_beq hfn'_p

/-- An emitted function is also a function in the broader module sense. -/
theorem Module.hasEmittedFunctionNamed_impl_hasFunctionNamed {env : Env} {m : Module} {name : String}
    (h : Module.hasEmittedFunctionNamed env m name) :
    Module.hasFunctionNamed m name := by
  simp only [Module.hasEmittedFunctionNamed, Module.emittedFunctions, List.any_eq_true] at h
  rcases h with ⟨fn, hmem, heq⟩
  have hmem_fn : fn ∈ m.functions := List.mem_of_mem_filter hmem
  have hmem_all : fn ∈ m.functions ++ m.tests ++ m.benches := by
    simp only [List.mem_append]
    exact Or.inl (Or.inl hmem_fn)
  simp only [Module.hasFunctionNamed, List.any_eq_true]
  exact ⟨fn, hmem_all, heq⟩

/-- In a module with unique function names, two functions with the same name are
    equal. -/
theorem Module.unique_function_name {m : Module}
    (hunique : Module.hasUniqueFunctionNames m)
    {fn1 fn2 : Function}
    (h1 : fn1 ∈ m.functions)
    (h2 : fn2 ∈ m.functions)
    (heq : fn1.name = fn2.name) :
    fn1 = fn2 := by
  have h1' : fn1 ∈ m.functions ++ m.tests ++ m.benches := by
    simp only [List.mem_append]
    exact Or.inl (Or.inl h1)
  have h2' : fn2 ∈ m.functions ++ m.tests ++ m.benches := by
    simp only [List.mem_append]
    exact Or.inl (Or.inl h2)
  simp only [Module.hasUniqueFunctionNames] at hunique
  exact List.inj_on_of_nodup_map hunique h1' h2' heq

/-- If a module has an emitted function named `name` and `findFunction` resolves
    `name` to `fn`, then `fn` is in the emitted set.  Uniqueness guarantees the
    resolvable function is the same one that satisfied `hasEmittedFunctionNamed`. -/
theorem Module.hasEmittedFunctionNamed_findFunction {env : Env} {m : Module} {name : String}
    (hunique : Module.hasUniqueFunctionNames m)
    (h : Module.hasEmittedFunctionNamed env m name)
    (fn : Function) (hm : m.findFunction name = some fn) :
    fn ∈ Module.emittedFunctions env m := by
  rcases Module.hasEmittedFunctionNamed_exists h with ⟨fn', hmem', heq_name'⟩
  have hmem_fn' : fn' ∈ m.functions := List.mem_of_mem_filter hmem'
  have hfn' : m.findFunction name = some fn' := by
    have hpred : (fn'.name == name) = true := by simp [heq_name']
    rcases find?_some_of_mem (p := fun f => f.name == name) hmem_fn' hpred
      with ⟨fn'', hfind, hfn''_p⟩
    have heq'' : fn'' = fn' := by
      have heq''_name : fn''.name = name := LawfulBEq.eq_of_beq hfn''_p
      have heq'_name : fn'.name = name := LawfulBEq.eq_of_beq hpred
      apply Module.unique_function_name hunique
        (List.mem_of_find?_eq_some hfind) hmem_fn'
      exact heq''_name.trans heq'_name.symm
    rw [heq''] at hfind
    exact hfind
  rw [hm] at hfn'
  have heq : fn = fn' := by injection hfn'
  exact heq.symm ▸ hmem'

/-- The emitted Verilog function keeps the original name. -/
theorem emitVFunction_name (fuel : Nat) (env : Env) (m : Module) (fn : Function) :
    (emitVFunction fuel env m fn).name = fn.name := by
  simp [emitVFunction]

/-- `List.find?` over the emitted function list returns the emitted version of
    the first function whose name matches.  W499: every function is emitted, so
    the reachability filter is gone. -/
theorem find?_map_emit (fuel : Nat) (env : Env) (m : Module)
    (fs : List Function) (name : String) (fn : Function)
    (h : List.find? (fun f => f.name == name) fs = some fn) :
    List.find? (fun f => f.name == name)
      (List.map (emitVFunction fuel env m) fs) =
    some (emitVFunction fuel env m fn) := by
  induction fs with
  | nil =>
      simp at h
  | cons f fs ih =>
      let g := emitVFunction fuel env m
      by_cases hname : (f.name == name) = true
      · -- `f` is the first name match, so it equals `fn`.
        have h_eq : f = fn := by
          simp [hname] at h
          exact h
        have hfn_name : f.name = name := LawfulBEq.eq_of_beq hname
        have hfind : List.find? (fun f => f.name == name) (List.map g (f :: fs)) =
          some (g fn) := by
          rw [List.map_cons]
          have hhead : ((g f).name == name) = true := by
            rw [emitVFunction_name, hfn_name]
            simp
          simp only [List.find?, hhead]
          rw [show g f = g fn by rw [h_eq]]
        exact hfind
      · -- `f` does not match the name, so the emitted head is skipped.
        have htail : List.find? (fun f => f.name == name) fs = some fn := by
          simp [hname] at h
          exact h
        have hfn_ne : f.name ≠ name := by
          intro hbad
          have hbeq : (f.name == name) = true := by
            rw [hbad]
            simp
          simp [hbeq] at hname
        have hbeq_false : (f.name == name) = false := by
          cases h : (f.name == name) <;> simp_all
        have hfind : List.find? (fun f => f.name == name) (List.map g (f :: fs)) =
          List.find? (fun f => f.name == name) (List.map g fs) := by
          rw [List.map_cons]
          have hhead : ((g f).name == name) = false := by
            rw [emitVFunction_name, hbeq_false]
          simp only [List.find?, hhead]
        rw [hfind]
        exact ih htail

/-- The Verilog module emitted from a lowerable module contains the emitted
    version of any non-host-only function that can be found in the t27 module. -/
theorem emit_function_lookup (fuel : Nat) (env : Env) (m : Module) (name : String)
    (fn : Function)
    (hunique : Module.hasUniqueFunctionNames m)
    (hm : m.findFunction name = some fn)
    (hfn : fn ∈ Module.emittedFunctions env m) :
    List.find? (fun f => f.name == name)
      (emitModuleFuel fuel env m).functions =
    some (emitVFunction fuel env m fn) := by
  have hfn_name : fn.name = name := Module.findFunction_name hm
  have hpred : (fn.name == name) = true := by simp [hfn_name]
  have ⟨fn', hfind, hfn'_p⟩ := find?_some_of_mem (p := fun f => f.name == name)
    hfn hpred
  have hfn'_mem : fn' ∈ Module.emittedFunctions env m :=
    List.mem_of_find?_eq_some hfind
  have h_eq : fn' = fn := by
    have heq_name : fn'.name = fn.name := by
      rw [LawfulBEq.eq_of_beq hfn'_p, hfn_name]
    exact Module.unique_function_name hunique
      (List.mem_of_mem_filter hfn'_mem)
      (List.mem_of_mem_filter hfn)
      heq_name
  have hmem : List.find? (fun f => f.name == name)
    (Module.emittedFunctions env m) = some fn := by
    rw [h_eq] at hfind
    exact hfind
  simp only [emitModuleFuel, hfn_name]
  exact find?_map_emit fuel env m (Module.emittedFunctions env m) name fn hmem

namespace Expr

/-- A function name in a list member also occurs in the combined list. -/
theorem functionNamesList'_mem {args : List Expr} {a : Expr} {x : String}
    (ha : a ∈ args) (hx : x ∈ a.functionNames') :
    x ∈ Expr.functionNamesList' args := by
  induction args with
  | nil =>
      simp at ha
  | cons e es ih =>
      rcases List.mem_cons.mp ha with (rfl | ha)
      · simp [hx]
      · simp [ih ha]

/-- A function name in a field expression also occurs in the field list. -/
theorem functionNamesFieldList'_mem {fields : List (String × Expr)}
    {p : String × Expr} {x : String}
    (hp : p ∈ fields) (hx : x ∈ p.2.functionNames') :
    x ∈ Expr.functionNamesFieldList' fields := by
  induction fields with
  | nil =>
      simp at hp
  | cons f fs ih =>
      rcases List.mem_cons.mp hp with (rfl | hp)
      · simp [hx]
      · simp [ih hp]

theorem callContext_binop {env m op lhs rhs}
    (h : Expr.callContext env m (Expr.binop op lhs rhs)) :
    Expr.callContext env m lhs ∧ Expr.callContext env m rhs := by
  constructor <;> intro x hx <;> apply h x <;> simp <;> try { tauto }

theorem callContext_unop {env m op e}
    (h : Expr.callContext env m (Expr.unop op e)) :
    Expr.callContext env m e := by
  intro x hx
  apply h x
  simp <;> try { tauto }

theorem callContext_fieldAccess {env m base field}
    (h : Expr.callContext env m (Expr.fieldAccess base field)) :
    Expr.callContext env m base := by
  intro x hx
  apply h x
  simp <;> try { tauto }

theorem callContext_index {env m base idx}
    (h : Expr.callContext env m (Expr.index base idx)) :
    Expr.callContext env m base ∧ Expr.callContext env m idx := by
  constructor <;> intro x hx <;> apply h x <;> simp <;> try { tauto }

theorem callContext_call {env m name args}
    (h : Expr.callContext env m (Expr.call name args)) :
    Env.isReachable env name = true
    ∧ ¬ Env.isHostOnly env name
    ∧ Module.hasEmittedFunctionNamed env m name = true
    ∧ ∀ a ∈ args, Expr.callContext env m a := by
  have h1 := (h name (by simp)).1
  have h2 := (h name (by simp)).2.1
  have h3 := (h name (by simp)).2.2
  constructor
  · exact h1
  constructor
  · exact h2
  constructor
  · exact h3
  · intro a ha x hx
    apply h x
    simp
    apply Or.inr
    exact functionNamesList'_mem ha hx

theorem callContext_structLit {env m name fields}
    (h : Expr.callContext env m (Expr.structLit name fields)) :
    ∀ p ∈ fields, Expr.callContext env m p.2 := by
  intro p hp x hx
  apply h x
  simp
  exact functionNamesFieldList'_mem hp hx

theorem callContext_arrayLit {env m ty elems}
    (h : Expr.callContext env m (Expr.arrayLit ty elems)) :
    ∀ a ∈ elems, Expr.callContext env m a := by
  intro a ha x hx
  apply h x
  simp
  exact functionNamesList'_mem ha hx

/-- A member of a combinational expression list is combinational. -/
theorem isCombinationalList'_mem {es : List Expr} (h : Expr.isCombinationalList' es = true) :
    ∀ a ∈ es, Expr.isCombinational' a := by
  induction es with
  | nil =>
      simp at h ⊢
  | cons e es ih =>
      simp at h ⊢
      constructor
      · exact h.1
      · intro a ha
        exact ih h.2 a ha

/-- A member of a combinational struct-literal field list is combinational. -/
theorem isCombinationalFieldList'_mem {fields : List (String × Expr)}
    (h : Expr.isCombinationalFieldList' fields = true) :
    ∀ (n : String) (e : Expr), (n, e) ∈ fields → Expr.isCombinational' e := by
  induction fields with
  | nil =>
      simp at h ⊢
  | cons f fs ih =>
      simp at h ⊢
      intro n e he
      rcases he with (rfl | he)
      · exact h.1
      · exact ih h.2 n e he

/-- Decompose combinationality of a binary operator expression. -/
theorem isCombinational_binop {op lhs rhs}
    (h : Expr.isCombinational (Expr.binop op lhs rhs)) :
    Expr.isCombinational lhs ∧ Expr.isCombinational rhs := by
  simp [Expr.isCombinational] at h ⊢
  exact h

/-- Decompose combinationality of a unary operator expression. -/
theorem isCombinational_unop {op e}
    (h : Expr.isCombinational (Expr.unop op e)) :
    Expr.isCombinational e := by
  simp [Expr.isCombinational] at h ⊢
  exact h

/-- Decompose combinationality of a field-access expression. -/
theorem isCombinational_fieldAccess {base field}
    (h : Expr.isCombinational (Expr.fieldAccess base field)) :
    Expr.isCombinational base := by
  simp [Expr.isCombinational] at h ⊢
  exact h

/-- Decompose combinationality of an index expression. -/
theorem isCombinational_index {base idx}
    (h : Expr.isCombinational (Expr.index base idx)) :
    Expr.isCombinational base ∧ Expr.isCombinational idx := by
  simp [Expr.isCombinational] at h ⊢
  exact h

/-- Decompose combinationality of a call expression. -/
theorem isCombinational_call {name args}
    (h : Expr.isCombinational (Expr.call name args)) :
    ∀ a ∈ args, Expr.isCombinational a := by
  simp [Expr.isCombinational] at h ⊢
  intro a ha
  exact isCombinationalList'_mem h a ha

/-- Decompose combinationality of a struct-literal expression. -/
theorem isCombinational_structLit {name fields}
    (h : Expr.isCombinational (Expr.structLit name fields)) :
    ∀ (n : String) (e : Expr), (n, e) ∈ fields → Expr.isCombinational e := by
  simp [Expr.isCombinational] at h ⊢
  intro n e he
  exact isCombinationalFieldList'_mem h n e he

/-- Decompose combinationality of an array-literal expression. -/
theorem isCombinational_arrayLit {ty elems}
    (h : Expr.isCombinational (Expr.arrayLit ty elems)) :
    ∀ a ∈ elems, Expr.isCombinational a := by
  simp [Expr.isCombinational] at h ⊢
  intro a ha
  exact isCombinationalList'_mem h a ha

/-- A member of a combinational switch case list is combinational. -/
theorem isCombinationalSwitchCaseList'_mem {cases : List (Expr × Expr)}
    (h : Expr.isCombinationalSwitchCaseList' cases = true) :
    ∀ (tag res : Expr), (tag, res) ∈ cases → Expr.isCombinational' tag ∧ Expr.isCombinational' res := by
  induction cases with
  | nil =>
      intro tag res he
      simp at he
  | cons c cs ih =>
      simp [Bool.and_eq_true, Expr.isCombinationalSwitchCaseList'] at h ⊢
      intro tag res he
      rcases he with (rfl | he)
      · tauto
      · exact ih (by tauto) tag res he

/-- A member of a switch case list also occurs in the combined function-name set. -/
theorem functionNamesSwitchCaseList'_mem {cases : List (Expr × Expr)} {p : Expr × Expr} {x : String}
    (hp : p ∈ cases) (hx : x ∈ p.1.functionNames') :
    x ∈ Expr.functionNamesSwitchCaseList' cases := by
  induction cases with
  | nil =>
      simp at hp
  | cons c cs ih =>
      rcases List.mem_cons.mp hp with (rfl | hp)
      · simp [hx]
      · simp [ih hp]

/-- A member of a switch case list also occurs in the combined function-name set. -/
theorem functionNamesSwitchCaseList'_mem_res {cases : List (Expr × Expr)} {p : Expr × Expr} {x : String}
    (hp : p ∈ cases) (hx : x ∈ p.2.functionNames') :
    x ∈ Expr.functionNamesSwitchCaseList' cases := by
  induction cases with
  | nil =>
      simp at hp
  | cons c cs ih =>
      rcases List.mem_cons.mp hp with (rfl | hp)
      · simp [hx]
      · simp [ih hp]

/-- The function-name list of a switch-case suffix is contained in the list for
    the suffix with one more case at the front. -/
theorem functionNamesSwitchCaseList'_subset_cons {cases : List (Expr × Expr)} {c : Expr × Expr} :
    Expr.functionNamesSwitchCaseList' cases ⊆ Expr.functionNamesSwitchCaseList' (c :: cases) := by
  induction cases with
  | nil =>
      intro x hx
      simp at hx
  | cons d ds ih =>
      intro x hx
      simp [Expr.functionNamesSwitchCaseList'] at hx ⊢
      rcases hx with (hx | hx | hx)
      · simp [hx]
      · simp [hx]
      · have h1 := ih hx
        simp [Expr.functionNamesSwitchCaseList'] at h1 ⊢
        rcases h1 with (h1 | h1 | h1)
        all_goals simp [h1]

/-- A switch expression's call-context decomposes into its discriminant, every
    tag/result arm, and the default expression. -/
theorem callContext_switch {env m disc cases default}
    (h : Expr.callContext env m (Expr.switch disc cases default)) :
    Expr.callContext env m disc
    ∧ (∀ p ∈ cases, Expr.callContext env m p.1 ∧ Expr.callContext env m p.2)
    ∧ Expr.callContext env m default := by
  simp only [Expr.callContext, Expr.functionNames, Expr.functionNames'] at h ⊢
  constructor
  · intro x hx
    apply h x
    simp [hx]
  constructor
  · intro p hp
    constructor
    · intro x hx
      apply h x
      have hmem := functionNamesSwitchCaseList'_mem hp hx
      simp [hmem]
    · intro x hx
      apply h x
      have hmem := functionNamesSwitchCaseList'_mem_res hp hx
      simp [hmem]
  · intro x hx
    apply h x
    simp [hx]

/-- Tail of a switch expression retains call context: every name reachable in the
    smaller switch is also reachable in the original. -/
theorem callContext_switch_tail {env m disc c cs default}
    (h : Expr.callContext env m (Expr.switch disc (c :: cs) default)) :
    Expr.callContext env m (Expr.switch disc cs default) := by
  simp only [Expr.callContext, Expr.functionNames, Expr.functionNames'] at h ⊢
  intro x hx
  apply h x
  simp only [Expr.functionNames, Expr.functionNames', Expr.functionNamesSwitchCaseList', List.mem_append] at hx ⊢
  cases cs with
  | nil => tauto
  | cons d ds => tauto

/-- Decompose combinationality of a switch expression.  Empty case lists are
    combinational, so the nil branch is reachable. -/
theorem isCombinational_switch {disc cases default}
    (h : Expr.isCombinational (Expr.switch disc cases default)) :
    Expr.isCombinational disc
    ∧ (∀ p ∈ cases, Expr.isCombinational p.1 ∧ Expr.isCombinational p.2)
    ∧ Expr.isCombinational default := by
  cases cases with
  | nil =>
      simp [Expr.isCombinational, Expr.isCombinational', Bool.and_eq_true] at h ⊢
      tauto
  | cons c cs =>
      rcases c with ⟨tag, res⟩
      simp [Expr.isCombinational, Expr.isCombinational', Expr.isCombinationalSwitchCaseList',
        Bool.and_eq_true] at h
      constructor
      · exact h.1.1
      constructor
      · intro p hp
        cases hp
        case head =>
          constructor
          · exact h.1.2.1.1
          · exact h.1.2.1.2
        case tail hp =>
          have hmem := Expr.isCombinationalSwitchCaseList'_mem h.1.2.2 p.1 p.2 hp
          simp [Expr.isCombinational, Expr.isCombinational'] at *
          constructor
          · tauto
          · tauto
      · exact h.2

/-- Tail of a switch expression retains combinationality. -/
theorem isCombinational_switch_tail {disc c cs default}
    (h : Expr.isCombinational (Expr.switch disc (c :: cs) default)) :
    Expr.isCombinational (Expr.switch disc cs default) := by
  cases cs with
  | nil =>
      simp [Expr.isCombinational, Expr.isCombinational', Expr.isCombinationalSwitchCaseList',
        Bool.and_eq_true] at h ⊢
      tauto
  | cons d ds =>
      simp [Expr.isCombinational, Expr.isCombinational', Expr.isCombinationalSwitchCaseList',
        Bool.and_eq_true] at h ⊢
      tauto

end Expr

namespace Stmt

@[simp]
theorem functionNames'_assign {lhs rhs} :
    (Stmt.assign lhs rhs).functionNames' = rhs.functionNames' := by
  simp [Stmt.functionNames']

@[simp]
theorem functionNames'_varDecl {name ty e} :
    (Stmt.varDecl name ty (some e)).functionNames' = e.functionNames' := by
  simp [Stmt.functionNames']

@[simp]
theorem functionNames'_constDecl {name ty e} :
    (Stmt.constDecl name ty (some e)).functionNames' = e.functionNames' := by
  simp [Stmt.functionNames']

@[simp]
theorem functionNames'_return {e} :
    (Stmt.return_ (some e)).functionNames' = e.functionNames' := by
  simp [Stmt.functionNames']

@[simp]
theorem functionNames'_bareCall {e} :
    (Stmt.bareCall e).functionNames' = e.functionNames' := by
  simp [Stmt.functionNames']

@[simp]
theorem functionNames_assign {lhs rhs} :
    (Stmt.assign lhs rhs).functionNames = rhs.functionNames := by
  simp [Stmt.functionNames]

@[simp]
theorem functionNames_varDecl {name ty e} :
    (Stmt.varDecl name ty (some e)).functionNames = e.functionNames := by
  simp [Stmt.functionNames]

@[simp]
theorem functionNames_constDecl {name ty e} :
    (Stmt.constDecl name ty (some e)).functionNames = e.functionNames := by
  simp [Stmt.functionNames]

@[simp]
theorem functionNames_return {e} :
    (Stmt.return_ (some e)).functionNames = e.functionNames := by
  simp [Stmt.functionNames]

@[simp]
theorem functionNames_bareCall {e} :
    (Stmt.bareCall e).functionNames = e.functionNames := by
  simp [Stmt.functionNames]

@[simp]
theorem stmt_functionNames_eq (s : Stmt) : s.functionNames = s.functionNames' := by rfl

@[simp]
theorem expr_functionNames_eq (e : Expr) : e.functionNames = e.functionNames' := by rfl

theorem functionNamesList'_eq (ss : List Stmt) :
    Stmt.functionNamesList' ss = ss.flatMap Stmt.functionNames' := by
  induction ss with
  | nil => simp [Stmt.functionNamesList']
  | cons s ss ih => simp [Stmt.functionNamesList', ih]

@[simp]
theorem functionNamesSwitchCaseList'_eq (cs : List (Expr × List Stmt)) :
    Stmt.functionNamesSwitchCaseList' cs =
    cs.flatMap (fun p => p.1.functionNames' ++ p.2.flatMap Stmt.functionNames') := by
  induction cs with
  | nil => simp [Stmt.functionNamesSwitchCaseList']
  | cons p cs ih =>
      rcases p with ⟨tag, body⟩
      simp [Stmt.functionNamesSwitchCaseList', functionNamesList'_eq, ih]

@[simp]
theorem functionNames_ifThenElse {cond then_ else_} :
    (Stmt.ifThenElse cond then_ else_).functionNames' =
    cond.functionNames' ++ then_.flatMap Stmt.functionNames' ++ else_.flatMap Stmt.functionNames' := by
  simp [Stmt.functionNames', functionNamesList'_eq]

@[simp]
theorem functionNames_forLoop {var range body} :
    (Stmt.forLoop var range body).functionNames' =
    range.functionNames' ++ body.flatMap Stmt.functionNames' := by
  simp [Stmt.functionNames', functionNamesList'_eq]

@[simp]
theorem functionNames_whileLoop {cond body} :
    (Stmt.whileLoop cond body).functionNames' =
    cond.functionNames' ++ body.flatMap Stmt.functionNames' := by
  simp [Stmt.functionNames', functionNamesList'_eq]

@[simp]
theorem functionNames'_switch {disc cases default} :
    (Stmt.switch disc cases default).functionNames' =
    disc.functionNames' ++
    cases.flatMap (fun p => p.1.functionNames' ++ p.2.flatMap Stmt.functionNames') ++
    default.flatMap Stmt.functionNames' := by
  simp [Stmt.functionNames', functionNamesList'_eq, functionNamesSwitchCaseList'_eq]

@[simp]
theorem functionNames_switch {disc cases default} :
    (Stmt.switch disc cases default).functionNames =
    disc.functionNames ++
    cases.flatMap (fun p => p.1.functionNames ++ p.2.flatMap Stmt.functionNames) ++
    default.flatMap Stmt.functionNames := by
  rw [stmt_functionNames_eq, functionNames'_switch]
  have h_body_fun : Stmt.functionNames' = Stmt.functionNames := by
    funext s
    rw [stmt_functionNames_eq]
  have h_cases : ∀ p : Expr × List Stmt, p.1.functionNames' ++ p.2.flatMap Stmt.functionNames' =
                   p.1.functionNames ++ p.2.flatMap Stmt.functionNames := by
    intro p
    rw [expr_functionNames_eq, h_body_fun]
  simp [h_cases, h_body_fun, expr_functionNames_eq]

theorem callContext_assign {env m lhs rhs}
    (h : Stmt.callContext env m (Stmt.assign lhs rhs)) :
    Expr.callContext env m rhs := by
  intro x hx
  exact h x (by simpa using hx)

theorem callContext_varDecl {env m name ty e}
    (h : Stmt.callContext env m (Stmt.varDecl name ty (some e))) :
    Expr.callContext env m e := by
  intro x hx
  exact h x (by simpa using hx)

theorem callContext_constDecl {env m name ty e}
    (h : Stmt.callContext env m (Stmt.constDecl name ty (some e))) :
    Expr.callContext env m e := by
  intro x hx
  exact h x (by simpa using hx)

theorem callContext_return {env m e}
    (h : Stmt.callContext env m (Stmt.return_ (some e))) :
    Expr.callContext env m e := by
  intro x hx
  exact h x (by simpa using hx)

theorem callContext_bareCall {env m e}
    (h : Stmt.callContext env m (Stmt.bareCall e)) :
    Expr.callContext env m e := by
  intro x hx
  exact h x (by simpa using hx)

theorem callContext_ifThenElse {env m cond then_ else_}
    (h : Stmt.callContext env m (Stmt.ifThenElse cond then_ else_)) :
    Expr.callContext env m cond
    ∧ Stmt.callContextList env m then_
    ∧ Stmt.callContextList env m else_ := by
  simp only [Stmt.callContext, stmt_functionNames_eq] at h
  constructor
  · intro x hx
    rw [expr_functionNames_eq] at hx
    have hmem : x ∈ (Stmt.ifThenElse cond then_ else_).functionNames' := by
      rw [functionNames_ifThenElse]
      apply List.mem_append_left (List.flatMap Stmt.functionNames' else_)
      apply List.mem_append_left (List.flatMap Stmt.functionNames' then_)
      exact hx
    exact h x hmem
  constructor
  · intro s hs x hx
    rw [stmt_functionNames_eq] at hx
    have hmem : x ∈ (Stmt.ifThenElse cond then_ else_).functionNames' := by
      rw [functionNames_ifThenElse]
      apply List.mem_append_left (List.flatMap Stmt.functionNames' else_)
      apply List.mem_append_right cond.functionNames'
      exact List.mem_flatMap_of_mem hs hx
    exact h x hmem
  · intro s hs x hx
    rw [stmt_functionNames_eq] at hx
    have hmem : x ∈ (Stmt.ifThenElse cond then_ else_).functionNames' := by
      rw [functionNames_ifThenElse]
      apply List.mem_append_right (cond.functionNames' ++ List.flatMap Stmt.functionNames' then_)
      exact List.mem_flatMap_of_mem hs hx
    exact h x hmem

theorem callContext_forLoop {env m var range body}
    (h : Stmt.callContext env m (Stmt.forLoop var range body)) :
    Expr.callContext env m range ∧ Stmt.callContextList env m body := by
  simp only [Stmt.callContext, stmt_functionNames_eq] at h
  constructor
  · intro x hx
    rw [expr_functionNames_eq] at hx
    have hmem : x ∈ (Stmt.forLoop var range body).functionNames' := by
      rw [functionNames_forLoop]
      apply List.mem_append_left (List.flatMap Stmt.functionNames' body)
      exact hx
    exact h x hmem
  · intro s hs x hx
    rw [stmt_functionNames_eq] at hx
    have hmem : x ∈ (Stmt.forLoop var range body).functionNames' := by
      rw [functionNames_forLoop]
      apply List.mem_append_right range.functionNames'
      exact List.mem_flatMap_of_mem hs hx
    exact h x hmem

theorem callContext_whileLoop {env m cond body}
    (h : Stmt.callContext env m (Stmt.whileLoop cond body)) :
    Expr.callContext env m cond ∧ Stmt.callContextList env m body := by
  simp only [Stmt.callContext, stmt_functionNames_eq] at h
  constructor
  · intro x hx
    rw [expr_functionNames_eq] at hx
    have hmem : x ∈ (Stmt.whileLoop cond body).functionNames' := by
      rw [functionNames_whileLoop]
      apply List.mem_append_left (List.flatMap Stmt.functionNames' body)
      exact hx
    exact h x hmem
  · intro s hs x hx
    rw [stmt_functionNames_eq] at hx
    have hmem : x ∈ (Stmt.whileLoop cond body).functionNames' := by
      rw [functionNames_whileLoop]
      apply List.mem_append_right cond.functionNames'
      exact List.mem_flatMap_of_mem hs hx
    exact h x hmem

theorem callContext_switch {env m disc cases default}
    (h : Stmt.callContext env m (Stmt.switch disc cases default)) :
    Expr.callContext env m disc
    ∧ (∀ p ∈ cases, Expr.callContext env m p.1 ∧ Stmt.callContextList env m p.2)
    ∧ Stmt.callContextList env m default := by
  simp only [Stmt.callContext, stmt_functionNames_eq, functionNames'_switch,
    expr_functionNames_eq] at h ⊢
  constructor
  · intro x hx
    rw [expr_functionNames_eq] at hx
    apply h x
    apply List.mem_append_left (default.flatMap Stmt.functionNames')
    apply List.mem_append_left (cases.flatMap (fun p => p.1.functionNames' ++ p.2.flatMap Stmt.functionNames'))
    exact hx
  constructor
  · intro p hp
    constructor
    · intro x hx
      rw [expr_functionNames_eq] at hx
      apply h x
      apply List.mem_append_left (default.flatMap Stmt.functionNames')
      apply List.mem_append_right disc.functionNames'
      apply List.mem_flatMap_of_mem hp
      apply List.mem_append_left (p.2.flatMap Stmt.functionNames')
      exact hx
    · intro s hs x hx
      rw [stmt_functionNames_eq] at hx
      apply h x
      apply List.mem_append_left (default.flatMap Stmt.functionNames')
      apply List.mem_append_right disc.functionNames'
      apply List.mem_flatMap_of_mem hp
      apply List.mem_append_right p.1.functionNames'
      apply List.mem_flatMap_of_mem hs
      exact hx
  · intro s hs x hx
    rw [stmt_functionNames_eq] at hx
    apply h x
    apply List.mem_append_right (disc.functionNames' ++ cases.flatMap (fun p => p.1.functionNames' ++ p.2.flatMap Stmt.functionNames'))
    apply List.mem_flatMap_of_mem hs
    exact hx

theorem callContext_list_mem {env m} {ss : List Stmt} {s : Stmt}
    (h : Stmt.callContextList env m ss) (hs : s ∈ ss) :
    Stmt.callContext env m s := by
  exact h s hs

theorem callContext_list_tail {env m} {s : Stmt} {ss : List Stmt}
    (h : Stmt.callContextList env m (s :: ss)) :
    Stmt.callContextList env m ss := by
  intro s' hs'
  exact h s' (by simp [hs'])

theorem isCombinational_assign {lhs rhs}
    (h : Stmt.isCombinational (Stmt.assign lhs rhs)) :
    ∃ name, lhs = Expr.identifier name ∧ Expr.isCombinational rhs := by
  cases lhs with
  | identifier name =>
      use name
      simp [Stmt.isCombinational] at h ⊢
      exact h
  | _ =>
      simp [Stmt.isCombinational] at h
      all_goals contradiction

theorem isCombinational_varDecl {name ty e}
    (h : Stmt.isCombinational (Stmt.varDecl name ty (some e))) :
    Expr.isCombinational e := by
  simp [Stmt.isCombinational] at h ⊢
  exact h

theorem isCombinational_constDecl {name ty e}
    (h : Stmt.isCombinational (Stmt.constDecl name ty (some e))) :
    Expr.isCombinational e := by
  simp [Stmt.isCombinational] at h ⊢
  exact h

theorem isCombinational_return {e}
    (h : Stmt.isCombinational (Stmt.return_ (some e))) :
    Expr.isCombinational e := by
  simp [Stmt.isCombinational] at h ⊢
  exact h

theorem isCombinational_bareCall {e}
    (h : Stmt.isCombinational (Stmt.bareCall e)) :
    Expr.isCombinational e := by
  simp [Stmt.isCombinational] at h ⊢
  exact h

theorem isCombinationalList'_eq (ss : List Stmt) :
    Stmt.isCombinationalList' ss = Stmt.isCombinationalList ss := by
  induction ss with
  | nil => simp [Stmt.isCombinationalList', Stmt.isCombinationalList]
  | cons s ss ih =>
      simp [Stmt.isCombinationalList', Stmt.isCombinationalList, ih]

theorem isCombinational_ifThenElse {cond then_ else_}
    (h : Stmt.isCombinational (Stmt.ifThenElse cond then_ else_)) :
    Expr.isCombinational cond
    ∧ Stmt.isCombinationalList then_
    ∧ Stmt.isCombinationalList else_ := by
  simp only [Stmt.isCombinational, Expr.isCombinational, Stmt.isCombinational', isCombinationalList'_eq,
    Bool.and_eq_true, and_assoc] at h ⊢
  exact h

theorem isCombinational_forLoop {var range body}
    (h : Stmt.isCombinational (Stmt.forLoop var range body)) :
    Expr.isCombinational range ∧ Stmt.isCombinationalList body := by
  simp only [Stmt.isCombinational, Expr.isCombinational] at h
  contradiction

theorem isCombinational_whileLoop {cond body}
    (h : Stmt.isCombinational (Stmt.whileLoop cond body)) :
    Expr.isCombinational cond ∧ Stmt.isCombinationalList body := by
  simp only [Stmt.isCombinational, Expr.isCombinational] at h
  contradiction

/-- A member of a combinational switch case list is combinational. -/
theorem isCombinationalSwitchCaseList'_mem {cases : List (Expr × List Stmt)}
    (h : Stmt.isCombinationalSwitchCaseList' cases = true) :
    ∀ (tag : Expr) (body : List Stmt), (tag, body) ∈ cases →
      Expr.isCombinational' tag ∧ Stmt.isCombinationalList' body := by
  induction cases with
  | nil =>
      intro tag body he
      simp at he
  | cons c cs ih =>
      intro tag body he
      rcases c with ⟨tag0, body0⟩
      simp [Bool.and_eq_true] at h
      rcases List.mem_cons.mp he with (⟨rfl, rfl⟩ | he')
      · simp [h.1.1, h.1.2]
      · exact ih (by tauto) tag body he'

/-- Decompose combinationality of a switch statement. -/
theorem isCombinational_switch {disc cases default}
    (h : Stmt.isCombinational (Stmt.switch disc cases default)) :
    Expr.isCombinational disc
    ∧ (∀ p ∈ cases, Expr.isCombinational p.1 ∧ Stmt.isCombinationalList p.2)
    ∧ Stmt.isCombinationalList default := by
  simp only [Stmt.isCombinational, Expr.isCombinational, Stmt.isCombinational', Bool.and_eq_true] at h ⊢
  constructor
  · exact h.1.1
  constructor
  · intro p hp
    rcases p with ⟨tag, body⟩
    constructor
    · exact (isCombinationalSwitchCaseList'_mem h.1.2 tag body hp).1
    · rw [← isCombinationalList'_eq body]
      exact (isCombinationalSwitchCaseList'_mem h.1.2 tag body hp).2
  · rw [← isCombinationalList'_eq default]
    exact h.2

theorem isCombinationalList_head {s ss}
    (h : Stmt.isCombinationalList (s :: ss)) :
    Stmt.isCombinational s := by
  simp [Stmt.isCombinationalList, Stmt.isCombinational] at h ⊢
  exact h.1

theorem isCombinationalList_tail {s ss}
    (h : Stmt.isCombinationalList (s :: ss)) :
    Stmt.isCombinationalList ss := by
  simp [Stmt.isCombinationalList] at h ⊢
  exact h.2

/-- Wrapper and primed sequential list forms coincide. -/
theorem isSequentialList'_eq (ss : List Stmt) :
    Stmt.isSequentialList' ss = Stmt.isSequentialList ss := by
  induction ss with
  | nil => simp [Stmt.isSequentialList', Stmt.isSequentialList]
  | cons s ss ih =>
      simp [Stmt.isSequentialList', Stmt.isSequentialList, ih]

theorem isSequential_assign {lhs rhs}
    (h : Stmt.isSequential (Stmt.assign lhs rhs)) :
    ∃ name, lhs = Expr.identifier name ∧ Expr.isCombinational rhs := by
  cases lhs with
  | identifier name =>
      use name
      simp [Stmt.isSequential] at h ⊢
      exact h
  | _ =>
      simp [Stmt.isSequential] at h
      all_goals contradiction

theorem isSequential_varDecl {name ty e}
    (h : Stmt.isSequential (Stmt.varDecl name ty (some e))) :
    Expr.isCombinational e := by
  simp [Stmt.isSequential] at h ⊢
  exact h

theorem isSequential_constDecl {name ty e}
    (h : Stmt.isSequential (Stmt.constDecl name ty (some e))) :
    Expr.isCombinational e := by
  simp [Stmt.isSequential] at h ⊢
  exact h

theorem isSequential_return {e}
    (h : Stmt.isSequential (Stmt.return_ (some e))) :
    Expr.isCombinational e := by
  simp [Stmt.isSequential] at h ⊢
  exact h

theorem isSequential_bareCall {e}
    (h : Stmt.isSequential (Stmt.bareCall e)) :
    Expr.isCombinational e := by
  simp [Stmt.isSequential] at h ⊢
  exact h

theorem isSequential_ifThenElse {cond then_ else_}
    (h : Stmt.isSequential (Stmt.ifThenElse cond then_ else_)) :
    Expr.isCombinational cond
    ∧ Stmt.isSequentialList then_
    ∧ Stmt.isSequentialList else_ := by
  simp only [Stmt.isSequential, Expr.isCombinational, Stmt.isSequential', Stmt.isSequentialList,
    isSequentialList'_eq, Bool.and_eq_true, and_assoc] at h ⊢
  exact h

theorem isSequential_forLoop {var range body}
    (h : Stmt.isSequential (Stmt.forLoop var range body)) :
    Expr.isCombinational range ∧ Stmt.isSequentialList body := by
  simp only [Stmt.isSequential, Expr.isCombinational, Stmt.isSequential', Stmt.isSequentialList,
    isSequentialList'_eq, Bool.and_eq_true] at h ⊢
  exact h

theorem isSequential_whileLoop {cond body}
    (h : Stmt.isSequential (Stmt.whileLoop cond body)) :
    Expr.isCombinational cond ∧ Stmt.isSequentialList body := by
  simp only [Stmt.isSequential, Expr.isCombinational, Stmt.isSequential', Stmt.isSequentialList,
    isSequentialList'_eq, Bool.and_eq_true] at h ⊢
  exact h

/-- A member of a sequential switch case list is sequential. -/
theorem isSequentialSwitchCaseList'_mem {cases : List (Expr × List Stmt)}
    (h : Stmt.isSequentialSwitchCaseList' cases = true) :
    ∀ (tag : Expr) (body : List Stmt), (tag, body) ∈ cases →
      Expr.isCombinational' tag ∧ Stmt.isSequentialList' body := by
  induction cases with
  | nil =>
      intro tag body he
      simp at he
  | cons c cs ih =>
      intro tag body he
      rcases c with ⟨tag0, body0⟩
      simp [Bool.and_eq_true] at h
      rcases List.mem_cons.mp he with (⟨rfl, rfl⟩ | he')
      · simp [h.1.1, h.1.2]
      · exact ih (by tauto) tag body he'

/-- Decompose sequentiality of a switch statement. -/
theorem isSequential_switch {disc cases default}
    (h : Stmt.isSequential (Stmt.switch disc cases default)) :
    Expr.isCombinational disc
    ∧ (∀ p ∈ cases, Expr.isCombinational p.1 ∧ Stmt.isSequentialList p.2)
    ∧ Stmt.isSequentialList default := by
  simp only [Stmt.isSequential, Expr.isCombinational, Stmt.isSequential', Bool.and_eq_true] at h ⊢
  constructor
  · exact h.1.1
  constructor
  · intro p hp
    rcases p with ⟨tag, body⟩
    constructor
    · exact (isSequentialSwitchCaseList'_mem h.1.2 tag body hp).1
    · rw [← isSequentialList'_eq body]
      exact (isSequentialSwitchCaseList'_mem h.1.2 tag body hp).2
  · rw [← isSequentialList'_eq default]
    exact h.2

theorem isSequentialList_head {s ss}
    (h : Stmt.isSequentialList (s :: ss)) :
    Stmt.isSequential s := by
  simp [Stmt.isSequentialList, Stmt.isSequential] at h ⊢
  exact h.1

theorem isSequentialList_tail {s ss}
    (h : Stmt.isSequentialList (s :: ss)) :
    Stmt.isSequentialList ss := by
  simp [Stmt.isSequentialList] at h ⊢
  exact h.2

end Stmt

/-- If a statement occurs in a function body, every name in the statement also
    occurs in the function's name set. -/
theorem Function.functionNames_mem {fn : Function} {s : Stmt} {x : String}
    (hs : s ∈ fn.body) (hx : x ∈ Stmt.functionNames s) :
    x ∈ fn.functionNames := by
  simp only [Function.functionNames]
  rw [List.mem_flatMap]
  exact ⟨s, hs, hx⟩

/-- `Module.isCombinational` implies that any emitted (non-host-only) function
    body in the module is combinational.  Host-only helpers and host-side
    tests/benches are not part of the synthesizable model. -/
theorem Module.isCombinational_function_body {env : Env} {m : Module} {fn : Function}
    (hcomb : Module.isCombinational env m)
    (hfn : fn ∈ m.functions)
    (hhost : ¬ Env.isHostOnly env fn.name) :
    Stmt.isCombinationalList fn.body := by
  simp only [Module.isCombinational, Function.isCombinational, Stmt.isCombinationalList,
    Stmt.isCombinational, Bool.and_eq_true, List.all_iff] at hcomb ⊢
  have h_funcs := hcomb.2
  specialize h_funcs fn hfn
  rw [if_neg hhost] at h_funcs
  simp only [Stmt.isCombinational, List.all_iff] at h_funcs ⊢
  exact h_funcs

/-- `Module.isSequential` implies that any emitted (non-host-only) function body
    in the module is sequential. -/
theorem Module.isSequential_function_body {env : Env} {m : Module} {fn : Function}
    (hseq : Module.isSequential env m)
    (hfn : fn ∈ m.functions)
    (hhost : ¬ Env.isHostOnly env fn.name) :
    Stmt.isSequentialList fn.body := by
  simp only [Module.isSequential, Function.isSequential, Stmt.isSequentialList,
    Stmt.isSequential, Bool.and_eq_true, List.all_iff] at hseq ⊢
  have h_funcs := hseq.2
  specialize h_funcs fn hfn
  rw [if_neg hhost] at h_funcs
  simp only [Stmt.isSequential, List.all_iff] at h_funcs ⊢
  exact h_funcs

/-- Parameter lookup on the t27 side and on the emitted Verilog side agree:
    the Verilog side only adds a width annotation to each parameter name; the
    lookup by name returns the same argument value. -/
theorem paramLookup_eq {params : List (String × Ty)} (argVals : List Value)
    (env : Env) (name : String) :
    Option.map (fun x => x.2)
      (((params.map (fun p => (p.1, widthOfType defaultFuel env p.2))).zip argVals).find?
        (fun p => p.1.1 == name)) =
    Option.map (fun x => x.2)
      ((params.zip argVals).find? (fun p => p.1.1 == name)) := by
  induction params generalizing argVals with
  | nil =>
      simp
  | cons p ps ih =>
      cases argVals with
      | nil =>
          simp
      | cons v vs =>
          by_cases hp : p.1 == name <;> simp [hp, ih vs]

section FuelFacts

/-- `defaultFuel` is positive so that it can be written as `fuel + 1`. -/
theorem defaultFuel_pos : defaultFuel > 0 := by decide

section EmitEq

/-! Default-fuel reduction lemmas for emitExpr.
    Emission is fuel-independent (it recurses on the AST), so all recursive
    positions use defaultFuel directly. -/

theorem emitExpr_default_boolLit (env m) (b : Bool) :
    emitExpr defaultFuel env m (Expr.boolLit b) =
    VExpr.lit 1 (toString (if b then (1 : Int) else (0 : Int))) := by
  unfold emitExpr; cases b <;> rfl

theorem emitExpr_default_intLit (env m) (n : Int) :
    emitExpr defaultFuel env m (Expr.intLit n) = VExpr.lit 32 (toString n) := by
  conv => lhs; unfold emitExpr

theorem emitExpr_default_identifier (env m) (name : String) :
    emitExpr defaultFuel env m (Expr.identifier name) = VExpr.ident name := by
  conv => lhs; unfold emitExpr

theorem emitExpr_default_binop (env m) (op : String) (lhs rhs : Expr) :
    emitExpr defaultFuel env m (Expr.binop op lhs rhs) =
    VExpr.binop op (emitExpr defaultFuel env m lhs) (emitExpr defaultFuel env m rhs) := by
  conv => lhs; unfold emitExpr

theorem emitExpr_default_unop (env m) (op : String) (e : Expr) :
    emitExpr defaultFuel env m (Expr.unop op e) =
    VExpr.unop op (emitExpr defaultFuel env m e) := by
  conv => lhs; unfold emitExpr

theorem emitExpr_default_fieldAccess (env m) (base : Expr) (field : String) :
    emitExpr defaultFuel env m (Expr.fieldAccess base field) =
    let baseV := emitExpr defaultFuel env m base
    match Expr.typeOf env m base with
    | some (.struct sname) =>
        let fields := env.structFields sname
        let offset := fields.foldl (fun acc p =>
          if p.1 < field then acc + widthOfType defaultFuel env p.2 else acc) 0
        let w := match fields.find? (fun p => p.1 == field) with
          | some ty => widthOfType defaultFuel env ty.2
          | none => 1
        VExpr.slice baseV (offset + w - 1) offset
    | _ => VExpr.slice baseV 0 0 := by
  rfl

theorem emitExpr_default_index (env m) (base idx : Expr) :
    emitExpr defaultFuel env m (Expr.index base idx) =
    VExpr.index (emitExpr defaultFuel env m base) (emitExpr defaultFuel env m idx)
      (indexElemWidth defaultFuel env m base) := by
  rfl

theorem emitExpr_default_call (env m) (name : String) (args : List Expr) :
    emitExpr defaultFuel env m (Expr.call name args) =
    VExpr.call name (emitExprList defaultFuel env m args) := by
  conv => lhs; unfold emitExpr

theorem emitExpr_default_structLit (env m) (name : String)
    (fields : List (String × Expr)) :
    emitExpr defaultFuel env m (Expr.structLit name fields) =
    VExpr.concat (emitFieldExprs defaultFuel env m fields) := by
  unfold emitExpr; rfl

theorem emitExpr_default_arrayLit (env m) (ty : Ty) (elems : List Expr) :
    emitExpr defaultFuel env m (Expr.arrayLit ty elems) =
    VExpr.concat (emitExprList defaultFuel env m elems) := by
  unfold emitExpr; rfl

theorem emitExpr_default_enumVal (env m) (enum variant : String) :
    emitExpr defaultFuel env m (Expr.enumVal enum variant) =
    VExpr.lit 32 (toString (env.enumValue enum variant |>.getD 0)) := by
  unfold emitExpr; rfl

/-- The outer emission of a `switch` expands to the local recursive helper.
    Isolating this rewrite prevents `unfold emitExpr` from touching the right-hand
    side of equivalence theorems. -/
theorem emitExpr_switch_eq (fuel env m) (disc default : Expr)
    (cases : List (Expr × Expr)) :
    emitExpr fuel env m (Expr.switch disc cases default) =
    (let discV := emitExpr fuel env m disc
     let defaultV := emitExpr fuel env m default
     emitExpr.go fuel env m discV defaultV cases) := by
  conv =>
    pattern (emitExpr _ _ _ (Expr.switch _ _ _))
    unfold emitExpr

/-- The local `emitExpr.go` helper is exactly the nested ternary fold used by the
    total evaluator's switch congruence lemmas. -/
theorem emitExpr.go_foldr (fuel env m) (discV defv : VExpr)
    (cases : List (Expr × Expr)) :
    emitExpr.go fuel env m discV defv cases =
    cases.foldr (init := defv)
      (fun (tag, res) acc =>
        VExpr.ternary
          (VExpr.binop "==" discV (emitExpr fuel env m tag))
          (emitExpr fuel env m res)
          acc) := by
  induction cases with
  | nil =>
      simp [emitExpr.go]
  | cons c cs ih =>
      rcases c with ⟨tag, res⟩
      simp [emitExpr.go, List.foldr_cons, ih]
      all_goals try { rfl }

/-- Default-fuel emission of a `switch` expression is the nested ternary fold
    over the emitted discriminant and default. -/
theorem emitExpr_default_switch (env m) (disc : Expr) (cases : List (Expr × Expr))
    (default : Expr) :
    emitExpr defaultFuel env m (Expr.switch disc cases default) =
    cases.foldr (init := emitExpr defaultFuel env m default)
      (fun (tag, res) acc =>
        VExpr.ternary
          (VExpr.binop "==" (emitExpr defaultFuel env m disc) (emitExpr defaultFuel env m tag))
          (emitExpr defaultFuel env m res)
          acc) := by
  rw [emitExpr_switch_eq, emitExpr.go_foldr]
  all_goals try { rfl }

/-! The structural list helpers coincide with the usual `List.map` formulation,
    which is what the total evaluator's congruence lemmas expect. -/

theorem emitExprList_eq_map (fuel env m) (es : List Expr) :
    emitExprList fuel env m es = es.map (emitExpr fuel env m) := by
  induction es with
  | nil => rfl
  | cons e es ih =>
      simp [emitExprList, ih]

theorem emitFieldExprs_eq_map (fuel env m) (fields : List (String × Expr)) :
    emitFieldExprs fuel env m fields = fields.map (fun p => emitExpr fuel env m p.2) := by
  induction fields with
  | nil => rfl
  | cons p fields ih =>
      simp [emitFieldExprs, ih]

/-! Default-fuel reduction lemmas for emitStmt / emitStmts. -/

theorem emitStmt_default_assign (env m) (lhs rhs : Expr) :
    emitStmt defaultFuel env m (Stmt.assign lhs rhs) =
    VStmt.assign (emitExpr defaultFuel env m lhs) (emitExpr defaultFuel env m rhs) := by
  conv => lhs; unfold emitStmt; rfl

theorem emitStmt_default_varDecl_some (env m) (name : String) (ty : Ty) (e : Expr) :
    emitStmt defaultFuel env m (Stmt.varDecl name ty (some e)) =
    VStmt.assign (VExpr.ident name) (emitExpr defaultFuel env m e) := by
  conv => lhs; unfold emitStmt
  simp [Option.map_some, Option.getD_some]

theorem emitStmt_default_constDecl_some (env m) (name : String) (ty : Ty) (e : Expr) :
    emitStmt defaultFuel env m (Stmt.constDecl name ty (some e)) =
    VStmt.localparam name (widthOfType defaultFuel env ty)
      (emitExpr defaultFuel env m e) := by
  conv => lhs; unfold emitStmt
  simp [Option.map_some, Option.getD_some]

theorem emitStmt_default_return_some (env m) (e : Expr) :
    emitStmt defaultFuel env m (Stmt.return_ (some e)) =
    VStmt.assign (VExpr.ident "__return") (emitExpr defaultFuel env m e) := by
  conv => lhs; unfold emitStmt
  simp [Option.map_some, Option.getD_some]

theorem emitStmt_default_ifThenElse (env m) (cond : Expr) (then_ else_ : List Stmt) :
    emitStmt defaultFuel env m (Stmt.ifThenElse cond then_ else_) =
    VStmt.ifThenElse (emitExpr defaultFuel env m cond)
      (emitStmts defaultFuel env m then_)
      (emitStmts defaultFuel env m else_) := by
  conv => lhs; unfold emitStmt; rfl

theorem emitSwitchCases_eq_map (fuel env m) (cases : List (Expr × List Stmt)) :
    emitSwitchCases fuel env m cases =
    cases.map (fun p => (emitExpr fuel env m p.1, emitStmts fuel env m p.2)) := by
  induction cases with
  | nil => simp [emitSwitchCases]
  | cons c cs ih =>
      rcases c with ⟨tag, body⟩
      simp [emitSwitchCases, ih]

theorem emitStmt_default_switch (env m) (disc : Expr) (cases : List (Expr × List Stmt))
    (default : List Stmt) :
    emitStmt defaultFuel env m (Stmt.switch disc cases default) =
    VStmt.switch (emitExpr defaultFuel env m disc)
      (cases.map (fun p => (emitExpr defaultFuel env m p.1, emitStmts defaultFuel env m p.2)))
      (emitStmts defaultFuel env m default) := by
  conv => lhs; unfold emitStmt
  rw [emitSwitchCases_eq_map]

theorem emitStmt_default_forLoop (env m) (var : String) (range : Expr) (body : List Stmt) :
    emitStmt defaultFuel env m (Stmt.forLoop var range body) =
    VStmt.forLoop var (emitExpr defaultFuel env m range) (emitStmts defaultFuel env m body) := by
  conv => lhs; unfold emitStmt; rfl

theorem emitStmt_default_whileLoop (env m) (cond : Expr) (body : List Stmt) :
    emitStmt defaultFuel env m (Stmt.whileLoop cond body) =
    VStmt.whileLoop (emitExpr defaultFuel env m cond) (emitStmts defaultFuel env m body) := by
  conv => lhs; unfold emitStmt; rfl

theorem emitStmt_default_bareCall (env m) (e : Expr) :
    emitStmt defaultFuel env m (Stmt.bareCall e) =
    VStmt.taskCall "" [emitExpr defaultFuel env m e] := by
  conv => lhs; unfold emitStmt; rfl

theorem emitStmt_default_break (env m) :
    emitStmt defaultFuel env m Stmt.break = VStmt.break := by
  conv => lhs; unfold emitStmt; rfl

theorem emitStmt_default_continue (env m) :
    emitStmt defaultFuel env m Stmt.continue = VStmt.continue := by
  conv => lhs; unfold emitStmt; rfl

theorem emitStmts_default_cons (env m) (s : Stmt) (ss : List Stmt) :
    emitStmts defaultFuel env m (s :: ss) =
    emitStmt defaultFuel env m s :: emitStmts defaultFuel env m ss := by
  conv => lhs; unfold emitStmts
  simp [List.map, emitStmts]

theorem emitStmts_default_nil (env m) :
    emitStmts defaultFuel env m [] = [] := by
  conv => lhs; unfold emitStmts
  simp [List.map, emitStmts]

end EmitEq


section EvalEq

/-! Reduction lemmas for total evaluation at zero and positive fuel. -/

/-- At zero fuel all expression evaluators return `none`. -/
theorem evalExprTotal_zero {env m val e} :
    evalExprTotal 0 env m val e = none := by
  cases e <;> simp [evalExprTotal]

theorem evalVExprTotal_zero {env vm val e} :
    evalVExprTotal 0 env vm val e = none := by
  cases e <;> simp [evalVExprTotal]

theorem evalForLoopTotal_zero {env m val var i n body} :
    evalForLoopTotal 0 env m val var i n body = none := by
  simp [evalForLoopTotal]

theorem evalVForLoopTotal_zero {env vm val var i n body} :
    evalVForLoopTotal 0 env vm val var i n body = none := by
  simp [evalVForLoopTotal]

theorem evalWhileLoopTotal_zero {env m val cond body} :
    evalWhileLoopTotal 0 env m val cond body = none := by
  simp [evalWhileLoopTotal]

theorem evalVWhileLoopTotal_zero {env vm val cond body} :
    evalVWhileLoopTotal 0 env vm val cond body = none := by
  simp [evalVWhileLoopTotal]

/-- At positive fuel a boolean literal reduces to a one-bit value. -/
theorem evalExprTotal_succ_boolLit (fuel env m) (b : Bool) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.boolLit b) =
    some ⟨1, if b then 1#1 else 0#1⟩ := by
  conv => lhs; unfold evalExprTotal; dsimp only [Option.bind]; rfl
  cases b <;> rfl

theorem evalExprTotal_succ_intLit (fuel env m) (n : Int) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.intLit n) =
    some ⟨32, BitVec.ofInt 32 n⟩ := by
  conv => lhs; unfold evalExprTotal; dsimp only [Option.bind]; rfl

theorem evalExprTotal_succ_identifier (fuel env m) (name : String) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.identifier name) = val name := by
  conv => lhs; unfold evalExprTotal; dsimp only [Option.bind]; rfl

theorem evalExprTotal_succ_binop (fuel env m) (op : String) (lhs rhs : Expr) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.binop op lhs rhs) =
    (do
      let l ← evalExprTotal fuel env m val lhs
      let r ← evalExprTotal fuel env m val rhs
      evalBinop op l r) := by
  conv => lhs; unfold evalExprTotal; dsimp only [Option.bind]; rfl

theorem evalExprTotal_succ_unop (fuel env m) (op : String) (e : Expr) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.unop op e) =
    (do
      let v ← evalExprTotal fuel env m val e
      evalUnop op v) := by
  conv => lhs; unfold evalExprTotal; dsimp only [Option.bind]; rfl

theorem evalExprTotal_succ_fieldAccess (fuel env m) (base : Expr) (field : String)
    (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.fieldAccess base field) =
    (do
      let baseV ← evalExprTotal fuel env m val base
      match Expr.typeOf env m base with
      | some (.struct sname) =>
          let off := structFieldOffsetTotal fuel env sname field
          let w := structFieldWidthTotal fuel env sname field
          let hi := off + w - 1
          if _h : off ≤ hi && hi < baseV.width then
            some ⟨hi - off + 1, BitVec.extractLsb' off (hi - off + 1) baseV.bits⟩
          else
            none
      | _ =>
          if _h : baseV.width > 0 then
            some ⟨1, BitVec.extractLsb' 0 1 baseV.bits⟩
          else
            none) := by
    cases h : Expr.typeOf env m base with
    | some ty =>
        cases ty with
        | struct sname =>
            simp only [h, evalExprTotal, structFieldOffsetTotal, structFieldWidthTotal]
            try { apply Option.bind_congr_ext rfl; intro baseV; try { split }; all_goals try { simp_all }; all_goals try { rfl } }
        | _ =>
            simp only [h, evalExprTotal]
            try { apply Option.bind_congr_ext rfl; intro baseV; try { split }; all_goals try { simp_all }; all_goals try { rfl } }
    | none =>
        simp only [h, evalExprTotal]
        try { apply Option.bind_congr_ext rfl; intro baseV; try { split }; all_goals try { simp_all }; all_goals try { rfl } }

theorem evalExprTotal_succ_index (fuel env m) (base idx : Expr) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.index base idx) =
    (do
      let baseV ← evalExprTotal fuel env m val base
      let idxV ← evalExprTotal fuel env m val idx
      let n := idxV.bits.toNat
      let elemW := match Expr.typeOf env m base with
        | some (.array _ elem) => widthOfType fuel env elem
        | _ => 8
      if _h : elemW > 0 && n * elemW + elemW - 1 < baseV.width then
        some ⟨elemW, BitVec.extractLsb' (n * elemW) elemW baseV.bits⟩
      else
        none) := by
    cases h : Expr.typeOf env m base with
    | some ty =>
        cases ty with
        | array n elem =>
            simp only [evalExprTotal, widthOfType]
            rw [h]
            try { apply Option.bind_congr_ext rfl; intro baseV; apply Option.bind_congr_ext rfl; intro idxV;
                  try { split }; all_goals try { simp_all }; all_goals try { rfl } }
        | _ =>
            simp only [evalExprTotal]
            rw [h]
            try { apply Option.bind_congr_ext rfl; intro baseV; apply Option.bind_congr_ext rfl; intro idxV;
                  try { split }; all_goals try { simp_all }; all_goals try { rfl } }
    | none =>
        simp only [evalExprTotal]
        rw [h]
        try { apply Option.bind_congr_ext rfl; intro baseV; apply Option.bind_congr_ext rfl; intro idxV;
              try { split }; all_goals try { simp_all }; all_goals try { rfl } }

theorem evalExprTotal_succ_call (fuel env m) (name : String) (args : List Expr) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.call name args) =
    (match m.findFunction name with
    | some fn =>
        (do
          let argVals ← args.mapM (evalExprTotal fuel env m val)
          evalFunctionTotal fuel env m fn argVals val)
    | none => none) := by
    conv => lhs; unfold evalExprTotal; dsimp only
    cases h : m.findFunction name with
    | some fn => simp [h]
    | none => simp [h]

theorem evalExprTotal_succ_structLit (fuel env m) (name : String) (fields : List (String × Expr))
    (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.structLit name fields) =
    (do
      let vs ← fields.mapM (fun p => evalExprTotal fuel env m val p.2)
      some (Value.concatList vs)) := by
  conv => lhs; unfold evalExprTotal; dsimp only [Option.bind]; rfl

theorem evalExprTotal_succ_arrayLit (fuel env m) (ty : Ty) (elems : List Expr) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.arrayLit ty elems) =
    (do
      let vs ← elems.mapM (evalExprTotal fuel env m val)
      some (Value.concatList vs)) := by
  conv => lhs; unfold evalExprTotal; dsimp only [Option.bind]; rfl

theorem evalExprTotal_succ_enumVal (fuel env m) (enum variant : String) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.enumVal enum variant) =
    some ⟨32, BitVec.ofInt 32 (env.enumValue enum variant |>.getD 0)⟩ := by
  conv => lhs; unfold evalExprTotal; dsimp only [Option.bind]; rfl

/-- A `switch` expression is evaluated by the switch-case walker at the same
    fuel. -/
theorem evalExprTotal_succ_switch (fuel env m) (disc : Expr) (cases : List (Expr × Expr))
    (default : Expr) (val : Valuation) :
    evalExprTotal (fuel + 1) env m val (Expr.switch disc cases default) =
    evalSwitchCasesTotal (fuel + 1) env m val disc default cases := by
  conv => lhs; unfold evalExprTotal; dsimp only [Option.bind]; rfl

/-- The switch-case walker is definitionally equal to evaluating the `switch`
    expression.  Used to re-associate the emitted nested ternary. -/
theorem evalExprTotal_eq_switch (fuel env m) (disc : Expr) (cases : List (Expr × Expr))
    (default : Expr) (val : Valuation) :
    evalExprTotal fuel env m val (Expr.switch disc cases default) =
    evalSwitchCasesTotal fuel env m val disc default cases := by
  cases fuel with
  | zero =>
      cases cases with
      | nil => simp [evalExprTotal, evalSwitchCasesTotal]
      | cons c cs =>
          simp [evalExprTotal]
          unfold evalSwitchCasesTotal
          simp
  | succ fuel =>
      simp [evalExprTotal]

/-! Verilog expression evaluator reductions. -/

theorem evalVExprTotal_succ_lit (fuel env vm) (w : Nat) (s : String) (val : Valuation) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.lit w s) =
    (String.toInt? s).map (fun n => ⟨w, BitVec.ofInt w n⟩) := by
    cases h : s.toInt? with
    | some n => simp [h, evalVExprTotal]
    | none => simp [h, evalVExprTotal]

theorem evalVExprTotal_succ_ident (fuel env vm) (name : String) (val : Valuation) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.ident name) = val name := by
  conv => lhs; unfold evalVExprTotal; dsimp only [Option.bind]; rfl

theorem evalVExprTotal_succ_binop (fuel env vm) (op : String) (lhs rhs : VExpr) (val : Valuation) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.binop op lhs rhs) =
    (do
      let l ← evalVExprTotal fuel env vm val lhs
      let r ← evalVExprTotal fuel env vm val rhs
      evalBinop op l r) := by
  conv => lhs; unfold evalVExprTotal; dsimp only [Option.bind]; rfl

theorem evalVExprTotal_succ_unop (fuel env vm) (op : String) (e : VExpr) (val : Valuation) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.unop op e) =
    (do
      let v ← evalVExprTotal fuel env vm val e
      evalUnop op v) := by
  conv => lhs; unfold evalVExprTotal; dsimp only [Option.bind]; rfl

theorem evalVExprTotal_succ_slice (fuel env vm) (e : VExpr) (hi lo : Nat) (val : Valuation) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.slice e hi lo) =
    (do
      let v ← evalVExprTotal fuel env vm val e
      if _h : lo ≤ hi && hi < v.width then
        some ⟨hi - lo + 1, BitVec.extractLsb' lo (hi - lo + 1) v.bits⟩
      else
        none) := by
  conv => lhs; unfold evalVExprTotal; dsimp only [Option.bind]; rfl

theorem evalVExprTotal_succ_index (fuel env vm) (base idx : VExpr) (w : Nat) (val : Valuation) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.index base idx w) =
    (do
      let baseV ← evalVExprTotal fuel env vm val base
      let idxV ← evalVExprTotal fuel env vm val idx
      let n := idxV.bits.toNat
      if _h : w > 0 && n * w + w - 1 < baseV.width then
        some ⟨w, BitVec.extractLsb' (n * w) w baseV.bits⟩
      else
        none) := by
  conv => lhs; unfold evalVExprTotal; dsimp only [Option.bind]; rfl

theorem evalVExprTotal_succ_call (fuel env vm) (name : String) (args : List VExpr) (val : Valuation) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.call name args) =
    (do
      let argVals ← args.mapM (evalVExprTotal fuel env vm val)
      match vm.functions.find? (fun f => f.name == name) with
      | some fn => evalVFunctionTotal fuel env vm fn argVals val
      | none => none) := by
    conv => lhs; unfold evalVExprTotal; dsimp only
    cases h : vm.functions.find? (fun f => f.name == name) with
    | some fn =>
        simp only [h]
        try { apply Option.bind_congr_ext rfl; intro argVals; rfl }
    | none =>
        simp only [h]
        try { apply Option.bind_congr_ext rfl; intro argVals; rfl }

theorem evalVExprTotal_succ_concat (fuel env vm) (es : List VExpr) (val : Valuation) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.concat es) =
    (do
      let vs ← es.mapM (evalVExprTotal fuel env vm val)
      some (Value.concatList vs)) := by
  conv => lhs; unfold evalVExprTotal; dsimp only [Option.bind]; rfl

/-- At positive fuel a ternary expression reduces to the condition evaluation
    followed by the chosen branch. -/
theorem evalVExprTotal_succ_ternary (fuel env vm) (cond then_ else_ : VExpr) (val : Valuation) :
    evalVExprTotal (fuel + 1) env vm val (VExpr.ternary cond then_ else_) =
    (do
      let c ← evalVExprTotal fuel env vm val cond
      if c.bits.toNat > 0 then evalVExprTotal fuel env vm val then_
      else evalVExprTotal fuel env vm val else_) := by
  conv => lhs; unfold evalVExprTotal; dsimp only [Option.bind]; rfl

/-- The nested ternary emitted for a `switch` evaluates to the same value as the
    Verilog ternary-case walker.  The proof mirrors the fuel bookkeeping of the
    two definitions. -/
theorem evalVExprTotal_foldr_ternary_cases (fuel env m vm) (disc default : Expr)
    (cases : List (Expr × Expr)) (val : Valuation) :
    evalVExprTotal fuel env vm val
      (cases.foldr (init := emitExpr defaultFuel env m default)
        (fun p acc =>
          (VExpr.binop "==" (emitExpr defaultFuel env m disc) (emitExpr defaultFuel env m p.1)).ternary
            (emitExpr defaultFuel env m p.2) acc)) =
    evalVTernaryCasesTotal fuel env vm val
      (emitExpr defaultFuel env m disc)
      (emitExpr defaultFuel env m default)
      (cases.map (fun p => (emitExpr defaultFuel env m p.1, emitExpr defaultFuel env m p.2))) := by
  revert cases
  induction fuel using Nat.strong_induction_on with
  | h fuel ih_fuel =>
      intro cases
      cases cases with
      | nil =>
          simp [evalVTernaryCasesTotal]
      | cons c cs =>
          rcases c with ⟨tag, res⟩
          cases fuel with
          | zero =>
              simp [evalVExprTotal, evalVTernaryCasesTotal]
          | succ fuel =>
              cases fuel with
              | zero =>
                  simp [evalVExprTotal, evalVTernaryCasesTotal]
              | succ fuel =>
                  simp [List.foldr_cons, List.map_cons, evalVTernaryCasesTotal]
                  rw [evalVExprTotal_succ_ternary]
                  rw [evalVExprTotal_succ_binop]
                  cases h1 : evalVExprTotal fuel env vm val (emitExpr defaultFuel env m disc) with
                  | none =>
                      simp [h1, evalVExprTotal, evalVTernaryCasesTotal]
                  | some d =>
                      cases h2 : evalVExprTotal fuel env vm val (emitExpr defaultFuel env m tag) with
                      | none =>
                          simp [h1, h2, evalVExprTotal, evalVTernaryCasesTotal]
                      | some t =>
                          cases h3 : evalBinop "==" d t with
                          | none =>
                              simp [h1, h2, h3, evalVExprTotal, evalVTernaryCasesTotal]
                          | some eq =>
                              simp only [h1, h2, h3, Bind.bind, Option.bind]
                              split_ifs
                              · rfl
                              · exact ih_fuel (fuel + 1) (by omega) cs

/-! Function and statement evaluator reductions. -/

theorem evalFunctionTotal_succ (fuel env m fn args base) :
    evalFunctionTotal (fuel + 1) env m fn args base =
    (evalStmtsTotal fuel env m
      (fun name =>
        (fn.params.zip args).find? (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name))
      fn.body).bind (fun final => final "__return") := by
    conv => lhs; unfold evalFunctionTotal; dsimp only [Bind.bind, Option.bind]
    rfl

theorem evalVFunctionTotal_succ (fuel env vm fn args base) :
    evalVFunctionTotal (fuel + 1) env vm fn args base =
    (evalVStmtsTotal fuel env vm
      (fun name =>
        (fn.params.zip args).find? (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name))
      fn.body).bind (fun final => final "__return") := by
    conv => lhs; unfold evalVFunctionTotal; dsimp only [Bind.bind, Option.bind]
    rfl

theorem evalStmtTotal_succ_assign_ident (fuel env m) (name : String) (rhs : Expr) (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val (Stmt.assign (Expr.identifier name) rhs) =
    (evalExprTotal fuel env m val rhs).bind (fun rv =>
      some (fun x => if x == name then some rv else val x)) := by
    cases h : evalExprTotal fuel env m val rhs with
    | none => simp [h, evalStmtTotal, Option.bind]
    | some rv => simp [h, evalStmtTotal, Option.bind]

theorem evalStmtTotal_succ_varDecl_some (fuel env m) (name : String) (ty : Ty) (e : Expr)
    (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val (Stmt.varDecl name ty (some e)) =
    (evalExprTotal fuel env m val e).bind (fun rv =>
      some (fun x => if x == name then some rv else val x)) := by
    cases h : evalExprTotal fuel env m val e with
    | none => simp [h, evalStmtTotal, Option.bind]
    | some rv => simp [h, evalStmtTotal, Option.bind]

theorem evalStmtTotal_succ_constDecl_some (fuel env m) (name : String) (ty : Ty) (e : Expr)
    (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val (Stmt.constDecl name ty (some e)) =
    (evalExprTotal fuel env m val e).bind (fun rv =>
      some (fun x => if x == name then some rv else val x)) := by
    cases h : evalExprTotal fuel env m val e with
    | none => simp [h, evalStmtTotal, Option.bind]
    | some rv => simp [h, evalStmtTotal, Option.bind]

theorem evalStmtTotal_succ_return_some (fuel env m) (e : Expr) (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val (Stmt.return_ (some e)) =
    (evalExprTotal fuel env m val e).bind (fun rv =>
      some (setFlag val returnFlag rv)) := by
    cases h : evalExprTotal fuel env m val e with
    | none => simp [h, evalStmtTotal, Option.bind]
    | some rv =>
        simp [h, evalStmtTotal, Option.bind, setFlag, returnFlag]
        try { funext x; simp }

theorem evalStmtTotal_succ_break (fuel env m) (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val Stmt.break =
    some (setFlag val breakFlag ⟨1, 1#1⟩) := by
    simp [evalStmtTotal, Option.bind]

theorem evalStmtTotal_succ_continue (fuel env m) (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val Stmt.continue =
    some (setFlag val continueFlag ⟨1, 1#1⟩) := by
    simp [evalStmtTotal, Option.bind]

theorem evalStmtTotal_succ_ifThenElse (fuel env m) (cond : Expr) (then_ else_ : List Stmt) (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val (Stmt.ifThenElse cond then_ else_) =
    (evalExprTotal fuel env m val cond).bind (fun c =>
      if c.bits.toNat > 0 then evalStmtsTotal fuel env m val then_
      else evalStmtsTotal fuel env m val else_) := by
    cases h : evalExprTotal fuel env m val cond with
    | none => simp [h, evalStmtTotal, Option.bind]
    | some c =>
        simp [h, evalStmtTotal, Option.bind]
        try { split; rfl }

theorem evalStmtTotal_succ_forLoop (fuel env m) (var : String) (range : Expr) (body : List Stmt) (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val (Stmt.forLoop var range body) =
    (evalExprTotal fuel env m val range).bind (fun r =>
      evalForLoopTotal fuel env m val var 0 r.bits.toNat body) := by
    cases h : evalExprTotal fuel env m val range with
    | none => simp [h, evalStmtTotal, Option.bind]
    | some r => simp [h, evalStmtTotal, Option.bind]

theorem evalStmtTotal_succ_whileLoop (fuel env m) (cond : Expr) (body : List Stmt) (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val (Stmt.whileLoop cond body) =
    evalWhileLoopTotal fuel env m val cond body := by
    simp [evalStmtTotal, Option.bind]

theorem evalStmtTotal_succ_switch (fuel env m) (disc : Expr) (cases : List (Expr × List Stmt))
    (default : List Stmt) (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val (Stmt.switch disc cases default) =
    (evalExprTotal fuel env m val disc).bind (fun d =>
      evalSwitchStmtCasesTotal fuel env m val d default cases) := by
    cases h : evalExprTotal fuel env m val disc with
    | none => simp [h, evalStmtTotal, Option.bind]
    | some d => simp [h, evalStmtTotal, Option.bind]

theorem evalStmtTotal_succ_bareCall (fuel env m) (e : Expr) (val : Valuation) :
    evalStmtTotal (fuel + 1) env m val (Stmt.bareCall e) = some val := by
    simp [evalStmtTotal]

theorem evalStmtsTotal_succ_cons (fuel env m) (s : Stmt) (ss : List Stmt) (val : Valuation) :
    evalStmtsTotal (fuel + 1) env m val (s :: ss) =
    if hasExitFlag val then some val
    else (evalStmtTotal fuel env m val s).bind (fun val' =>
      evalStmtsTotal fuel env m val' ss) := by
    by_cases h : hasExitFlag val
    · simp [h, evalStmtsTotal]
    · cases h' : evalStmtTotal fuel env m val s with
      | none => simp [h, h', evalStmtsTotal, Option.bind]
      | some val' => simp [h, h', evalStmtsTotal, Option.bind]

theorem evalStmtsTotal_succ_nil (fuel env m) (val : Valuation) :
    evalStmtsTotal (fuel + 1) env m val [] = some val := by
    simp [evalStmtsTotal]

/-! Verilog statement reductions. -/

theorem evalVStmtTotal_succ_assign_ident (fuel env vm) (name : String) (rhs : VExpr) (val : Valuation) :
    evalVStmtTotal (fuel + 1) env vm val (VStmt.assign (VExpr.ident name) rhs) =
    (evalVExprTotal fuel env vm val rhs).bind (fun rv =>
      some (val.set name (some rv))) := by
    cases h : evalVExprTotal fuel env vm val rhs with
    | none => simp [h, evalVStmtTotal, Option.bind, Valuation.set_eq]
    | some rv => simp [h, evalVStmtTotal, Option.bind, Valuation.set_eq]

theorem evalVStmtTotal_succ_localparam (fuel env vm) (name : String) (w : Nat) (e : VExpr)
    (val : Valuation) :
    evalVStmtTotal (fuel + 1) env vm val (VStmt.localparam name w e) =
    (evalVExprTotal fuel env vm val e).bind (fun rv =>
      some (val.set name (some rv))) := by
    cases h : evalVExprTotal fuel env vm val e with
    | none => simp [h, evalVStmtTotal, Option.bind, Valuation.set_eq]
    | some rv => simp [h, evalVStmtTotal, Option.bind, Valuation.set_eq]

theorem evalVStmtTotal_succ_ifThenElse (fuel env vm) (cond : VExpr) (then_ else_ : List VStmt)
    (val : Valuation) :
    evalVStmtTotal (fuel + 1) env vm val (VStmt.ifThenElse cond then_ else_) =
    (evalVExprTotal fuel env vm val cond).bind (fun c =>
      if c.bits.toNat > 0 then evalVStmtsTotal fuel env vm val then_
      else evalVStmtsTotal fuel env vm val else_) := by
    cases h : evalVExprTotal fuel env vm val cond with
    | none => simp [h, evalVStmtTotal, Option.bind]
    | some c =>
        simp [h, evalVStmtTotal, Option.bind]
        try { split; rfl }

theorem evalVStmtTotal_succ_forLoop (fuel env vm) (var : String) (range : VExpr) (body : List VStmt)
    (val : Valuation) :
    evalVStmtTotal (fuel + 1) env vm val (VStmt.forLoop var range body) =
    (evalVExprTotal fuel env vm val range).bind (fun r =>
      evalVForLoopTotal fuel env vm val var 0 r.bits.toNat body) := by
    cases h : evalVExprTotal fuel env vm val range with
    | none => simp [h, evalVStmtTotal, Option.bind]
    | some r => simp [h, evalVStmtTotal, Option.bind]

theorem evalVStmtTotal_succ_whileLoop (fuel env vm) (cond : VExpr) (body : List VStmt)
    (val : Valuation) :
    evalVStmtTotal (fuel + 1) env vm val (VStmt.whileLoop cond body) =
    evalVWhileLoopTotal fuel env vm val cond body := by
    simp [evalVStmtTotal, Option.bind]

theorem evalVStmtTotal_succ_switch (fuel env vm) (disc : VExpr) (cases : List (VExpr × List VStmt))
    (default : List VStmt) (val : Valuation) :
    evalVStmtTotal (fuel + 1) env vm val (VStmt.switch disc cases default) =
    (evalVExprTotal fuel env vm val disc).bind (fun d =>
      evalVSwitchStmtCasesTotal fuel env vm val d default cases) := by
    cases h : evalVExprTotal fuel env vm val disc with
    | none => simp [h, evalVStmtTotal, Option.bind]
    | some d => simp [h, evalVStmtTotal, Option.bind]

theorem evalVStmtTotal_succ_taskCall (fuel env vm) (name : String) (args : List VExpr)
    (val : Valuation) :
    evalVStmtTotal (fuel + 1) env vm val (VStmt.taskCall name args) = some val := by
    simp [evalVStmtTotal]

theorem evalVStmtTotal_succ_break (fuel env vm) (val : Valuation) :
    evalVStmtTotal (fuel + 1) env vm val VStmt.break =
    some (setFlag val breakFlag ⟨1, 1#1⟩) := by
    simp [evalVStmtTotal, Option.bind]

theorem evalVStmtTotal_succ_continue (fuel env vm) (val : Valuation) :
    evalVStmtTotal (fuel + 1) env vm val VStmt.continue =
    some (setFlag val continueFlag ⟨1, 1#1⟩) := by
    simp [evalVStmtTotal, Option.bind]

theorem evalVStmtsTotal_succ_cons (fuel env vm) (s : VStmt) (ss : List VStmt) (val : Valuation) :
    evalVStmtsTotal (fuel + 1) env vm val (s :: ss) =
    if hasExitFlag val then some val
    else (evalVStmtTotal fuel env vm val s).bind (fun val' =>
      evalVStmtsTotal fuel env vm val' ss) := by
    by_cases h : hasExitFlag val
    · simp [h, evalVStmtsTotal]
    · cases h' : evalVStmtTotal fuel env vm val s with
      | none => simp [h, h', evalVStmtsTotal, Option.bind]
      | some val' => simp [h, h', evalVStmtsTotal, Option.bind]

theorem evalVStmtsTotal_succ_nil (fuel env vm) (val : Valuation) :
    evalVStmtsTotal (fuel + 1) env vm val [] = some val := by
    simp [evalVStmtsTotal]

end EvalEq

section EquivProof

variable (env₀ : Env) (m₀ : Module)
variable (vm0 : VModule)
variable (hctx₀ : Module.callContext env₀ m₀)
variable (hunique₀ : Module.hasUniqueFunctionNames m₀)

/-- Forward-simulation predicate for expressions at fuel `fuel`. -/
def P_expr (fuel : Nat) : Prop :=
  ∀ (val : Valuation) (e : Expr) (ve : VExpr)
    (heq : emitExpr defaultFuel env₀ m₀ e = ve)
    (hcc : Expr.callContext env₀ m₀ e)
    (hcomb : Expr.isCombinational e)
    (hval : Valuation.equiv val val),
    evalExprTotal fuel env₀ m₀ val e = evalVExprTotal fuel env₀ vm0 val ve

/-- Forward-simulation predicate for statements at fuel `fuel`. -/
def P_stmt (fuel : Nat) : Prop :=
  ∀ (val : Valuation) (s : Stmt) (vs : VStmt)
    (heq : emitStmt defaultFuel env₀ m₀ s = vs)
    (hcc : Stmt.callContext env₀ m₀ s)
    (hseq : Stmt.isSequential s)
    (hval : Valuation.equiv val val),
    evalStmtTotal fuel env₀ m₀ val s = evalVStmtTotal fuel env₀ vm0 val vs

/-- Forward-simulation predicate for statement lists at fuel `fuel`. -/
def P_stmts (fuel : Nat) : Prop :=
  ∀ (val : Valuation) (ss : List Stmt) (vss : List VStmt)
    (heq : emitStmts defaultFuel env₀ m₀ ss = vss)
    (hcc : Stmt.callContextList env₀ m₀ ss)
    (hseq : Stmt.isSequentialList ss)
    (hval : Valuation.equiv val val),
    evalStmtsTotal fuel env₀ m₀ val ss = evalVStmtsTotal fuel env₀ vm0 val vss

/-- Forward-simulation predicate for function bodies at fuel `fuel`. -/
def P_function (fuel : Nat) : Prop :=
  ∀ (base : Valuation) (fn : Function) (vfn : VFunction) (argVals : List Value)
    (heq : emitVFunction defaultFuel env₀ m₀ fn = vfn)
    (hcc : Stmt.callContextList env₀ m₀ fn.body)
    (hseq : Stmt.isSequentialList fn.body)
    (hbase : Valuation.equiv base base),
    evalFunctionTotal fuel env₀ m₀ fn argVals base =
    evalVFunctionTotal fuel env₀ vm0 vfn argVals base

/-- Forward-simulation predicate for bounded `forLoop` bodies at fuel `fuel`.
    The loop is treated as a separate predicate because it recurses on `n`,
    but each iteration steps down to the smaller fuel used by the statement
    induction hypothesis. -/
def P_forLoop (fuel : Nat) : Prop :=
  ∀ (val : Valuation) (var : String) (i n : Nat) (body : List Stmt) (vbody : List VStmt)
    (heq : emitStmts defaultFuel env₀ m₀ body = vbody)
    (hcc : Stmt.callContextList env₀ m₀ body)
    (hseq : Stmt.isSequentialList body)
    (hval : Valuation.equiv val val),
    evalForLoopTotal fuel env₀ m₀ val var i n body =
    evalVForLoopTotal fuel env₀ vm0 val var i n vbody

/-- Forward-simulation predicate for bounded `whileLoop` bodies at fuel `fuel`.
    Unlike `forLoop`, the loop recurses only on fuel because the condition is
    re-evaluated dynamically at every iteration. -/
def P_whileLoop (fuel : Nat) : Prop :=
  ∀ (val : Valuation) (cond : Expr) (body : List Stmt) (vcond : VExpr) (vbody : List VStmt)
    (heq_cond : emitExpr defaultFuel env₀ m₀ cond = vcond)
    (heq_body : emitStmts defaultFuel env₀ m₀ body = vbody)
    (hcc_cond : Expr.callContext env₀ m₀ cond)
    (hcc_body : Stmt.callContextList env₀ m₀ body)
    (hcomb_cond : Expr.isCombinational cond)
    (hseq_body : Stmt.isSequentialList body)
    (hval : Valuation.equiv val val),
    evalWhileLoopTotal fuel env₀ m₀ val cond body =
    evalVWhileLoopTotal fuel env₀ vm0 val vcond vbody

/-- Forward-simulation predicate for switch case bodies at fuel `fuel`.  Each case
    body is a statement list; the case walker needs to know that the chosen body
    evaluates the same way on both sides. -/
def P_switchCases (fuel : Nat) : Prop :=
  ∀ (val : Valuation) (d : Value)
    (default : List Stmt) (cases : List (Expr × List Stmt))
    (defaultV : List VStmt) (casesV : List (VExpr × List VStmt))
    (heq_default : emitStmts defaultFuel env₀ m₀ default = defaultV)
    (heq_cases : casesV = cases.map (fun p =>
      (emitExpr defaultFuel env₀ m₀ p.1, emitStmts defaultFuel env₀ m₀ p.2)))
    (hcc_default : Stmt.callContextList env₀ m₀ default)
    (hcc_cases : ∀ p ∈ cases, Expr.callContext env₀ m₀ p.1 ∧ Stmt.callContextList env₀ m₀ p.2)
    (hseq_default : Stmt.isSequentialList default)
    (hseq_cases : ∀ p ∈ cases, Expr.isCombinational p.1 ∧ Stmt.isSequentialList p.2)
    (hval : Valuation.equiv val val)
    (ih_expr : ∀ (e : Expr) (ve : VExpr), emitExpr defaultFuel env₀ m₀ e = ve →
      Expr.callContext env₀ m₀ e → Expr.isCombinational e → evalExprTotal fuel env₀ m₀ val e =
      evalVExprTotal fuel env₀ vm0 val ve)
    (ih_stmts : ∀ (ss : List Stmt) (vss : List VStmt), emitStmts defaultFuel env₀ m₀ ss = vss →
      Stmt.callContextList env₀ m₀ ss → Stmt.isSequentialList ss →
      evalStmtsTotal fuel env₀ m₀ val ss = evalVStmtsTotal fuel env₀ vm0 val vss),
    evalSwitchStmtCasesTotal fuel env₀ m₀ val d default cases =
    evalVSwitchStmtCasesTotal fuel env₀ vm0 val d defaultV casesV

/-- The switch-case walkers agree when each tag evaluates the same way on both
    sides and each body list evaluates the same way.  The proof is by induction on
    the case list; the chosen case is the same on both sides because the tag
    equality tests agree. -/
theorem P_switchCases_holds (fuel : Nat) (val : Valuation) (d : Value)
    (default : List Stmt) (cases : List (Expr × List Stmt))
    (defaultV : List VStmt)
    (heq_default : emitStmts defaultFuel env₀ m₀ default = defaultV)
    (hcc_default : Stmt.callContextList env₀ m₀ default)
    (hcc_cases : ∀ p ∈ cases, Expr.callContext env₀ m₀ p.1 ∧ Stmt.callContextList env₀ m₀ p.2)
    (hseq_default : Stmt.isSequentialList default)
    (hseq_cases : ∀ p ∈ cases, Expr.isCombinational p.1 ∧ Stmt.isSequentialList p.2)
    (hval : Valuation.equiv val val)
    (ih_expr : ∀ (e : Expr) (ve : VExpr), emitExpr defaultFuel env₀ m₀ e = ve →
      Expr.callContext env₀ m₀ e → Expr.isCombinational e → evalExprTotal fuel env₀ m₀ val e =
      evalVExprTotal fuel env₀ vm0 val ve)
    (ih_stmts : ∀ (ss : List Stmt) (vss : List VStmt), emitStmts defaultFuel env₀ m₀ ss = vss →
      Stmt.callContextList env₀ m₀ ss → Stmt.isSequentialList ss →
      evalStmtsTotal fuel env₀ m₀ val ss = evalVStmtsTotal fuel env₀ vm0 val vss) :
    evalSwitchStmtCasesTotal fuel env₀ m₀ val d default cases =
    evalVSwitchStmtCasesTotal fuel env₀ vm0 val d defaultV
      (cases.map (fun p => (emitExpr defaultFuel env₀ m₀ p.1, emitStmts defaultFuel env₀ m₀ p.2))) := by
  induction cases with
  | nil =>
      simp [evalSwitchStmtCasesTotal, evalVSwitchStmtCasesTotal]
      exact ih_stmts default defaultV heq_default hcc_default hseq_default
  | cons c cs ih =>
      rcases c with ⟨tag, body⟩
      simp [evalSwitchStmtCasesTotal, evalVSwitchStmtCasesTotal]
      have htag : evalExprTotal fuel env₀ m₀ val tag =
                evalVExprTotal fuel env₀ vm0 val (emitExpr defaultFuel env₀ m₀ tag) := by
        apply ih_expr tag (emitExpr defaultFuel env₀ m₀ tag) rfl
        · exact (hcc_cases ⟨tag, body⟩ (by simp)).1
        · exact (hseq_cases ⟨tag, body⟩ (by simp)).1
      rw [htag]
      apply Option.bind_congr_ext rfl
      intro t
      apply Option.bind_congr_ext rfl
      intro eq
      split_ifs with hmatch
      · exact ih_stmts body (emitStmts defaultFuel env₀ m₀ body) rfl
          (hcc_cases ⟨tag, body⟩ (by simp)).2
          (hseq_cases ⟨tag, body⟩ (by simp)).2
      · have hcc_tail : ∀ p ∈ cs, Expr.callContext env₀ m₀ p.1 ∧ Stmt.callContextList env₀ m₀ p.2 := by
          intro p hp
          exact hcc_cases p (by simp [hp])
        have hseq_tail : ∀ p ∈ cs, Expr.isCombinational p.1 ∧ Stmt.isSequentialList p.2 := by
          intro p hp
          exact hseq_cases p (by simp [hp])
        exact ih hcc_tail hseq_tail

/-- `P_expr` at a successor fuel follows from the strong induction hypothesis at
    all smaller fuels by structural induction on the expression.  This is needed
    for the empty `switch` case, where the default sub-expression is evaluated at
    the current fuel rather than a smaller one. -/
theorem P_expr_succ (hvm0 : vm0 = emitModuleFuel defaultFuel env₀ m₀)
    (hseq₀ : Module.isSequential env₀ m₀)
    (hctx₀ : Module.callContext env₀ m₀)
    (hunique₀ : Module.hasUniqueFunctionNames m₀)
    (fuel : Nat)
    (ih : ∀ k, k ≤ fuel →
      P_expr env₀ m₀ vm0 k ∧
      P_stmt env₀ m₀ vm0 k ∧
      P_stmts env₀ m₀ vm0 k ∧
      P_function env₀ m₀ vm0 k ∧
      P_forLoop env₀ m₀ vm0 k ∧
      P_whileLoop env₀ m₀ vm0 k)
    (val : Valuation) (e : Expr) (ve : VExpr)
    (heq : emitExpr defaultFuel env₀ m₀ e = ve)
    (hcc : Expr.callContext env₀ m₀ e)
    (hcomb : Expr.isCombinational e)
    (hval : Valuation.equiv val val) :
    evalExprTotal (fuel + 1) env₀ m₀ val e =
    evalVExprTotal (fuel + 1) env₀ vm0 val ve := by
  rw [← heq]
  induction e using Expr.induction_on_lists with
  | boolLit b =>
      cases b
      · simp [evalExprTotal_succ_boolLit, evalVExprTotal_succ_lit, evalVExprTotal_lit_toString, emitExpr_default_boolLit]
      · simp [evalExprTotal_succ_boolLit, evalVExprTotal_succ_lit, evalVExprTotal_lit_toString, emitExpr_default_boolLit]
  | intLit n =>
      simp [evalExprTotal_succ_intLit, evalVExprTotal_succ_lit, evalVExprTotal_lit_toString, emitExpr_default_intLit]
  | f32Lit s =>
      simp [Expr.isCombinational] at hcomb
      all_goals contradiction
  | stringLit s =>
      simp [Expr.isCombinational] at hcomb
      all_goals contradiction
  | identifier name =>
      simp [evalExprTotal_succ_identifier, evalVExprTotal_succ_ident, emitExpr_default_identifier, hval name]
  | binop op l r hl hr =>
      have hcc_l := (Expr.callContext_binop hcc).1
      have hcc_r := (Expr.callContext_binop hcc).2
      have hcomb_l := (Expr.isCombinational_binop hcomb).1
      have hcomb_r := (Expr.isCombinational_binop hcomb).2
      have h_l := (ih fuel (by omega)).1 val l (emitExpr defaultFuel env₀ m₀ l) rfl hcc_l hcomb_l hval
      have h_r := (ih fuel (by omega)).1 val r (emitExpr defaultFuel env₀ m₀ r) rfl hcc_r hcomb_r hval
      simp only [evalExprTotal_succ_binop, evalVExprTotal_succ_binop, emitExpr_default_binop]
      rw [h_l, h_r]
  | unop op e h_e =>
      have hcc_e := Expr.callContext_unop hcc
      have hcomb_e := Expr.isCombinational_unop hcomb
      have h_e' := (ih fuel (by omega)).1 val e (emitExpr defaultFuel env₀ m₀ e) rfl hcc_e hcomb_e hval
      simp only [evalExprTotal_succ_unop, evalVExprTotal_succ_unop, emitExpr_default_unop]
      rw [h_e']
  | fieldAccess base field h_b =>
      have hcc_b := Expr.callContext_fieldAccess hcc
      have hcomb_b := Expr.isCombinational_fieldAccess hcomb
      have h_b' := (ih fuel (by omega)).1 val base (emitExpr defaultFuel env₀ m₀ base) rfl hcc_b hcomb_b hval
      cases hty : Expr.typeOf env₀ m₀ base with
      | some ty =>
          cases ty with
          | struct sname =>
              simp only [hty, evalExprTotal_succ_fieldAccess, evalVExprTotal_succ_slice, emitExpr_default_fieldAccess]
              apply Option.bind_congr_ext h_b'; intro baseV
              generalize hfind : List.find? (fun p => p.1 == field) (env₀.structFields sname) = found at *
              cases found with
              | some ty =>
                  simp (config := { zeta := true }) only [hfind, structFieldOffsetTotal, structFieldWidthTotal, widthOfType, widthOfType_fuel_independent]
                  split_ifs <;> try { conv => lhs; rw [hfind] } <;>
                    try { simp_all [offset_le_add_sub_one, slice_width_eq] }
                  all_goals try { rfl }
              | none =>
                  simp (config := { zeta := true }) only [hfind, structFieldOffsetTotal, structFieldWidthTotal, widthOfType, widthOfType_fuel_independent]
                  split_ifs <;> try { conv => lhs; rw [hfind] } <;>
                    try { simp_all [offset_le_add_sub_one, slice_width_eq] }
                  all_goals try { rfl }
          | _ =>
              simp only [hty, evalExprTotal_succ_fieldAccess, evalVExprTotal_succ_slice, emitExpr_default_fieldAccess]
              rw [h_b']
              apply Option.bind_congr_ext rfl; intro baseV
              simp
      | none =>
          simp only [hty, evalExprTotal_succ_fieldAccess, evalVExprTotal_succ_slice, emitExpr_default_fieldAccess]
          rw [h_b']
          apply Option.bind_congr_ext rfl; intro baseV
          simp
  | index base idx h_b h_i =>
      have hcc_b := (Expr.callContext_index hcc).1
      have hcc' := (Expr.callContext_index hcc).2
      have hcomb_b := (Expr.isCombinational_index hcomb).1
      have hcomb' := (Expr.isCombinational_index hcomb).2
      have h_b' := (ih fuel (by omega)).1 val base (emitExpr defaultFuel env₀ m₀ base) rfl hcc_b hcomb_b hval
      have h_i' := (ih fuel (by omega)).1 val idx (emitExpr defaultFuel env₀ m₀ idx) rfl hcc' hcomb' hval
      cases hty : Expr.typeOf env₀ m₀ base with
      | some ty =>
          cases ty with
          | array n elem =>
              simp only [evalExprTotal_succ_index, evalVExprTotal_succ_index, emitExpr_default_index]
              rw [hty, h_b', h_i']
              apply Option.bind_congr_ext rfl; intro baseV
              apply Option.bind_congr_ext rfl; intro idxV
              have hiw : indexElemWidth defaultFuel env₀ m₀ base = widthOfType defaultFuel env₀ elem :=
                indexElemWidth_eq defaultFuel env₀ m₀ base hty
              rw [hiw, widthOfType_fuel_independent]
              all_goals try { split <;> rfl }
          | _ =>
              simp only [evalExprTotal_succ_index, evalVExprTotal_succ_index, emitExpr_default_index]
              rw [hty, h_b', h_i']
              apply Option.bind_congr_ext rfl; intro baseV
              apply Option.bind_congr_ext rfl; intro idxV
              have hw : indexElemWidth defaultFuel env₀ m₀ base = 8 := by
                simp [indexElemWidth, hty]
              simp [hty, hw]
              all_goals rw [hw]
              all_goals try { split <;> rfl }
      | none =>
          simp only [evalExprTotal_succ_index, evalVExprTotal_succ_index, emitExpr_default_index]
          rw [hty, h_b', h_i']
          apply Option.bind_congr_ext rfl; intro baseV
          apply Option.bind_congr_ext rfl; intro idxV
          have hw : indexElemWidth defaultFuel env₀ m₀ base = 8 := by
            simp [indexElemWidth, hty]
          simp [hty, hw]
          all_goals rw [hw]
          all_goals try { split <;> rfl }
  | call name args h_args =>
      have hcc_call := Expr.callContext_call hcc
      have hcomb_args := Expr.isCombinational_call hcomb
      have hargs_ctx := hcc_call.2.2.2
      simp only [evalExprTotal_succ_call, evalVExprTotal_succ_call, emitExpr_default_call]
      cases hfn : m₀.findFunction name with
      | none =>
          rcases Module.findFunction_of_hasEmittedFunctionNamed hcc_call.2.2.1
            with ⟨fn, hfn', heq_name⟩
          rw [hfn] at hfn'
          contradiction
      | some fn =>
          have hfn_name : fn.name = name := Module.findFunction_name hfn
          have hhost : ¬ Env.isHostOnly env₀ fn.name := by
            rw [hfn_name]
            exact hcc_call.2.1
          have hmem_emitted : fn ∈ Module.emittedFunctions env₀ m₀ :=
            Module.hasEmittedFunctionNamed_findFunction hunique₀ hcc_call.2.2.1 fn hfn
          have hlookup :
            List.find? (fun f => f.name == name) vm0.functions =
            some (emitVFunction defaultFuel env₀ m₀ fn) := by
            rw [hvm0]
            exact emit_function_lookup defaultFuel env₀ m₀ name fn hunique₀ hfn hmem_emitted
          rw [hlookup]
          rw [emitExprList_eq_map]
          have h_args :
            args.mapM (evalExprTotal fuel env₀ m₀ val) =
            (args.map (emitExpr defaultFuel env₀ m₀)).mapM (evalVExprTotal fuel env₀ vm0 val) := by
            have h_eta : args.mapM (evalExprTotal fuel env₀ m₀ val) =
              List.mapM (fun a => evalExprTotal fuel env₀ m₀ val a) args := by rfl
            rw [h_eta]
            rw [List.mapM_map]
            apply List.mapM_congr'
            intro a ha
            exact (ih fuel (by omega)).1 val a (emitExpr defaultFuel env₀ m₀ a) rfl
              (hargs_ctx a ha) (hcomb_args a ha) hval
          apply Option.bind_congr_ext h_args; intro argVals
          have hcc_body : Stmt.callContextList env₀ m₀ fn.body :=
            hctx₀.2 fn (Module.findFunction_mem hfn) hhost
          have hseq_body :=
            Module.isSequential_function_body hseq₀
              (Module.findFunction_mem hfn) hhost
          apply (ih fuel (by omega)).2.2.2.1 val fn (emitVFunction defaultFuel env₀ m₀ fn) argVals rfl hcc_body hseq_body
            (Valuation.equiv_refl val)
  | structLit name fields h_fields =>
      have hcc_fields := Expr.callContext_structLit hcc
      have hcomb_fields := Expr.isCombinational_structLit hcomb
      simp only [evalExprTotal_succ_structLit, evalVExprTotal_succ_concat, emitExpr_default_structLit]
      rw [emitFieldExprs_eq_map]
      have h_fields' :
        List.mapM (fun p => evalExprTotal fuel env₀ m₀ val p.2) fields =
        List.mapM (fun ve => evalVExprTotal fuel env₀ vm0 val ve)
          (fields.map (fun p => emitExpr defaultFuel env₀ m₀ p.2)) := by
        rw [List.mapM_map]
        apply List.mapM_congr'
        intro p hp
        exact (ih fuel (by omega)).1 val p.2 (emitExpr defaultFuel env₀ m₀ p.2) rfl
          (hcc_fields p hp) (hcomb_fields p.1 p.2 hp) hval
      apply Option.bind_congr_ext h_fields'; intro vs
      rfl
  | arrayLit ty elems h_elems =>
      have hcc_elems := Expr.callContext_arrayLit hcc
      have hcomb_elems := Expr.isCombinational_arrayLit hcomb
      simp only [evalExprTotal_succ_arrayLit, evalVExprTotal_succ_concat, emitExpr_default_arrayLit]
      rw [emitExprList_eq_map]
      have h_elems' :
        List.mapM (fun a => evalExprTotal fuel env₀ m₀ val a) elems =
        List.mapM (fun ve => evalVExprTotal fuel env₀ vm0 val ve)
          (elems.map (emitExpr defaultFuel env₀ m₀)) := by
        rw [List.mapM_map]
        apply List.mapM_congr'
        intro a ha
        exact (ih fuel (by omega)).1 val a (emitExpr defaultFuel env₀ m₀ a) rfl
          (hcc_elems a ha) (hcomb_elems a ha) hval
      apply Option.bind_congr_ext h_elems'; intro vs
      rfl
  | enumVal enum variant =>
      rw [evalExprTotal_succ_enumVal, emitExpr_default_enumVal, evalVExprTotal_lit_toString]
  | len base h_b =>
      simp [Expr.isCombinational] at hcomb
      all_goals contradiction
  | contains base item h_b h_i =>
      simp [Expr.isCombinational] at hcomb
      all_goals contradiction
  | switch disc cases default h_disc h_cases h_default =>
      have hcc_s := Expr.callContext_switch hcc
      have hcomb_s := Expr.isCombinational_switch hcomb
      simp only [evalExprTotal_succ_switch, emitExpr_default_switch]
      rw [evalVExprTotal_foldr_ternary_cases]
      cases cases with
      | nil =>
          have heq_default : emitExpr defaultFuel env₀ m₀ default = ve := by
            rw [← heq]
            rw [emitExpr_default_switch]
            all_goals rfl
          simp [evalSwitchCasesTotal, evalVTernaryCasesTotal]
          exact h_default heq_default hcc_s.2.2 hcomb_s.2.2
      | cons c cs =>
          rcases c with ⟨tag, res⟩
          have hcc_tag : Expr.callContext env₀ m₀ tag := by
            apply (hcc_s.2.1 ⟨tag, res⟩ (by simp)).1
          have hcc_res : Expr.callContext env₀ m₀ res := by
            apply (hcc_s.2.1 ⟨tag, res⟩ (by simp)).2
          have hcomb_tag : Expr.isCombinational tag := by
            apply (hcomb_s.2.1 ⟨tag, res⟩ (by simp)).1
          have hcomb_res : Expr.isCombinational res := by
            apply (hcomb_s.2.1 ⟨tag, res⟩ (by simp)).2
          have hcc_tail : Expr.callContext env₀ m₀ (Expr.switch disc cs default) :=
            Expr.callContext_switch_tail hcc
          have hcomb_tail : Expr.isCombinational (Expr.switch disc cs default) :=
            Expr.isCombinational_switch_tail hcomb
          cases fuel with
          | zero =>
              simp [evalSwitchCasesTotal, evalVTernaryCasesTotal]
          | succ fuel =>
              have h_disc := (ih fuel (by omega)).1 val disc
                (emitExpr defaultFuel env₀ m₀ disc) rfl hcc_s.1 hcomb_s.1 hval
              have h_tag := (ih fuel (by omega)).1 val tag
                (emitExpr defaultFuel env₀ m₀ tag) rfl hcc_tag hcomb_tag hval
              have h_res := (ih (fuel + 1) (by omega)).1 val res
                (emitExpr defaultFuel env₀ m₀ res) rfl hcc_res hcomb_res hval
              have h_tail := (ih (fuel + 1) (by omega)).1 val
                (Expr.switch disc cs default)
                (emitExpr defaultFuel env₀ m₀ (Expr.switch disc cs default)) rfl
                hcc_tail hcomb_tail hval
              have h_tail_cases :
                evalSwitchCasesTotal (fuel + 1) env₀ m₀ val disc default cs =
                evalVTernaryCasesTotal (fuel + 1) env₀ vm0 val
                  (emitExpr defaultFuel env₀ m₀ disc)
                  (emitExpr defaultFuel env₀ m₀ default)
                  (cs.map (fun c =>
                    (emitExpr defaultFuel env₀ m₀ c.1, emitExpr defaultFuel env₀ m₀ c.2))) := by
                rw [← evalExprTotal_eq_switch (fuel + 1) env₀ m₀ disc cs default val]
                rw [h_tail]
                rw [emitExpr_default_switch]
                exact evalVExprTotal_foldr_ternary_cases (fuel + 1) env₀ m₀ vm0 disc default cs val
              simp [evalSwitchCasesTotal, evalVTernaryCasesTotal]
              rw [h_disc]
              apply Option.bind_congr_ext rfl; intro d
              rw [h_tag]
              apply Option.bind_congr_ext rfl; intro t
              apply Option.bind_congr_ext rfl; intro eq
              split_ifs with hmatch
              · exact h_res
              · exact h_tail_cases
  | unsupportedIcarus r =>
      simp [Expr.isCombinational] at hcomb
      all_goals contradiction


/-- Combined forward-simulation invariant.  The proof is a single induction on
    fuel; at `fuel + 1` every recursive sub-evaluation happens at the smaller
    fuel `fuel`, which is exactly what the induction hypothesis covers. -/
theorem all_equiv (hvm0 : vm0 = emitModuleFuel defaultFuel env₀ m₀)
    (hseq₀ : Module.isSequential env₀ m₀)
    (hctx₀ : Module.callContext env₀ m₀)
    (hunique₀ : Module.hasUniqueFunctionNames m₀)
    (fuel : Nat) :
    P_expr env₀ m₀ vm0 fuel ∧
    P_stmt env₀ m₀ vm0 fuel ∧
    P_stmts env₀ m₀ vm0 fuel ∧
    P_function env₀ m₀ vm0 fuel ∧
    P_forLoop env₀ m₀ vm0 fuel ∧
    P_whileLoop env₀ m₀ vm0 fuel := by
  induction fuel using Nat.strong_induction_on with
  | h fuel ih =>
      cases fuel with
      | zero =>
          constructor
          · intro val e ve heq hcc hseq hval
            rw [← heq]
            cases e <;> unfold evalExprTotal evalVExprTotal <;> rfl
          constructor
          · intro val s vs heq hcc hseq hval
            rw [← heq]
            cases s
            all_goals
              simp [evalStmtTotal, evalVStmtTotal, evalForLoopTotal_zero, evalVForLoopTotal_zero]
          constructor
          · intro val ss vss heq hcc hseq hval
            rw [← heq]
            cases ss <;> unfold evalStmtsTotal evalVStmtsTotal <;> rfl
          constructor
          · intro base fn vfn argVals heq hcc hseq hbase
            rw [← heq]
            unfold evalFunctionTotal evalVFunctionTotal
            rfl
          constructor
          · -- P_forLoop 0
            intro val var i n body vbody heq hcc hseq hval
            rw [← heq]
            simp [evalForLoopTotal_zero, evalVForLoopTotal_zero]
          · -- P_whileLoop 0
            intro val cond body vcond vbody heq_cond heq_body hcc_cond hcc_body hcomb_cond hseq_body hval
            rw [← heq_cond, ← heq_body]
            simp [evalWhileLoopTotal_zero, evalVWhileLoopTotal_zero]
      | succ fuel =>
      have h_ih : ∀ k, k ≤ fuel →
          P_expr env₀ m₀ vm0 k ∧
          P_stmt env₀ m₀ vm0 k ∧
          P_stmts env₀ m₀ vm0 k ∧
          P_function env₀ m₀ vm0 k ∧
          P_forLoop env₀ m₀ vm0 k ∧
          P_whileLoop env₀ m₀ vm0 k := by
        intro k hk
        exact ih k (by omega)
      have ih_expr := (h_ih fuel (by omega)).1
      have ih_stmt := (h_ih fuel (by omega)).2.1
      have ih_stmts := (h_ih fuel (by omega)).2.2.1
      have ih_fn := (h_ih fuel (by omega)).2.2.2.1
      have ih_loop := (h_ih fuel (by omega)).2.2.2.2.1
      have ih_while := (h_ih fuel (by omega)).2.2.2.2.2
      constructor
      · -- P_expr (fuel + 1)
        intro val e ve heq hcc_e hcomb_e hval
        exact P_expr_succ env₀ m₀ vm0 hvm0 hseq₀ hctx₀ hunique₀ fuel h_ih val e ve heq hcc_e hcomb_e hval
      constructor
      · -- P_stmt (fuel + 1)
        intro val s vs heq hcc_s hseq_s hval
        rw [← heq]
        cases s with
        | assign lhs rhs =>
            rcases Stmt.isSequential_assign hseq_s with ⟨name, rfl, hcomb_r⟩
            have hcc_r := Stmt.callContext_assign hcc_s
            simp only [emitStmt_default_assign, emitExpr_default_identifier,
              evalStmtTotal_succ_assign_ident, evalVStmtTotal_succ_assign_ident]
            have h_r := ih_expr val rhs (emitExpr defaultFuel env₀ m₀ rhs) rfl hcc_r hcomb_r hval
            rw [h_r]
            apply Option.bind_congr_ext rfl
            intro rv
            apply congr_arg some
            funext x
            simp [Valuation.set_eq]
        | varDecl name ty init =>
            cases init with
            | none =>
                simp [Stmt.isSequential] at hseq_s
            | some e =>
                have hcc_e := Stmt.callContext_varDecl hcc_s
                have hcomb_e := Stmt.isSequential_varDecl hseq_s
                simp only [emitStmt_default_varDecl_some,
                  evalStmtTotal_succ_varDecl_some, evalVStmtTotal_succ_assign_ident]
                have h_e := ih_expr val e (emitExpr defaultFuel env₀ m₀ e) rfl hcc_e hcomb_e hval
                rw [h_e]
                apply Option.bind_congr_ext rfl
                intro rv
                apply congr_arg some
                funext x
                simp [Valuation.set_eq]
        | constDecl name ty init =>
            cases init with
            | none =>
                simp [Stmt.isSequential] at hseq_s
            | some e =>
                have hcc_e := Stmt.callContext_constDecl hcc_s
                have hcomb_e := Stmt.isSequential_constDecl hseq_s
                simp only [emitStmt_default_constDecl_some,
                  evalStmtTotal_succ_constDecl_some, evalVStmtTotal_succ_localparam]
                have h_e := ih_expr val e (emitExpr defaultFuel env₀ m₀ e) rfl hcc_e hcomb_e hval
                rw [h_e]
                apply Option.bind_congr_ext rfl
                intro rv
                apply congr_arg some
                funext x
                simp [Valuation.set_eq]
        | return_ e =>
            cases e with
            | none =>
                simp [Stmt.isSequential] at hseq_s
            | some e =>
                have hcc_e := Stmt.callContext_return hcc_s
                have hcomb_e := Stmt.isSequential_return hseq_s
                simp only [emitStmt_default_return_some,
                  evalStmtTotal_succ_return_some, evalVStmtTotal_succ_assign_ident]
                have h_e := ih_expr val e (emitExpr defaultFuel env₀ m₀ e) rfl hcc_e hcomb_e hval
                rw [h_e]
                apply Option.bind_congr_ext rfl
                intro rv
                apply congr_arg some
                funext x
                simp [Valuation.set_eq, setFlag, returnFlag]
                try { by_cases h : x == "__return"; simp [h] }
        | bareCall e =>
            have hcomb_e := Stmt.isSequential_bareCall hseq_s
            have hcc_e := Stmt.callContext_bareCall hcc_s
            simp [emitStmt_default_bareCall,
              evalStmtTotal_succ_bareCall, evalVStmtTotal_succ_taskCall]
            -- The t27 bare-call semantics ignores `e`, matching the Verilog
            -- task-call semantics, so no expression IH is needed.
        | ifThenElse cond then_ else_ =>
            have hcomb_cond := (Stmt.isSequential_ifThenElse hseq_s).1
            have hcomb_then := (Stmt.isSequential_ifThenElse hseq_s).2.1
            have hcomb_else := (Stmt.isSequential_ifThenElse hseq_s).2.2
            have hcc_cond := (Stmt.callContext_ifThenElse hcc_s).1
            have hcc_then := (Stmt.callContext_ifThenElse hcc_s).2.1
            have hcc_else := (Stmt.callContext_ifThenElse hcc_s).2.2
            simp only [emitStmt_default_ifThenElse,
              evalStmtTotal_succ_ifThenElse, evalVStmtTotal_succ_ifThenElse]
            have h_cond := ih_expr val cond (emitExpr defaultFuel env₀ m₀ cond) rfl hcc_cond hcomb_cond hval
            rw [h_cond]
            apply Option.bind_congr_ext rfl
            intro c
            split
            · exact ih_stmts val then_ (emitStmts defaultFuel env₀ m₀ then_) rfl hcc_then hcomb_then hval
            · exact ih_stmts val else_ (emitStmts defaultFuel env₀ m₀ else_) rfl hcc_else hcomb_else hval
        | switch disc cases default =>
            have hcomb_disc := (Stmt.isSequential_switch hseq_s).1
            have hseq_cases := (Stmt.isSequential_switch hseq_s).2.1
            have hseq_default := (Stmt.isSequential_switch hseq_s).2.2
            have hcc_disc := (Stmt.callContext_switch hcc_s).1
            have hcc_cases := (Stmt.callContext_switch hcc_s).2.1
            have hcc_default := (Stmt.callContext_switch hcc_s).2.2
            simp only [emitStmt_default_switch,
              evalStmtTotal_succ_switch, evalVStmtTotal_succ_switch]
            have h_disc := ih_expr val disc (emitExpr defaultFuel env₀ m₀ disc) rfl hcc_disc hcomb_disc hval
            rw [h_disc]
            apply Option.bind_congr_ext rfl
            intro d
            exact P_switchCases_holds env₀ m₀ vm0 fuel val d default cases
              (emitStmts defaultFuel env₀ m₀ default) rfl hcc_default hcc_cases hseq_default hseq_cases hval
              (fun e ve heq hcc hcomb => ih_expr val e ve heq hcc hcomb hval)
              (fun ss vss heq hcc hseq => ih_stmts val ss vss heq hcc hseq hval)
        | forLoop var range body =>
            have hcomb_range := (Stmt.isSequential_forLoop hseq_s).1
            have hseq_body := (Stmt.isSequential_forLoop hseq_s).2
            have hcc_range := (Stmt.callContext_forLoop hcc_s).1
            have hcc_body := (Stmt.callContext_forLoop hcc_s).2
            simp only [emitStmt_default_forLoop,
              evalStmtTotal_succ_forLoop, evalVStmtTotal_succ_forLoop]
            have h_range := ih_expr val range (emitExpr defaultFuel env₀ m₀ range) rfl hcc_range hcomb_range hval
            rw [h_range]
            apply Option.bind_congr_ext rfl
            intro r
            exact ih_loop val var 0 r.bits.toNat body
              (emitStmts defaultFuel env₀ m₀ body) rfl hcc_body hseq_body hval
        | whileLoop cond body =>
            have hcomb_cond := (Stmt.isSequential_whileLoop hseq_s).1
            have hseq_body := (Stmt.isSequential_whileLoop hseq_s).2
            have hcc_cond := (Stmt.callContext_whileLoop hcc_s).1
            have hcc_body := (Stmt.callContext_whileLoop hcc_s).2
            simp only [emitStmt_default_whileLoop,
              evalStmtTotal_succ_whileLoop, evalVStmtTotal_succ_whileLoop]
            exact ih_while val cond body
              (emitExpr defaultFuel env₀ m₀ cond) (emitStmts defaultFuel env₀ m₀ body)
              rfl rfl hcc_cond hcc_body hcomb_cond hseq_body hval
        | «break» =>
            simp only [emitStmt_default_break, evalStmtTotal_succ_break, evalVStmtTotal_succ_break]
        | «continue» =>
            simp only [emitStmt_default_continue, evalStmtTotal_succ_continue, evalVStmtTotal_succ_continue]
      constructor
      · -- P_stmts (fuel + 1)
        intro val ss vss heq hcc_ss hseq_ss hval
        rw [← heq]
        cases ss with
        | nil =>
            simp [emitStmts_default_nil, evalStmtsTotal_succ_nil, evalVStmtsTotal_succ_nil]
        | cons s ss =>
            have hcc_s := Stmt.callContext_list_mem (s := s) hcc_ss (by simp)
            have hcc_ss' := Stmt.callContext_list_tail hcc_ss
            have hseq_s := Stmt.isSequentialList_head (s := s) hseq_ss
            have hseq_ss' := Stmt.isSequentialList_tail hseq_ss
            simp only [evalStmtsTotal_succ_cons, evalVStmtsTotal_succ_cons, emitStmts_default_cons]
            have h_s := ih_stmt val s (emitStmt defaultFuel env₀ m₀ s) rfl hcc_s hseq_s hval
            rw [h_s]
            split_ifs with hexit
            · rfl
            · apply Option.bind_congr_ext rfl
              intro val'
              exact ih_stmts val' ss (emitStmts defaultFuel env₀ m₀ ss) rfl hcc_ss' hseq_ss' (Valuation.equiv_refl val')
      constructor
      · -- P_function (fuel + 1)
        intro base fn vfn argVals heq hcc_fn hseq_fn hbase
        rw [← heq]
        simp only [evalFunctionTotal_succ, evalVFunctionTotal_succ, emitVFunction]
        have hinit :
          (fun name =>
            (fn.params.zip argVals).find? (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name)) =
          (fun name =>
            ((fn.params.map (fun p => (p.1, widthOfType defaultFuel env₀ p.2))).zip argVals).find?
              (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name)) := by
          funext name
          simp [paramLookup_eq]
        rw [hinit]
        rw [ih_stmts
          (fun name =>
            ((fn.params.map (fun p => (p.1, widthOfType defaultFuel env₀ p.2))).zip argVals).find?
              (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name))
          fn.body (emitStmts defaultFuel env₀ m₀ fn.body) rfl hcc_fn hseq_fn
          (Valuation.equiv_refl _)]
      constructor
      · -- P_forLoop (fuel + 1)
        intro val var i n body vbody heq hcc hseq hval
        rw [← heq]
        cases n with
        | zero =>
            simp [evalForLoopTotal, evalVForLoopTotal]
        | succ n =>
            simp only [evalForLoopTotal, evalVForLoopTotal]
            have h_body := ih_stmts
              (fun x => if x == var then some ⟨32, BitVec.ofNat 32 i⟩ else val x)
              body (emitStmts defaultFuel env₀ m₀ body) rfl hcc hseq
              (Valuation.equiv_refl _)
            rw [h_body]
            apply Option.bind_congr_ext rfl
            intro val'
            split_ifs with hexit
            · rfl
            · exact ih_loop (clearLoopFlags val') var (i + 1) n body
                (emitStmts defaultFuel env₀ m₀ body) rfl hcc hseq
                (Valuation.equiv_refl _)
      · -- P_whileLoop (fuel + 1)
        intro val cond body vcond vbody heq_cond heq_body hcc_cond hcc_body hcomb_cond hseq_body hval
        rw [← heq_cond, ← heq_body]
        simp only [evalWhileLoopTotal, evalVWhileLoopTotal]
        have h_cond := ih_expr val cond (emitExpr defaultFuel env₀ m₀ cond) rfl hcc_cond hcomb_cond hval
        rw [h_cond]
        apply Option.bind_congr_ext rfl
        intro c
        split_ifs with hcond
        · have h_body := ih_stmts val body (emitStmts defaultFuel env₀ m₀ body) rfl hcc_body hseq_body (Valuation.equiv_refl _)
          rw [h_body]
          apply Option.bind_congr_ext rfl
          intro val'
          split_ifs with hexit
          · rfl
          · exact ih_while (clearLoopFlags val') cond body
              (emitExpr defaultFuel env₀ m₀ cond) (emitStmts defaultFuel env₀ m₀ body)
              rfl rfl hcc_cond hcc_body hcomb_cond hseq_body (Valuation.equiv_refl _)
        · rfl

end EquivProof

/-- `evalModuleFunctionTotal` as a single `Option.bind` chain. -/
theorem evalModuleFunctionTotal_bind (fuel : Nat) (env : Env) (m : Module) (fnName : String)
    (args : List Value) :
    evalModuleFunctionTotal fuel env m fnName args =
    (evalStmtsTotal fuel env m (fun _ => none) m.globals).bind (fun initVal =>
      (m.findFunction fnName).bind (fun fn =>
        evalFunctionTotal fuel env m fn args initVal)) := by
  unfold evalModuleFunctionTotal
  cases evalStmtsTotal fuel env m (fun _ => none) m.globals <;> cases m.findFunction fnName <;> rfl

/-- `evalVModuleTotal` as a single `Option.bind` chain. -/
theorem evalVModuleTotal_bind (fuel : Nat) (env : Env) (vm : VModule) (fnName : String)
    (args : List Value) :
    evalVModuleTotal fuel env vm fnName args =
    (evalVStmtsTotal fuel env vm (fun _ => none) vm.globals).bind (fun initVal =>
      (List.find? (fun f => f.name == fnName) vm.functions).bind (fun fn =>
        evalVFunctionTotal fuel env vm fn args initVal)) := by
  unfold evalVModuleTotal
  cases evalVStmtsTotal fuel env vm (fun _ => none) vm.globals <;>
    cases List.find? (fun f => f.name == fnName) vm.functions <;> rfl

/-- Wrapper that exposes the generic equivalence theorem in the shape required
    by `Soundness.lean`.

    W501: the theorem is parameterized over any emitted function name, not just
    `main`.  The only remaining well-formedness assumptions are lowerability,
    combinationality, unique function names, the module-level call-context
    invariant, and the fact that the chosen function is not a host-only helper. -/
theorem module_value_equiv_proved (env : Env) (m : Module)
    (_h : Module.isLowerable env m)
    (hunique : Module.hasUniqueFunctionNames m)
    (hcomb : Module.isCombinational env m)
    (hctx : Module.callContext env m)
    (fnName : String)
    (fn : Function)
    (args : List Value)
    (hm : m.findFunction fnName = some fn)
    (hhost : ¬ Env.isHostOnly env fn.name) :
    evalModuleFunctionTotal defaultFuel env m fnName args =
    evalVModuleTotal defaultFuel env (emitModule env m) fnName args := by
  let vm := emitModuleFuel defaultFuel env m
  have hvm : vm = emitModule env m := by simp [emitModule, vm]
  have hseq : Module.isSequential env m := Module.isCombinational_implies_isSequential env m hcomb
  have hcomb_globals : Stmt.isCombinationalList m.globals := by
    have h : ∀ s, s ∈ m.globals → Stmt.isCombinational s = true := by
      simp only [Module.isCombinational, Bool.and_eq_true, List.all_eq_true] at hcomb
      exact hcomb.1
    simp only [Stmt.isCombinationalList, Stmt.isCombinational, List.all_eq_true]
    exact h
  have hseq_globals : Stmt.isSequentialList m.globals :=
    Stmt.isCombinationalList_implies_isSequentialList m.globals hcomb_globals
  have hcomb_fn : Stmt.isCombinationalList fn.body :=
    Module.isCombinational_function_body hcomb (Module.findFunction_mem hm) hhost
  have hseq_fn : Stmt.isSequentialList fn.body :=
    Stmt.isCombinationalList_implies_isSequentialList fn.body hcomb_fn
  have hhost_eq : (Env.isHostOnly env fn.name) = false := by
    simp only [Bool.not_eq_true] at hhost ⊢
    exact hhost
  have hmem_emitted : fn ∈ Module.emittedFunctions env m := by
    simp only [Module.emittedFunctions, List.mem_filter]
    exact ⟨Module.findFunction_mem hm, by simp [hhost_eq]⟩
  have hlookup :
    List.find? (fun f => f.name == fnName) vm.functions =
    some (emitVFunction defaultFuel env m fn) := by
    rw [hvm]
    exact emit_function_lookup defaultFuel env m fnName fn hunique hm hmem_emitted
  rw [← hvm]
  rw [evalModuleFunctionTotal_bind, evalVModuleTotal_bind]
  have h_globals := (all_equiv env m vm (by rfl) hseq hctx hunique defaultFuel).2.2.1
    (fun _ => none) m.globals
    (emitStmts defaultFuel env m m.globals) rfl hctx.1 hseq_globals (fun _ => rfl)
  rw [h_globals]
  apply Option.bind_congr_ext rfl
  intro initVal
  rw [hm, hlookup]
  apply (all_equiv env m vm (by rfl) hseq hctx hunique defaultFuel).2.2.2.1 initVal fn
    (emitVFunction defaultFuel env m fn) args rfl
    (hctx.2 fn (Module.findFunction_mem hm) hhost) hseq_fn
    (Valuation.equiv_refl initVal)

/-- Sequential variant: same conclusion from the weaker `Module.isSequential`
    hypothesis.  This is the theorem `Soundness.lean` uses for modules that
    contain bounded `forLoop` bodies. -/
theorem module_value_equiv_proved_sequential (env : Env) (m : Module)
    (_h : Module.isLowerable env m)
    (hunique : Module.hasUniqueFunctionNames m)
    (hseq : Module.isSequential env m)
    (hctx : Module.callContext env m)
    (fnName : String)
    (fn : Function)
    (args : List Value)
    (hm : m.findFunction fnName = some fn)
    (hhost : ¬ Env.isHostOnly env fn.name) :
    evalModuleFunctionTotal defaultFuel env m fnName args =
    evalVModuleTotal defaultFuel env (emitModule env m) fnName args := by
  let vm := emitModuleFuel defaultFuel env m
  have hvm : vm = emitModule env m := by simp [emitModule, vm]
  have hseq_globals : Stmt.isSequentialList m.globals := by
    have h : ∀ s, s ∈ m.globals → Stmt.isSequential s = true := by
      simp only [Module.isSequential, Bool.and_eq_true, List.all_eq_true] at hseq
      exact hseq.1
    simp only [Stmt.isSequentialList, Stmt.isSequential, List.all_eq_true]
    exact h
  have hseq_fn : Stmt.isSequentialList fn.body :=
    Module.isSequential_function_body hseq (Module.findFunction_mem hm) hhost
  have hhost_eq : (Env.isHostOnly env fn.name) = false := by
    simp only [Bool.not_eq_true] at hhost ⊢
    exact hhost
  have hmem_emitted : fn ∈ Module.emittedFunctions env m := by
    simp only [Module.emittedFunctions, List.mem_filter]
    exact ⟨Module.findFunction_mem hm, by simp [hhost_eq]⟩
  have hlookup :
    List.find? (fun f => f.name == fnName) vm.functions =
    some (emitVFunction defaultFuel env m fn) := by
    rw [hvm]
    exact emit_function_lookup defaultFuel env m fnName fn hunique hm hmem_emitted
  rw [← hvm]
  rw [evalModuleFunctionTotal_bind, evalVModuleTotal_bind]
  have h_globals := (all_equiv env m vm (by rfl) hseq hctx hunique defaultFuel).2.2.1
    (fun _ => none) m.globals
    (emitStmts defaultFuel env m m.globals) rfl hctx.1 hseq_globals (fun _ => rfl)
  rw [h_globals]
  apply Option.bind_congr_ext rfl
  intro initVal
  rw [hm, hlookup]
  apply (all_equiv env m vm (by rfl) hseq hctx hunique defaultFuel).2.2.2.1 initVal fn
    (emitVFunction defaultFuel env m fn) args rfl
    (hctx.2 fn (Module.findFunction_mem hm) hhost) hseq_fn
    (Valuation.equiv_refl initVal)

/-- Convenience corollary for the common `main` entry point (sequential). -/
theorem module_value_equiv_main_sequential (env : Env) (m : Module)
    (_h : Module.isLowerable env m)
    (hunique : Module.hasUniqueFunctionNames m)
    (hseq : Module.isSequential env m)
    (hctx : Module.callContext env m)
    (mainFn : Function)
    (args : List Value)
    (hm : m.findFunction "main" = some mainFn)
    (hmain : ¬ Env.isHostOnly env mainFn.name) :
    evalModuleFunctionTotal defaultFuel env m "main" args =
    evalVModuleTotal defaultFuel env (emitModule env m) "main" args := by
  exact module_value_equiv_proved_sequential env m _h hunique hseq hctx "main" mainFn args hm hmain

/-- Convenience corollary for the common `main` entry point. -/
theorem module_value_equiv_main (env : Env) (m : Module)
    (_h : Module.isLowerable env m)
    (hunique : Module.hasUniqueFunctionNames m)
    (hcomb : Module.isCombinational env m)
    (hctx : Module.callContext env m)
    (mainFn : Function)
    (args : List Value)
    (hm : m.findFunction "main" = some mainFn)
    (hmain : ¬ Env.isHostOnly env mainFn.name) :
    evalModuleFunctionTotal defaultFuel env m "main" args =
    evalVModuleTotal defaultFuel env (emitModule env m) "main" args := by
  exact module_value_equiv_proved env m _h hunique hcomb hctx "main" mainFn args hm hmain

end FuelFacts

end Trinity.IcarusLowerable
