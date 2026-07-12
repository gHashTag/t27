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
import Trinity.IcarusLowerable.SemanticsTotal
import Trinity.IcarusLowerable.AstInduction
import Trinity.IcarusLowerable.Equivalence

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

/-- W497 bridge: the total and partial t27 evaluators agree on the scalar-struct
    witness. -/
theorem scalar_struct_total_partial_t27_bridge :
  evalModuleFunctionTotal defaultFuel scalarStructEnv scalarStructModule "main" [] =
  evalModuleFunction scalarStructEnv scalarStructModule "main" [] := by
  native_decide

/-- W497 bridge: the total and partial Verilog evaluators agree on the
    scalar-struct witness. -/
theorem scalar_struct_total_partial_v_bridge :
  evalVModuleTotal defaultFuel scalarStructEnv (emitModule scalarStructEnv scalarStructModule) "main" [] =
  evalVModule scalarStructEnv (emitModule scalarStructEnv scalarStructModule) "main" := by
  native_decide

/-- W497 bridge: the total evaluators agree with the partial evaluators on the
    W493 nested-identifier witness. -/
theorem w493_nested_identifier_total_partial_bridge :
  evalModuleFunctionTotal defaultFuel w493NestedIdentifierEnv w493NestedIdentifierModule "get_y" [] =
  evalModuleFunction w493NestedIdentifierEnv w493NestedIdentifierModule "get_y" [] := by
  native_decide

/-- Generic value-preservation theorem for the Icarus-lowerable combinational
    subset.  W501: generalized to any emitted function name, not just `main`.
    The only remaining assumptions are lowerability, combinationality, unique
    function names, the module-level call-context invariant, and the fact that
    the chosen function is not a host-only helper. -/
theorem module_value_equiv_statement (env : Env) (m : Module)
    (h : Module.isLowerable env m)
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
  exact module_value_equiv_proved env m h hunique hcomb hctx fnName fn args hm hhost

/-- Convenience corollary: the original `main`-specific shape of the theorem. -/
theorem module_value_equiv_main_statement (env : Env) (m : Module)
    (h : Module.isLowerable env m)
    (hunique : Module.hasUniqueFunctionNames m)
    (hcomb : Module.isCombinational env m)
    (hctx : Module.callContext env m)
    (mainFn : Function)
    (args : List Value)
    (hm : m.findFunction "main" = some mainFn)
    (hmain : ¬ Env.isHostOnly env mainFn.name) :
    evalModuleFunctionTotal defaultFuel env m "main" args =
    evalVModuleTotal defaultFuel env (emitModule env m) "main" args := by
  exact module_value_equiv_main env m h hunique hcomb hctx mainFn args hm hmain

/-- W501: the non-main-entry witness is lowerable. -/
theorem w501_non_main_entry_lowerable :
  Module.isLowerable w501NonMainEnv w501NonMainModule := by
  native_decide

/-- W501: value preservation for the non-`main` function `get_y`. This exercises
    the generalized `module_value_equiv_statement` directly on an emitted helper
    rather than on the `main` entry point. -/
theorem w501_non_main_entry_value_equiv :
  evalModuleFunctionTotal defaultFuel w501NonMainEnv w501NonMainModule "get_y" [] =
  evalVModuleTotal defaultFuel w501NonMainEnv (emitModule w501NonMainEnv w501NonMainModule) "get_y" [] := by
  have hlowerable : Module.isLowerable w501NonMainEnv w501NonMainModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w501NonMainModule := by
    simp [Module.hasUniqueFunctionNames, w501NonMainModule, w501NonMainMakePt, w501NonMainGetY, w501NonMainMain]
  have hcomb : Module.isCombinational w501NonMainEnv w501NonMainModule := by native_decide
  have hctx : Module.callContext w501NonMainEnv w501NonMainModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w501NonMainEnv, w501NonMainModule, w501NonMainMakePt, w501NonMainGetY, w501NonMainMain]
    all_goals native_decide
  have hfind : w501NonMainModule.findFunction "get_y" = some w501NonMainGetY := by
    simp [Module.findFunction, w501NonMainModule, w501NonMainMakePt, w501NonMainGetY, w501NonMainMain]
  have hhost : ¬ Env.isHostOnly w501NonMainEnv w501NonMainGetY.name := by
    simp [Env.isHostOnly, w501NonMainEnv, w501NonMainGetY]
  exact module_value_equiv_statement w501NonMainEnv w501NonMainModule
    hlowerable hunique hcomb hctx "get_y" w501NonMainGetY [] hfind hhost

/-- W502-A: the non-main-called-from-emitted witness is lowerable. -/
theorem w502_non_main_called_from_emitted_lowerable :
  Module.isLowerable w502NonMainCalledFromEmittedEnv w502NonMainCalledFromEmittedModule := by
  native_decide

/-- W502-A: value preservation for the non-`main` function `caller`, which calls
    another emitted function `helper`. -/
theorem w502_non_main_called_from_emitted_value_equiv :
  evalModuleFunctionTotal defaultFuel w502NonMainCalledFromEmittedEnv w502NonMainCalledFromEmittedModule "caller" [] =
  evalVModuleTotal defaultFuel w502NonMainCalledFromEmittedEnv (emitModule w502NonMainCalledFromEmittedEnv w502NonMainCalledFromEmittedModule) "caller" [] := by
  have hlowerable : Module.isLowerable w502NonMainCalledFromEmittedEnv w502NonMainCalledFromEmittedModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w502NonMainCalledFromEmittedModule := by
    simp [Module.hasUniqueFunctionNames, w502NonMainCalledFromEmittedModule, w502NonMainCalledFromEmittedHelper, w502NonMainCalledFromEmittedCaller, w502NonMainCalledFromEmittedMain]
  have hcomb : Module.isCombinational w502NonMainCalledFromEmittedEnv w502NonMainCalledFromEmittedModule := by native_decide
  have hctx : Module.callContext w502NonMainCalledFromEmittedEnv w502NonMainCalledFromEmittedModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w502NonMainCalledFromEmittedEnv, w502NonMainCalledFromEmittedModule, w502NonMainCalledFromEmittedHelper, w502NonMainCalledFromEmittedCaller, w502NonMainCalledFromEmittedMain]
    all_goals native_decide
  have hfind : w502NonMainCalledFromEmittedModule.findFunction "caller" = some w502NonMainCalledFromEmittedCaller := by
    simp [Module.findFunction, w502NonMainCalledFromEmittedModule, w502NonMainCalledFromEmittedHelper, w502NonMainCalledFromEmittedCaller, w502NonMainCalledFromEmittedMain]
  have hhost : ¬ Env.isHostOnly w502NonMainCalledFromEmittedEnv w502NonMainCalledFromEmittedCaller.name := by
    simp [Env.isHostOnly, w502NonMainCalledFromEmittedEnv, w502NonMainCalledFromEmittedCaller]
  exact module_value_equiv_statement w502NonMainCalledFromEmittedEnv w502NonMainCalledFromEmittedModule
    hlowerable hunique hcomb hctx "caller" w502NonMainCalledFromEmittedCaller [] hfind hhost

/-- W502-B: the chain-leaf witness is lowerable. -/
theorem w502_non_main_chain_leaf_lowerable :
  Module.isLowerable w502NonMainChainLeafEnv w502NonMainChainLeafModule := by
  native_decide

/-- W502-B: value preservation for the non-`main` leaf function `leaf` at the end
    of a three-function emitted chain. -/
theorem w502_non_main_chain_leaf_value_equiv :
  evalModuleFunctionTotal defaultFuel w502NonMainChainLeafEnv w502NonMainChainLeafModule "leaf" [] =
  evalVModuleTotal defaultFuel w502NonMainChainLeafEnv (emitModule w502NonMainChainLeafEnv w502NonMainChainLeafModule) "leaf" [] := by
  have hlowerable : Module.isLowerable w502NonMainChainLeafEnv w502NonMainChainLeafModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w502NonMainChainLeafModule := by
    simp [Module.hasUniqueFunctionNames, w502NonMainChainLeafModule, w502NonMainChainLeafLeaf, w502NonMainChainLeafMid, w502NonMainChainLeafTop, w502NonMainChainLeafMain]
  have hcomb : Module.isCombinational w502NonMainChainLeafEnv w502NonMainChainLeafModule := by native_decide
  have hctx : Module.callContext w502NonMainChainLeafEnv w502NonMainChainLeafModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w502NonMainChainLeafEnv, w502NonMainChainLeafModule, w502NonMainChainLeafLeaf, w502NonMainChainLeafMid, w502NonMainChainLeafTop, w502NonMainChainLeafMain]
    all_goals native_decide
  have hfind : w502NonMainChainLeafModule.findFunction "leaf" = some w502NonMainChainLeafLeaf := by
    simp [Module.findFunction, w502NonMainChainLeafModule, w502NonMainChainLeafLeaf, w502NonMainChainLeafMid, w502NonMainChainLeafTop, w502NonMainChainLeafMain]
  have hhost : ¬ Env.isHostOnly w502NonMainChainLeafEnv w502NonMainChainLeafLeaf.name := by
    simp [Env.isHostOnly, w502NonMainChainLeafEnv, w502NonMainChainLeafLeaf]
  exact module_value_equiv_statement w502NonMainChainLeafEnv w502NonMainChainLeafModule
    hlowerable hunique hcomb hctx "leaf" w502NonMainChainLeafLeaf [] hfind hhost

/-- W502-C: the scalar-struct-param helper witness is lowerable. -/
theorem w502_non_main_helper_struct_param_lowerable :
  Module.isLowerable w502NonMainHelperStructParamEnv w502NonMainHelperStructParamModule := by
  native_decide

/-- W502-C: value preservation for the non-`main` helper `helper` that takes a
    scalar struct parameter. -/
theorem w502_non_main_helper_struct_param_value_equiv :
  evalModuleFunctionTotal defaultFuel w502NonMainHelperStructParamEnv w502NonMainHelperStructParamModule "helper" [⟨32, BitVec.ofInt 32 5⟩] =
  evalVModuleTotal defaultFuel w502NonMainHelperStructParamEnv (emitModule w502NonMainHelperStructParamEnv w502NonMainHelperStructParamModule) "helper" [⟨32, BitVec.ofInt 32 5⟩] := by
  have hlowerable : Module.isLowerable w502NonMainHelperStructParamEnv w502NonMainHelperStructParamModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w502NonMainHelperStructParamModule := by
    simp [Module.hasUniqueFunctionNames, w502NonMainHelperStructParamModule, w502NonMainHelperStructParamHelper, w502NonMainHelperStructParamMain]
  have hcomb : Module.isCombinational w502NonMainHelperStructParamEnv w502NonMainHelperStructParamModule := by native_decide
  have hctx : Module.callContext w502NonMainHelperStructParamEnv w502NonMainHelperStructParamModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w502NonMainHelperStructParamEnv, w502NonMainHelperStructParamModule, w502NonMainHelperStructParamHelper, w502NonMainHelperStructParamMain]
    all_goals native_decide
  have hfind : w502NonMainHelperStructParamModule.findFunction "helper" = some w502NonMainHelperStructParamHelper := by
    simp [Module.findFunction, w502NonMainHelperStructParamModule, w502NonMainHelperStructParamHelper, w502NonMainHelperStructParamMain]
  have hhost : ¬ Env.isHostOnly w502NonMainHelperStructParamEnv w502NonMainHelperStructParamHelper.name := by
    simp [Env.isHostOnly, w502NonMainHelperStructParamEnv, w502NonMainHelperStructParamHelper]
  exact module_value_equiv_statement w502NonMainHelperStructParamEnv w502NonMainHelperStructParamModule
    hlowerable hunique hcomb hctx "helper" w502NonMainHelperStructParamHelper [⟨32, BitVec.ofInt 32 5⟩] hfind hhost

/-- W502-D: the multiple-non-main-entries witness is lowerable. -/
theorem w502_multiple_non_main_entries_lowerable :
  Module.isLowerable w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule := by
  native_decide

/-- W502-D: value preservation for the non-`main` entry point `a`. -/
theorem w502_multiple_non_main_entries_a_value_equiv :
  evalModuleFunctionTotal defaultFuel w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule "a" [] =
  evalVModuleTotal defaultFuel w502MultipleNonMainEntriesEnv (emitModule w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule) "a" [] := by
  have hlowerable : Module.isLowerable w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w502MultipleNonMainEntriesModule := by
    simp [Module.hasUniqueFunctionNames, w502MultipleNonMainEntriesModule, w502MultipleNonMainEntriesA, w502MultipleNonMainEntriesB, w502MultipleNonMainEntriesMain]
  have hcomb : Module.isCombinational w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule := by native_decide
  have hctx : Module.callContext w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w502MultipleNonMainEntriesEnv, w502MultipleNonMainEntriesModule, w502MultipleNonMainEntriesA, w502MultipleNonMainEntriesB, w502MultipleNonMainEntriesMain]
    all_goals native_decide
  have hfind : w502MultipleNonMainEntriesModule.findFunction "a" = some w502MultipleNonMainEntriesA := by
    simp [Module.findFunction, w502MultipleNonMainEntriesModule, w502MultipleNonMainEntriesA, w502MultipleNonMainEntriesB, w502MultipleNonMainEntriesMain]
  have hhost : ¬ Env.isHostOnly w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesA.name := by
    simp [Env.isHostOnly, w502MultipleNonMainEntriesEnv, w502MultipleNonMainEntriesA]
  exact module_value_equiv_statement w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule
    hlowerable hunique hcomb hctx "a" w502MultipleNonMainEntriesA [] hfind hhost

/-- W502-D: value preservation for the non-`main` entry point `b`. -/
theorem w502_multiple_non_main_entries_b_value_equiv :
  evalModuleFunctionTotal defaultFuel w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule "b" [] =
  evalVModuleTotal defaultFuel w502MultipleNonMainEntriesEnv (emitModule w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule) "b" [] := by
  have hlowerable : Module.isLowerable w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w502MultipleNonMainEntriesModule := by
    simp [Module.hasUniqueFunctionNames, w502MultipleNonMainEntriesModule, w502MultipleNonMainEntriesA, w502MultipleNonMainEntriesB, w502MultipleNonMainEntriesMain]
  have hcomb : Module.isCombinational w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule := by native_decide
  have hctx : Module.callContext w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w502MultipleNonMainEntriesEnv, w502MultipleNonMainEntriesModule, w502MultipleNonMainEntriesA, w502MultipleNonMainEntriesB, w502MultipleNonMainEntriesMain]
    all_goals native_decide
  have hfind : w502MultipleNonMainEntriesModule.findFunction "b" = some w502MultipleNonMainEntriesB := by
    simp [Module.findFunction, w502MultipleNonMainEntriesModule, w502MultipleNonMainEntriesA, w502MultipleNonMainEntriesB, w502MultipleNonMainEntriesMain]
  have hhost : ¬ Env.isHostOnly w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesB.name := by
    simp [Env.isHostOnly, w502MultipleNonMainEntriesEnv, w502MultipleNonMainEntriesB]
  exact module_value_equiv_statement w502MultipleNonMainEntriesEnv w502MultipleNonMainEntriesModule
    hlowerable hunique hcomb hctx "b" w502MultipleNonMainEntriesB [] hfind hhost

/-- W503-A: the conditional-return witness is lowerable. -/
theorem w503_if_return_lowerable :
  Module.isLowerable w503IfReturnEnv w503IfReturnModule := by
  native_decide

/-- W503-A: the conditional-return witness is combinational, so the generic
    equivalence theorem applies. -/
theorem w503_if_return_combinational :
  Module.isCombinational w503IfReturnEnv w503IfReturnModule := by
  native_decide

/-- W503-A: value preservation for `pick(true)`. -/
theorem w503_if_return_pick_true_value_equiv :
  evalModuleFunctionTotal defaultFuel w503IfReturnEnv w503IfReturnModule "pick" [⟨1, BitVec.ofNat 1 1⟩] =
  evalVModuleTotal defaultFuel w503IfReturnEnv (emitModule w503IfReturnEnv w503IfReturnModule) "pick" [⟨1, BitVec.ofNat 1 1⟩] := by
  have hlowerable : Module.isLowerable w503IfReturnEnv w503IfReturnModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w503IfReturnModule := by
    simp [Module.hasUniqueFunctionNames, w503IfReturnModule, w503IfReturnPick]
  have hcomb : Module.isCombinational w503IfReturnEnv w503IfReturnModule := by native_decide
  have hctx : Module.callContext w503IfReturnEnv w503IfReturnModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w503IfReturnEnv, w503IfReturnModule, w503IfReturnPick]
  have hfind : w503IfReturnModule.findFunction "pick" = some w503IfReturnPick := by
    simp [Module.findFunction, w503IfReturnModule, w503IfReturnPick]
  have hhost : ¬ Env.isHostOnly w503IfReturnEnv w503IfReturnPick.name := by
    simp [Env.isHostOnly, w503IfReturnEnv, w503IfReturnPick]
  exact module_value_equiv_statement w503IfReturnEnv w503IfReturnModule
    hlowerable hunique hcomb hctx "pick" w503IfReturnPick [⟨1, BitVec.ofNat 1 1⟩] hfind hhost

/-- W503-A: value preservation for `pick(false)`. -/
theorem w503_if_return_pick_false_value_equiv :
  evalModuleFunctionTotal defaultFuel w503IfReturnEnv w503IfReturnModule "pick" [⟨1, BitVec.ofNat 1 0⟩] =
  evalVModuleTotal defaultFuel w503IfReturnEnv (emitModule w503IfReturnEnv w503IfReturnModule) "pick" [⟨1, BitVec.ofNat 1 0⟩] := by
  have hlowerable : Module.isLowerable w503IfReturnEnv w503IfReturnModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w503IfReturnModule := by
    simp [Module.hasUniqueFunctionNames, w503IfReturnModule, w503IfReturnPick]
  have hcomb : Module.isCombinational w503IfReturnEnv w503IfReturnModule := by native_decide
  have hctx : Module.callContext w503IfReturnEnv w503IfReturnModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w503IfReturnEnv, w503IfReturnModule, w503IfReturnPick]
  have hfind : w503IfReturnModule.findFunction "pick" = some w503IfReturnPick := by
    simp [Module.findFunction, w503IfReturnModule, w503IfReturnPick]
  have hhost : ¬ Env.isHostOnly w503IfReturnEnv w503IfReturnPick.name := by
    simp [Env.isHostOnly, w503IfReturnEnv, w503IfReturnPick]
  exact module_value_equiv_statement w503IfReturnEnv w503IfReturnModule
    hlowerable hunique hcomb hctx "pick" w503IfReturnPick [⟨1, BitVec.ofNat 1 0⟩] hfind hhost

/-- W503-B: the bounded for-loop accumulator witness is lowerable. -/
theorem w503_for_accumulator_lowerable :
  Module.isLowerable w503ForAccumulatorEnv w503ForAccumulatorModule := by
  native_decide

/-- W503-B: direct value preservation for `sum_three`.  The generic theorem does
    not apply because the body contains a `forLoop`, but the total evaluators on
    both sides agree by computation. -/
theorem w503_for_accumulator_sum_three_value_equiv :
  evalModuleFunctionTotal defaultFuel w503ForAccumulatorEnv w503ForAccumulatorModule "sum_three" [] =
  evalVModuleTotal defaultFuel w503ForAccumulatorEnv (emitModule w503ForAccumulatorEnv w503ForAccumulatorModule) "sum_three" [] := by
  native_decide

end Trinity.IcarusLowerable
