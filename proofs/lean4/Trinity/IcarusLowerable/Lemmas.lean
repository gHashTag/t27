/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 491
  Representative lowerability lemmas for the four W490 lowerability classes.

  These are not full semantic correctness proofs (no Verilog semantics is
  modeled here). They are machine-checkable demonstrations that the
  `IsIcarusLowerable` predicate accepts the exact patterns that the Rust
  backend now lowers.

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

import Trinity.IcarusLowerable.Ast
import Trinity.IcarusLowerable.Predicate

namespace Trinity.IcarusLowerable

/-- Environment for scalar-struct-literal witness. -/
def scalarStructEnv : Env := {
  structs := [("Pt", [("x", .u8), ("y", .u8)])],
  constructors := [("make_pt", "Pt")],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["main"]
}

def scalarStructMain : Function := {
  name := "main",
  params := [],
  ret := some (.struct "Pt"),
  body := [.return_ (some (.structLit "Pt" [("x", .intLit 1), ("y", .intLit 2)]))]
}

def scalarStructModule : Module := {
  name := "scalar_struct_literal",
  imports := [],
  globals := [],
  functions := [scalarStructMain],
  tests := [],
  benches := []
}

/-- A scalar struct literal whose fields are numeric literals is lowerable. -/
theorem scalar_struct_literal_lowerable :
  Module.isLowerable scalarStructEnv scalarStructModule := by
  native_decide

/-- Environment for imported-constructor expression-context witness. -/
def importedCtorEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u8)])],
  constructors := [("make_pt", "Pt")],
  enums := [],
  imports := [("make_pt", ("geom", "make_pt"))],
  hostOnly := [],
  reachable := ["main"]
}

def importedCtorMain : Function := {
  name := "main",
  params := [],
  ret := some .u8,
  body := [.return_ (some (.fieldAccess (.call "make_pt" [.intLit 1, .intLit 2, .intLit 3]) "coords"))]
}

def importedCtorModule : Module := {
  name := "imported_constructor_expr_context",
  imports := [{ path := "geom", items := ["make_pt"] }],
  globals := [],
  functions := [importedCtorMain],
  tests := [],
  benches := []
}

/-- An imported constructor call used in expression context for a leaf-lowerable
    field is lowerable when the import resolves and arity matches. -/
theorem imported_constructor_expr_context_lowerable :
  Module.isLowerable importedCtorEnv importedCtorModule := by
  native_decide

/-- Environment for array-field index on struct-return call witness. -/
def arrayFieldEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u8)])],
  constructors := [("make_pt", "Pt")],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["main"]
}

def arrayFieldMain : Function := {
  name := "main",
  params := [],
  ret := some .u8,
  body := [.return_ (some (.index (.fieldAccess (.call "make_pt" [.intLit 1, .intLit 2, .intLit 3]) "coords") (.intLit 1)))]
}

def arrayFieldModule : Module := {
  name := "array_field_index_on_struct_return_call",
  imports := [],
  globals := [],
  functions := [arrayFieldMain],
  tests := [],
  benches := []
}

/-- Indexing an array-typed field of a scalar struct-return call with a literal
    index is lowerable when the element type is leaf-lowerable. -/
theorem array_field_index_on_struct_return_call_lowerable :
  Module.isLowerable arrayFieldEnv arrayFieldModule := by
  native_decide

/-- Environment for variable-index local array-field access witness. -/
def varIndexEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u8)])],
  constructors := [("make_pt", "Pt")],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["main"]
}

def varIndexMain : Function := {
  name := "main",
  params := [("i", .u8)],
  ret := some .u8,
  body := [
    .varDecl "p" (.struct "Pt") (some (.structLit "Pt" [("coords", .arrayLit (.array 3 .u8) [.intLit 1, .intLit 2, .intLit 3])])),
    .return_ (some (.index (.fieldAccess (.identifier "p") "coords") (.identifier "i")))
  ]
}

def varIndexModule : Module := {
  name := "variable_index_local_array_field",
  imports := [],
  globals := [],
  functions := [varIndexMain],
  tests := [],
  benches := []
}

/-- A variable-index access on a local struct variable's array-typed field is
    lowerable when the variable is declared in a reachable function, the
    initializer is a lowerable array literal, and the index expression is
    lowerable. -/
theorem variable_index_local_array_field_lowerable :
  Module.isLowerable varIndexEnv varIndexModule := by
  native_decide

/-- Environment for string-helper non-lowerability witness. -/
def stringHelperEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := ["describe"],
  reachable := ["main"]
}

def stringHelperFn : Function := {
  name := "describe",
  params := [],
  ret := some .string,
  body := [.return_ (some (.stringLit "ok"))]
}

def stringHelperMain : Function := {
  name := "main",
  params := [],
  ret := some .string,
  body := [.return_ (some (.call "describe" []))]
}

def stringHelperModule : Module := {
  name := "string_helper_not_lowerable",
  imports := [],
  globals := [],
  functions := [stringHelperFn, stringHelperMain],
  tests := [],
  benches := []
}

/-- A string-return helper is not lowerable in synthesizable context. -/
theorem string_helper_not_lowerable :
  ¬ Module.isLowerable stringHelperEnv stringHelperModule := by
  native_decide

/- W495 witness environments and modules. -/

/-- Environment for W493 nested-struct field from scalar-struct identifier. -/
def w493NestedIdentifierEnv : Env := {
  structs := [("Inner", [("y", .u32)]), ("Outer", [("x", .struct "Inner")])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_inner", "make_outer", "get_y"]
}

def w493NestedIdentifierMakeInner : Function := {
  name := "make_inner",
  params := [("a", .u32)],
  ret := some (.struct "Inner"),
  body := [.return_ (some (.structLit "Inner" [("y", .binop "+" (.identifier "a") (.intLit 1))]))]
}

def w493NestedIdentifierMakeOuter : Function := {
  name := "make_outer",
  params := [("inner", .struct "Inner")],
  ret := some (.struct "Outer"),
  body := [.return_ (some (.structLit "Outer" [("x", .identifier "inner")]))]
}

def w493NestedIdentifierGetY : Function := {
  name := "get_y",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.fieldAccess
        (.fieldAccess
          (.call "make_outer" [.call "make_inner" [.intLit 5]])
          "x")
        "y"))
  ]
}

def w493NestedIdentifierModule : Module := {
  name := "w493_nested_struct_field_from_identifier_lowerable",
  imports := [],
  globals := [],
  functions := [w493NestedIdentifierMakeInner, w493NestedIdentifierMakeOuter, w493NestedIdentifierGetY],
  tests := [],
  benches := []
}

/-- Environment for W493 local scalar-struct variable used as struct-literal field. -/
def w493LocalScalarEnv : Env := {
  structs := [("Inner", [("y", .u32)]), ("Outer", [("x", .struct "Inner")])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_inner", "make_outer", "get_y"]
}

def w493LocalScalarMakeInner : Function := {
  name := "make_inner",
  params := [("a", .u32)],
  ret := some (.struct "Inner"),
  body := [.return_ (some (.structLit "Inner" [("y", .binop "+" (.identifier "a") (.intLit 1))]))]
}

def w493LocalScalarMakeOuter : Function := {
  name := "make_outer",
  params := [],
  ret := some (.struct "Outer"),
  body := [
    .varDecl "inner" (.struct "Inner") (some (.call "make_inner" [.intLit 5])),
    .return_ (some (.structLit "Outer" [("x", .identifier "inner")]))
  ]
}

def w493LocalScalarGetY : Function := {
  name := "get_y",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.fieldAccess
        (.fieldAccess
          (.call "make_outer" [])
          "x")
        "y"))
  ]
}

def w493LocalScalarModule : Module := {
  name := "w493_local_scalar_struct_field_lowerable",
  imports := [],
  globals := [],
  functions := [w493LocalScalarMakeInner, w493LocalScalarMakeOuter, w493LocalScalarGetY],
  tests := [],
  benches := []
}

/-- Environment for W493 module-level scalar-struct constant used as struct-literal
    field. -/
def w493ModuleScalarEnv : Env := {
  structs := [("Inner", [("y", .u32)]), ("Outer", [("x", .struct "Inner")])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_outer", "get_y"],
  vars := [("INNER_CONST", .struct "Inner")]
}

def w493ModuleScalarConst : Stmt :=
  .constDecl "INNER_CONST" (.struct "Inner") (some (.structLit "Inner" [("y", .intLit 7)]))

def w493ModuleScalarMakeOuter : Function := {
  name := "make_outer",
  params := [],
  ret := some (.struct "Outer"),
  body := [.return_ (some (.structLit "Outer" [("x", .identifier "INNER_CONST")]))]
}

def w493ModuleScalarGetY : Function := {
  name := "get_y",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.fieldAccess
        (.fieldAccess
          (.call "make_outer" [])
          "x")
        "y"))
  ]
}

def w493ModuleScalarModule : Module := {
  name := "w493_module_scalar_struct_field_lowerable",
  imports := [],
  globals := [w493ModuleScalarConst],
  functions := [w493ModuleScalarMakeOuter, w493ModuleScalarGetY],
  tests := [],
  benches := []
}

/-- Environment for W493 literal-index module-level array-of-struct element used
    as struct-literal field. -/
def w493ModuleAosEnv : Env := {
  structs := [("Inner", [("y", .u32)]), ("Outer", [("x", .struct "Inner")])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_outer", "get_y"],
  vars := [("CHOICES", .array 2 (.struct "Inner"))]
}

def w493ModuleAosConst : Stmt :=
  .constDecl "CHOICES" (.array 2 (.struct "Inner"))
    (some (.arrayLit (.array 2 (.struct "Inner"))
      [.structLit "Inner" [("y", .intLit 1)],
       .structLit "Inner" [("y", .intLit 2)]]))

def w493ModuleAosMakeOuter : Function := {
  name := "make_outer",
  params := [],
  ret := some (.struct "Outer"),
  body := [.return_ (some (.structLit "Outer" [("x", .index (.identifier "CHOICES") (.intLit 0))]))]
}

def w493ModuleAosGetY : Function := {
  name := "get_y",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.fieldAccess
        (.fieldAccess
          (.call "make_outer" [])
          "x")
        "y"))
  ]
}

def w493ModuleAosModule : Module := {
  name := "w493_module_aos_element_field_lowerable",
  imports := [],
  globals := [w493ModuleAosConst],
  functions := [w493ModuleAosMakeOuter, w493ModuleAosGetY],
  tests := [],
  benches := []
}

/-- W501: environment for a module whose equivalence property is stated for a
    non-`main` emitted function. -/
def w501NonMainEnv : Env := {
  structs := [("Pt", [("x", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["main", "get_y", "make_pt"]
}

def w501NonMainMakePt : Function := {
  name := "make_pt",
  params := [],
  ret := some (.struct "Pt"),
  body := [.return_ (some (.structLit "Pt" [("x", .intLit 42)]))]
}

def w501NonMainGetY : Function := {
  name := "get_y",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.fieldAccess (.call "make_pt" []) "x"))]
}

def w501NonMainMain : Function := {
  name := "main",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.call "get_y" []))]
}

def w501NonMainModule : Module := {
  name := "w501_non_main_entry_function",
  imports := [],
  globals := [],
  functions := [w501NonMainMakePt, w501NonMainGetY, w501NonMainMain],
  tests := [],
  benches := []
}

/- W502 witness environments and modules. -/

/-- W502-A: a non-`main` function called from another emitted function. -/
def w502NonMainCalledFromEmittedEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["helper", "caller", "main"]
}

def w502NonMainCalledFromEmittedHelper : Function := {
  name := "helper",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.intLit 1))]
}

def w502NonMainCalledFromEmittedCaller : Function := {
  name := "caller",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.call "helper" []))]
}

def w502NonMainCalledFromEmittedMain : Function := {
  name := "main",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.call "caller" []))]
}

def w502NonMainCalledFromEmittedModule : Module := {
  name := "w502_non_main_called_from_emitted",
  imports := [],
  globals := [],
  functions := [w502NonMainCalledFromEmittedHelper, w502NonMainCalledFromEmittedCaller, w502NonMainCalledFromEmittedMain],
  tests := [],
  benches := []
}

/-- W502-B: chain of three emitted functions ending in a non-`main` leaf. -/
def w502NonMainChainLeafEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["leaf", "mid", "top", "main"]
}

def w502NonMainChainLeafLeaf : Function := {
  name := "leaf",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.intLit 7))]
}

def w502NonMainChainLeafMid : Function := {
  name := "mid",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.call "leaf" []))]
}

def w502NonMainChainLeafTop : Function := {
  name := "top",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.call "mid" []))]
}

def w502NonMainChainLeafMain : Function := {
  name := "main",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.call "top" []))]
}

def w502NonMainChainLeafModule : Module := {
  name := "w502_non_main_chain_leaf",
  imports := [],
  globals := [],
  functions := [w502NonMainChainLeafLeaf, w502NonMainChainLeafMid, w502NonMainChainLeafTop, w502NonMainChainLeafMain],
  tests := [],
  benches := []
}

/-- W502-C: helper taking a scalar struct parameter. -/
def w502NonMainHelperStructParamEnv : Env := {
  structs := [("Pt", [("x", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["helper", "main"]
}

def w502NonMainHelperStructParamHelper : Function := {
  name := "helper",
  params := [("p", .struct "Pt")],
  ret := some .u32,
  body := [.return_ (some (.fieldAccess (.identifier "p") "x"))]
}

def w502NonMainHelperStructParamMain : Function := {
  name := "main",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.call "helper" [.structLit "Pt" [("x", .intLit 5)]]))]
}

def w502NonMainHelperStructParamModule : Module := {
  name := "w502_non_main_helper_struct_param",
  imports := [],
  globals := [],
  functions := [w502NonMainHelperStructParamHelper, w502NonMainHelperStructParamMain],
  tests := [],
  benches := []
}

/-- W502-D: module with multiple non-`main` entry points. -/
def w502MultipleNonMainEntriesEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["a", "b", "main"]
}

def w502MultipleNonMainEntriesA : Function := {
  name := "a",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.intLit 11))]
}

def w502MultipleNonMainEntriesB : Function := {
  name := "b",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.intLit 13))]
}

def w502MultipleNonMainEntriesMain : Function := {
  name := "main",
  params := [],
  ret := some .u32,
  body := [.return_ (some (.binop "+" (.call "a" []) (.call "b" [])))]
}

def w502MultipleNonMainEntriesModule : Module := {
  name := "w502_multiple_non_main_entries",
  imports := [],
  globals := [],
  functions := [w502MultipleNonMainEntriesA, w502MultipleNonMainEntriesB, w502MultipleNonMainEntriesMain],
  tests := [],
  benches := []
}

/-- W503: environment for the conditional-return witness. -/
def w503IfReturnEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["pick"]
}

def w503IfReturnPick : Function := {
  name := "pick",
  params := [("flag", .bool)],
  ret := some .u32,
  body := [
    .ifThenElse (.identifier "flag")
      [.return_ (some (.intLit 7))]
      [.return_ (some (.intLit 11))]
  ]
}

def w503IfReturnModule : Module := {
  name := "w503_if_return",
  imports := [],
  globals := [],
  functions := [w503IfReturnPick],
  tests := [],
  benches := []
}

/-- W503: environment for the bounded for-loop accumulator witness. -/
def w503ForAccumulatorEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["sum_three"]
}

def w503ForAccumulatorSumThree : Function := {
  name := "sum_three",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "acc" .u32 (some (.intLit 0)),
    .forLoop "i" (.intLit 3) [
      .assign (.identifier "acc") (.binop "+" (.identifier "acc") (.intLit 1))
    ],
    .return_ (some (.identifier "acc"))
  ]
}

def w503ForAccumulatorModule : Module := {
  name := "w503_for_accumulator",
  imports := [],
  globals := [],
  functions := [w503ForAccumulatorSumThree],
  tests := [],
  benches := []
}

/-- W504: environment for the bounded for-loop with parameter witness. -/
def w504ForSumEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["sum_n"]
}

def w504ForSumSumN : Function := {
  name := "sum_n",
  params := [("n", .u32)],
  ret := some .u32,
  body := [
    .varDecl "acc" .u32 (some (.intLit 0)),
    .forLoop "i" (.identifier "n") [
      .assign (.identifier "acc") (.binop "+" (.identifier "acc") (.identifier "i"))
    ],
    .return_ (some (.identifier "acc"))
  ]
}

def w504ForSumModule : Module := {
  name := "w504_for_sum",
  imports := [],
  globals := [],
  functions := [w504ForSumSumN],
  tests := [],
  benches := []
}

/- W505 witness environments and modules: adversarial sequential constructs. -/

/-- W505-A: environment for nested `ifThenElse` with four return arms. -/
def w505NestedIfEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["classify"]
}

def w505NestedIfClassify : Function := {
  name := "classify",
  params := [("x", .u32)],
  ret := some .u32,
  body := [
    .ifThenElse (.binop "<" (.identifier "x") (.intLit 5))
      [
        .ifThenElse (.binop "<" (.identifier "x") (.intLit 2))
          [.return_ (some (.intLit 1))]
          [.return_ (some (.intLit 2))]
      ]
      [
        .ifThenElse (.binop "<" (.identifier "x") (.intLit 8))
          [.return_ (some (.intLit 3))]
          [.return_ (some (.intLit 4))]
      ]
  ]
}

def w505NestedIfModule : Module := {
  name := "w505_nested_if",
  imports := [],
  globals := [],
  functions := [w505NestedIfClassify],
  tests := [],
  benches := []
}

/-- W505-B: environment for `ifThenElse` inside a bounded `forLoop`. -/
def w505IfInForEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["conditional_sum"]
}

def w505IfInForConditionalSum : Function := {
  name := "conditional_sum",
  params := [("x", .u32), ("y", .u32), ("z", .u32)],
  ret := some .u32,
  body := [
    .varDecl "acc" .u32 (some (.intLit 0)),
    .forLoop "i" (.intLit 4) [
      .ifThenElse (.binop ">" (.identifier "x") (.identifier "y"))
        [.assign (.identifier "acc") (.binop "+" (.identifier "acc") (.identifier "z"))]
        [.assign (.identifier "acc") (.binop "+" (.identifier "acc") (.intLit 1))]
    ],
    .return_ (some (.identifier "acc"))
  ]
}

def w505IfInForModule : Module := {
  name := "w505_if_in_for",
  imports := [],
  globals := [],
  functions := [w505IfInForConditionalSum],
  tests := [],
  benches := []
}

/-- W505-C: environment for bounded `forLoop` whose range is a parameter. -/
def w505ForVarRangeEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["sum_range"]
}

def w505ForVarRangeSumRange : Function := {
  name := "sum_range",
  params := [("n", .u32)],
  ret := some .u32,
  body := [
    .varDecl "acc" .u32 (some (.intLit 0)),
    .forLoop "i" (.identifier "n") [
      .assign (.identifier "acc") (.binop "+" (.identifier "acc") (.identifier "i"))
    ],
    .return_ (some (.identifier "acc"))
  ]
}

def w505ForVarRangeModule : Module := {
  name := "w505_for_var_range",
  imports := [],
  globals := [],
  functions := [w505ForVarRangeSumRange],
  tests := [],
  benches := []
}

/-- W505-D: environment for bounded `forLoop` used to compute a return value. -/
def w505ForReturnEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["factorial"]
}

def w505ForReturnFactorial : Function := {
  name := "factorial",
  params := [("n", .u32)],
  ret := some .u32,
  body := [
    .varDecl "acc" .u32 (some (.intLit 1)),
    .forLoop "i" (.binop "+" (.identifier "n") (.intLit 1)) [
      .assign (.identifier "acc") (.binop "*" (.identifier "acc") (.identifier "i"))
    ],
    .return_ (some (.identifier "acc"))
  ]
}

def w505ForReturnModule : Module := {
  name := "w505_for_return",
  imports := [],
  globals := [],
  functions := [w505ForReturnFactorial],
  tests := [],
  benches := []
}

/-- W505-E: environment for bounded `forLoop` with a locally-declared loop body variable. -/
def w505ForLocalVarInitEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["fill_init"]
}

def w505ForLocalVarInitFillInit : Function := {
  name := "fill_init",
  params := [("n", .u32)],
  ret := some .u32,
  body := [
    .varDecl "table" .u32 (some (.intLit 0)),
    .forLoop "i" (.identifier "n") [
      .varDecl "slot" .u32 (some (.binop "*" (.identifier "i") (.intLit 7))),
      .assign (.identifier "table") (.binop "+" (.identifier "table") (.identifier "slot"))
    ],
    .return_ (some (.identifier "table"))
  ]
}

def w505ForLocalVarInitModule : Module := {
  name := "w505_for_local_var_init",
  imports := [],
  globals := [],
  functions := [w505ForLocalVarInitFillInit],
  tests := [],
  benches := []
}

end Trinity.IcarusLowerable
