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

/- W508 theorems: `break`/`continue` in bounded loops. -/

/-- W508-A: the while-loop break witness is sequential. -/
theorem w508_break_search_sequential :
  Module.isSequential w508BreakSearchEnv w508BreakSearchModule := by
  native_decide

/-- W508-A: value preservation for `find_target(1)` (returns index 2). -/
theorem w508_break_search_value_equiv :
  evalModuleFunctionTotal defaultFuel w508BreakSearchEnv w508BreakSearchModule "find_target"
    [⟨32, BitVec.ofNat 32 1⟩] =
  evalVModuleTotal defaultFuel w508BreakSearchEnv (emitModule w508BreakSearchEnv w508BreakSearchModule) "find_target"
    [⟨32, BitVec.ofNat 32 1⟩] := by
  have hlowerable : Module.isLowerable w508BreakSearchEnv w508BreakSearchModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w508BreakSearchModule := by
    simp [Module.hasUniqueFunctionNames, w508BreakSearchModule, w508BreakSearchFindTarget]
  have hseq : Module.isSequential w508BreakSearchEnv w508BreakSearchModule := by native_decide
  have hctx : Module.callContext w508BreakSearchEnv w508BreakSearchModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w508BreakSearchEnv, w508BreakSearchModule, w508BreakSearchFindTarget]
  have hfind : w508BreakSearchModule.findFunction "find_target" = some w508BreakSearchFindTarget := by
    simp [Module.findFunction, w508BreakSearchModule, w508BreakSearchFindTarget]
  have hhost : ¬ Env.isHostOnly w508BreakSearchEnv w508BreakSearchFindTarget.name := by
    simp [Env.isHostOnly, w508BreakSearchEnv, w508BreakSearchFindTarget]
  exact module_value_equiv_proved_sequential w508BreakSearchEnv w508BreakSearchModule
    hlowerable hunique hseq hctx "find_target" w508BreakSearchFindTarget [⟨32, BitVec.ofNat 32 1⟩] hfind hhost

/-- W508-B: the for-loop continue witness is sequential. -/
theorem w508_continue_sum_sequential :
  Module.isSequential w508ContinueSumEnv w508ContinueSumModule := by
  native_decide

/-- W508-B: value preservation for `sum_odd()` (returns 25). -/
theorem w508_continue_sum_value_equiv :
  evalModuleFunctionTotal defaultFuel w508ContinueSumEnv w508ContinueSumModule "sum_odd" [] =
  evalVModuleTotal defaultFuel w508ContinueSumEnv (emitModule w508ContinueSumEnv w508ContinueSumModule) "sum_odd" [] := by
  have hlowerable : Module.isLowerable w508ContinueSumEnv w508ContinueSumModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w508ContinueSumModule := by
    simp [Module.hasUniqueFunctionNames, w508ContinueSumModule, w508ContinueSumSumOdd]
  have hseq : Module.isSequential w508ContinueSumEnv w508ContinueSumModule := by native_decide
  have hctx : Module.callContext w508ContinueSumEnv w508ContinueSumModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w508ContinueSumEnv, w508ContinueSumModule, w508ContinueSumSumOdd]
  have hfind : w508ContinueSumModule.findFunction "sum_odd" = some w508ContinueSumSumOdd := by
    simp [Module.findFunction, w508ContinueSumModule, w508ContinueSumSumOdd]
  have hhost : ¬ Env.isHostOnly w508ContinueSumEnv w508ContinueSumSumOdd.name := by
    simp [Env.isHostOnly, w508ContinueSumEnv, w508ContinueSumSumOdd]
  exact module_value_equiv_proved_sequential w508ContinueSumEnv w508ContinueSumModule
    hlowerable hunique hseq hctx "sum_odd" w508ContinueSumSumOdd [] hfind hhost

/-- W508-C: the nested break witness is sequential. -/
theorem w508_break_nested_sequential :
  Module.isSequential w508BreakNestedEnv w508BreakNestedModule := by
  native_decide

/-- W508-C: value preservation for `find_pair()` (returns 3). -/
theorem w508_break_nested_value_equiv :
  evalModuleFunctionTotal defaultFuel w508BreakNestedEnv w508BreakNestedModule "find_pair" [] =
  evalVModuleTotal defaultFuel w508BreakNestedEnv (emitModule w508BreakNestedEnv w508BreakNestedModule) "find_pair" [] := by
  have hlowerable : Module.isLowerable w508BreakNestedEnv w508BreakNestedModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w508BreakNestedModule := by
    simp [Module.hasUniqueFunctionNames, w508BreakNestedModule, w508BreakNestedFindPair]
  have hseq : Module.isSequential w508BreakNestedEnv w508BreakNestedModule := by native_decide
  have hctx : Module.callContext w508BreakNestedEnv w508BreakNestedModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w508BreakNestedEnv, w508BreakNestedModule, w508BreakNestedFindPair]
  have hfind : w508BreakNestedModule.findFunction "find_pair" = some w508BreakNestedFindPair := by
    simp [Module.findFunction, w508BreakNestedModule, w508BreakNestedFindPair]
  have hhost : ¬ Env.isHostOnly w508BreakNestedEnv w508BreakNestedFindPair.name := by
    simp [Env.isHostOnly, w508BreakNestedEnv, w508BreakNestedFindPair]
  exact module_value_equiv_proved_sequential w508BreakNestedEnv w508BreakNestedModule
    hlowerable hunique hseq hctx "find_pair" w508BreakNestedFindPair [] hfind hhost

/-- W508-D: environment for an invalid `break` outside any loop. -/
def w508BreakOutsideLoopEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["bad"]
}

/-- W508-D: function with a top-level `break`, which is not lowerable. -/
def w508BreakOutsideLoopBad : Function := {
  name := "bad",
  params := [],
  ret := some .u32,
  body := [.break]
}

/-- W508-D: module containing the out-of-loop break. -/
def w508BreakOutsideLoopModule : Module := {
  name := "w508_break_outside_loop",
  imports := [],
  globals := [],
  functions := [w508BreakOutsideLoopBad],
  tests := [],
  benches := []
}

/-- W508-D: a `break` outside a loop makes the module not lowerable. -/
theorem w508_break_outside_loop_not_lowerable :
  ¬ Module.isLowerable w508BreakOutsideLoopEnv w508BreakOutsideLoopModule := by
  native_decide

/- W509 theorems: direct lowering of array-typed struct fields. -/

/-- W509: `Module.callContext` is decidable for concrete modules, so the
    witness proofs can discharge it with `native_decide`. -/
instance w509_callContext_decidable (env m) : Decidable (Module.callContext env m) := by
  unfold Module.callContext Stmt.callContextList Stmt.callContext
  infer_instance

/-- W509-A: the direct-read witness is sequential. -/
theorem w509_array_field_direct_sequential :
  Module.isSequential w509ArrayFieldDirectEnv w509ArrayFieldDirectModule := by
  native_decide

/-- W509-A: value preservation for `sum_local_pt()` (returns 13). -/
theorem w509_array_field_direct_sum_local_pt_value_equiv :
  evalModuleFunctionTotal defaultFuel w509ArrayFieldDirectEnv w509ArrayFieldDirectModule "sum_local_pt" [] =
  evalVModuleTotal defaultFuel w509ArrayFieldDirectEnv (emitModule w509ArrayFieldDirectEnv w509ArrayFieldDirectModule) "sum_local_pt" [] := by
  have hlowerable : Module.isLowerable w509ArrayFieldDirectEnv w509ArrayFieldDirectModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w509ArrayFieldDirectModule := by
    simp [Module.hasUniqueFunctionNames, w509ArrayFieldDirectModule, w509ArrayFieldDirectSumLocalPt, w509ArrayFieldDirectSum2DLocalPt]
  have hseq : Module.isSequential w509ArrayFieldDirectEnv w509ArrayFieldDirectModule := by native_decide
  have hctx : Module.callContext w509ArrayFieldDirectEnv w509ArrayFieldDirectModule := by native_decide
  have hfind : w509ArrayFieldDirectModule.findFunction "sum_local_pt" = some w509ArrayFieldDirectSumLocalPt := by
    simp [Module.findFunction, w509ArrayFieldDirectModule, w509ArrayFieldDirectSumLocalPt, w509ArrayFieldDirectSum2DLocalPt]
  have hhost : ¬ Env.isHostOnly w509ArrayFieldDirectEnv w509ArrayFieldDirectSumLocalPt.name := by
    simp [Env.isHostOnly, w509ArrayFieldDirectEnv, w509ArrayFieldDirectSumLocalPt]
  exact module_value_equiv_proved_sequential w509ArrayFieldDirectEnv w509ArrayFieldDirectModule
    hlowerable hunique hseq hctx "sum_local_pt" w509ArrayFieldDirectSumLocalPt [] hfind hhost

/-- W509-A: value preservation for `sum_2d_local_pt()` (returns 10). -/
theorem w509_array_field_direct_sum_2d_local_pt_value_equiv :
  evalModuleFunctionTotal defaultFuel w509ArrayFieldDirectEnv w509ArrayFieldDirectModule "sum_2d_local_pt" [] =
  evalVModuleTotal defaultFuel w509ArrayFieldDirectEnv (emitModule w509ArrayFieldDirectEnv w509ArrayFieldDirectModule) "sum_2d_local_pt" [] := by
  have hlowerable : Module.isLowerable w509ArrayFieldDirectEnv w509ArrayFieldDirectModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w509ArrayFieldDirectModule := by
    simp [Module.hasUniqueFunctionNames, w509ArrayFieldDirectModule, w509ArrayFieldDirectSumLocalPt, w509ArrayFieldDirectSum2DLocalPt]
  have hseq : Module.isSequential w509ArrayFieldDirectEnv w509ArrayFieldDirectModule := by native_decide
  have hctx : Module.callContext w509ArrayFieldDirectEnv w509ArrayFieldDirectModule := by native_decide
  have hfind : w509ArrayFieldDirectModule.findFunction "sum_2d_local_pt" = some w509ArrayFieldDirectSum2DLocalPt := by
    simp [Module.findFunction, w509ArrayFieldDirectModule, w509ArrayFieldDirectSumLocalPt, w509ArrayFieldDirectSum2DLocalPt]
  have hhost : ¬ Env.isHostOnly w509ArrayFieldDirectEnv w509ArrayFieldDirectSum2DLocalPt.name := by
    simp [Env.isHostOnly, w509ArrayFieldDirectEnv, w509ArrayFieldDirectSum2DLocalPt]
  exact module_value_equiv_proved_sequential w509ArrayFieldDirectEnv w509ArrayFieldDirectModule
    hlowerable hunique hseq hctx "sum_2d_local_pt" w509ArrayFieldDirectSum2DLocalPt [] hfind hhost

/-- W509-B: the packed-param witness is sequential. -/
theorem w509_array_field_param_sequential :
  Module.isSequential w509ArrayFieldParamEnv w509ArrayFieldParamModule := by
  native_decide

/-- W509-B: value preservation for `sum_pt(Pt{[1,2,3],7})` (returns 13). -/
theorem w509_array_field_param_sum_pt_value_equiv :
  evalModuleFunctionTotal defaultFuel w509ArrayFieldParamEnv w509ArrayFieldParamModule "sum_pt"
    [⟨32, BitVec.ofNat 32 0x01020307⟩] =
  evalVModuleTotal defaultFuel w509ArrayFieldParamEnv (emitModule w509ArrayFieldParamEnv w509ArrayFieldParamModule) "sum_pt"
    [⟨32, BitVec.ofNat 32 0x01020307⟩] := by
  have hlowerable : Module.isLowerable w509ArrayFieldParamEnv w509ArrayFieldParamModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w509ArrayFieldParamModule := by
    simp [Module.hasUniqueFunctionNames, w509ArrayFieldParamModule, w509ArrayFieldParamSumPt, w509ArrayFieldParamSum2DPt]
  have hseq : Module.isSequential w509ArrayFieldParamEnv w509ArrayFieldParamModule := by native_decide
  have hctx : Module.callContext w509ArrayFieldParamEnv w509ArrayFieldParamModule := by native_decide
  have hfind : w509ArrayFieldParamModule.findFunction "sum_pt" = some w509ArrayFieldParamSumPt := by
    simp [Module.findFunction, w509ArrayFieldParamModule, w509ArrayFieldParamSumPt, w509ArrayFieldParamSum2DPt]
  have hhost : ¬ Env.isHostOnly w509ArrayFieldParamEnv w509ArrayFieldParamSumPt.name := by
    simp [Env.isHostOnly, w509ArrayFieldParamEnv, w509ArrayFieldParamSumPt]
  exact module_value_equiv_proved_sequential w509ArrayFieldParamEnv w509ArrayFieldParamModule
    hlowerable hunique hseq hctx "sum_pt" w509ArrayFieldParamSumPt [⟨32, BitVec.ofNat 32 0x01020307⟩] hfind hhost

/-- W509-B: value preservation for `sum_2d_pt(Pt2{[1..6],1})` (returns 10). -/
theorem w509_array_field_param_sum_2d_pt_value_equiv :
  evalModuleFunctionTotal defaultFuel w509ArrayFieldParamEnv w509ArrayFieldParamModule "sum_2d_pt"
    [⟨56, BitVec.ofNat 56 0x01020304050601⟩] =
  evalVModuleTotal defaultFuel w509ArrayFieldParamEnv (emitModule w509ArrayFieldParamEnv w509ArrayFieldParamModule) "sum_2d_pt"
    [⟨56, BitVec.ofNat 56 0x01020304050601⟩] := by
  have hlowerable : Module.isLowerable w509ArrayFieldParamEnv w509ArrayFieldParamModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w509ArrayFieldParamModule := by
    simp [Module.hasUniqueFunctionNames, w509ArrayFieldParamModule, w509ArrayFieldParamSumPt, w509ArrayFieldParamSum2DPt]
  have hseq : Module.isSequential w509ArrayFieldParamEnv w509ArrayFieldParamModule := by native_decide
  have hctx : Module.callContext w509ArrayFieldParamEnv w509ArrayFieldParamModule := by native_decide
  have hfind : w509ArrayFieldParamModule.findFunction "sum_2d_pt" = some w509ArrayFieldParamSum2DPt := by
    simp [Module.findFunction, w509ArrayFieldParamModule, w509ArrayFieldParamSumPt, w509ArrayFieldParamSum2DPt]
  have hhost : ¬ Env.isHostOnly w509ArrayFieldParamEnv w509ArrayFieldParamSum2DPt.name := by
    simp [Env.isHostOnly, w509ArrayFieldParamEnv, w509ArrayFieldParamSum2DPt]
  exact module_value_equiv_proved_sequential w509ArrayFieldParamEnv w509ArrayFieldParamModule
    hlowerable hunique hseq hctx "sum_2d_pt" w509ArrayFieldParamSum2DPt [⟨56, BitVec.ofNat 56 0x01020304050601⟩] hfind hhost

/-- W509-C: the packed-return witness is sequential. -/
theorem w509_array_field_return_sequential :
  Module.isSequential w509ArrayFieldReturnEnv w509ArrayFieldReturnModule := by
  native_decide

/-- W509-C: value preservation for `sum_returned_pt()` (returns 13). -/
theorem w509_array_field_return_sum_returned_pt_value_equiv :
  evalModuleFunctionTotal defaultFuel w509ArrayFieldReturnEnv w509ArrayFieldReturnModule "sum_returned_pt" [] =
  evalVModuleTotal defaultFuel w509ArrayFieldReturnEnv (emitModule w509ArrayFieldReturnEnv w509ArrayFieldReturnModule) "sum_returned_pt" [] := by
  have hlowerable : Module.isLowerable w509ArrayFieldReturnEnv w509ArrayFieldReturnModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w509ArrayFieldReturnModule := by
    simp [Module.hasUniqueFunctionNames, w509ArrayFieldReturnModule, w509ArrayFieldReturnMakePt, w509ArrayFieldReturnSumReturnedPt, w509ArrayFieldReturnMakePt2, w509ArrayFieldReturnSumReturnedPt2]
  have hseq : Module.isSequential w509ArrayFieldReturnEnv w509ArrayFieldReturnModule := by native_decide
  have hctx : Module.callContext w509ArrayFieldReturnEnv w509ArrayFieldReturnModule := by native_decide
  have hfind : w509ArrayFieldReturnModule.findFunction "sum_returned_pt" = some w509ArrayFieldReturnSumReturnedPt := by
    simp [Module.findFunction, w509ArrayFieldReturnModule, w509ArrayFieldReturnMakePt, w509ArrayFieldReturnSumReturnedPt, w509ArrayFieldReturnMakePt2, w509ArrayFieldReturnSumReturnedPt2]
  have hhost : ¬ Env.isHostOnly w509ArrayFieldReturnEnv w509ArrayFieldReturnSumReturnedPt.name := by
    simp [Env.isHostOnly, w509ArrayFieldReturnEnv, w509ArrayFieldReturnSumReturnedPt]
  exact module_value_equiv_proved_sequential w509ArrayFieldReturnEnv w509ArrayFieldReturnModule
    hlowerable hunique hseq hctx "sum_returned_pt" w509ArrayFieldReturnSumReturnedPt [] hfind hhost

/-- W509-C: value preservation for `sum_returned_pt2()` (returns 10). -/
theorem w509_array_field_return_sum_returned_pt2_value_equiv :
  evalModuleFunctionTotal defaultFuel w509ArrayFieldReturnEnv w509ArrayFieldReturnModule "sum_returned_pt2" [] =
  evalVModuleTotal defaultFuel w509ArrayFieldReturnEnv (emitModule w509ArrayFieldReturnEnv w509ArrayFieldReturnModule) "sum_returned_pt2" [] := by
  have hlowerable : Module.isLowerable w509ArrayFieldReturnEnv w509ArrayFieldReturnModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w509ArrayFieldReturnModule := by
    simp [Module.hasUniqueFunctionNames, w509ArrayFieldReturnModule, w509ArrayFieldReturnMakePt, w509ArrayFieldReturnSumReturnedPt, w509ArrayFieldReturnMakePt2, w509ArrayFieldReturnSumReturnedPt2]
  have hseq : Module.isSequential w509ArrayFieldReturnEnv w509ArrayFieldReturnModule := by native_decide
  have hctx : Module.callContext w509ArrayFieldReturnEnv w509ArrayFieldReturnModule := by native_decide
  have hfind : w509ArrayFieldReturnModule.findFunction "sum_returned_pt2" = some w509ArrayFieldReturnSumReturnedPt2 := by
    simp [Module.findFunction, w509ArrayFieldReturnModule, w509ArrayFieldReturnMakePt, w509ArrayFieldReturnSumReturnedPt, w509ArrayFieldReturnMakePt2, w509ArrayFieldReturnSumReturnedPt2]
  have hhost : ¬ Env.isHostOnly w509ArrayFieldReturnEnv w509ArrayFieldReturnSumReturnedPt2.name := by
    simp [Env.isHostOnly, w509ArrayFieldReturnEnv, w509ArrayFieldReturnSumReturnedPt2]
  exact module_value_equiv_proved_sequential w509ArrayFieldReturnEnv w509ArrayFieldReturnModule
    hlowerable hunique hseq hctx "sum_returned_pt2" w509ArrayFieldReturnSumReturnedPt2 [] hfind hhost

/- W510 theorems: element-level writes into packed array-typed struct fields.
   These use direct `native_decide` computational equivalence rather than the
   generic sequential theorem, because the current sequentiality predicate only
   accepts identifier LHS assignments. The shallow model still evaluates both the
   source and emitted Verilog to the same packed bit-vector value. -/

/-- W510-A: writing and reading back a variable-index element of a packed 1-D
    array field yields the same value on both sides of the model. -/
theorem w510_array_field_write_var_index_value_equiv :
  evalModuleFunctionTotal defaultFuel w510ArrayFieldWriteVarIndexEnv w510ArrayFieldWriteVarIndexModule "write_and_read"
    [⟨8, BitVec.ofNat 8 1⟩] =
  evalVModuleTotal defaultFuel w510ArrayFieldWriteVarIndexEnv
    (emitModule w510ArrayFieldWriteVarIndexEnv w510ArrayFieldWriteVarIndexModule) "write_and_read"
    [⟨8, BitVec.ofNat 8 1⟩] := by
  native_decide

/-- W510-B: writing and summing a variable-index row of a packed 2-D array field
    yields the same value on both sides of the model. -/
theorem w510_array_field_write_2d_slice_value_equiv :
  evalModuleFunctionTotal defaultFuel w510ArrayFieldWrite2DSliceEnv w510ArrayFieldWrite2DSliceModule "write_row"
    [⟨8, BitVec.ofNat 8 1⟩] =
  evalVModuleTotal defaultFuel w510ArrayFieldWrite2DSliceEnv
    (emitModule w510ArrayFieldWrite2DSliceEnv w510ArrayFieldWrite2DSliceModule) "write_row"
    [⟨8, BitVec.ofNat 8 1⟩] := by
  native_decide

/-- W510-C: mutating a packed array field before returning the whole struct,
    then reading the mutated element, yields the same value on both sides of the
    model. -/
theorem w510_array_field_write_return_copy_value_equiv :
  evalModuleFunctionTotal defaultFuel w510ArrayFieldWriteReturnCopyEnv w510ArrayFieldWriteReturnCopyModule "check" [] =
  evalVModuleTotal defaultFuel w510ArrayFieldWriteReturnCopyEnv
    (emitModule w510ArrayFieldWriteReturnCopyEnv w510ArrayFieldWriteReturnCopyModule) "check" [] := by
  native_decide

/- W511 theorems: module-level packed scalar structs with fixed-size scalar
   array fields.  The generic sequential value-preservation theorem applies
   because module-level const/var declarations are evaluated before the function
   body and the functions themselves are combinational. -/

/-- W511-A: the module-level read witness is sequential. -/
theorem w511_module_array_field_read_sequential :
  Module.isSequential w511ModuleArrayFieldReadEnv w511ModuleArrayFieldReadModule := by
  native_decide

/-- W511-A: value preservation for `read_coord(1)` (returns 20). -/
theorem w511_module_array_field_read_value_equiv :
  evalModuleFunctionTotal defaultFuel w511ModuleArrayFieldReadEnv w511ModuleArrayFieldReadModule "read_coord"
    [⟨8, BitVec.ofNat 8 1⟩] =
  evalVModuleTotal defaultFuel w511ModuleArrayFieldReadEnv
    (emitModule w511ModuleArrayFieldReadEnv w511ModuleArrayFieldReadModule) "read_coord"
    [⟨8, BitVec.ofNat 8 1⟩] := by
  have hlowerable : Module.isLowerable w511ModuleArrayFieldReadEnv w511ModuleArrayFieldReadModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w511ModuleArrayFieldReadModule := by
    simp [Module.hasUniqueFunctionNames, w511ModuleArrayFieldReadModule, w511ModuleArrayFieldReadCoord]
  have hseq : Module.isSequential w511ModuleArrayFieldReadEnv w511ModuleArrayFieldReadModule := by native_decide
  have hctx : Module.callContext w511ModuleArrayFieldReadEnv w511ModuleArrayFieldReadModule := by native_decide
  have hfind : w511ModuleArrayFieldReadModule.findFunction "read_coord" = some w511ModuleArrayFieldReadCoord := by
    simp [Module.findFunction, w511ModuleArrayFieldReadModule, w511ModuleArrayFieldReadCoord]
  have hhost : ¬ Env.isHostOnly w511ModuleArrayFieldReadEnv w511ModuleArrayFieldReadCoord.name := by
    simp [Env.isHostOnly, w511ModuleArrayFieldReadEnv, w511ModuleArrayFieldReadCoord]
  exact module_value_equiv_proved_sequential w511ModuleArrayFieldReadEnv w511ModuleArrayFieldReadModule
    hlowerable hunique hseq hctx "read_coord" w511ModuleArrayFieldReadCoord [⟨8, BitVec.ofNat 8 1⟩] hfind hhost

/-- W511-B: the module-level 2-D init witness is sequential. -/
theorem w511_module_array_field_init_sequential :
  Module.isSequential w511ModuleArrayFieldInitEnv w511ModuleArrayFieldInitModule := by
  native_decide

/-- W511-B: value preservation for `sum_row(1)` (returns 26). -/
theorem w511_module_array_field_init_value_equiv :
  evalModuleFunctionTotal defaultFuel w511ModuleArrayFieldInitEnv w511ModuleArrayFieldInitModule "sum_row"
    [⟨8, BitVec.ofNat 8 1⟩] =
  evalVModuleTotal defaultFuel w511ModuleArrayFieldInitEnv
    (emitModule w511ModuleArrayFieldInitEnv w511ModuleArrayFieldInitModule) "sum_row"
    [⟨8, BitVec.ofNat 8 1⟩] := by
  have hlowerable : Module.isLowerable w511ModuleArrayFieldInitEnv w511ModuleArrayFieldInitModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w511ModuleArrayFieldInitModule := by
    simp [Module.hasUniqueFunctionNames, w511ModuleArrayFieldInitModule, w511ModuleArrayFieldInitSumRow]
  have hseq : Module.isSequential w511ModuleArrayFieldInitEnv w511ModuleArrayFieldInitModule := by native_decide
  have hctx : Module.callContext w511ModuleArrayFieldInitEnv w511ModuleArrayFieldInitModule := by native_decide
  have hfind : w511ModuleArrayFieldInitModule.findFunction "sum_row" = some w511ModuleArrayFieldInitSumRow := by
    simp [Module.findFunction, w511ModuleArrayFieldInitModule, w511ModuleArrayFieldInitSumRow]
  have hhost : ¬ Env.isHostOnly w511ModuleArrayFieldInitEnv w511ModuleArrayFieldInitSumRow.name := by
    simp [Env.isHostOnly, w511ModuleArrayFieldInitEnv, w511ModuleArrayFieldInitSumRow]
  exact module_value_equiv_proved_sequential w511ModuleArrayFieldInitEnv w511ModuleArrayFieldInitModule
    hlowerable hunique hseq hctx "sum_row" w511ModuleArrayFieldInitSumRow [⟨8, BitVec.ofNat 8 1⟩] hfind hhost

/-- W511-C: the module-level whole-struct copy witness is lowerable, but not
    structurally sequential because `g_dst` is declared without an initializer at
    module scope and then assigned inside a function.  We prove value preservation
    directly by computation, matching the W510 element-write witnesses. -/
theorem w511_module_array_field_copy_value_equiv :
  evalModuleFunctionTotal defaultFuel w511ModuleArrayFieldCopyEnv w511ModuleArrayFieldCopyModule "copy_and_check"
    [⟨8, BitVec.ofNat 8 1⟩] =
  evalVModuleTotal defaultFuel w511ModuleArrayFieldCopyEnv
    (emitModule w511ModuleArrayFieldCopyEnv w511ModuleArrayFieldCopyModule) "copy_and_check"
    [⟨8, BitVec.ofNat 8 1⟩] := by
  native_decide

/- W512 theorems: arrays of structs with packed array-typed element fields. -/

/-- W512-A: the packed-AOS read witness is lowerable. -/
theorem w512_aos_read_lowerable :
  Module.isLowerable w512AosReadEnv w512AosReadModule := by
  native_decide

/-- W512-A: the packed-AOS read witness is combinational. -/
theorem w512_aos_read_combinational :
  Module.isCombinational w512AosReadEnv w512AosReadModule := by
  native_decide

/-- W512-A: value preservation for `read_tag(arr, 0)` (returns 1). -/
theorem w512_aos_read_tag_value_equiv :
  evalModuleFunctionTotal defaultFuel w512AosReadEnv w512AosReadModule "read_tag"
    [⟨256, BitVec.ofNat 256 0x000000010000000A000000140000001E0000000200000028000000320000003C⟩,
     ⟨8, BitVec.ofNat 8 0⟩] =
  evalVModuleTotal defaultFuel w512AosReadEnv (emitModule w512AosReadEnv w512AosReadModule) "read_tag"
    [⟨256, BitVec.ofNat 256 0x000000010000000A000000140000001E0000000200000028000000320000003C⟩,
     ⟨8, BitVec.ofNat 8 0⟩] := by
  have hlowerable : Module.isLowerable w512AosReadEnv w512AosReadModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w512AosReadModule := by
    simp [Module.hasUniqueFunctionNames, w512AosReadModule, w512AosReadTag, w512AosReadVal]
  have hcomb : Module.isCombinational w512AosReadEnv w512AosReadModule := by native_decide
  have hctx : Module.callContext w512AosReadEnv w512AosReadModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w512AosReadEnv, w512AosReadModule, w512AosReadTag, w512AosReadVal]
  have hfind : w512AosReadModule.findFunction "read_tag" = some w512AosReadTag := by
    simp [Module.findFunction, w512AosReadModule, w512AosReadTag, w512AosReadVal]
  have hhost : ¬ Env.isHostOnly w512AosReadEnv w512AosReadTag.name := by
    simp [Env.isHostOnly, w512AosReadEnv, w512AosReadTag]
  exact module_value_equiv_statement w512AosReadEnv w512AosReadModule
    hlowerable hunique hcomb hctx "read_tag" w512AosReadTag
    [⟨256, BitVec.ofNat 256 0x000000010000000A000000140000001E0000000200000028000000320000003C⟩,
     ⟨8, BitVec.ofNat 8 0⟩] hfind hhost

/-- W512-A: value preservation for `read_val(arr, 1, 1)` (returns 50). -/
theorem w512_aos_read_val_value_equiv :
  evalModuleFunctionTotal defaultFuel w512AosReadEnv w512AosReadModule "read_val"
    [⟨256, BitVec.ofNat 256 0x000000010000000A000000140000001E0000000200000028000000320000003C⟩,
     ⟨8, BitVec.ofNat 8 1⟩, ⟨8, BitVec.ofNat 8 1⟩] =
  evalVModuleTotal defaultFuel w512AosReadEnv (emitModule w512AosReadEnv w512AosReadModule) "read_val"
    [⟨256, BitVec.ofNat 256 0x000000010000000A000000140000001E0000000200000028000000320000003C⟩,
     ⟨8, BitVec.ofNat 8 1⟩, ⟨8, BitVec.ofNat 8 1⟩] := by
  have hlowerable : Module.isLowerable w512AosReadEnv w512AosReadModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w512AosReadModule := by
    simp [Module.hasUniqueFunctionNames, w512AosReadModule, w512AosReadTag, w512AosReadVal]
  have hcomb : Module.isCombinational w512AosReadEnv w512AosReadModule := by native_decide
  have hctx : Module.callContext w512AosReadEnv w512AosReadModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w512AosReadEnv, w512AosReadModule, w512AosReadTag, w512AosReadVal]
  have hfind : w512AosReadModule.findFunction "read_val" = some w512AosReadVal := by
    simp [Module.findFunction, w512AosReadModule, w512AosReadTag, w512AosReadVal]
  have hhost : ¬ Env.isHostOnly w512AosReadEnv w512AosReadVal.name := by
    simp [Env.isHostOnly, w512AosReadEnv, w512AosReadVal]
  exact module_value_equiv_statement w512AosReadEnv w512AosReadModule
    hlowerable hunique hcomb hctx "read_val" w512AosReadVal
    [⟨256, BitVec.ofNat 256 0x000000010000000A000000140000001E0000000200000028000000320000003C⟩,
     ⟨8, BitVec.ofNat 8 1⟩, ⟨8, BitVec.ofNat 8 1⟩] hfind hhost

/-- W512-B: the packed-AOS element-write witness is lowerable. -/
theorem w512_aos_write_lowerable :
  Module.isLowerable w512AosWriteEnv w512AosWriteModule := by
  native_decide

/-- W512-B: value preservation for `overwrite_and_read()` (shallow model no-op).
    The assignment target is not a bare identifier, so the shallow evaluator
    leaves the local unchanged on both sides; the theorem confirms the emitted
    Verilog also contains no placeholder. -/
theorem w512_aos_write_value_equiv :
  evalModuleFunctionTotal defaultFuel w512AosWriteEnv w512AosWriteModule "overwrite_and_read" [] =
  evalVModuleTotal defaultFuel w512AosWriteEnv
    (emitModule w512AosWriteEnv w512AosWriteModule) "overwrite_and_read" [] := by
  native_decide

/-- W512-C: the packed-AOS return witness is lowerable. -/
theorem w512_aos_return_lowerable :
  Module.isLowerable w512AosReturnEnv w512AosReturnModule := by
  native_decide

/-- W512-C: the packed-AOS return witness is combinational. -/
theorem w512_aos_return_combinational :
  Module.isCombinational w512AosReturnEnv w512AosReturnModule := by
  native_decide

/-- W512-C: value preservation for `read_returned()` (returns 60). -/
theorem w512_aos_return_value_equiv :
  evalModuleFunctionTotal defaultFuel w512AosReturnEnv w512AosReturnModule "read_returned" [] =
  evalVModuleTotal defaultFuel w512AosReturnEnv (emitModule w512AosReturnEnv w512AosReturnModule) "read_returned" [] := by
  have hlowerable : Module.isLowerable w512AosReturnEnv w512AosReturnModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w512AosReturnModule := by
    simp [Module.hasUniqueFunctionNames, w512AosReturnModule, w512AosReturnMakeArr, w512AosReturnReadReturned]
  have hcomb : Module.isCombinational w512AosReturnEnv w512AosReturnModule := by native_decide
  have hctx : Module.callContext w512AosReturnEnv w512AosReturnModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w512AosReturnEnv, w512AosReturnModule, w512AosReturnMakeArr, w512AosReturnReadReturned]
    all_goals native_decide
  have hfind : w512AosReturnModule.findFunction "read_returned" = some w512AosReturnReadReturned := by
    simp [Module.findFunction, w512AosReturnModule, w512AosReturnMakeArr, w512AosReturnReadReturned]
  have hhost : ¬ Env.isHostOnly w512AosReturnEnv w512AosReturnReadReturned.name := by
    simp [Env.isHostOnly, w512AosReturnEnv, w512AosReturnReadReturned]
  exact module_value_equiv_statement w512AosReturnEnv w512AosReturnModule
    hlowerable hunique hcomb hctx "read_returned" w512AosReturnReadReturned [] hfind hhost

/- W513 theorems: function-local packed-element arrays of structs with fixed-size
   scalar array fields. -/

/-- W513-A: the function-local packed-AOS read witness is lowerable. -/
theorem w513_local_aos_read_lowerable :
  Module.isLowerable w513LocalAosReadEnv w513LocalAosReadModule := by
  native_decide

/-- W513-A: the function-local packed-AOS read witness is combinational. -/
theorem w513_local_aos_read_combinational :
  Module.isCombinational w513LocalAosReadEnv w513LocalAosReadModule := by
  native_decide

/-- W513-A: value preservation for `read_fixed()` (returns 51). -/
theorem w513_local_aos_read_fixed_value_equiv :
  evalModuleFunctionTotal defaultFuel w513LocalAosReadEnv w513LocalAosReadModule "read_fixed" [] =
  evalVModuleTotal defaultFuel w513LocalAosReadEnv (emitModule w513LocalAosReadEnv w513LocalAosReadModule) "read_fixed" [] := by
  have hlowerable : Module.isLowerable w513LocalAosReadEnv w513LocalAosReadModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w513LocalAosReadModule := by
    simp [Module.hasUniqueFunctionNames, w513LocalAosReadModule, w513LocalAosReadFixed, w513LocalAosReadIndexed]
  have hcomb : Module.isCombinational w513LocalAosReadEnv w513LocalAosReadModule := by native_decide
  have hctx : Module.callContext w513LocalAosReadEnv w513LocalAosReadModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w513LocalAosReadEnv, w513LocalAosReadModule, w513LocalAosReadFixed, w513LocalAosReadIndexed]
  have hfind : w513LocalAosReadModule.findFunction "read_fixed" = some w513LocalAosReadFixed := by
    simp [Module.findFunction, w513LocalAosReadModule, w513LocalAosReadFixed, w513LocalAosReadIndexed]
  have hhost : ¬ Env.isHostOnly w513LocalAosReadEnv w513LocalAosReadFixed.name := by
    simp [Env.isHostOnly, w513LocalAosReadEnv, w513LocalAosReadFixed]
  exact module_value_equiv_statement w513LocalAosReadEnv w513LocalAosReadModule
    hlowerable hunique hcomb hctx "read_fixed" w513LocalAosReadFixed [] hfind hhost

/-- W513-A: value preservation for `read_indexed(1, 2)` (returns 60). -/
theorem w513_local_aos_read_indexed_value_equiv :
  evalModuleFunctionTotal defaultFuel w513LocalAosReadEnv w513LocalAosReadModule "read_indexed"
    [⟨8, BitVec.ofNat 8 1⟩, ⟨8, BitVec.ofNat 8 2⟩] =
  evalVModuleTotal defaultFuel w513LocalAosReadEnv (emitModule w513LocalAosReadEnv w513LocalAosReadModule) "read_indexed"
    [⟨8, BitVec.ofNat 8 1⟩, ⟨8, BitVec.ofNat 8 2⟩] := by
  have hlowerable : Module.isLowerable w513LocalAosReadEnv w513LocalAosReadModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w513LocalAosReadModule := by
    simp [Module.hasUniqueFunctionNames, w513LocalAosReadModule, w513LocalAosReadFixed, w513LocalAosReadIndexed]
  have hcomb : Module.isCombinational w513LocalAosReadEnv w513LocalAosReadModule := by native_decide
  have hctx : Module.callContext w513LocalAosReadEnv w513LocalAosReadModule := by
    simp [Module.callContext, Stmt.callContextList, Stmt.callContext, Stmt.functionNames, w513LocalAosReadEnv, w513LocalAosReadModule, w513LocalAosReadFixed, w513LocalAosReadIndexed]
  have hfind : w513LocalAosReadModule.findFunction "read_indexed" = some w513LocalAosReadIndexed := by
    simp [Module.findFunction, w513LocalAosReadModule, w513LocalAosReadFixed, w513LocalAosReadIndexed]
  have hhost : ¬ Env.isHostOnly w513LocalAosReadEnv w513LocalAosReadIndexed.name := by
    simp [Env.isHostOnly, w513LocalAosReadEnv, w513LocalAosReadIndexed]
  exact module_value_equiv_statement w513LocalAosReadEnv w513LocalAosReadModule
    hlowerable hunique hcomb hctx "read_indexed" w513LocalAosReadIndexed
    [⟨8, BitVec.ofNat 8 1⟩, ⟨8, BitVec.ofNat 8 2⟩] hfind hhost

/-- W513-B: the function-local packed-AOS write witness is lowerable. -/
theorem w513_local_aos_write_lowerable :
  Module.isLowerable w513LocalAosWriteEnv w513LocalAosWriteModule := by
  native_decide

/-- W513-B: value preservation for `modify_fixed()` (returns 106). -/
theorem w513_local_aos_write_fixed_value_equiv :
  evalModuleFunctionTotal defaultFuel w513LocalAosWriteEnv w513LocalAosWriteModule "modify_fixed" [] =
  evalVModuleTotal defaultFuel w513LocalAosWriteEnv (emitModule w513LocalAosWriteEnv w513LocalAosWriteModule) "modify_fixed" [] := by
  native_decide

/-- W513-B: value preservation for `modify_indexed(1, 2, 88)` (returns 88). -/
theorem w513_local_aos_write_indexed_value_equiv :
  evalModuleFunctionTotal defaultFuel w513LocalAosWriteEnv w513LocalAosWriteModule "modify_indexed"
    [⟨8, BitVec.ofNat 8 1⟩, ⟨8, BitVec.ofNat 8 2⟩, ⟨32, BitVec.ofNat 32 88⟩] =
  evalVModuleTotal defaultFuel w513LocalAosWriteEnv (emitModule w513LocalAosWriteEnv w513LocalAosWriteModule) "modify_indexed"
    [⟨8, BitVec.ofNat 8 1⟩, ⟨8, BitVec.ofNat 8 2⟩, ⟨32, BitVec.ofNat 32 88⟩] := by
  native_decide

/-- W513-C: the function-local packed-AOS return witness is lowerable. -/
theorem w513_local_aos_return_lowerable :
  Module.isLowerable w513LocalAosReturnEnv w513LocalAosReturnModule := by
  native_decide

/-- W513-C: value preservation for `make_local()` returning the mutated packed AOS. -/
theorem w513_local_aos_return_make_local_value_equiv :
  evalModuleFunctionTotal defaultFuel w513LocalAosReturnEnv w513LocalAosReturnModule "make_local" [] =
  evalVModuleTotal defaultFuel w513LocalAosReturnEnv (emitModule w513LocalAosReturnEnv w513LocalAosReturnModule) "make_local" [] := by
  native_decide

/-- W513-C: value preservation for `read_returned()` (returns 77). -/
theorem w513_local_aos_return_read_returned_value_equiv :
  evalModuleFunctionTotal defaultFuel w513LocalAosReturnEnv w513LocalAosReturnModule "read_returned" [] =
  evalVModuleTotal defaultFuel w513LocalAosReturnEnv (emitModule w513LocalAosReturnEnv w513LocalAosReturnModule) "read_returned" [] := by
  native_decide

/- W515 theorems: function-local packed scalar struct copy initializers. -/

/-- W515-A: the local-to-local packed scalar struct copy witness is lowerable. -/
theorem w515_local_copy_lowerable :
  Module.isLowerable w515LocalCopyEnv w515LocalCopyModule := by
  native_decide

/-- W515-A: value preservation for `copy_and_sum()` (returns 118). -/
theorem w515_local_copy_value_equiv :
  evalModuleFunctionTotal defaultFuel w515LocalCopyEnv w515LocalCopyModule "copy_and_sum" [] =
  evalVModuleTotal defaultFuel w515LocalCopyEnv (emitModule w515LocalCopyEnv w515LocalCopyModule) "copy_and_sum" [] := by
  native_decide

/-- W515-B: the module-to-local packed scalar struct copy witness is lowerable. -/
theorem w515_module_to_local_copy_lowerable :
  Module.isLowerable w515ModuleToLocalCopyEnv w515ModuleToLocalCopyModule := by
  native_decide

/-- W515-B: value preservation for `copy_and_sum()` (returns 123). -/
theorem w515_module_to_local_copy_value_equiv :
  evalModuleFunctionTotal defaultFuel w515ModuleToLocalCopyEnv w515ModuleToLocalCopyModule "copy_and_sum" [] =
  evalVModuleTotal defaultFuel w515ModuleToLocalCopyEnv (emitModule w515ModuleToLocalCopyEnv w515ModuleToLocalCopyModule) "copy_and_sum" [] := by
  native_decide

/-- W515-C: the return-to-local packed scalar struct copy witness is lowerable. -/
theorem w515_return_to_local_copy_lowerable :
  Module.isLowerable w515ReturnToLocalCopyEnv w515ReturnToLocalCopyModule := by
  native_decide

/-- W515-C: value preservation for `copy_and_sum()` (returns 107). -/
theorem w515_return_to_local_copy_value_equiv :
  evalModuleFunctionTotal defaultFuel w515ReturnToLocalCopyEnv w515ReturnToLocalCopyModule "copy_and_sum" [] =
  evalVModuleTotal defaultFuel w515ReturnToLocalCopyEnv (emitModule w515ReturnToLocalCopyEnv w515ReturnToLocalCopyModule) "copy_and_sum" [] := by
  native_decide

/- W521 theorems: multi-dimensional arrays-of-structs passed as function
   parameters. -/

/-- W521-A: the 2-D register-mode AOS parameter witness is lowerable. -/
theorem w521_aos_param_2d_scalar_lowerable :
  Module.isLowerable w521AosParam2DScalarEnv w521AosParam2DScalarModule := by
  native_decide

/-- W521-A: the caller function is combinational. -/
theorem w521_aos_param_2d_scalar_caller_combinational :
  Module.isCombinational w521AosParam2DScalarEnv w521AosParam2DScalarModule := by
  native_decide

/-- W521-A: value preservation for `caller()` (returns 23 = g_grid[1][2].x + y). -/
theorem w521_aos_param_2d_scalar_caller_value_equiv :
  evalModuleFunctionTotal defaultFuel w521AosParam2DScalarEnv w521AosParam2DScalarModule "caller" [] =
  evalVModuleTotal defaultFuel w521AosParam2DScalarEnv (emitModule w521AosParam2DScalarEnv w521AosParam2DScalarModule) "caller" [] := by
  have hlowerable : Module.isLowerable w521AosParam2DScalarEnv w521AosParam2DScalarModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w521AosParam2DScalarModule := by
    simp [Module.hasUniqueFunctionNames, w521AosParam2DScalarModule, w521AosParam2DScalarReadParam, w521AosParam2DScalarCaller]
  have hcomb : Module.isCombinational w521AosParam2DScalarEnv w521AosParam2DScalarModule := by native_decide
  have hctx : Module.callContext w521AosParam2DScalarEnv w521AosParam2DScalarModule := by native_decide
  have hfind : w521AosParam2DScalarModule.findFunction "caller" = some w521AosParam2DScalarCaller := by
    simp [Module.findFunction, w521AosParam2DScalarModule, w521AosParam2DScalarReadParam, w521AosParam2DScalarCaller]
  have hhost : ¬ Env.isHostOnly w521AosParam2DScalarEnv w521AosParam2DScalarCaller.name := by
    simp [Env.isHostOnly, w521AosParam2DScalarEnv, w521AosParam2DScalarCaller]
  exact module_value_equiv_statement w521AosParam2DScalarEnv w521AosParam2DScalarModule
    hlowerable hunique hcomb hctx "caller" w521AosParam2DScalarCaller [] hfind hhost

/-- W521-B: the 2-D packed-element AOS parameter witness is lowerable. -/
theorem w521_aos_param_2d_array_field_lowerable :
  Module.isLowerable w521AosParam2DArrayFieldEnv w521AosParam2DArrayFieldModule := by
  native_decide

/-- W521-B: the caller function is sequential. -/
theorem w521_aos_param_2d_array_field_caller_sequential :
  Module.isSequential w521AosParam2DArrayFieldEnv w521AosParam2DArrayFieldModule := by
  native_decide

/-- W521-B: value preservation for `caller()` (returns 7 = arr[0][1].data[2]). -/
theorem w521_aos_param_2d_array_field_caller_value_equiv :
  evalModuleFunctionTotal defaultFuel w521AosParam2DArrayFieldEnv w521AosParam2DArrayFieldModule "caller" [] =
  evalVModuleTotal defaultFuel w521AosParam2DArrayFieldEnv (emitModule w521AosParam2DArrayFieldEnv w521AosParam2DArrayFieldModule) "caller" [] := by
  have hlowerable : Module.isLowerable w521AosParam2DArrayFieldEnv w521AosParam2DArrayFieldModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w521AosParam2DArrayFieldModule := by
    simp [Module.hasUniqueFunctionNames, w521AosParam2DArrayFieldModule, w521AosParam2DArrayFieldReadBuf, w521AosParam2DArrayFieldCaller]
  have hseq : Module.isSequential w521AosParam2DArrayFieldEnv w521AosParam2DArrayFieldModule := by native_decide
  have hctx : Module.callContext w521AosParam2DArrayFieldEnv w521AosParam2DArrayFieldModule := by native_decide
  have hfind : w521AosParam2DArrayFieldModule.findFunction "caller" = some w521AosParam2DArrayFieldCaller := by
    simp [Module.findFunction, w521AosParam2DArrayFieldModule, w521AosParam2DArrayFieldReadBuf, w521AosParam2DArrayFieldCaller]
  have hhost : ¬ Env.isHostOnly w521AosParam2DArrayFieldEnv w521AosParam2DArrayFieldCaller.name := by
    simp [Env.isHostOnly, w521AosParam2DArrayFieldEnv, w521AosParam2DArrayFieldCaller]
  exact module_value_equiv_proved_sequential w521AosParam2DArrayFieldEnv w521AosParam2DArrayFieldModule
    hlowerable hunique hseq hctx "caller" w521AosParam2DArrayFieldCaller [] hfind hhost

/- W524 theorems: module-level 2-D packed-element AOS parameter. -/

/-- W524: the module-level 2-D packed-element AOS parameter witness is lowerable. -/
theorem w524_aos_param_2d_packed_module_lowerable :
  Module.isLowerable w524AosParam2DPackedModuleEnv w524AosParam2DPackedModule := by
  native_decide

/-- W524: the module is sequential because `sum_bufs` declares a local `total`. -/
theorem w524_aos_param_2d_packed_module_caller_sequential :
  Module.isSequential w524AosParam2DPackedModuleEnv w524AosParam2DPackedModule := by
  native_decide

/-- W524: value preservation for `caller()` (returns 136 = sum of g_bufs data). -/
theorem w524_aos_param_2d_packed_module_caller_value_equiv :
  evalModuleFunctionTotal defaultFuel w524AosParam2DPackedModuleEnv w524AosParam2DPackedModule "caller" [] =
  evalVModuleTotal defaultFuel w524AosParam2DPackedModuleEnv (emitModule w524AosParam2DPackedModuleEnv w524AosParam2DPackedModule) "caller" [] := by
  have hlowerable : Module.isLowerable w524AosParam2DPackedModuleEnv w524AosParam2DPackedModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w524AosParam2DPackedModule := by
    simp [Module.hasUniqueFunctionNames, w524AosParam2DPackedModule, w524AosParam2DPackedModuleSumBufs, w524AosParam2DPackedModuleCaller]
  have hseq : Module.isSequential w524AosParam2DPackedModuleEnv w524AosParam2DPackedModule := by native_decide
  have hctx : Module.callContext w524AosParam2DPackedModuleEnv w524AosParam2DPackedModule := by native_decide
  have hfind : w524AosParam2DPackedModule.findFunction "caller" = some w524AosParam2DPackedModuleCaller := by
    simp [Module.findFunction, w524AosParam2DPackedModule, w524AosParam2DPackedModuleSumBufs, w524AosParam2DPackedModuleCaller]
  have hhost : ¬ Env.isHostOnly w524AosParam2DPackedModuleEnv w524AosParam2DPackedModuleCaller.name := by
    simp [Env.isHostOnly, w524AosParam2DPackedModuleEnv, w524AosParam2DPackedModuleCaller]
  exact module_value_equiv_proved_sequential w524AosParam2DPackedModuleEnv w524AosParam2DPackedModule
    hlowerable hunique hseq hctx "caller" w524AosParam2DPackedModuleCaller [] hfind hhost

/- W529 theorems: module/function 2-D scalar-struct AoS cross-boundary lowering. -/

/-- W529-A: the module-level 2-D packed constant witness is lowerable. -/
theorem w529_module_2d_struct_array_const_lowerable :
  Module.isLowerable w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule := by
  native_decide

/-- W529-A: the module-level 2-D packed constant witness is combinational. -/
theorem w529_module_2d_struct_array_const_combinational :
  Module.isCombinational w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule := by
  native_decide

/-- W529-A: value preservation for variable-index read of the module constant. -/
theorem w529_module_2d_struct_array_const_read_var_value_equiv :
  evalModuleFunctionTotal defaultFuel w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule "read_const" [Value.mk 32 1, Value.mk 32 0] =
  evalVModuleTotal defaultFuel w529Module2DStructArrayConstEnv (emitModule w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule) "read_const" [Value.mk 32 1, Value.mk 32 0] := by
  have hlowerable : Module.isLowerable w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w529Module2DStructArrayConstModule := by
    simp [Module.hasUniqueFunctionNames, w529Module2DStructArrayConstModule, w529Module2DStructArrayConstReadVar, w529Module2DStructArrayConstReadLiteral]
  have hcomb : Module.isCombinational w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule := by native_decide
  have hctx : Module.callContext w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule := by native_decide
  have hfind : w529Module2DStructArrayConstModule.findFunction "read_const" = some w529Module2DStructArrayConstReadVar := by
    simp [Module.findFunction, w529Module2DStructArrayConstModule, w529Module2DStructArrayConstReadVar, w529Module2DStructArrayConstReadLiteral]
  have hhost : ¬ Env.isHostOnly w529Module2DStructArrayConstEnv w529Module2DStructArrayConstReadVar.name := by
    simp [Env.isHostOnly, w529Module2DStructArrayConstEnv, w529Module2DStructArrayConstReadVar]
  exact module_value_equiv_statement w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule
    hlowerable hunique hcomb hctx "read_const" w529Module2DStructArrayConstReadVar [Value.mk 32 1, Value.mk 32 0] hfind hhost

/-- W529-A: value preservation for literal-index read of the module constant. -/
theorem w529_module_2d_struct_array_const_read_literal_value_equiv :
  evalModuleFunctionTotal defaultFuel w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule "read_const_literal" [] =
  evalVModuleTotal defaultFuel w529Module2DStructArrayConstEnv (emitModule w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule) "read_const_literal" [] := by
  have hlowerable : Module.isLowerable w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w529Module2DStructArrayConstModule := by
    simp [Module.hasUniqueFunctionNames, w529Module2DStructArrayConstModule, w529Module2DStructArrayConstReadVar, w529Module2DStructArrayConstReadLiteral]
  have hcomb : Module.isCombinational w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule := by native_decide
  have hctx : Module.callContext w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule := by native_decide
  have hfind : w529Module2DStructArrayConstModule.findFunction "read_const_literal" = some w529Module2DStructArrayConstReadLiteral := by
    simp [Module.findFunction, w529Module2DStructArrayConstModule, w529Module2DStructArrayConstReadVar, w529Module2DStructArrayConstReadLiteral]
  have hhost : ¬ Env.isHostOnly w529Module2DStructArrayConstEnv w529Module2DStructArrayConstReadLiteral.name := by
    simp [Env.isHostOnly, w529Module2DStructArrayConstEnv, w529Module2DStructArrayConstReadLiteral]
  exact module_value_equiv_statement w529Module2DStructArrayConstEnv w529Module2DStructArrayConstModule
    hlowerable hunique hcomb hctx "read_const_literal" w529Module2DStructArrayConstReadLiteral [] hfind hhost

/-- W529-B: the module-level 2-D packed variable witness is lowerable. -/
theorem w529_module_2d_struct_array_var_lowerable :
  Module.isLowerable w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule := by
  native_decide

/-- W529-B: the module-level 2-D packed variable witness is combinational. -/
theorem w529_module_2d_struct_array_var_combinational :
  Module.isCombinational w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule := by
  native_decide

/-- W529-B: value preservation for variable-index read of the module variable. -/
theorem w529_module_2d_struct_array_var_read_var_value_equiv :
  evalModuleFunctionTotal defaultFuel w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule "read_var" [Value.mk 32 1, Value.mk 32 2] =
  evalVModuleTotal defaultFuel w529Module2DStructArrayVarEnv (emitModule w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule) "read_var" [Value.mk 32 1, Value.mk 32 2] := by
  have hlowerable : Module.isLowerable w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w529Module2DStructArrayVarModule := by
    simp [Module.hasUniqueFunctionNames, w529Module2DStructArrayVarModule, w529Module2DStructArrayVarReadVar, w529Module2DStructArrayVarReadLiteral]
  have hcomb : Module.isCombinational w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule := by native_decide
  have hctx : Module.callContext w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule := by native_decide
  have hfind : w529Module2DStructArrayVarModule.findFunction "read_var" = some w529Module2DStructArrayVarReadVar := by
    simp [Module.findFunction, w529Module2DStructArrayVarModule, w529Module2DStructArrayVarReadVar, w529Module2DStructArrayVarReadLiteral]
  have hhost : ¬ Env.isHostOnly w529Module2DStructArrayVarEnv w529Module2DStructArrayVarReadVar.name := by
    simp [Env.isHostOnly, w529Module2DStructArrayVarEnv, w529Module2DStructArrayVarReadVar]
  exact module_value_equiv_statement w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule
    hlowerable hunique hcomb hctx "read_var" w529Module2DStructArrayVarReadVar [Value.mk 32 1, Value.mk 32 2] hfind hhost

/-- W529-B: value preservation for literal-index read of the module variable. -/
theorem w529_module_2d_struct_array_var_read_literal_value_equiv :
  evalModuleFunctionTotal defaultFuel w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule "read_var_literal" [] =
  evalVModuleTotal defaultFuel w529Module2DStructArrayVarEnv (emitModule w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule) "read_var_literal" [] := by
  have hlowerable : Module.isLowerable w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w529Module2DStructArrayVarModule := by
    simp [Module.hasUniqueFunctionNames, w529Module2DStructArrayVarModule, w529Module2DStructArrayVarReadVar, w529Module2DStructArrayVarReadLiteral]
  have hcomb : Module.isCombinational w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule := by native_decide
  have hctx : Module.callContext w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule := by native_decide
  have hfind : w529Module2DStructArrayVarModule.findFunction "read_var_literal" = some w529Module2DStructArrayVarReadLiteral := by
    simp [Module.findFunction, w529Module2DStructArrayVarModule, w529Module2DStructArrayVarReadVar, w529Module2DStructArrayVarReadLiteral]
  have hhost : ¬ Env.isHostOnly w529Module2DStructArrayVarEnv w529Module2DStructArrayVarReadLiteral.name := by
    simp [Env.isHostOnly, w529Module2DStructArrayVarEnv, w529Module2DStructArrayVarReadLiteral]
  exact module_value_equiv_statement w529Module2DStructArrayVarEnv w529Module2DStructArrayVarModule
    hlowerable hunique hcomb hctx "read_var_literal" w529Module2DStructArrayVarReadLiteral [] hfind hhost

/-- W529-C: the 2-D AoS function-parameter witness is lowerable. -/
theorem w529_function_2d_struct_array_param_lowerable :
  Module.isLowerable w529Function2DStructArrayParamEnv w529Function2DStructArrayParamModule := by
  native_decide

/-- W529-C: the 2-D AoS function-parameter witness is combinational. -/
theorem w529_function_2d_struct_array_param_combinational :
  Module.isCombinational w529Function2DStructArrayParamEnv w529Function2DStructArrayParamModule := by
  native_decide

/-- W529-C: value preservation for the caller that passes a packed 2-D AoS into
    helper functions. -/
theorem w529_function_2d_struct_array_param_caller_value_equiv :
  evalModuleFunctionTotal defaultFuel w529Function2DStructArrayParamEnv w529Function2DStructArrayParamModule "caller" [] =
  evalVModuleTotal defaultFuel w529Function2DStructArrayParamEnv (emitModule w529Function2DStructArrayParamEnv w529Function2DStructArrayParamModule) "caller" [] := by
  have hlowerable : Module.isLowerable w529Function2DStructArrayParamEnv w529Function2DStructArrayParamModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w529Function2DStructArrayParamModule := by
    simp [Module.hasUniqueFunctionNames, w529Function2DStructArrayParamModule, w529Function2DStructArrayParamSum, w529Function2DStructArrayParamVaridx, w529Function2DStructArrayParamMakeGrid, w529Function2DStructArrayParamCaller]
  have hcomb : Module.isCombinational w529Function2DStructArrayParamEnv w529Function2DStructArrayParamModule := by native_decide
  have hctx : Module.callContext w529Function2DStructArrayParamEnv w529Function2DStructArrayParamModule := by native_decide
  have hfind : w529Function2DStructArrayParamModule.findFunction "caller" = some w529Function2DStructArrayParamCaller := by
    simp [Module.findFunction, w529Function2DStructArrayParamModule, w529Function2DStructArrayParamSum, w529Function2DStructArrayParamVaridx, w529Function2DStructArrayParamMakeGrid, w529Function2DStructArrayParamCaller]
  have hhost : ¬ Env.isHostOnly w529Function2DStructArrayParamEnv w529Function2DStructArrayParamCaller.name := by
    simp [Env.isHostOnly, w529Function2DStructArrayParamEnv, w529Function2DStructArrayParamCaller]
  exact module_value_equiv_statement w529Function2DStructArrayParamEnv w529Function2DStructArrayParamModule
    hlowerable hunique hcomb hctx "caller" w529Function2DStructArrayParamCaller [] hfind hhost

/-- W529-D: the 2-D AoS function-return witness is lowerable. -/
theorem w529_function_2d_struct_array_return_lowerable :
  Module.isLowerable w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule := by
  native_decide

/-- W529-D: the 2-D AoS function-return witness is sequential because consumers
    bind the returned array to a local variable. -/
theorem w529_function_2d_struct_array_return_sequential :
  Module.isSequential w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule := by
  native_decide

/-- W529-D: value preservation for the local-copy consumer with literal indices. -/
theorem w529_function_2d_struct_array_return_sum_value_equiv :
  evalModuleFunctionTotal defaultFuel w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule "sum_returned" [] =
  evalVModuleTotal defaultFuel w529Function2DStructArrayReturnEnv (emitModule w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule) "sum_returned" [] := by
  have hlowerable : Module.isLowerable w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w529Function2DStructArrayReturnModule := by
    simp [Module.hasUniqueFunctionNames, w529Function2DStructArrayReturnModule, w529Function2DStructArrayReturnMakeGrid, w529Function2DStructArrayReturnSum, w529Function2DStructArrayReturnVaridx]
  have hseq : Module.isSequential w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule := by native_decide
  have hctx : Module.callContext w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule := by native_decide
  have hfind : w529Function2DStructArrayReturnModule.findFunction "sum_returned" = some w529Function2DStructArrayReturnSum := by
    simp [Module.findFunction, w529Function2DStructArrayReturnModule, w529Function2DStructArrayReturnMakeGrid, w529Function2DStructArrayReturnSum, w529Function2DStructArrayReturnVaridx]
  have hhost : ¬ Env.isHostOnly w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnSum.name := by
    simp [Env.isHostOnly, w529Function2DStructArrayReturnEnv, w529Function2DStructArrayReturnSum]
  exact module_value_equiv_proved_sequential w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule
    hlowerable hunique hseq hctx "sum_returned" w529Function2DStructArrayReturnSum [] hfind hhost

/-- W529-D: value preservation for the local-copy consumer with variable indices. -/
theorem w529_function_2d_struct_array_return_varidx_value_equiv :
  evalModuleFunctionTotal defaultFuel w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule "varidx_returned" [Value.mk 32 1, Value.mk 32 2] =
  evalVModuleTotal defaultFuel w529Function2DStructArrayReturnEnv (emitModule w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule) "varidx_returned" [Value.mk 32 1, Value.mk 32 2] := by
  have hlowerable : Module.isLowerable w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w529Function2DStructArrayReturnModule := by
    simp [Module.hasUniqueFunctionNames, w529Function2DStructArrayReturnModule, w529Function2DStructArrayReturnMakeGrid, w529Function2DStructArrayReturnSum, w529Function2DStructArrayReturnVaridx]
  have hseq : Module.isSequential w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule := by native_decide
  have hctx : Module.callContext w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule := by native_decide
  have hfind : w529Function2DStructArrayReturnModule.findFunction "varidx_returned" = some w529Function2DStructArrayReturnVaridx := by
    simp [Module.findFunction, w529Function2DStructArrayReturnModule, w529Function2DStructArrayReturnMakeGrid, w529Function2DStructArrayReturnSum, w529Function2DStructArrayReturnVaridx]
  have hhost : ¬ Env.isHostOnly w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnVaridx.name := by
    simp [Env.isHostOnly, w529Function2DStructArrayReturnEnv, w529Function2DStructArrayReturnVaridx]
  exact module_value_equiv_proved_sequential w529Function2DStructArrayReturnEnv w529Function2DStructArrayReturnModule
    hlowerable hunique hseq hctx "varidx_returned" w529Function2DStructArrayReturnVaridx [Value.mk 32 1, Value.mk 32 2] hfind hhost

/- W545 theorems: primitive scalar array function returns used to initialize
   module-level globals. -/

/-- W545: the call-init primitive scalar array witness is lowerable. -/
theorem w545_call_init_returns_array_lowerable :
  Module.isLowerable w545CallInitReturnsArrayEnv w545CallInitReturnsArrayModule := by
  native_decide

/-- W545: the call-init primitive scalar array witness is sequential because the
    module global is initialized by a pure combinational call and the test block
    contains only bare assertions. -/
theorem w545_call_init_returns_array_sequential :
  Module.isSequential w545CallInitReturnsArrayEnv w545CallInitReturnsArrayModule := by
  native_decide

/-- W545: value preservation for `seq()` returning the packed vector `[1,2,3]`. -/
theorem w545_call_init_returns_array_value_equiv :
  evalModuleFunctionTotal defaultFuel w545CallInitReturnsArrayEnv w545CallInitReturnsArrayModule "seq" [] =
  evalVModuleTotal defaultFuel w545CallInitReturnsArrayEnv (emitModule w545CallInitReturnsArrayEnv w545CallInitReturnsArrayModule) "seq" [] := by
  have hlowerable : Module.isLowerable w545CallInitReturnsArrayEnv w545CallInitReturnsArrayModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w545CallInitReturnsArrayModule := by
    simp [Module.hasUniqueFunctionNames, w545CallInitReturnsArrayModule, w545CallInitReturnsArraySeq]
  have hseq : Module.isSequential w545CallInitReturnsArrayEnv w545CallInitReturnsArrayModule := by native_decide
  have hctx : Module.callContext w545CallInitReturnsArrayEnv w545CallInitReturnsArrayModule := by
    native_decide
  have hfind : w545CallInitReturnsArrayModule.findFunction "seq" = some w545CallInitReturnsArraySeq := by
    simp [Module.findFunction, w545CallInitReturnsArrayModule, w545CallInitReturnsArraySeq]
  have hhost : ¬ Env.isHostOnly w545CallInitReturnsArrayEnv w545CallInitReturnsArraySeq.name := by
    simp [Env.isHostOnly, w545CallInitReturnsArrayEnv, w545CallInitReturnsArraySeq]
  exact module_value_equiv_proved_sequential w545CallInitReturnsArrayEnv w545CallInitReturnsArrayModule
    hlowerable hunique hseq hctx "seq" w545CallInitReturnsArraySeq [] hfind hhost

/- W546 theorems: function-local primitive scalar arrays initialized or reassigned
   from packed-vector function calls. -/

/-- W546-A: the local-call-init primitive scalar array witness is lowerable. -/
theorem w546_local_call_init_returns_array_lowerable :
  Module.isLowerable w546LocalCallInitReturnsArrayEnv w546LocalCallInitReturnsArrayModule := by
  native_decide

/-- W546-A: value preservation for `check()` (returns 6). -/
theorem w546_local_call_init_returns_array_value_equiv :
  evalModuleFunctionTotal defaultFuel w546LocalCallInitReturnsArrayEnv w546LocalCallInitReturnsArrayModule "check" [] =
  evalVModuleTotal defaultFuel w546LocalCallInitReturnsArrayEnv (emitModule w546LocalCallInitReturnsArrayEnv w546LocalCallInitReturnsArrayModule) "check" [] := by
  have hlowerable : Module.isLowerable w546LocalCallInitReturnsArrayEnv w546LocalCallInitReturnsArrayModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w546LocalCallInitReturnsArrayModule := by
    simp [Module.hasUniqueFunctionNames, w546LocalCallInitReturnsArrayModule, w546LocalCallInitReturnsArraySeq, w546LocalCallInitReturnsArrayCheck]
  have hcomb : Module.isCombinational w546LocalCallInitReturnsArrayEnv w546LocalCallInitReturnsArrayModule := by native_decide
  have hctx : Module.callContext w546LocalCallInitReturnsArrayEnv w546LocalCallInitReturnsArrayModule := by
    native_decide
  have hfind : w546LocalCallInitReturnsArrayModule.findFunction "check" = some w546LocalCallInitReturnsArrayCheck := by
    simp [Module.findFunction, w546LocalCallInitReturnsArrayModule, w546LocalCallInitReturnsArraySeq, w546LocalCallInitReturnsArrayCheck]
  have hhost : ¬ Env.isHostOnly w546LocalCallInitReturnsArrayEnv w546LocalCallInitReturnsArrayCheck.name := by
    simp [Env.isHostOnly, w546LocalCallInitReturnsArrayEnv, w546LocalCallInitReturnsArrayCheck]
  exact module_value_equiv_statement w546LocalCallInitReturnsArrayEnv w546LocalCallInitReturnsArrayModule
    hlowerable hunique hcomb hctx "check" w546LocalCallInitReturnsArrayCheck [] hfind hhost

/-- W546-B: the local-call-assign primitive scalar array witness is lowerable. -/
theorem w546_local_call_assign_returns_array_lowerable :
  Module.isLowerable w546LocalCallAssignReturnsArrayEnv w546LocalCallAssignReturnsArrayModule := by
  native_decide

/-- W546-B: value preservation for `check()` (returns 24). -/
theorem w546_local_call_assign_returns_array_value_equiv :
  evalModuleFunctionTotal defaultFuel w546LocalCallAssignReturnsArrayEnv w546LocalCallAssignReturnsArrayModule "check" [] =
  evalVModuleTotal defaultFuel w546LocalCallAssignReturnsArrayEnv (emitModule w546LocalCallAssignReturnsArrayEnv w546LocalCallAssignReturnsArrayModule) "check" [] := by
  have hlowerable : Module.isLowerable w546LocalCallAssignReturnsArrayEnv w546LocalCallAssignReturnsArrayModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w546LocalCallAssignReturnsArrayModule := by
    simp [Module.hasUniqueFunctionNames, w546LocalCallAssignReturnsArrayModule, w546LocalCallAssignReturnsArraySeq, w546LocalCallAssignReturnsArrayCheck]
  have hcomb : Module.isCombinational w546LocalCallAssignReturnsArrayEnv w546LocalCallAssignReturnsArrayModule := by native_decide
  have hctx : Module.callContext w546LocalCallAssignReturnsArrayEnv w546LocalCallAssignReturnsArrayModule := by
    native_decide
  have hfind : w546LocalCallAssignReturnsArrayModule.findFunction "check" = some w546LocalCallAssignReturnsArrayCheck := by
    simp [Module.findFunction, w546LocalCallAssignReturnsArrayModule, w546LocalCallAssignReturnsArraySeq, w546LocalCallAssignReturnsArrayCheck]
  have hhost : ¬ Env.isHostOnly w546LocalCallAssignReturnsArrayEnv w546LocalCallAssignReturnsArrayCheck.name := by
    simp [Env.isHostOnly, w546LocalCallAssignReturnsArrayEnv, w546LocalCallAssignReturnsArrayCheck]
  exact module_value_equiv_statement w546LocalCallAssignReturnsArrayEnv w546LocalCallAssignReturnsArrayModule
    hlowerable hunique hcomb hctx "check" w546LocalCallAssignReturnsArrayCheck [] hfind hhost

/- W547 theorems: signed primitive scalar array function returns. -/

theorem w547_signed_call_init_returns_array_lowerable :
  Module.isLowerable w547SignedCallInitReturnsArrayEnv w547SignedCallInitReturnsArrayModule := by
  native_decide

theorem w547_signed_call_init_returns_array_value_equiv :
  evalModuleFunctionTotal defaultFuel w547SignedCallInitReturnsArrayEnv w547SignedCallInitReturnsArrayModule "check" [] =
  evalVModuleTotal defaultFuel w547SignedCallInitReturnsArrayEnv (emitModule w547SignedCallInitReturnsArrayEnv w547SignedCallInitReturnsArrayModule) "check" [] := by
  have hlowerable : Module.isLowerable w547SignedCallInitReturnsArrayEnv w547SignedCallInitReturnsArrayModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w547SignedCallInitReturnsArrayModule := by
    simp [Module.hasUniqueFunctionNames, w547SignedCallInitReturnsArrayModule, w547SignedCallInitReturnsArraySeq, w547SignedCallInitReturnsArrayCheck]
  have hcomb : Module.isCombinational w547SignedCallInitReturnsArrayEnv w547SignedCallInitReturnsArrayModule := by native_decide
  have hctx : Module.callContext w547SignedCallInitReturnsArrayEnv w547SignedCallInitReturnsArrayModule := by native_decide
  have hfind : w547SignedCallInitReturnsArrayModule.findFunction "check" = some w547SignedCallInitReturnsArrayCheck := by
    simp [Module.findFunction, w547SignedCallInitReturnsArrayModule, w547SignedCallInitReturnsArraySeq, w547SignedCallInitReturnsArrayCheck]
  have hhost : ¬ Env.isHostOnly w547SignedCallInitReturnsArrayEnv w547SignedCallInitReturnsArrayCheck.name := by
    simp [Env.isHostOnly, w547SignedCallInitReturnsArrayEnv, w547SignedCallInitReturnsArrayCheck]
  exact module_value_equiv_statement w547SignedCallInitReturnsArrayEnv w547SignedCallInitReturnsArrayModule
    hlowerable hunique hcomb hctx "check" w547SignedCallInitReturnsArrayCheck [] hfind hhost

theorem w547_signed_element_compare_lowerable :
  Module.isLowerable w547SignedCallInitReturnsArrayEnv w547SignedElementCompareModule := by
  native_decide

theorem w547_signed_element_compare_value_equiv :
  evalModuleFunctionTotal defaultFuel w547SignedCallInitReturnsArrayEnv w547SignedElementCompareModule "seq" [] =
  evalVModuleTotal defaultFuel w547SignedCallInitReturnsArrayEnv (emitModule w547SignedCallInitReturnsArrayEnv w547SignedElementCompareModule) "seq" [] := by
  have hlowerable : Module.isLowerable w547SignedCallInitReturnsArrayEnv w547SignedElementCompareModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w547SignedElementCompareModule := by
    simp [Module.hasUniqueFunctionNames, w547SignedElementCompareModule, w547SignedElementCompareSeq]
  have hcomb : Module.isCombinational w547SignedCallInitReturnsArrayEnv w547SignedElementCompareModule := by native_decide
  have hctx : Module.callContext w547SignedCallInitReturnsArrayEnv w547SignedElementCompareModule := by native_decide
  have hfind : w547SignedElementCompareModule.findFunction "seq" = some w547SignedElementCompareSeq := by
    simp [Module.findFunction, w547SignedElementCompareModule, w547SignedElementCompareSeq]
  have hhost : ¬ Env.isHostOnly w547SignedCallInitReturnsArrayEnv w547SignedElementCompareSeq.name := by
    simp [Env.isHostOnly, w547SignedCallInitReturnsArrayEnv, w547SignedElementCompareSeq]
  exact module_value_equiv_statement w547SignedCallInitReturnsArrayEnv w547SignedElementCompareModule
    hlowerable hunique hcomb hctx "seq" w547SignedElementCompareSeq [] hfind hhost

/- W548 theorems: multi-dimensional primitive scalar array function returns. -/

/- Unsigned 2-D packed primitive array return with element indexing. -/

theorem w548_2d_call_init_returns_array_lowerable :
  Module.isLowerable w548TwoDCallInitReturnsArrayEnv w548TwoDCallInitReturnsArrayModule := by
  native_decide

theorem w548_2d_call_init_returns_array_value_equiv :
  evalModuleFunctionTotal defaultFuel w548TwoDCallInitReturnsArrayEnv w548TwoDCallInitReturnsArrayModule "sum" [] =
  evalVModuleTotal defaultFuel w548TwoDCallInitReturnsArrayEnv (emitModule w548TwoDCallInitReturnsArrayEnv w548TwoDCallInitReturnsArrayModule) "sum" [] := by
  have hlowerable : Module.isLowerable w548TwoDCallInitReturnsArrayEnv w548TwoDCallInitReturnsArrayModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w548TwoDCallInitReturnsArrayModule := by
    simp [Module.hasUniqueFunctionNames, w548TwoDCallInitReturnsArrayModule, w548TwoDCallInitReturnsArrayGrid, w548TwoDCallInitReturnsArraySum]
  have hcomb : Module.isCombinational w548TwoDCallInitReturnsArrayEnv w548TwoDCallInitReturnsArrayModule := by native_decide
  have hctx : Module.callContext w548TwoDCallInitReturnsArrayEnv w548TwoDCallInitReturnsArrayModule := by native_decide
  have hfind : w548TwoDCallInitReturnsArrayModule.findFunction "sum" = some w548TwoDCallInitReturnsArraySum := by
    simp [Module.findFunction, w548TwoDCallInitReturnsArrayModule, w548TwoDCallInitReturnsArrayGrid, w548TwoDCallInitReturnsArraySum]
  have hhost : ¬ Env.isHostOnly w548TwoDCallInitReturnsArrayEnv w548TwoDCallInitReturnsArraySum.name := by
    simp [Env.isHostOnly, w548TwoDCallInitReturnsArrayEnv, w548TwoDCallInitReturnsArraySum]
  exact module_value_equiv_statement w548TwoDCallInitReturnsArrayEnv w548TwoDCallInitReturnsArrayModule
    hlowerable hunique hcomb hctx "sum" w548TwoDCallInitReturnsArraySum [] hfind hhost

/- Signed 2-D packed primitive array return with element indexing. -/

theorem w548_2d_signed_element_read_lowerable :
  Module.isLowerable w548TwoDSignedElementReadEnv w548TwoDSignedElementReadModule := by
  native_decide

theorem w548_2d_signed_element_read_value_equiv :
  evalModuleFunctionTotal defaultFuel w548TwoDSignedElementReadEnv w548TwoDSignedElementReadModule "diag" [] =
  evalVModuleTotal defaultFuel w548TwoDSignedElementReadEnv (emitModule w548TwoDSignedElementReadEnv w548TwoDSignedElementReadModule) "diag" [] := by
  have hlowerable : Module.isLowerable w548TwoDSignedElementReadEnv w548TwoDSignedElementReadModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w548TwoDSignedElementReadModule := by
    simp [Module.hasUniqueFunctionNames, w548TwoDSignedElementReadModule, w548TwoDSignedElementReadSigns, w548TwoDSignedElementReadDiag]
  have hcomb : Module.isCombinational w548TwoDSignedElementReadEnv w548TwoDSignedElementReadModule := by native_decide
  have hctx : Module.callContext w548TwoDSignedElementReadEnv w548TwoDSignedElementReadModule := by native_decide
  have hfind : w548TwoDSignedElementReadModule.findFunction "diag" = some w548TwoDSignedElementReadDiag := by
    simp [Module.findFunction, w548TwoDSignedElementReadModule, w548TwoDSignedElementReadSigns, w548TwoDSignedElementReadDiag]
  have hhost : ¬ Env.isHostOnly w548TwoDSignedElementReadEnv w548TwoDSignedElementReadDiag.name := by
    simp [Env.isHostOnly, w548TwoDSignedElementReadEnv, w548TwoDSignedElementReadDiag]
  exact module_value_equiv_statement w548TwoDSignedElementReadEnv w548TwoDSignedElementReadModule
    hlowerable hunique hcomb hctx "diag" w548TwoDSignedElementReadDiag [] hfind hhost

/- W549 theorems: three-dimensional primitive scalar array function returns. -/

theorem w549_3d_call_init_returns_array_lowerable :
  Module.isLowerable w549ThreeDCallInitReturnsArrayEnv w549ThreeDCallInitReturnsArrayModule := by
  native_decide

theorem w549_3d_call_init_returns_array_value_equiv :
  evalModuleFunctionTotal defaultFuel w549ThreeDCallInitReturnsArrayEnv w549ThreeDCallInitReturnsArrayModule "check" [] =
  evalVModuleTotal defaultFuel w549ThreeDCallInitReturnsArrayEnv (emitModule w549ThreeDCallInitReturnsArrayEnv w549ThreeDCallInitReturnsArrayModule) "check" [] := by
  have hlowerable : Module.isLowerable w549ThreeDCallInitReturnsArrayEnv w549ThreeDCallInitReturnsArrayModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w549ThreeDCallInitReturnsArrayModule := by
    simp [Module.hasUniqueFunctionNames, w549ThreeDCallInitReturnsArrayModule, w549ThreeDCallInitReturnsArrayCube, w549ThreeDCallInitReturnsArrayCheck]
  have hcomb : Module.isCombinational w549ThreeDCallInitReturnsArrayEnv w549ThreeDCallInitReturnsArrayModule := by native_decide
  have hctx : Module.callContext w549ThreeDCallInitReturnsArrayEnv w549ThreeDCallInitReturnsArrayModule := by native_decide
  have hfind : w549ThreeDCallInitReturnsArrayModule.findFunction "check" = some w549ThreeDCallInitReturnsArrayCheck := by
    simp [Module.findFunction, w549ThreeDCallInitReturnsArrayModule, w549ThreeDCallInitReturnsArrayCube, w549ThreeDCallInitReturnsArrayCheck]
  have hhost : ¬ Env.isHostOnly w549ThreeDCallInitReturnsArrayEnv w549ThreeDCallInitReturnsArrayCheck.name := by
    simp [Env.isHostOnly, w549ThreeDCallInitReturnsArrayEnv, w549ThreeDCallInitReturnsArrayCheck]
  exact module_value_equiv_statement w549ThreeDCallInitReturnsArrayEnv w549ThreeDCallInitReturnsArrayModule
    hlowerable hunique hcomb hctx "check" w549ThreeDCallInitReturnsArrayCheck [] hfind hhost

/- W550 theorems: four-dimensional primitive scalar array function returns. -/

theorem w550_4d_call_init_returns_array_lowerable :
  Module.isLowerable w550FourDCallInitReturnsArrayEnv w550FourDCallInitReturnsArrayModule := by
  native_decide

theorem w550_4d_call_init_returns_array_value_equiv :
  evalModuleFunctionTotal defaultFuel w550FourDCallInitReturnsArrayEnv w550FourDCallInitReturnsArrayModule "check" [] =
  evalVModuleTotal defaultFuel w550FourDCallInitReturnsArrayEnv (emitModule w550FourDCallInitReturnsArrayEnv w550FourDCallInitReturnsArrayModule) "check" [] := by
  have hlowerable : Module.isLowerable w550FourDCallInitReturnsArrayEnv w550FourDCallInitReturnsArrayModule := by native_decide
  have hunique : Module.hasUniqueFunctionNames w550FourDCallInitReturnsArrayModule := by
    simp [Module.hasUniqueFunctionNames, w550FourDCallInitReturnsArrayModule, w550FourDCallInitReturnsArrayHyper, w550FourDCallInitReturnsArrayCheck]
  have hcomb : Module.isCombinational w550FourDCallInitReturnsArrayEnv w550FourDCallInitReturnsArrayModule := by native_decide
  have hctx : Module.callContext w550FourDCallInitReturnsArrayEnv w550FourDCallInitReturnsArrayModule := by native_decide
  have hfind : w550FourDCallInitReturnsArrayModule.findFunction "check" = some w550FourDCallInitReturnsArrayCheck := by
    simp [Module.findFunction, w550FourDCallInitReturnsArrayModule, w550FourDCallInitReturnsArrayHyper, w550FourDCallInitReturnsArrayCheck]
  have hhost : ¬ Env.isHostOnly w550FourDCallInitReturnsArrayEnv w550FourDCallInitReturnsArrayCheck.name := by
    simp [Env.isHostOnly, w550FourDCallInitReturnsArrayEnv, w550FourDCallInitReturnsArrayCheck]
  exact module_value_equiv_statement w550FourDCallInitReturnsArrayEnv w550FourDCallInitReturnsArrayModule
    hlowerable hunique hcomb hctx "check" w550FourDCallInitReturnsArrayCheck [] hfind hhost

end Trinity.IcarusLowerable
