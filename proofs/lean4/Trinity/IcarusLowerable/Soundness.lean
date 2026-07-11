/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 492
  Soundness lemmas for the Icarus-lowerability predicate.

  The predicate and the emitter model are both partial computational
  functions, so the soundness claim is proved per concrete module with
  `native_decide` rather than by a single structural induction over the AST.

  The intended contract:
    Module.isLowerable env m  →  emitModule env m has no placeholder

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

import Trinity.IcarusLowerable.Ast
import Trinity.IcarusLowerable.Predicate
import Trinity.IcarusLowerable.Verilog
import Trinity.IcarusLowerable.Emitter
import Trinity.IcarusLowerable.Lemmas
import Trinity.IcarusLowerable.Semantics

namespace Trinity.IcarusLowerable

/-- A module is *sound* when lowerability implies a placeholder-free modeled
    Verilog output.  This is the top-level contract that the gate enforces. -/
def Module.isSound (env : Env) (m : Module) : Prop :=
  Module.isLowerable env m → ¬ (emitModule env m).hasPlaceholder

/-- The scalar-struct-literal witness satisfies the soundness contract. -/
theorem scalar_struct_sound :
  Module.isSound scalarStructEnv scalarStructModule := by
  unfold Module.isSound
  intro h
  native_decide

/-- The imported-constructor expression-context witness satisfies the soundness
    contract. -/
theorem imported_ctor_sound :
  Module.isSound importedCtorEnv importedCtorModule := by
  unfold Module.isSound
  intro h
  native_decide

/-- The array-field index on struct-return call witness satisfies the soundness
    contract. -/
theorem array_field_sound :
  Module.isSound arrayFieldEnv arrayFieldModule := by
  unfold Module.isSound
  intro h
  native_decide

/-- The variable-index local array-field witness satisfies the soundness
    contract. -/
theorem var_index_sound :
  Module.isSound varIndexEnv varIndexModule := by
  unfold Module.isSound
  intro h
  native_decide

/-- W494: value preservation for the scalar-struct-literal witness.
    The t27 module and the emitted shallow Verilog module compute the same
    packed bit-vector value for the `main` function return. -/
theorem scalar_struct_value_equiv :
  evalFunction scalarStructEnv scalarStructModule scalarStructMain []
    = evalVModule scalarStructEnv (emitModule scalarStructEnv scalarStructModule) := by
  native_decide

end Trinity.IcarusLowerable
