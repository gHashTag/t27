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

/-- W503-B: direct value preservation for `sum_three`.  The total evaluators on
    both sides agree by computation. -/
theorem w503_for_accumulator_sum_three_value_equiv :
  evalModuleFunctionTotal defaultFuel w503ForAccumulatorEnv w503ForAccumulatorModule "sum_three" [] =
  evalVModuleTotal defaultFuel w503ForAccumulatorEnv (emitModule w503ForAccumulatorEnv w503ForAccumulatorModule) "sum_three" [] := by
  native_decide

/-- W504-A: the bounded for-loop with parameter witness is lowerable. -/
theorem w504_for_sum_lowerable :
  Module.isLowerable w504ForSumEnv w504ForSumModule := by
  native_decide

/-- W504-A: the bounded for-loop with parameter witness is sequential. -/
theorem w504_for_sum_sequential :
  Module.isSequential w504ForSumEnv w504ForSumModule := by
  native_decide

/-- W504-A: generic sequential value preservation for `sum_n(5)`.  This is the
    first witness whose equivalence follows directly from `module_value_equiv_proved_sequential`
    rather than from a native-decide computation of the full loop. -/
theorem w504_for_sum_value_equiv :
  evalModuleFunctionTotal defaultFuel w504ForSumEnv w504ForSumModule "sum_n" [⟨32, BitVec.ofNat 32 5⟩] =
  evalVModuleTotal defaultFuel w504ForSumEnv (emitModule w504ForSumEnv w504ForSumModule) "sum_n" [⟨32, BitVec.ofNat 32 5⟩] := by
  have hlowerable : Module.isLowerable w504ForSumEnv w504ForSumModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w504ForSumModule := by
    simp [Module.hasUniqueFunctionNames, w504ForSumModule, w504ForSumSumN]
  have hseq : Module.isSequential w504ForSumEnv w504ForSumModule := by native_decide
  have hctx : Module.callContext w504ForSumEnv w504ForSumModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w504ForSumEnv, w504ForSumModule, w504ForSumSumN]
  have hfind : w504ForSumModule.findFunction "sum_n" = some w504ForSumSumN := by
    simp [Module.findFunction, w504ForSumModule, w504ForSumSumN]
  have hhost : ¬ Env.isHostOnly w504ForSumEnv w504ForSumSumN.name := by
    simp [Env.isHostOnly, w504ForSumEnv, w504ForSumSumN]
  exact module_value_equiv_proved_sequential w504ForSumEnv w504ForSumModule
    hlowerable hunique hseq hctx "sum_n" w504ForSumSumN [⟨32, BitVec.ofNat 32 5⟩] hfind hhost

/- W505 theorems: adversarial sequential witnesses. -/

/-- W505-A: nested `ifThenElse` witness is lowerable. -/
theorem w505_nested_if_lowerable :
  Module.isLowerable w505NestedIfEnv w505NestedIfModule := by
  native_decide

/-- W505-A: nested if-return witness is sequential. -/
theorem w505_nested_if_sequential :
  Module.isSequential w505NestedIfEnv w505NestedIfModule := by
  native_decide

/-- W505-A: value preservation for `classify(9)`. -/
theorem w505_nested_if_value_equiv :
  evalModuleFunctionTotal defaultFuel w505NestedIfEnv w505NestedIfModule "classify" [⟨32, BitVec.ofNat 32 9⟩] =
  evalVModuleTotal defaultFuel w505NestedIfEnv (emitModule w505NestedIfEnv w505NestedIfModule) "classify" [⟨32, BitVec.ofNat 32 9⟩] := by
  have hlowerable : Module.isLowerable w505NestedIfEnv w505NestedIfModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w505NestedIfModule := by
    simp [Module.hasUniqueFunctionNames, w505NestedIfModule, w505NestedIfClassify]
  have hseq : Module.isSequential w505NestedIfEnv w505NestedIfModule := by native_decide
  have hctx : Module.callContext w505NestedIfEnv w505NestedIfModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w505NestedIfEnv, w505NestedIfModule, w505NestedIfClassify]
  have hfind : w505NestedIfModule.findFunction "classify" = some w505NestedIfClassify := by
    simp [Module.findFunction, w505NestedIfModule, w505NestedIfClassify]
  have hhost : ¬ Env.isHostOnly w505NestedIfEnv w505NestedIfClassify.name := by
    simp [Env.isHostOnly, w505NestedIfEnv, w505NestedIfClassify]
  exact module_value_equiv_proved_sequential w505NestedIfEnv w505NestedIfModule
    hlowerable hunique hseq hctx "classify" w505NestedIfClassify [⟨32, BitVec.ofNat 32 9⟩] hfind hhost

/-- W505-B: `ifThenElse` inside a bounded `forLoop` is lowerable. -/
theorem w505_if_in_for_lowerable :
  Module.isLowerable w505IfInForEnv w505IfInForModule := by
  native_decide

/-- W505-B: conditional accumulation inside a loop is sequential. -/
theorem w505_if_in_for_sequential :
  Module.isSequential w505IfInForEnv w505IfInForModule := by
  native_decide

/-- W505-B: value preservation for `conditional_sum(3,2,5)`. -/
theorem w505_if_in_for_value_equiv :
  evalModuleFunctionTotal defaultFuel w505IfInForEnv w505IfInForModule "conditional_sum"
    [⟨32, BitVec.ofNat 32 3⟩, ⟨32, BitVec.ofNat 32 2⟩, ⟨32, BitVec.ofNat 32 5⟩] =
  evalVModuleTotal defaultFuel w505IfInForEnv (emitModule w505IfInForEnv w505IfInForModule) "conditional_sum"
    [⟨32, BitVec.ofNat 32 3⟩, ⟨32, BitVec.ofNat 32 2⟩, ⟨32, BitVec.ofNat 32 5⟩] := by
  have hlowerable : Module.isLowerable w505IfInForEnv w505IfInForModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w505IfInForModule := by
    simp [Module.hasUniqueFunctionNames, w505IfInForModule, w505IfInForConditionalSum]
  have hseq : Module.isSequential w505IfInForEnv w505IfInForModule := by native_decide
  have hctx : Module.callContext w505IfInForEnv w505IfInForModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w505IfInForEnv, w505IfInForModule, w505IfInForConditionalSum]
  have hfind : w505IfInForModule.findFunction "conditional_sum" = some w505IfInForConditionalSum := by
    simp [Module.findFunction, w505IfInForModule, w505IfInForConditionalSum]
  have hhost : ¬ Env.isHostOnly w505IfInForEnv w505IfInForConditionalSum.name := by
    simp [Env.isHostOnly, w505IfInForEnv, w505IfInForConditionalSum]
  exact module_value_equiv_proved_sequential w505IfInForEnv w505IfInForModule
    hlowerable hunique hseq hctx "conditional_sum" w505IfInForConditionalSum
    [⟨32, BitVec.ofNat 32 3⟩, ⟨32, BitVec.ofNat 32 2⟩, ⟨32, BitVec.ofNat 32 5⟩] hfind hhost

/-- W505-C: bounded `forLoop` with a parameter range is lowerable. -/
theorem w505_for_var_range_lowerable :
  Module.isLowerable w505ForVarRangeEnv w505ForVarRangeModule := by
  native_decide

/-- W505-C: parameter-range loop is sequential. -/
theorem w505_for_var_range_sequential :
  Module.isSequential w505ForVarRangeEnv w505ForVarRangeModule := by
  native_decide

/-- W505-C: value preservation for `sum_range(5)`. -/
theorem w505_for_var_range_value_equiv :
  evalModuleFunctionTotal defaultFuel w505ForVarRangeEnv w505ForVarRangeModule "sum_range" [⟨32, BitVec.ofNat 32 5⟩] =
  evalVModuleTotal defaultFuel w505ForVarRangeEnv (emitModule w505ForVarRangeEnv w505ForVarRangeModule) "sum_range" [⟨32, BitVec.ofNat 32 5⟩] := by
  have hlowerable : Module.isLowerable w505ForVarRangeEnv w505ForVarRangeModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w505ForVarRangeModule := by
    simp [Module.hasUniqueFunctionNames, w505ForVarRangeModule, w505ForVarRangeSumRange]
  have hseq : Module.isSequential w505ForVarRangeEnv w505ForVarRangeModule := by native_decide
  have hctx : Module.callContext w505ForVarRangeEnv w505ForVarRangeModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w505ForVarRangeEnv, w505ForVarRangeModule, w505ForVarRangeSumRange]
  have hfind : w505ForVarRangeModule.findFunction "sum_range" = some w505ForVarRangeSumRange := by
    simp [Module.findFunction, w505ForVarRangeModule, w505ForVarRangeSumRange]
  have hhost : ¬ Env.isHostOnly w505ForVarRangeEnv w505ForVarRangeSumRange.name := by
    simp [Env.isHostOnly, w505ForVarRangeEnv, w505ForVarRangeSumRange]
  exact module_value_equiv_proved_sequential w505ForVarRangeEnv w505ForVarRangeModule
    hlowerable hunique hseq hctx "sum_range" w505ForVarRangeSumRange [⟨32, BitVec.ofNat 32 5⟩] hfind hhost

/-- W505-D: bounded `forLoop` returning a computed value is lowerable. -/
theorem w505_for_return_lowerable :
  Module.isLowerable w505ForReturnEnv w505ForReturnModule := by
  native_decide

/-- W505-D: factorial-style loop is sequential. -/
theorem w505_for_return_sequential :
  Module.isSequential w505ForReturnEnv w505ForReturnModule := by
  native_decide

/-- W505-D: value preservation for `factorial(5)`. -/
theorem w505_for_return_value_equiv :
  evalModuleFunctionTotal defaultFuel w505ForReturnEnv w505ForReturnModule "factorial" [⟨32, BitVec.ofNat 32 5⟩] =
  evalVModuleTotal defaultFuel w505ForReturnEnv (emitModule w505ForReturnEnv w505ForReturnModule) "factorial" [⟨32, BitVec.ofNat 32 5⟩] := by
  have hlowerable : Module.isLowerable w505ForReturnEnv w505ForReturnModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w505ForReturnModule := by
    simp [Module.hasUniqueFunctionNames, w505ForReturnModule, w505ForReturnFactorial]
  have hseq : Module.isSequential w505ForReturnEnv w505ForReturnModule := by native_decide
  have hctx : Module.callContext w505ForReturnEnv w505ForReturnModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w505ForReturnEnv, w505ForReturnModule, w505ForReturnFactorial]
  have hfind : w505ForReturnModule.findFunction "factorial" = some w505ForReturnFactorial := by
    simp [Module.findFunction, w505ForReturnModule, w505ForReturnFactorial]
  have hhost : ¬ Env.isHostOnly w505ForReturnEnv w505ForReturnFactorial.name := by
    simp [Env.isHostOnly, w505ForReturnEnv, w505ForReturnFactorial]
  exact module_value_equiv_proved_sequential w505ForReturnEnv w505ForReturnModule
    hlowerable hunique hseq hctx "factorial" w505ForReturnFactorial [⟨32, BitVec.ofNat 32 5⟩] hfind hhost

/-- W505-E: bounded `forLoop` with a local body variable is lowerable. -/
theorem w505_for_local_var_init_lowerable :
  Module.isLowerable w505ForLocalVarInitEnv w505ForLocalVarInitModule := by
  native_decide

/-- W505-E: loop with local body variable is sequential. -/
theorem w505_for_local_var_init_sequential :
  Module.isSequential w505ForLocalVarInitEnv w505ForLocalVarInitModule := by
  native_decide

/-- W505-E: value preservation for `fill_init(4)`. -/
theorem w505_for_local_var_init_value_equiv :
  evalModuleFunctionTotal defaultFuel w505ForLocalVarInitEnv w505ForLocalVarInitModule "fill_init" [⟨32, BitVec.ofNat 32 4⟩] =
  evalVModuleTotal defaultFuel w505ForLocalVarInitEnv (emitModule w505ForLocalVarInitEnv w505ForLocalVarInitModule) "fill_init" [⟨32, BitVec.ofNat 32 4⟩] := by
  have hlowerable : Module.isLowerable w505ForLocalVarInitEnv w505ForLocalVarInitModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w505ForLocalVarInitModule := by
    simp [Module.hasUniqueFunctionNames, w505ForLocalVarInitModule, w505ForLocalVarInitFillInit]
  have hseq : Module.isSequential w505ForLocalVarInitEnv w505ForLocalVarInitModule := by native_decide
  have hctx : Module.callContext w505ForLocalVarInitEnv w505ForLocalVarInitModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w505ForLocalVarInitEnv, w505ForLocalVarInitModule, w505ForLocalVarInitFillInit]
  have hfind : w505ForLocalVarInitModule.findFunction "fill_init" = some w505ForLocalVarInitFillInit := by
    simp [Module.findFunction, w505ForLocalVarInitModule, w505ForLocalVarInitFillInit]
  have hhost : ¬ Env.isHostOnly w505ForLocalVarInitEnv w505ForLocalVarInitFillInit.name := by
    simp [Env.isHostOnly, w505ForLocalVarInitEnv, w505ForLocalVarInitFillInit]
  exact module_value_equiv_proved_sequential w505ForLocalVarInitEnv w505ForLocalVarInitModule
    hlowerable hunique hseq hctx "fill_init" w505ForLocalVarInitFillInit [⟨32, BitVec.ofNat 32 4⟩] hfind hhost


/-- W506: statement-level scalar switch dispatch is lowerable. -/
theorem w506_switch_lowerable :
  Module.isLowerable w506SwitchEnv w506SwitchModule := by
  native_decide

/-- W506: statement-level scalar switch dispatch is sequential. -/
theorem w506_switch_sequential :
  Module.isSequential w506SwitchEnv w506SwitchModule := by
  native_decide

/-- W506: value preservation for `main(1)` via the generic sequential
    equivalence theorem.  The source `switch` and the emitted Verilog `case`
    compute the same `u32` result. -/
theorem w506_switch_value_equiv :
  evalModuleFunctionTotal defaultFuel w506SwitchEnv w506SwitchModule "main"
    [⟨32, BitVec.ofNat 32 1⟩] =
  evalVModuleTotal defaultFuel w506SwitchEnv (emitModule w506SwitchEnv w506SwitchModule) "main"
    [⟨32, BitVec.ofNat 32 1⟩] := by
  have hlowerable : Module.isLowerable w506SwitchEnv w506SwitchModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w506SwitchModule := by
    simp [Module.hasUniqueFunctionNames, w506SwitchModule, w506SwitchMain]
  have hseq : Module.isSequential w506SwitchEnv w506SwitchModule := by native_decide
  have hctx : Module.callContext w506SwitchEnv w506SwitchModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w506SwitchEnv, w506SwitchModule, w506SwitchMain]
  have hfind : w506SwitchModule.findFunction "main" = some w506SwitchMain := by
    simp [Module.findFunction, w506SwitchModule, w506SwitchMain]
  have hhost : ¬ Env.isHostOnly w506SwitchEnv w506SwitchMain.name := by
    simp [Env.isHostOnly, w506SwitchEnv, w506SwitchMain]
  exact module_value_equiv_proved_sequential w506SwitchEnv w506SwitchModule
    hlowerable hunique hseq hctx "main" w506SwitchMain [⟨32, BitVec.ofNat 32 1⟩] hfind hhost

/- W507 theorems: bounded `while` loops. -/

/-- W507-A: the bounded while-loop counter witness is lowerable. -/
theorem w507_while_counter_lowerable :
  Module.isLowerable w507WhileCounterEnv w507WhileCounterModule := by
  native_decide

/-- W507-A: the bounded while-loop counter witness is sequential. -/
theorem w507_while_counter_sequential :
  Module.isSequential w507WhileCounterEnv w507WhileCounterModule := by
  native_decide

/-- W507-A: value preservation for `count_to(3)`. -/
theorem w507_while_counter_value_equiv :
  evalModuleFunctionTotal defaultFuel w507WhileCounterEnv w507WhileCounterModule "count_to"
    [⟨32, BitVec.ofNat 32 3⟩] =
  evalVModuleTotal defaultFuel w507WhileCounterEnv (emitModule w507WhileCounterEnv w507WhileCounterModule) "count_to"
    [⟨32, BitVec.ofNat 32 3⟩] := by
  have hlowerable : Module.isLowerable w507WhileCounterEnv w507WhileCounterModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w507WhileCounterModule := by
    simp [Module.hasUniqueFunctionNames, w507WhileCounterModule, w507WhileCounterCountTo]
  have hseq : Module.isSequential w507WhileCounterEnv w507WhileCounterModule := by native_decide
  have hctx : Module.callContext w507WhileCounterEnv w507WhileCounterModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w507WhileCounterEnv, w507WhileCounterModule, w507WhileCounterCountTo]
  have hfind : w507WhileCounterModule.findFunction "count_to" = some w507WhileCounterCountTo := by
    simp [Module.findFunction, w507WhileCounterModule, w507WhileCounterCountTo]
  have hhost : ¬ Env.isHostOnly w507WhileCounterEnv w507WhileCounterCountTo.name := by
    simp [Env.isHostOnly, w507WhileCounterEnv, w507WhileCounterCountTo]
  exact module_value_equiv_proved_sequential w507WhileCounterEnv w507WhileCounterModule
    hlowerable hunique hseq hctx "count_to" w507WhileCounterCountTo [⟨32, BitVec.ofNat 32 3⟩] hfind hhost

/-- W507-B: the while-loop linear-search witness is lowerable. -/
theorem w507_while_search_lowerable :
  Module.isLowerable w507WhileSearchEnv w507WhileSearchModule := by
  native_decide

/-- W507-B: the while-loop linear-search witness is sequential. -/
theorem w507_while_search_sequential :
  Module.isSequential w507WhileSearchEnv w507WhileSearchModule := by
  native_decide

/-- W507-B: value preservation for `find_index(1)` (returns index 2). -/
theorem w507_while_search_value_equiv :
  evalModuleFunctionTotal defaultFuel w507WhileSearchEnv w507WhileSearchModule "find_index"
    [⟨32, BitVec.ofNat 32 1⟩] =
  evalVModuleTotal defaultFuel w507WhileSearchEnv (emitModule w507WhileSearchEnv w507WhileSearchModule) "find_index"
    [⟨32, BitVec.ofNat 32 1⟩] := by
  have hlowerable : Module.isLowerable w507WhileSearchEnv w507WhileSearchModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w507WhileSearchModule := by
    simp [Module.hasUniqueFunctionNames, w507WhileSearchModule, w507WhileSearchFindIndex]
  have hseq : Module.isSequential w507WhileSearchEnv w507WhileSearchModule := by native_decide
  have hctx : Module.callContext w507WhileSearchEnv w507WhileSearchModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w507WhileSearchEnv, w507WhileSearchModule, w507WhileSearchFindIndex]
  have hfind : w507WhileSearchModule.findFunction "find_index" = some w507WhileSearchFindIndex := by
    simp [Module.findFunction, w507WhileSearchModule, w507WhileSearchFindIndex]
  have hhost : ¬ Env.isHostOnly w507WhileSearchEnv w507WhileSearchFindIndex.name := by
    simp [Env.isHostOnly, w507WhileSearchEnv, w507WhileSearchFindIndex]
  exact module_value_equiv_proved_sequential w507WhileSearchEnv w507WhileSearchModule
    hlowerable hunique hseq hctx "find_index" w507WhileSearchFindIndex [⟨32, BitVec.ofNat 32 1⟩] hfind hhost

/-- W507-C: the nested while-inside-for witness is lowerable. -/
theorem w507_while_nested_lowerable :
  Module.isLowerable w507WhileNestedEnv w507WhileNestedModule := by
  native_decide

/-- W507-C: the nested while-inside-for witness is sequential. -/
theorem w507_while_nested_sequential :
  Module.isSequential w507WhileNestedEnv w507WhileNestedModule := by
  native_decide

/-- W507-C: value preservation for `nested_sum()` (returns 6). -/
theorem w507_while_nested_value_equiv :
  evalModuleFunctionTotal defaultFuel w507WhileNestedEnv w507WhileNestedModule "nested_sum" [] =
  evalVModuleTotal defaultFuel w507WhileNestedEnv (emitModule w507WhileNestedEnv w507WhileNestedModule) "nested_sum" [] := by
  have hlowerable : Module.isLowerable w507WhileNestedEnv w507WhileNestedModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w507WhileNestedModule := by
    simp [Module.hasUniqueFunctionNames, w507WhileNestedModule, w507WhileNestedNestedSum]
  have hseq : Module.isSequential w507WhileNestedEnv w507WhileNestedModule := by native_decide
  have hctx : Module.callContext w507WhileNestedEnv w507WhileNestedModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w507WhileNestedEnv, w507WhileNestedModule, w507WhileNestedNestedSum]
  have hfind : w507WhileNestedModule.findFunction "nested_sum" = some w507WhileNestedNestedSum := by
    simp [Module.findFunction, w507WhileNestedModule, w507WhileNestedNestedSum]
  have hhost : ¬ Env.isHostOnly w507WhileNestedEnv w507WhileNestedNestedSum.name := by
    simp [Env.isHostOnly, w507WhileNestedEnv, w507WhileNestedNestedSum]
  exact module_value_equiv_proved_sequential w507WhileNestedEnv w507WhileNestedModule
    hlowerable hunique hseq hctx "nested_sum" w507WhileNestedNestedSum [] hfind hhost

end Trinity.IcarusLowerable