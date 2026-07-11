/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 495
  Soundness and value-preservation lemmas for the Icarus-lowerability predicate.

  The predicate and the emitter model are both partial computational
  functions, so the claims are proved per concrete module with `native_decide`
  rather than by a single structural induction over the AST.

  The intended contracts:
    Module.isLowerable env m  →  emitModule env m has no placeholder
    Module.isLowerable env m  →
      evalModuleFunction env m "main" [] = evalVModule env (emitModule env m) "main"

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

/-- W495: the W493 nested-struct-field-from-identifier witness is lowerable. -/
theorem w493_nested_identifier_lowerable :
  Module.isLowerable w493NestedIdentifierEnv w493NestedIdentifierModule := by
  native_decide

/-- W495: the W493 local-scalar-struct-field witness is lowerable. -/
theorem w493_local_scalar_lowerable :
  Module.isLowerable w493LocalScalarEnv w493LocalScalarModule := by
  native_decide

/-- W495: the W493 module-scalar-struct-field witness is lowerable. -/
theorem w493_module_scalar_lowerable :
  Module.isLowerable w493ModuleScalarEnv w493ModuleScalarModule := by
  native_decide

/-- W495: the W493 module-AOS-element-field witness is lowerable. -/
theorem w493_module_aos_lowerable :
  Module.isLowerable w493ModuleAosEnv w493ModuleAosModule := by
  native_decide

/-- W494/W495: value preservation for the scalar-struct-literal witness.
    The t27 module and the emitted shallow Verilog module compute the same
    packed bit-vector value for the `main` function return. -/
theorem scalar_struct_value_equiv :
  evalModuleFunction scalarStructEnv scalarStructModule "main" []
    = evalVModule scalarStructEnv (emitModule scalarStructEnv scalarStructModule) "main" := by
  native_decide

/-- W495: value preservation for nested-struct field access from a scalar-struct
    identifier parameter. -/
theorem w493_nested_identifier_value_equiv :
  evalModuleFunction w493NestedIdentifierEnv w493NestedIdentifierModule "get_y" []
    = evalVModule w493NestedIdentifierEnv (emitModule w493NestedIdentifierEnv w493NestedIdentifierModule) "get_y" := by
  native_decide

/-- W495: value preservation for struct-literal field initialized from a local
    scalar-struct variable. -/
theorem w493_local_scalar_value_equiv :
  evalModuleFunction w493LocalScalarEnv w493LocalScalarModule "get_y" []
    = evalVModule w493LocalScalarEnv (emitModule w493LocalScalarEnv w493LocalScalarModule) "get_y" := by
  native_decide

/-- W495: value preservation for struct-literal field initialized from a module-
    level scalar-struct constant. -/
theorem w493_module_scalar_value_equiv :
  evalModuleFunction w493ModuleScalarEnv w493ModuleScalarModule "get_y" []
    = evalVModule w493ModuleScalarEnv (emitModule w493ModuleScalarEnv w493ModuleScalarModule) "get_y" := by
  native_decide

/-- W495: value preservation for struct-literal field initialized from a literal-
    index element of a module-level array-of-struct constant. -/
theorem w493_module_aos_value_equiv :
  evalModuleFunction w493ModuleAosEnv w493ModuleAosModule "get_y" []
    = evalVModule w493ModuleAosEnv (emitModule w493ModuleAosEnv w493ModuleAosModule) "get_y" := by
  native_decide

/-- Generic value-preservation theorem statement.  The current model proves
    this for the representative witness set; a full structural proof is future
    work once the model covers conditionals and loops. -/
theorem module_value_equiv_statement (env : Env) (m : Module)
    (h : Module.isLowerable env m)
    (mainFn : Function)
    (hm : m.findFunction "main" = some mainFn) :
    evalModuleFunction env m "main" [] =
    evalVModule env (emitModule env m) "main" := by
  -- Proved by `native_decide` on concrete lowerable modules; the structural
  -- induction remains open for parametric modules with conditionals/loops.
  sorry

end Trinity.IcarusLowerable
