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


/-- W506: environment for statement-level `switch` dispatch on a scalar. -/
def w506SwitchEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["main"]
}

/-- W506: function that uses `Stmt.switch` to dispatch on a `u32` parameter. -/
def w506SwitchMain : Function := {
  name := "main",
  params := [("sel", .u32)],
  ret := some .u32,
  body := [
    .varDecl "r" .u32 (some (.intLit 0)),
    .switch (.identifier "sel")
      [
        (.intLit 0, [.assign (.identifier "r") (.intLit 10)]),
        (.intLit 1, [.assign (.identifier "r") (.intLit 20)])
      ]
      [.assign (.identifier "r") (.intLit 99)],
    .return_ (some (.identifier "r"))
  ]
}

/-- W506: module containing the statement-level switch witness. -/
def w506SwitchModule : Module := {
  name := "w506_switch",
  imports := [],
  globals := [],
  functions := [w506SwitchMain],
  tests := [],
  benches := []
}

/- W507 witness environments and modules: bounded `while` loops. -/

/-- W507-A: environment for the bounded while-loop counter witness. -/
def w507WhileCounterEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["count_to"]
}

/-- W507-A: function that counts up to its parameter using a `while` loop. -/
def w507WhileCounterCountTo : Function := {
  name := "count_to",
  params := [("n", .u32)],
  ret := some .u32,
  body := [
    .varDecl "i" .u32 (some (.intLit 0)),
    .varDecl "acc" .u32 (some (.intLit 0)),
    .whileLoop (.binop "<" (.identifier "i") (.identifier "n")) [
      .assign (.identifier "acc") (.binop "+" (.identifier "acc") (.intLit 1)),
      .assign (.identifier "i") (.binop "+" (.identifier "i") (.intLit 1))
    ],
    .return_ (some (.identifier "acc"))
  ]
}

/-- W507-A: module containing the while-loop counter witness. -/
def w507WhileCounterModule : Module := {
  name := "w507_while_counter",
  imports := [],
  globals := [],
  functions := [w507WhileCounterCountTo],
  tests := [],
  benches := []
}

/-- W507-B: environment for the while-loop linear-search witness. -/
def w507WhileSearchEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["find_index"]
}

/-- W507-B: function that scans a fixed array with a `while` loop. -/
def w507WhileSearchFindIndex : Function := {
  name := "find_index",
  params := [("target", .u32)],
  ret := some .u32,
  body := [
    .varDecl "arr" (.array 5 .u32)
      (some (.arrayLit (.array 5 .u32) [.intLit 3, .intLit 7, .intLit 1, .intLit 9, .intLit 2])),
    .varDecl "i" .u32 (some (.intLit 0)),
    .varDecl "found" .u32 (some (.intLit 5)),
    .whileLoop (.binop "<" (.identifier "i") (.intLit 5)) [
      .ifThenElse (.binop "==" (.index (.identifier "arr") (.identifier "i")) (.identifier "target"))
        [.assign (.identifier "found") (.identifier "i")]
        [],
      .assign (.identifier "i") (.binop "+" (.identifier "i") (.intLit 1))
    ],
    .return_ (some (.identifier "found"))
  ]
}

/-- W507-B: module containing the while-loop search witness. -/
def w507WhileSearchModule : Module := {
  name := "w507_while_search",
  imports := [],
  globals := [],
  functions := [w507WhileSearchFindIndex],
  tests := [],
  benches := []
}

/-- W507-C: environment for the nested while-inside-for witness. -/
def w507WhileNestedEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["nested_sum"]
}

/-- W507-C: function that nests a `while` loop inside a bounded `for` loop. -/
def w507WhileNestedNestedSum : Function := {
  name := "nested_sum",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "acc" .u32 (some (.intLit 0)),
    .forLoop "outer" (.intLit 4) [
      .varDecl "inner" .u32 (some (.intLit 0)),
      .whileLoop (.binop "<" (.identifier "inner") (.identifier "outer")) [
        .assign (.identifier "acc") (.binop "+" (.identifier "acc") (.intLit 1)),
        .assign (.identifier "inner") (.binop "+" (.identifier "inner") (.intLit 1))
      ]
    ],
    .return_ (some (.identifier "acc"))
  ]
}

/-- W507-C: module containing the nested while-loop witness. -/
def w507WhileNestedModule : Module := {
  name := "w507_while_nested",
  imports := [],
  globals := [],
  functions := [w507WhileNestedNestedSum],
  tests := [],
  benches := []
}

/- W508 witness environments and modules: `break`/`continue` in bounded loops. -/

/-- W508-A: environment for the `break` inside a `while` loop witness. -/
def w508BreakSearchEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["find_target"]
}

/-- W508-A: function that breaks out of a `while` loop when a target is found. -/
def w508BreakSearchFindTarget : Function := {
  name := "find_target",
  params := [("target", .u32)],
  ret := some .u32,
  body := [
    .varDecl "arr" (.array 5 .u32)
      (some (.arrayLit (.array 5 .u32) [.intLit 3, .intLit 7, .intLit 1, .intLit 9, .intLit 2])),
    .varDecl "i" .u32 (some (.intLit 0)),
    .varDecl "found" .u32 (some (.intLit 5)),
    .whileLoop (.binop "<" (.identifier "i") (.intLit 5)) [
      .ifThenElse (.binop "==" (.index (.identifier "arr") (.identifier "i")) (.identifier "target"))
        [.assign (.identifier "found") (.identifier "i"), .break]
        [],
      .assign (.identifier "i") (.binop "+" (.identifier "i") (.intLit 1))
    ],
    .return_ (some (.identifier "found"))
  ]
}

/-- W508-A: module containing the while-break witness. -/
def w508BreakSearchModule : Module := {
  name := "w508_break_search",
  imports := [],
  globals := [],
  functions := [w508BreakSearchFindTarget],
  tests := [],
  benches := []
}

/-- W508-B: environment for the `continue` inside a `for` loop witness. -/
def w508ContinueSumEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["sum_odd"]
}

/-- W508-B: function that skips even iterations with `continue`. -/
def w508ContinueSumSumOdd : Function := {
  name := "sum_odd",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "acc" .u32 (some (.intLit 0)),
    .forLoop "i" (.intLit 10) [
      .ifThenElse (.binop "==" (.binop "%" (.identifier "i") (.intLit 2)) (.intLit 0))
        [.continue]
        [],
      .assign (.identifier "acc") (.binop "+" (.identifier "acc") (.identifier "i"))
    ],
    .return_ (some (.identifier "acc"))
  ]
}

/-- W508-B: module containing the for-continue witness. -/
def w508ContinueSumModule : Module := {
  name := "w508_continue_sum",
  imports := [],
  globals := [],
  functions := [w508ContinueSumSumOdd],
  tests := [],
  benches := []
}

/-- W508-C: environment for the `break` out of a nested loop witness. -/
def w508BreakNestedEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["find_pair"]
}

/-- W508-C: function that breaks from an inner `while` loop nested in a `for`. -/
def w508BreakNestedFindPair : Function := {
  name := "find_pair",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "result" .u32 (some (.intLit 0)),
    .forLoop "i" (.intLit 5) [
      .varDecl "j" .u32 (some (.intLit 0)),
      .whileLoop (.binop "<" (.identifier "j") (.intLit 5)) [
        .ifThenElse (.binop "==" (.binop "+" (.binop "*" (.identifier "i") (.intLit 5)) (.identifier "j")) (.intLit 13))
          [.assign (.identifier "result") (.identifier "j"), .break]
          [],
        .assign (.identifier "j") (.binop "+" (.identifier "j") (.intLit 1))
      ]
    ],
    .return_ (some (.identifier "result"))
  ]
}

/-- W508-C: module containing the nested-break witness. -/
def w508BreakNestedModule : Module := {
  name := "w508_break_nested",
  imports := [],
  globals := [],
  functions := [w508BreakNestedFindPair],
  tests := [],
  benches := []
}

/-- W508-A: a `break` inside a bounded `while` loop is lowerable. -/
theorem w508_break_search_lowerable :
  Module.isLowerable w508BreakSearchEnv w508BreakSearchModule := by
  native_decide

/-- W508-B: a `continue` inside a bounded `for` loop is lowerable. -/
theorem w508_continue_sum_lowerable :
  Module.isLowerable w508ContinueSumEnv w508ContinueSumModule := by
  native_decide

/-- W508-C: a `break` from a nested `while` loop is lowerable. -/
theorem w508_break_nested_lowerable :
  Module.isLowerable w508BreakNestedEnv w508BreakNestedModule := by
  native_decide

/- W509 witness environments and modules: direct lowering of array-typed
   struct fields as packed vectors. -/

/-- W509-A: environment for a local scalar struct with an array-typed field. -/
def w509ArrayFieldDirectEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u8), ("tag", .u8)]),
              ("Pt2", [("grid", .array 2 (.array 3 .u8)), ("tag", .u8)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["sum_local_pt", "sum_2d_local_pt"]
}

/-- W509-A: sum the elements of a 1-D array field of a local struct variable. -/
def w509ArrayFieldDirectSumLocalPt : Function := {
  name := "sum_local_pt",
  params := [],
  ret := some .u8,
  body := [
    .varDecl "p" (.struct "Pt")
      (some (.structLit "Pt" [
        ("coords", .arrayLit (.array 3 .u8) [.intLit 1, .intLit 2, .intLit 3]),
        ("tag", .intLit 7)
      ])),
    .varDecl "s" .u8 (some (.intLit 0)),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 0))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 1))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 2))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.fieldAccess (.identifier "p") "tag")),
    .return_ (some (.identifier "s"))
  ]
}

/-- W509-A: sum selected elements of a 2-D array field of a local struct variable. -/
def w509ArrayFieldDirectSum2DLocalPt : Function := {
  name := "sum_2d_local_pt",
  params := [],
  ret := some .u8,
  body := [
    .varDecl "p" (.struct "Pt2")
      (some (.structLit "Pt2" [
        ("grid", .arrayLit (.array 2 (.array 3 .u8)) [
          .arrayLit (.array 3 .u8) [.intLit 1, .intLit 2, .intLit 3],
          .arrayLit (.array 3 .u8) [.intLit 4, .intLit 5, .intLit 6]
        ]),
        ("tag", .intLit 1)
      ])),
    .varDecl "s" .u8 (some (.intLit 0)),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "p") "grid") (.intLit 0)) (.intLit 0))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "p") "grid") (.intLit 0)) (.intLit 2))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "p") "grid") (.intLit 1)) (.intLit 1))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.fieldAccess (.identifier "p") "tag")),
    .return_ (some (.identifier "s"))
  ]
}

def w509ArrayFieldDirectModule : Module := {
  name := "w509_array_field_direct",
  imports := [],
  globals := [],
  functions := [w509ArrayFieldDirectSumLocalPt, w509ArrayFieldDirectSum2DLocalPt],
  tests := [],
  benches := []
}

/-- W509-A: local scalar struct with 1-D and 2-D array fields is lowerable. -/
theorem w509_array_field_direct_lowerable :
  Module.isLowerable w509ArrayFieldDirectEnv w509ArrayFieldDirectModule := by
  native_decide

/-- W509-B: environment for a scalar struct with an array-typed field passed as a
    packed-vector parameter. -/
def w509ArrayFieldParamEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u8), ("tag", .u8)]),
              ("Pt2", [("grid", .array 2 (.array 3 .u8)), ("tag", .u8)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["sum_pt", "sum_2d_pt"]
}

/-- W509-B: sum the elements of an array field received as a packed struct param. -/
def w509ArrayFieldParamSumPt : Function := {
  name := "sum_pt",
  params := [("p", .struct "Pt")],
  ret := some .u8,
  body := [
    .varDecl "s" .u8 (some (.intLit 0)),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 0))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 1))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 2))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.fieldAccess (.identifier "p") "tag")),
    .return_ (some (.identifier "s"))
  ]
}

/-- W509-B: sum selected elements of a 2-D array field received as a packed struct param. -/
def w509ArrayFieldParamSum2DPt : Function := {
  name := "sum_2d_pt",
  params := [("p", .struct "Pt2")],
  ret := some .u8,
  body := [
    .varDecl "s" .u8 (some (.intLit 0)),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "p") "grid") (.intLit 0)) (.intLit 0))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "p") "grid") (.intLit 0)) (.intLit 2))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "p") "grid") (.intLit 1)) (.intLit 1))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.fieldAccess (.identifier "p") "tag")),
    .return_ (some (.identifier "s"))
  ]
}

def w509ArrayFieldParamModule : Module := {
  name := "w509_array_field_param",
  imports := [],
  globals := [],
  functions := [w509ArrayFieldParamSumPt, w509ArrayFieldParamSum2DPt],
  tests := [],
  benches := []
}

/-- W509-B: packed-vector passing of a scalar struct with array-typed fields is
    lowerable. -/
theorem w509_array_field_param_lowerable :
  Module.isLowerable w509ArrayFieldParamEnv w509ArrayFieldParamModule := by
  native_decide

/-- W509-C: environment for a scalar struct with an array-typed field returned from
    a function as a packed vector. -/
def w509ArrayFieldReturnEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u8), ("tag", .u8)]),
              ("Pt2", [("grid", .array 2 (.array 3 .u8)), ("tag", .u8)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_pt", "sum_returned_pt", "make_pt2", "sum_returned_pt2"]
}

/-- W509-C: construct and return a scalar struct with a 1-D array field. -/
def w509ArrayFieldReturnMakePt : Function := {
  name := "make_pt",
  params := [("x", .u8), ("y", .u8), ("z", .u8), ("t", .u8)],
  ret := some (.struct "Pt"),
  body := [
    .return_ (some (.structLit "Pt" [
      ("coords", .arrayLit (.array 3 .u8) [.identifier "x", .identifier "y", .identifier "z"]),
      ("tag", .identifier "t")
    ]))
  ]
}

/-- W509-C: read the array field of a returned struct. -/
def w509ArrayFieldReturnSumReturnedPt : Function := {
  name := "sum_returned_pt",
  params := [],
  ret := some .u8,
  body := [
    .varDecl "p" (.struct "Pt")
      (some (.call "make_pt" [.intLit 1, .intLit 2, .intLit 3, .intLit 7])),
    .varDecl "s" .u8 (some (.intLit 0)),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 0))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 1))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 2))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.fieldAccess (.identifier "p") "tag")),
    .return_ (some (.identifier "s"))
  ]
}

/-- W509-C: construct and return a scalar struct with a 2-D array field. -/
def w509ArrayFieldReturnMakePt2 : Function := {
  name := "make_pt2",
  params := [("a", .u8), ("b", .u8), ("t", .u8)],
  ret := some (.struct "Pt2"),
  body := [
    .return_ (some (.structLit "Pt2" [
      ("grid", .arrayLit (.array 2 (.array 3 .u8)) [
        .arrayLit (.array 3 .u8) [.identifier "a", .binop "+" (.identifier "a") (.intLit 1), .binop "+" (.identifier "a") (.intLit 2)],
        .arrayLit (.array 3 .u8) [.identifier "b", .binop "+" (.identifier "b") (.intLit 1), .binop "+" (.identifier "b") (.intLit 2)]
      ]),
      ("tag", .identifier "t")
    ]))
  ]
}

/-- W509-C: read the 2-D array field of a returned struct. -/
def w509ArrayFieldReturnSumReturnedPt2 : Function := {
  name := "sum_returned_pt2",
  params := [],
  ret := some .u8,
  body := [
    .varDecl "p" (.struct "Pt2")
      (some (.call "make_pt2" [.intLit 1, .intLit 4, .intLit 2])),
    .varDecl "s" .u8 (some (.intLit 0)),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "p") "grid") (.intLit 0)) (.intLit 0))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "p") "grid") (.intLit 0)) (.intLit 2))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "p") "grid") (.intLit 1)) (.intLit 1))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.fieldAccess (.identifier "p") "tag")),
    .return_ (some (.identifier "s"))
  ]
}

def w509ArrayFieldReturnModule : Module := {
  name := "w509_array_field_return",
  imports := [],
  globals := [],
  functions := [w509ArrayFieldReturnMakePt, w509ArrayFieldReturnSumReturnedPt,
                w509ArrayFieldReturnMakePt2, w509ArrayFieldReturnSumReturnedPt2],
  tests := [],
  benches := []
}

/-- W509-C: returning a scalar struct with array-typed fields as a packed vector is
    lowerable. -/
theorem w509_array_field_return_lowerable :
  Module.isLowerable w509ArrayFieldReturnEnv w509ArrayFieldReturnModule := by
  native_decide

/- W510 witness environments and modules: element-level writes into packed
   scalar-array fields of local struct variables. -/

/-- W510-A: environment for a variable-index write into a 1-D scalar-array field. -/
def w510ArrayFieldWriteVarIndexEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["write_and_read"],
  vars := [("p", .struct "Pt")]
}

/-- W510-A: write a variable-index element of a packed 1-D array field and read it back. -/
def w510ArrayFieldWriteVarIndexFn : Function := {
  name := "write_and_read",
  params := [("i", .u8)],
  ret := some .u32,
  body := [
    .varDecl "p" (.struct "Pt")
      (some (.structLit "Pt" [
        ("coords", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
      ])),
    .assign
      (.index (.fieldAccess (.identifier "p") "coords") (.identifier "i"))
      (.intLit 99),
    .return_ (some
      (.index (.fieldAccess (.identifier "p") "coords") (.identifier "i")))
  ]
}

def w510ArrayFieldWriteVarIndexModule : Module := {
  name := "w510_array_field_write_var_index",
  imports := [],
  globals := [],
  functions := [w510ArrayFieldWriteVarIndexFn],
  tests := [],
  benches := []
}

/-- W510-A: variable-index write into a packed 1-D array field is lowerable. -/
theorem w510_array_field_write_var_index_lowerable :
  Module.isLowerable w510ArrayFieldWriteVarIndexEnv w510ArrayFieldWriteVarIndexModule := by
  native_decide

/-- W510-B: environment for a variable-index sub-array (row) write into a 2-D
    scalar-array field. -/
def w510ArrayFieldWrite2DSliceEnv : Env := {
  structs := [("Grid", [("cells", .array 3 (.array 4 .u32))])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["write_row"],
  vars := [("g", .struct "Grid")]
}

/-- W510-B: write a whole row of a packed 2-D array field with a variable outer
    index, then sum the row. -/
def w510ArrayFieldWrite2DSliceFn : Function := {
  name := "write_row",
  params := [("i", .u8)],
  ret := some .u32,
  body := [
    .varDecl "g" (.struct "Grid")
      (some (.structLit "Grid" [
        ("cells", .arrayLit (.array 3 (.array 4 .u32)) [
          .arrayLit (.array 4 .u32) [.intLit 1, .intLit 2, .intLit 3, .intLit 4],
          .arrayLit (.array 4 .u32) [.intLit 5, .intLit 6, .intLit 7, .intLit 8],
          .arrayLit (.array 4 .u32) [.intLit 9, .intLit 10, .intLit 11, .intLit 12]
        ])
      ])),
    .assign
      (.index (.fieldAccess (.identifier "g") "cells") (.identifier "i"))
      (.arrayLit (.array 4 .u32) [.intLit 0, .intLit 0, .intLit 0, .intLit 0]),
    .varDecl "s" .u32 (some (.intLit 0)),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "g") "cells") (.identifier "i")) (.intLit 0))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "g") "cells") (.identifier "i")) (.intLit 1))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "g") "cells") (.identifier "i")) (.intLit 2))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "g") "cells") (.identifier "i")) (.intLit 3))),
    .return_ (some (.identifier "s"))
  ]
}

def w510ArrayFieldWrite2DSliceModule : Module := {
  name := "w510_array_field_write_2d_slice",
  imports := [],
  globals := [],
  functions := [w510ArrayFieldWrite2DSliceFn],
  tests := [],
  benches := []
}

/-- W510-B: variable-index row write into a packed 2-D array field is lowerable. -/
theorem w510_array_field_write_2d_slice_lowerable :
  Module.isLowerable w510ArrayFieldWrite2DSliceEnv w510ArrayFieldWrite2DSliceModule := by
  native_decide

/-- W510-C: environment for a struct-return that has had an array-field element
    mutated in place. -/
def w510ArrayFieldWriteReturnCopyEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["mutate", "check"],
  vars := [("p", .struct "Pt")]
}

/-- W510-C: mutate a packed array field of a local struct and return the struct. -/
def w510ArrayFieldWriteReturnCopyMutate : Function := {
  name := "mutate",
  params := [],
  ret := some (.struct "Pt"),
  body := [
    .varDecl "p" (.struct "Pt")
      (some (.structLit "Pt" [
        ("coords", .arrayLit (.array 3 .u32) [.intLit 1, .intLit 2, .intLit 3])
      ])),
    .assign
      (.index (.fieldAccess (.identifier "p") "coords") (.intLit 1))
      (.intLit 42),
    .return_ (some (.identifier "p"))
  ]
}

/-- W510-C: read the mutated element from the returned packed struct. -/
def w510ArrayFieldWriteReturnCopyCheck : Function := {
  name := "check",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.index (.fieldAccess (.call "mutate" []) "coords") (.intLit 1)))
  ]
}

def w510ArrayFieldWriteReturnCopyModule : Module := {
  name := "w510_array_field_write_return_copy",
  imports := [],
  globals := [],
  functions := [w510ArrayFieldWriteReturnCopyMutate, w510ArrayFieldWriteReturnCopyCheck],
  tests := [],
  benches := []
}

/-- W510-C: in-place mutation of a packed array field before struct return is
    lowerable. -/
theorem w510_array_field_write_return_copy_lowerable :
  Module.isLowerable w510ArrayFieldWriteReturnCopyEnv w510ArrayFieldWriteReturnCopyModule := by
  native_decide

/- W511 witness environments and modules: module-level scalar structs with
   fixed-size scalar array fields are lowered to packed vectors, matching the
   W509/W510 function-local and parameter/return cases. -/

/-- W511-A: environment for a module-level packed scalar struct var with a 1-D
    array-typed field. -/
def w511ModuleArrayFieldReadEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["read_coord"],
  vars := [("g_p", .struct "Pt")]
}

/-- W511-A: read a variable-index element of a module-level packed array field. -/
def w511ModuleArrayFieldReadCoord : Function := {
  name := "read_coord",
  params := [("i", .u8)],
  ret := some .u32,
  body := [
    .return_ (some
      (.index (.fieldAccess (.identifier "g_p") "coords") (.identifier "i")))
  ]
}

def w511ModuleArrayFieldReadModule : Module := {
  name := "w511_module_array_field_read",
  imports := [],
  globals := [
    .varDecl "g_p" (.struct "Pt")
      (some (.structLit "Pt" [
        ("coords", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
      ]))
  ],
  functions := [w511ModuleArrayFieldReadCoord],
  tests := [],
  benches := []
}

/-- W511-A: module-level packed scalar struct var with a 1-D array field is
    lowerable. -/
theorem w511_module_array_field_read_lowerable :
  Module.isLowerable w511ModuleArrayFieldReadEnv w511ModuleArrayFieldReadModule := by
  native_decide

/-- W511-B: environment for a module-level packed scalar struct var with a 2-D
    array-typed field initialized from a struct literal. -/
def w511ModuleArrayFieldInitEnv : Env := {
  structs := [("Grid", [("cells", .array 3 (.array 4 .u32))])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["sum_row"],
  vars := [("g_g", .struct "Grid")]
}

/-- W511-B: sum a row of a module-level packed 2-D array field. -/
def w511ModuleArrayFieldInitSumRow : Function := {
  name := "sum_row",
  params := [("i", .u8)],
  ret := some .u32,
  body := [
    .varDecl "s" .u32 (some (.intLit 0)),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "g_g") "cells") (.identifier "i")) (.intLit 0))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "g_g") "cells") (.identifier "i")) (.intLit 1))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "g_g") "cells") (.identifier "i")) (.intLit 2))),
    .assign (.identifier "s") (.binop "+" (.identifier "s")
      (.index (.index (.fieldAccess (.identifier "g_g") "cells") (.identifier "i")) (.intLit 3))),
    .return_ (some (.identifier "s"))
  ]
}

def w511ModuleArrayFieldInitModule : Module := {
  name := "w511_module_array_field_init",
  imports := [],
  globals := [
    .varDecl "g_g" (.struct "Grid")
      (some (.structLit "Grid" [
        ("cells", .arrayLit (.array 3 (.array 4 .u32)) [
          .arrayLit (.array 4 .u32) [.intLit 1, .intLit 2, .intLit 3, .intLit 4],
          .arrayLit (.array 4 .u32) [.intLit 5, .intLit 6, .intLit 7, .intLit 8],
          .arrayLit (.array 4 .u32) [.intLit 9, .intLit 10, .intLit 11, .intLit 12]
        ])
      ]))
  ],
  functions := [w511ModuleArrayFieldInitSumRow],
  tests := [],
  benches := []
}

/-- W511-B: module-level packed scalar struct var with a 2-D array field is
    lowerable. -/
theorem w511_module_array_field_init_lowerable :
  Module.isLowerable w511ModuleArrayFieldInitEnv w511ModuleArrayFieldInitModule := by
  native_decide

/-- W511-C: environment for whole-struct copy between module-level packed scalar
    struct vars. -/
def w511ModuleArrayFieldCopyEnv : Env := {
  structs := [("Pt", [("coords", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["copy_and_check"],
  vars := [("g_src", .struct "Pt"), ("g_dst", .struct "Pt")]
}

/-- W511-C: copy one module-level packed struct var to another, then read back an
    array element. -/
def w511ModuleArrayFieldCopyAndCheck : Function := {
  name := "copy_and_check",
  params := [("i", .u8)],
  ret := some .u32,
  body := [
    .assign (.identifier "g_dst") (.identifier "g_src"),
    .return_ (some
      (.index (.fieldAccess (.identifier "g_dst") "coords") (.identifier "i")))
  ]
}

def w511ModuleArrayFieldCopyModule : Module := {
  name := "w511_module_array_field_copy",
  imports := [],
  globals := [
    .varDecl "g_src" (.struct "Pt")
      (some (.structLit "Pt" [
        ("coords", .arrayLit (.array 3 .u32) [.intLit 7, .intLit 8, .intLit 9])
      ])),
    .varDecl "g_dst" (.struct "Pt") none
  ],
  functions := [w511ModuleArrayFieldCopyAndCheck],
  tests := [],
  benches := []
}

/-- W511-C: whole-struct assignment between module-level packed scalar struct
    vars is lowerable. -/
theorem w511_module_array_field_copy_lowerable :
  Module.isLowerable w511ModuleArrayFieldCopyEnv w511ModuleArrayFieldCopyModule := by
  native_decide

/- W512 witness environments and modules: arrays of structs whose element struct
   has fixed-size scalar array fields, emitted as packed vectors per element. -/

/-- W512-A: environment for reading scalar and array-typed fields of a packed AOS
    parameter. -/
def w512AosReadEnv : Env := {
  structs := [("S", [("tag", .u32), ("vals", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["read_tag", "read_val"]
}

/-- W512-A: read the scalar field of a packed AOS element. -/
def w512AosReadTag : Function := {
  name := "read_tag",
  params := [("arr", .array 2 (.struct "S")), ("i", .u8)],
  ret := some .u32,
  body := [
    .return_ (some
      (.fieldAccess
        (.index (.identifier "arr") (.identifier "i"))
        "tag"))
  ]
}

/-- W512-A: read an element of an array-typed field inside a packed AOS element. -/
def w512AosReadVal : Function := {
  name := "read_val",
  params := [("arr", .array 2 (.struct "S")), ("i", .u8), ("j", .u8)],
  ret := some .u32,
  body := [
    .return_ (some
      (.index
        (.fieldAccess
          (.index (.identifier "arr") (.identifier "i"))
          "vals")
        (.identifier "j")))
  ]
}

def w512AosReadModule : Module := {
  name := "w512_aos_read",
  imports := [],
  globals := [],
  functions := [w512AosReadTag, w512AosReadVal],
  tests := [],
  benches := []
}

/-- W512-B: environment for writing a scalar field of a packed AOS local variable
    and reading it back.  The shallow model treats assignments to non-identifier
    LHS as no-ops, so the value-preservation theorem checks that both sides agree
    on the same (initial) value, matching the W510 element-write witnesses. -/
def w512AosWriteEnv : Env := {
  structs := [("S", [("tag", .u32), ("vals", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["overwrite_and_read"]
}

/-- W512-B: declare a packed AOS local, assign to arr[0].tag, and read it back. -/
def w512AosWriteOverwriteAndRead : Function := {
  name := "overwrite_and_read",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "arr" (.array 2 (.struct "S"))
      (some (.arrayLit (.array 2 (.struct "S")) [
        .structLit "S" [
          ("tag", .intLit 1),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
        ],
        .structLit "S" [
          ("tag", .intLit 2),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 40, .intLit 50, .intLit 60])
        ]
      ])),
    .assign
      (.fieldAccess (.index (.identifier "arr") (.intLit 0)) "tag")
      (.intLit 7),
    .return_ (some
      (.fieldAccess (.index (.identifier "arr") (.intLit 0)) "tag"))
  ]
}

def w512AosWriteModule : Module := {
  name := "w512_aos_write",
  imports := [],
  globals := [],
  functions := [w512AosWriteOverwriteAndRead],
  tests := [],
  benches := []
}

/-- W512-C: environment for returning an array of structs from a function and
    reading an array-typed field of one element. -/
def w512AosReturnEnv : Env := {
  structs := [("S", [("tag", .u32), ("vals", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_arr", "read_returned"]
}

/-- W512-C: construct and return a packed array of structs. -/
def w512AosReturnMakeArr : Function := {
  name := "make_arr",
  params := [],
  ret := some (.array 2 (.struct "S")),
  body := [
    .return_ (some (.arrayLit (.array 2 (.struct "S")) [
      .structLit "S" [
        ("tag", .intLit 1),
        ("vals", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
      ],
      .structLit "S" [
        ("tag", .intLit 2),
        ("vals", .arrayLit (.array 3 .u32) [.intLit 40, .intLit 50, .intLit 60])
      ]
    ]))
  ]
}

/-- W512-C: read arr[1].vals[2] from the returned packed AOS. -/
def w512AosReturnReadReturned : Function := {
  name := "read_returned",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.index
        (.fieldAccess
          (.index (.call "make_arr" []) (.intLit 1))
          "vals")
        (.intLit 2)))
  ]
}

def w512AosReturnModule : Module := {
  name := "w512_aos_return",
  imports := [],
  globals := [],
  functions := [w512AosReturnMakeArr, w512AosReturnReadReturned],
  tests := [],
  benches := []
}

/- W513 witness environments and modules: function-local packed-element arrays of
   structs with fixed-size scalar array fields. -/

/-- W513-A: environment for reading scalar and array-typed fields of a
    function-local packed AOS. -/
def w513LocalAosReadEnv : Env := {
  structs := [("S", [("tag", .u32), ("vals", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["read_fixed", "read_indexed"]
}

/-- W513-A: read the scalar field of a function-local packed AOS element. -/
def w513LocalAosReadFixed : Function := {
  name := "read_fixed",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "arr" (.array 2 (.struct "S"))
      (some (.arrayLit (.array 2 (.struct "S")) [
        .structLit "S" [
          ("tag", .intLit 1),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
        ],
        .structLit "S" [
          ("tag", .intLit 2),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 40, .intLit 50, .intLit 60])
        ]
      ])),
    .return_ (some
      (.binop "+"
        (.fieldAccess (.index (.identifier "arr") (.intLit 0)) "tag")
        (.index
          (.fieldAccess (.index (.identifier "arr") (.intLit 1)) "vals")
          (.intLit 1))))
  ]
}

/-- W513-A: read an array-typed field element from a function-local packed AOS
    using variable outer/inner indices. -/
def w513LocalAosReadIndexed : Function := {
  name := "read_indexed",
  params := [("i", .u8), ("j", .u8)],
  ret := some .u32,
  body := [
    .varDecl "arr" (.array 2 (.struct "S"))
      (some (.arrayLit (.array 2 (.struct "S")) [
        .structLit "S" [
          ("tag", .intLit 1),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
        ],
        .structLit "S" [
          ("tag", .intLit 2),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 40, .intLit 50, .intLit 60])
        ]
      ])),
    .return_ (some
      (.index
        (.fieldAccess
          (.index (.identifier "arr") (.identifier "i"))
          "vals")
        (.identifier "j")))
  ]
}

def w513LocalAosReadModule : Module := {
  name := "w513_local_aos_read",
  imports := [],
  globals := [],
  functions := [w513LocalAosReadFixed, w513LocalAosReadIndexed],
  tests := [],
  benches := []
}

/-- W513-B: environment for writing scalar/array-typed fields of a function-local
    packed AOS and reading them back. -/
def w513LocalAosWriteEnv : Env := {
  structs := [("S", [("tag", .u32), ("vals", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["modify_fixed", "modify_indexed"]
}

/-- W513-B: write scalar and array-typed fields of a function-local packed AOS
    with literal indices, then read them back. -/
def w513LocalAosWriteFixed : Function := {
  name := "modify_fixed",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "arr" (.array 2 (.struct "S"))
      (some (.arrayLit (.array 2 (.struct "S")) [
        .structLit "S" [
          ("tag", .intLit 1),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
        ],
        .structLit "S" [
          ("tag", .intLit 2),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 40, .intLit 50, .intLit 60])
        ]
      ])),
    .assign
      (.fieldAccess (.index (.identifier "arr") (.intLit 0)) "tag")
      (.intLit 7),
    .assign
      (.index
        (.fieldAccess (.index (.identifier "arr") (.intLit 1)) "vals")
        (.intLit 2))
      (.intLit 99),
    .return_ (some
      (.binop "+"
        (.fieldAccess (.index (.identifier "arr") (.intLit 0)) "tag")
        (.index
          (.fieldAccess (.index (.identifier "arr") (.intLit 1)) "vals")
          (.intLit 2))))
  ]
}

/-- W513-B: write a variable-index element of an array-typed field inside a
    function-local packed AOS and read it back. -/
def w513LocalAosWriteIndexed : Function := {
  name := "modify_indexed",
  params := [("i", .u8), ("j", .u8), ("v", .u32)],
  ret := some .u32,
  body := [
    .varDecl "arr" (.array 2 (.struct "S"))
      (some (.arrayLit (.array 2 (.struct "S")) [
        .structLit "S" [
          ("tag", .intLit 1),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
        ],
        .structLit "S" [
          ("tag", .intLit 2),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 40, .intLit 50, .intLit 60])
        ]
      ])),
    .assign
      (.index
        (.fieldAccess
          (.index (.identifier "arr") (.identifier "i"))
          "vals")
        (.identifier "j"))
      (.identifier "v"),
    .return_ (some
      (.index
        (.fieldAccess
          (.index (.identifier "arr") (.identifier "i"))
          "vals")
        (.identifier "j")))
  ]
}

def w513LocalAosWriteModule : Module := {
  name := "w513_local_aos_write",
  imports := [],
  globals := [],
  functions := [w513LocalAosWriteFixed, w513LocalAosWriteIndexed],
  tests := [],
  benches := []
}

/-- W513-C: environment for returning a function-local packed AOS from a function
    and reading a mutated element from the returned value. -/
def w513LocalAosReturnEnv : Env := {
  structs := [("S", [("tag", .u32), ("vals", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_local", "read_returned"]
}

/-- W513-C: construct a function-local packed AOS, mutate one element, and return
    the whole local identifier. -/
def w513LocalAosReturnMakeLocal : Function := {
  name := "make_local",
  params := [],
  ret := some (.array 2 (.struct "S")),
  body := [
    .varDecl "arr" (.array 2 (.struct "S"))
      (some (.arrayLit (.array 2 (.struct "S")) [
        .structLit "S" [
          ("tag", .intLit 1),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
        ],
        .structLit "S" [
          ("tag", .intLit 2),
          ("vals", .arrayLit (.array 3 .u32) [.intLit 40, .intLit 50, .intLit 60])
        ]
      ])),
    .assign
      (.index
        (.fieldAccess
          (.index (.identifier "arr") (.intLit 1))
          "vals")
        (.intLit 1))
      (.intLit 77),
    .return_ (some (.identifier "arr"))
  ]
}

/-- W513-C: read arr[1].vals[1] from the returned function-local packed AOS. -/
def w513LocalAosReturnReadReturned : Function := {
  name := "read_returned",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.index
        (.fieldAccess
          (.index (.call "make_local" []) (.intLit 1))
          "vals")
        (.intLit 1)))
  ]
}

def w513LocalAosReturnModule : Module := {
  name := "w513_local_aos_return",
  imports := [],
  globals := [],
  functions := [w513LocalAosReturnMakeLocal, w513LocalAosReturnReadReturned],
  tests := [],
  benches := []
}

/- W515 witness environments and modules: function-local packed scalar structs
   initialized by copying another packed scalar struct value (local-to-local,
   module-to-local, and return-to-local). -/

/-- W515-A: environment for local-to-local copy of a packed scalar struct. -/
def w515LocalCopyEnv : Env := {
  structs := [("S", [("tag", .u32), ("vals", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["copy_and_sum"]
}

/-- W515-A: copy a local packed scalar struct into another local, mutate the
    copy, and assert the original is unchanged. -/
def w515LocalCopyFn : Function := {
  name := "copy_and_sum",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "a" (.struct "S")
      (some (.structLit "S" [
        ("tag", .intLit 1),
        ("vals", .arrayLit (.array 3 .u32) [.intLit 10, .intLit 20, .intLit 30])
      ])),
    .varDecl "b" (.struct "S") (some (.identifier "a")),
    .assign (.fieldAccess (.identifier "b") "tag") (.intLit 7),
    .assign
      (.index (.fieldAccess (.identifier "b") "vals") (.intLit 0))
      (.intLit 100),
    .return_ (some
      (.binop "+"
        (.binop "+"
          (.fieldAccess (.identifier "a") "tag")
          (.index (.fieldAccess (.identifier "a") "vals") (.intLit 0)))
        (.binop "+"
          (.fieldAccess (.identifier "b") "tag")
          (.index (.fieldAccess (.identifier "b") "vals") (.intLit 0)))))
  ]
}

def w515LocalCopyModule : Module := {
  name := "w515_local_packed_struct_copy",
  imports := [],
  globals := [],
  functions := [w515LocalCopyFn],
  tests := [],
  benches := []
}

/-- W515-B: environment for module-to-local copy of a packed scalar struct. -/
def w515ModuleToLocalCopyEnv : Env := {
  structs := [("S", [("tag", .u32), ("vals", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["copy_and_sum"],
  vars := [("m", .struct "S")]
}

def w515ModuleToLocalCopyGlobal : Stmt :=
  .varDecl "m" (.struct "S")
    (some (.structLit "S" [
      ("tag", .intLit 5),
      ("vals", .arrayLit (.array 3 .u32) [.intLit 11, .intLit 22, .intLit 33])
    ]))

/-- W515-B: copy a module-level packed scalar struct into a function-local packed
    scalar struct, mutate the local, and sum both values to verify independence. -/
def w515ModuleToLocalCopyFn : Function := {
  name := "copy_and_sum",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "s" (.struct "S") (some (.identifier "m")),
    .assign (.fieldAccess (.identifier "s") "tag") (.intLit 7),
    .assign
      (.index (.fieldAccess (.identifier "s") "vals") (.intLit 0))
      (.intLit 100),
    .return_ (some
      (.binop "+"
        (.binop "+"
          (.fieldAccess (.identifier "m") "tag")
          (.index (.fieldAccess (.identifier "m") "vals") (.intLit 0)))
        (.binop "+"
          (.fieldAccess (.identifier "s") "tag")
          (.index (.fieldAccess (.identifier "s") "vals") (.intLit 0)))))
  ]
}

def w515ModuleToLocalCopyModule : Module := {
  name := "w515_module_to_local_packed_struct_copy",
  imports := [],
  globals := [w515ModuleToLocalCopyGlobal],
  functions := [w515ModuleToLocalCopyFn],
  tests := [],
  benches := []
}

/-- W515-C: environment for return-to-local copy of a packed scalar struct. -/
def w515ReturnToLocalCopyEnv : Env := {
  structs := [("S", [("tag", .u32), ("vals", .array 3 .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make", "copy_and_sum"]
}

/-- W515-C: function that returns a packed scalar struct by value. -/
def w515ReturnToLocalCopyMake : Function := {
  name := "make",
  params := [],
  ret := some (.struct "S"),
  body := [
    .return_ (some (.structLit "S" [
      ("tag", .intLit 3),
      ("vals", .arrayLit (.array 3 .u32) [.intLit 7, .intLit 8, .intLit 9])
    ]))
  ]
}

/-- W515-C: copy a returned packed scalar struct into a local, mutate it, and
    read it back. -/
def w515ReturnToLocalCopyFn : Function := {
  name := "copy_and_sum",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "s" (.struct "S") (some (.call "make" [])),
    .assign (.fieldAccess (.identifier "s") "tag") (.intLit 7),
    .assign
      (.index (.fieldAccess (.identifier "s") "vals") (.intLit 0))
      (.intLit 100),
    .return_ (some
      (.binop "+"
        (.fieldAccess (.identifier "s") "tag")
        (.index (.fieldAccess (.identifier "s") "vals") (.intLit 0))))
  ]
}

def w515ReturnToLocalCopyModule : Module := {
  name := "w515_local_packed_struct_return_copy",
  imports := [],
  globals := [],
  functions := [w515ReturnToLocalCopyMake, w515ReturnToLocalCopyFn],
  tests := [],
  benches := []
}

/- W521 witness environments and modules: multi-dimensional arrays-of-structs
   passed as function parameters. -/

/-- W521-A: environment for a 2-D register-mode AOS parameter passed from a module-
    level variable.  The element struct has only scalar fields. -/
def w521AosParam2DScalarEnv : Env := {
  structs := [("Pt", [("x", .u32), ("y", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["read_param", "caller"],
  vars := [("g_grid", .array 2 (.array 3 (.struct "Pt")))]
}

/-- W521-A: module-level 2-D AOS global initialized with flat struct-literal
    rows. -/
def w521AosParam2DScalarGlobal : Stmt :=
  .varDecl "g_grid" (.array 2 (.array 3 (.struct "Pt")))
    (some (.arrayLit (.array 2 (.array 3 (.struct "Pt"))) [
      .arrayLit (.array 3 (.struct "Pt")) [
        .structLit "Pt" [("x", .intLit 1), ("y", .intLit 2)],
        .structLit "Pt" [("x", .intLit 3), ("y", .intLit 4)],
        .structLit "Pt" [("x", .intLit 5), ("y", .intLit 6)]
      ],
      .arrayLit (.array 3 (.struct "Pt")) [
        .structLit "Pt" [("x", .intLit 7), ("y", .intLit 8)],
        .structLit "Pt" [("x", .intLit 9), ("y", .intLit 10)],
        .structLit "Pt" [("x", .intLit 11), ("y", .intLit 12)]
      ]
    ]))

/-- W521-A: helper that takes a 2-D AOS parameter and returns the sum of the
    scalar fields of element [i][j]. -/
def w521AosParam2DScalarReadParam : Function := {
  name := "read_param",
  params := [
    ("m", .array 2 (.array 3 (.struct "Pt"))),
    ("i", .u8),
    ("j", .u8)
  ],
  ret := some .u32,
  body := [
    .return_ (some
      (.binop "+"
        (.fieldAccess
          (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j"))
          "x")
        (.fieldAccess
          (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j"))
          "y")))
  ]
}

/-- W521-A: caller that passes the module-level 2-D AOS into the helper. -/
def w521AosParam2DScalarCaller : Function := {
  name := "caller",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.call "read_param" [
        (.identifier "g_grid"),
        (.intLit 1),
        (.intLit 2)
      ]))
  ]
}

def w521AosParam2DScalarModule : Module := {
  name := "w521_aos_param_2d_scalar",
  imports := [],
  globals := [w521AosParam2DScalarGlobal],
  functions := [w521AosParam2DScalarReadParam, w521AosParam2DScalarCaller],
  tests := [],
  benches := []
}

/-- W521-B: environment for a 2-D packed-element AOS parameter passed from a local
    variable.  The element struct has a fixed-size scalar array field. -/
def w521AosParam2DArrayFieldEnv : Env := {
  structs := [("Buf", [("data", .array 4 .u8), ("tag", .u8)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["read_buf", "caller"]
}

/-- W521-B: helper that takes a 2-D packed-element AOS parameter and reads
    m[i][j].data[k]. -/
def w521AosParam2DArrayFieldReadBuf : Function := {
  name := "read_buf",
  params := [
    ("m", .array 2 (.array 2 (.struct "Buf"))),
    ("i", .u8),
    ("j", .u8),
    ("k", .u8)
  ],
  ret := some .u8,
  body := [
    .return_ (some
      (.index
        (.fieldAccess
          (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j"))
          "data")
        (.identifier "k")))
  ]
}

/-- W521-B: caller that constructs a function-local 2-D packed-element AOS and
    passes it to the helper. -/
def w521AosParam2DArrayFieldCaller : Function := {
  name := "caller",
  params := [],
  ret := some .u8,
  body := [
    .varDecl "arr" (.array 2 (.array 2 (.struct "Buf")))
      (some (.arrayLit (.array 2 (.array 2 (.struct "Buf"))) [
        .arrayLit (.array 2 (.struct "Buf")) [
          .structLit "Buf" [
            ("data", .arrayLit (.array 4 .u8) [.intLit 1, .intLit 2, .intLit 3, .intLit 4]),
            ("tag", .intLit 10)
          ],
          .structLit "Buf" [
            ("data", .arrayLit (.array 4 .u8) [.intLit 5, .intLit 6, .intLit 7, .intLit 8]),
            ("tag", .intLit 11)
          ]
        ],
        .arrayLit (.array 2 (.struct "Buf")) [
          .structLit "Buf" [
            ("data", .arrayLit (.array 4 .u8) [.intLit 9, .intLit 10, .intLit 11, .intLit 12]),
            ("tag", .intLit 12)
          ],
          .structLit "Buf" [
            ("data", .arrayLit (.array 4 .u8) [.intLit 13, .intLit 14, .intLit 15, .intLit 16]),
            ("tag", .intLit 13)
          ]
        ]
      ])),
    .return_ (some
      (.call "read_buf" [
        (.identifier "arr"),
        (.intLit 0),
        (.intLit 1),
        (.intLit 2)
      ]))
  ]
}

def w521AosParam2DArrayFieldModule : Module := {
  name := "w521_aos_param_2d_array_field",
  imports := [],
  globals := [],
  functions := [w521AosParam2DArrayFieldReadBuf, w521AosParam2DArrayFieldCaller],
  tests := [],
  benches := []
}

/- W524 witness environment and module: module-level 2-D packed-element AOS
   parameter. The element struct has a fixed-size scalar array field, so each
   element is lowered as one packed vector and the module-level array is passed
   to the helper as a concatenation of element vectors. -/

def w524AosParam2DPackedModuleEnv : Env := {
  structs := [("Buf", [("data", (.array 4 (.u32))), ("tag", (.u8))])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["assert_eq", "sum_bufs"]
}

def w524AosParam2DPackedModuleGlobal : Stmt :=
  .constDecl "g_bufs" (.array 2 (.array 2 (.struct "Buf")))
    (some (.arrayLit (.array 2 (.array 2 (.struct "Buf"))) [
      .arrayLit (.array 2 (.struct "Buf")) [
        .structLit "Buf" [("data", .arrayLit (.array 4 (.u32)) [.intLit 1, .intLit 2, .intLit 3, .intLit 4]), ("tag", .intLit 10)],
        .structLit "Buf" [("data", .arrayLit (.array 4 (.u32)) [.intLit 5, .intLit 6, .intLit 7, .intLit 8]), ("tag", .intLit 11)]
      ],
      .arrayLit (.array 2 (.struct "Buf")) [
        .structLit "Buf" [("data", .arrayLit (.array 4 (.u32)) [.intLit 9, .intLit 10, .intLit 11, .intLit 12]), ("tag", .intLit 12)],
        .structLit "Buf" [("data", .arrayLit (.array 4 (.u32)) [.intLit 13, .intLit 14, .intLit 15, .intLit 16]), ("tag", .intLit 13)]
      ]
    ]))

def w524AosParam2DPackedModuleSumBufs : Function := {
  name := "sum_bufs",
  params := [("m", .array 2 (.array 2 (.struct "Buf")))],
  ret := some .u32,
  body := [
    .varDecl "total" .u32 (some (.intLit 0)),
    .forLoop "i" (.intLit 2) [
      .forLoop "j" (.intLit 2) [
        .forLoop "k" (.intLit 4) [
          .assign (.identifier "total")
            (.binop "+" (.identifier "total")
              (.index
                (.fieldAccess
                  (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j"))
                  "data")
                (.identifier "k")))
        ]
      ]
    ],
    .return_ (some (.identifier "total"))
  ]
}

def w524AosParam2DPackedModuleCaller : Function := {
  name := "caller",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some (.call "sum_bufs" [(.identifier "g_bufs")]))
  ]
}

def w524AosParam2DPackedModule : Module := {
  name := "w524_2d_packed_aos_param_module",
  imports := [],
  globals := [w524AosParam2DPackedModuleGlobal],
  functions := [w524AosParam2DPackedModuleSumBufs, w524AosParam2DPackedModuleCaller],
  tests := [{ name := "basic", params := [], ret := none, body := [.bareCall (.call "assert_eq" [(.call "caller" []), (.intLit 136)])] }],
  benches := [{ name := "throughput", params := [], ret := none, body := [] }]
}

/- W529 witnesses: module/function 2-D scalar-struct array-of-structures (AoS)
   cross-boundary lowering. These exercise the W528 Rust backend shapes inside
   the shallow Icarus-lowerability model. -/

/-- W529-A: environment for a module-level 2-D packed scalar-struct constant. -/
def w529Module2DStructArrayConstEnv : Env := {
  structs := [("Pt", [("x", .u32), ("y", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["read_const", "read_const_literal"],
  vars := [("grid", .array 2 (.array 3 (.struct "Pt")))]
}

/-- W529-A: module-level packed constant initialized with nested array/struct
    literals. -/
def w529Module2DStructArrayConstGlobal : Stmt :=
  .constDecl "grid" (.array 2 (.array 3 (.struct "Pt")))
    (some (.arrayLit (.array 2 (.array 3 (.struct "Pt"))) [
      .arrayLit (.array 3 (.struct "Pt")) [
        .structLit "Pt" [("x", .intLit 1), ("y", .intLit 2)],
        .structLit "Pt" [("x", .intLit 3), ("y", .intLit 4)],
        .structLit "Pt" [("x", .intLit 5), ("y", .intLit 6)]
      ],
      .arrayLit (.array 3 (.struct "Pt")) [
        .structLit "Pt" [("x", .intLit 7), ("y", .intLit 8)],
        .structLit "Pt" [("x", .intLit 9), ("y", .intLit 10)],
        .structLit "Pt" [("x", .intLit 11), ("y", .intLit 12)]
      ]
    ]))

/-- W529-A: variable-index read of the module-level constant. -/
def w529Module2DStructArrayConstReadVar : Function := {
  name := "read_const",
  params := [("i", .u32), ("j", .u32)],
  ret := some .u32,
  body := [
    .return_ (some
      (.binop "+"
        (.fieldAccess
          (.index (.index (.identifier "grid") (.identifier "i")) (.identifier "j"))
          "x")
        (.fieldAccess
          (.index (.index (.identifier "grid") (.identifier "i")) (.identifier "j"))
          "y")))
  ]
}

/-- W529-A: literal-index read of the module-level constant. -/
def w529Module2DStructArrayConstReadLiteral : Function := {
  name := "read_const_literal",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.binop "+"
        (.fieldAccess
          (.index (.index (.identifier "grid") (.intLit 0)) (.intLit 0))
          "x")
        (.fieldAccess
          (.index (.index (.identifier "grid") (.intLit 1)) (.intLit 2))
          "y")))
  ]
}

/-- W529-A: module with a packed module-level constant. -/
def w529Module2DStructArrayConstModule : Module := {
  name := "w529_module_2d_struct_array_const",
  imports := [],
  globals := [w529Module2DStructArrayConstGlobal],
  functions := [w529Module2DStructArrayConstReadVar, w529Module2DStructArrayConstReadLiteral],
  tests := [],
  benches := []
}

/-- W529-B: environment for a module-level 2-D packed scalar-struct variable. -/
def w529Module2DStructArrayVarEnv : Env := {
  structs := [("Pt", [("x", .u32), ("y", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["read_var", "read_var_literal"],
  vars := [("grid", .array 2 (.array 3 (.struct "Pt")))]
}

/-- W529-B: module-level packed variable initialized with nested array/struct
    literals. -/
def w529Module2DStructArrayVarGlobal : Stmt :=
  .varDecl "grid" (.array 2 (.array 3 (.struct "Pt")))
    (some (.arrayLit (.array 2 (.array 3 (.struct "Pt"))) [
      .arrayLit (.array 3 (.struct "Pt")) [
        .structLit "Pt" [("x", .intLit 1), ("y", .intLit 2)],
        .structLit "Pt" [("x", .intLit 3), ("y", .intLit 4)],
        .structLit "Pt" [("x", .intLit 5), ("y", .intLit 6)]
      ],
      .arrayLit (.array 3 (.struct "Pt")) [
        .structLit "Pt" [("x", .intLit 7), ("y", .intLit 8)],
        .structLit "Pt" [("x", .intLit 9), ("y", .intLit 10)],
        .structLit "Pt" [("x", .intLit 11), ("y", .intLit 12)]
      ]
    ]))

/-- W529-B: variable-index read of the module-level variable. -/
def w529Module2DStructArrayVarReadVar : Function := {
  name := "read_var",
  params := [("i", .u32), ("j", .u32)],
  ret := some .u32,
  body := [
    .return_ (some
      (.binop "+"
        (.fieldAccess
          (.index (.index (.identifier "grid") (.identifier "i")) (.identifier "j"))
          "x")
        (.fieldAccess
          (.index (.index (.identifier "grid") (.identifier "i")) (.identifier "j"))
          "y")))
  ]
}

/-- W529-B: literal-index read of the module-level variable. -/
def w529Module2DStructArrayVarReadLiteral : Function := {
  name := "read_var_literal",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.binop "+"
        (.fieldAccess
          (.index (.index (.identifier "grid") (.intLit 0)) (.intLit 0))
          "x")
        (.fieldAccess
          (.index (.index (.identifier "grid") (.intLit 1)) (.intLit 2))
          "y")))
  ]
}

/-- W529-B: module with a packed module-level variable. -/
def w529Module2DStructArrayVarModule : Module := {
  name := "w529_module_2d_struct_array_var",
  imports := [],
  globals := [w529Module2DStructArrayVarGlobal],
  functions := [w529Module2DStructArrayVarReadVar, w529Module2DStructArrayVarReadLiteral],
  tests := [],
  benches := []
}

/-- W529-C: environment for a 2-D AoS passed as a function parameter. -/
def w529Function2DStructArrayParamEnv : Env := {
  structs := [("Pt", [("x", .u32), ("y", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["sum_param", "varidx_param", "make_grid", "caller"],
  vars := []
}

/-- W529-C: helper that sums literal-index fields of the parameter array. -/
def w529Function2DStructArrayParamSum : Function := {
  name := "sum_param",
  params := [("m", .array 2 (.array 3 (.struct "Pt")))],
  ret := some .u32,
  body := [
    .return_ (some
      (.binop "+"
        (.binop "+"
          (.binop "+"
            (.fieldAccess (.index (.index (.identifier "m") (.intLit 0)) (.intLit 0)) "x")
            (.fieldAccess (.index (.index (.identifier "m") (.intLit 0)) (.intLit 2)) "y"))
          (.fieldAccess (.index (.index (.identifier "m") (.intLit 1)) (.intLit 1)) "x"))
        (.fieldAccess (.index (.index (.identifier "m") (.intLit 1)) (.intLit 2)) "y")))
  ]
}

/-- W529-C: helper that returns the sum of fields at variable indices. -/
def w529Function2DStructArrayParamVaridx : Function := {
  name := "varidx_param",
  params := [("m", .array 2 (.array 3 (.struct "Pt"))), ("i", .u32), ("j", .u32)],
  ret := some .u32,
  body := [
    .return_ (some
      (.binop "+"
        (.fieldAccess
          (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j"))
          "x")
        (.fieldAccess
          (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j"))
          "y")))
  ]
}

/-- W529-C: helper that constructs and returns the 2-D AoS. -/
def w529Function2DStructArrayParamMakeGrid : Function := {
  name := "make_grid",
  params := [],
  ret := some (.array 2 (.array 3 (.struct "Pt"))),
  body := [
    .return_ (some
      (.arrayLit (.array 2 (.array 3 (.struct "Pt"))) [
        .arrayLit (.array 3 (.struct "Pt")) [
          .structLit "Pt" [("x", .intLit 1), ("y", .intLit 2)],
          .structLit "Pt" [("x", .intLit 3), ("y", .intLit 4)],
          .structLit "Pt" [("x", .intLit 5), ("y", .intLit 6)]
        ],
        .arrayLit (.array 3 (.struct "Pt")) [
          .structLit "Pt" [("x", .intLit 7), ("y", .intLit 8)],
          .structLit "Pt" [("x", .intLit 9), ("y", .intLit 10)],
          .structLit "Pt" [("x", .intLit 11), ("y", .intLit 12)]
        ]
      ]))
  ]
}

/-- W529-C: caller that exercises both helpers with a freshly-built grid. -/
def w529Function2DStructArrayParamCaller : Function := {
  name := "caller",
  params := [],
  ret := some .u32,
  body := [
    .return_ (some
      (.binop "+"
        (.call "sum_param" [(.call "make_grid" [])])
        (.call "varidx_param" [(.call "make_grid" []), (.intLit 0), (.intLit 0)])))
  ]
}

/-- W529-C: module containing parameter helpers and caller. -/
def w529Function2DStructArrayParamModule : Module := {
  name := "w529_function_2d_struct_array_param",
  imports := [],
  globals := [],
  functions := [w529Function2DStructArrayParamSum, w529Function2DStructArrayParamVaridx, w529Function2DStructArrayParamMakeGrid, w529Function2DStructArrayParamCaller],
  tests := [],
  benches := []
}

/-- W529-D: environment for a 2-D AoS returned from a function and bound to a
    local variable. -/
def w529Function2DStructArrayReturnEnv : Env := {
  structs := [("Pt", [("x", .u32), ("y", .u32)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_grid", "sum_returned", "varidx_returned"],
  vars := []
}

/-- W529-D: helper that constructs and returns the 2-D AoS. -/
def w529Function2DStructArrayReturnMakeGrid : Function := {
  name := "make_grid",
  params := [],
  ret := some (.array 2 (.array 3 (.struct "Pt"))),
  body := [
    .return_ (some
      (.arrayLit (.array 2 (.array 3 (.struct "Pt"))) [
        .arrayLit (.array 3 (.struct "Pt")) [
          .structLit "Pt" [("x", .intLit 1), ("y", .intLit 2)],
          .structLit "Pt" [("x", .intLit 3), ("y", .intLit 4)],
          .structLit "Pt" [("x", .intLit 5), ("y", .intLit 6)]
        ],
        .arrayLit (.array 3 (.struct "Pt")) [
          .structLit "Pt" [("x", .intLit 7), ("y", .intLit 8)],
          .structLit "Pt" [("x", .intLit 9), ("y", .intLit 10)],
          .structLit "Pt" [("x", .intLit 11), ("y", .intLit 12)]
        ]
      ]))
  ]
}

/-- W529-D: function that copies the returned array into a local variable and
    sums literal-index fields. -/
def w529Function2DStructArrayReturnSum : Function := {
  name := "sum_returned",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "m" (.array 2 (.array 3 (.struct "Pt")))
      (some (.call "make_grid" [])),
    .return_ (some
      (.binop "+"
        (.binop "+"
          (.binop "+"
            (.fieldAccess (.index (.index (.identifier "m") (.intLit 0)) (.intLit 0)) "x")
            (.fieldAccess (.index (.index (.identifier "m") (.intLit 0)) (.intLit 2)) "y"))
          (.fieldAccess (.index (.index (.identifier "m") (.intLit 1)) (.intLit 1)) "x"))
        (.fieldAccess (.index (.index (.identifier "m") (.intLit 1)) (.intLit 2)) "y")))
  ]
}

/-- W529-D: function that copies the returned array into a local variable and
    sums variable-index fields. -/
def w529Function2DStructArrayReturnVaridx : Function := {
  name := "varidx_returned",
  params := [("i", .u32), ("j", .u32)],
  ret := some .u32,
  body := [
    .varDecl "m" (.array 2 (.array 3 (.struct "Pt")))
      (some (.call "make_grid" [])),
    .return_ (some
      (.binop "+"
        (.fieldAccess
          (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j"))
          "x")
        (.fieldAccess
          (.index (.index (.identifier "m") (.identifier "i")) (.identifier "j"))
          "y")))
  ]
}

/-- W529-D: module containing the return helper and its consumers. -/
def w529Function2DStructArrayReturnModule : Module := {
  name := "w529_function_2d_struct_array_return",
  imports := [],
  globals := [],
  functions := [w529Function2DStructArrayReturnMakeGrid, w529Function2DStructArrayReturnSum, w529Function2DStructArrayReturnVaridx],
  tests := [],
  benches := []
}

/- W545 witness: primitive scalar array returned from a function and used to
   initialize a module-level global. -/

/-- W545: helper function that constructs and returns a `[3]u8` packed vector. -/
def w545CallInitReturnsArraySeq : Function := {
  name := "seq",
  params := [],
  ret := some (.array 3 .u8),
  body := [
    .varDecl "a" (.array 3 .u8)
      (some (.arrayLit (.array 3 .u8) [.intLit 1, .intLit 2, .intLit 3])),
    .return_ (some (.identifier "a"))
  ]
}

def w545CallInitReturnsArrayEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["seq"]
}

/-- W545: module with a global initialized from a primitive scalar array return. -/
def w545CallInitReturnsArrayModule : Module := {
  name := "w545_call_init_returns_array",
  imports := [],
  globals := [
    .constDecl "a" (.array 3 .u8) (some (.call "seq" []))
  ],
  functions := [w545CallInitReturnsArraySeq],
  tests := [
    { name := "call_init_returns_array", params := [], ret := none, body := [
      .bareCall (.call "assert_eq" [.index (.identifier "a") (.intLit 0), .intLit 1]),
      .bareCall (.call "assert_eq" [.index (.identifier "a") (.intLit 1), .intLit 2]),
      .bareCall (.call "assert_eq" [.index (.identifier "a") (.intLit 2), .intLit 3])
    ]}
  ],
  benches := []
}

/- W546 witnesses: function-local primitive scalar arrays initialized or reassigned
   from packed-vector function calls. -/

/-- W546-A: helper function that returns a `[3]u8` packed vector. -/
def w546LocalCallInitReturnsArraySeq : Function := {
  name := "seq",
  params := [],
  ret := some (.array 3 .u8),
  body := [
    .return_ (some (.arrayLit (.array 3 .u8) [.intLit 1, .intLit 2, .intLit 3]))
  ]
}

/-- W546-A: function that binds the returned array to a local `let` and returns
    the element sum. -/
def w546LocalCallInitReturnsArrayCheck : Function := {
  name := "check",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "a" (.array 3 .u8) (some (.call "seq" [])),
    .return_ (some
      (.binop "+"
        (.binop "+"
          (.index (.identifier "a") (.intLit 0))
          (.index (.identifier "a") (.intLit 1)))
        (.index (.identifier "a") (.intLit 2))))
  ]
}

def w546LocalCallInitReturnsArrayEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["seq", "check"]
}

/-- W546-A: module with a function-local packed primitive array initializer. -/
def w546LocalCallInitReturnsArrayModule : Module := {
  name := "w546_local_call_init_returns_array",
  imports := [],
  globals := [],
  functions := [w546LocalCallInitReturnsArraySeq, w546LocalCallInitReturnsArrayCheck],
  tests := [
    { name := "local_call_init_returns_array", params := [], ret := none, body := [
      .bareCall (.call "assert_eq" [.call "check" [], .intLit 6])
    ]}
  ],
  benches := []
}

/-- W546-B: helper function that returns a different `[3]u8` packed vector. -/
def w546LocalCallAssignReturnsArraySeq : Function := {
  name := "seq",
  params := [],
  ret := some (.array 3 .u8),
  body := [
    .return_ (some (.arrayLit (.array 3 .u8) [.intLit 7, .intLit 8, .intLit 9]))
  ]
}

/-- W546-B: function that initializes a local array from a literal, reassigns it
    from a function call, and returns the element sum. -/
def w546LocalCallAssignReturnsArrayCheck : Function := {
  name := "check",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "a" (.array 3 .u8)
      (some (.arrayLit (.array 3 .u8) [.intLit 1, .intLit 2, .intLit 3])),
    .assign (.identifier "a") (.call "seq" []),
    .return_ (some
      (.binop "+"
        (.binop "+"
          (.index (.identifier "a") (.intLit 0))
          (.index (.identifier "a") (.intLit 1)))
        (.index (.identifier "a") (.intLit 2))))
  ]
}

def w546LocalCallAssignReturnsArrayEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["seq", "check"]
}

/-- W546-B: module with a function-local packed primitive array reassignment. -/
def w546LocalCallAssignReturnsArrayModule : Module := {
  name := "w546_local_call_assign_returns_array",
  imports := [],
  globals := [],
  functions := [w546LocalCallAssignReturnsArraySeq, w546LocalCallAssignReturnsArrayCheck],
  tests := [
    { name := "local_call_assign_returns_array", params := [], ret := none, body := [
      .bareCall (.call "assert_eq" [.call "check" [], .intLit 24])
    ]}
  ],
  benches := []
}

/- W547 witnesses: signed primitive scalar arrays initialized from packed-vector
   function calls. -/

/-- W547-A: helper function that returns a `[3]i8` signed packed vector. -/
def w547SignedCallInitReturnsArraySeq : Function := {
  name := "seq",
  params := [],
  ret := some (.array 3 .i8),
  body := [
    .return_ (some (.arrayLit (.array 3 .i8) [.intLit (-1), .intLit (-2), .intLit (-3)]))
  ]
}

/-- W547-A: function that binds the returned signed array to a local `let` and
    returns the signed element sum. -/
def w547SignedCallInitReturnsArrayCheck : Function := {
  name := "check",
  params := [],
  ret := some .i8,
  body := [
    .varDecl "a" (.array 3 .i8) (some (.call "seq" [])),
    .return_ (some
      (.binop "+"
        (.binop "+"
          (.index (.identifier "a") (.intLit 0))
          (.index (.identifier "a") (.intLit 1)))
        (.index (.identifier "a") (.intLit 2))))
  ]
}

def w547SignedCallInitReturnsArrayEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["seq", "check"]
}

/-- W547-A: module with a function-local signed packed primitive array initializer. -/
def w547SignedCallInitReturnsArrayModule : Module := {
  name := "w547_signed_call_init_returns_array",
  imports := [],
  globals := [],
  functions := [w547SignedCallInitReturnsArraySeq, w547SignedCallInitReturnsArrayCheck],
  tests := [
    { name := "signed_call_init_returns_array", params := [], ret := none, body := [
      .bareCall (.call "assert_eq" [.call "check" [], .intLit (-6)])
    ]}
  ],
  benches := []
}

/-- W547-B: helper function that returns a `[3]i8` signed packed vector. -/
def w547SignedElementCompareSeq : Function := {
  name := "seq",
  params := [],
  ret := some (.array 3 .i8),
  body := [
    .return_ (some (.arrayLit (.array 3 .i8) [.intLit (-1), .intLit (-2), .intLit (-3)]))
  ]
}

/-- W547-B: test block that binds the signed array locally and compares the first
    element against a signed literal. -/
def w547SignedElementCompareModule : Module := {
  name := "w547_signed_element_compare",
  imports := [],
  globals := [],
  functions := [w547SignedElementCompareSeq],
  tests := [
    { name := "signed_element_compare", params := [], ret := none, body := [
      .varDecl "a" (.array 3 .i8) (some (.call "seq" [])),
      .bareCall (.call "assert_eq" [.index (.identifier "a") (.intLit 0), .intLit (-1)])
    ]}
  ],
  benches := []
}

/- W548 witnesses: multi-dimensional primitive scalar arrays initialized from
   packed-vector function calls. -/

/-- W548-A: helper function that returns a `[2][3]u8` packed vector. -/
def w548TwoDCallInitReturnsArrayGrid : Function := {
  name := "grid",
  params := [],
  ret := some (.array 2 (.array 3 .u8)),
  body := [
    .return_ (some (.arrayLit (.array 2 (.array 3 .u8)) [
      .arrayLit (.array 3 .u8) [.intLit 1, .intLit 2, .intLit 3],
      .arrayLit (.array 3 .u8) [.intLit 4, .intLit 5, .intLit 6]
    ]))
  ]
}

/-- W548-A: function that binds the returned 2-D array to a local `let` and
    returns the sum of two elements indexed with the full multi-dimensional chain. -/
def w548TwoDCallInitReturnsArraySum : Function := {
  name := "sum",
  params := [],
  ret := some .u8,
  body := [
    .varDecl "m" (.array 2 (.array 3 .u8)) (some (.call "grid" [])),
    .return_ (some
      (.binop "+"
        (.index (.index (.identifier "m") (.intLit 0)) (.intLit 0))
        (.index (.index (.identifier "m") (.intLit 1)) (.intLit 2))))
  ]
}

def w548TwoDCallInitReturnsArrayEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["grid", "sum"]
}

/-- W548-A: module with a function-local 2-D packed primitive array initializer. -/
def w548TwoDCallInitReturnsArrayModule : Module := {
  name := "w548_2d_call_init_returns_array",
  imports := [],
  globals := [],
  functions := [w548TwoDCallInitReturnsArrayGrid, w548TwoDCallInitReturnsArraySum],
  tests := [
    { name := "two_d_unsigned_sum", params := [], ret := none, body := [
      .bareCall (.call "assert_eq" [.call "sum" [], .intLit 7])
    ]}
  ],
  benches := []
}

/-- W548-B: helper function that returns a `[2][2]i8` signed packed vector. -/
def w548TwoDSignedElementReadSigns : Function := {
  name := "signs",
  params := [],
  ret := some (.array 2 (.array 2 .i8)),
  body := [
    .return_ (some (.arrayLit (.array 2 (.array 2 .i8)) [
      .arrayLit (.array 2 .i8) [.intLit (-1), .intLit (-2)],
      .arrayLit (.array 2 .i8) [.intLit (-3), .intLit (-4)]
    ]))
  ]
}

/-- W548-B: function that binds the returned signed 2-D array to a local `let`
    and returns the diagonal signed element sum. -/
def w548TwoDSignedElementReadDiag : Function := {
  name := "diag",
  params := [],
  ret := some .i8,
  body := [
    .varDecl "m" (.array 2 (.array 2 .i8)) (some (.call "signs" [])),
    .return_ (some
      (.binop "+"
        (.index (.index (.identifier "m") (.intLit 0)) (.intLit 0))
        (.index (.index (.identifier "m") (.intLit 1)) (.intLit 1))))
  ]
}

def w548TwoDSignedElementReadEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["signs", "diag"]
}

/-- W548-B: module with a function-local signed 2-D packed primitive array initializer. -/
def w548TwoDSignedElementReadModule : Module := {
  name := "w548_2d_signed_element_read",
  imports := [],
  globals := [],
  functions := [w548TwoDSignedElementReadSigns, w548TwoDSignedElementReadDiag],
  tests := [
    { name := "two_d_signed_diag", params := [], ret := none, body := [
      .bareCall (.call "assert_eq" [.call "diag" [], .intLit (-5)])
    ]}
  ],
  benches := []
}

/- W549 witness: three-dimensional primitive scalar array initialized from a
   packed-vector function call. -/

/-- W549: helper function that returns a `[2][3][4]u8` packed vector. -/
def w549ThreeDCallInitReturnsArrayCube : Function := {
  name := "cube",
  params := [],
  ret := some (.array 2 (.array 3 (.array 4 .u8))),
  body := [
    .return_ (some (.arrayLit (.array 2 (.array 3 (.array 4 .u8))) [
      .arrayLit (.array 3 (.array 4 .u8)) [
        .arrayLit (.array 4 .u8) [.intLit 1, .intLit 2, .intLit 3, .intLit 4],
        .arrayLit (.array 4 .u8) [.intLit 5, .intLit 6, .intLit 7, .intLit 8],
        .arrayLit (.array 4 .u8) [.intLit 9, .intLit 10, .intLit 11, .intLit 12]
      ],
      .arrayLit (.array 3 (.array 4 .u8)) [
        .arrayLit (.array 4 .u8) [.intLit 13, .intLit 14, .intLit 15, .intLit 16],
        .arrayLit (.array 4 .u8) [.intLit 17, .intLit 18, .intLit 19, .intLit 20],
        .arrayLit (.array 4 .u8) [.intLit 21, .intLit 22, .intLit 23, .intLit 24]
      ]
    ]))
  ]
}

/-- W549: function that binds the returned 3-D array to a local `let` and returns
    the sum of four corner elements indexed through the full index chain. -/
def w549ThreeDCallInitReturnsArrayCheck : Function := {
  name := "check",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "m" (.array 2 (.array 3 (.array 4 .u8))) (some (.call "cube" [])),
    .return_ (some
      (.binop "+"
        (.binop "+"
          (.index (.index (.index (.identifier "m") (.intLit 0)) (.intLit 0)) (.intLit 0))
          (.index (.index (.index (.identifier "m") (.intLit 0)) (.intLit 2)) (.intLit 3)))
        (.binop "+"
          (.index (.index (.index (.identifier "m") (.intLit 1)) (.intLit 0)) (.intLit 0))
          (.index (.index (.index (.identifier "m") (.intLit 1)) (.intLit 2)) (.intLit 3)))))
  ]
}

def w549ThreeDCallInitReturnsArrayEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["cube", "check"]
}

/-- W549: module with a function-local 3-D packed primitive array initializer. -/
def w549ThreeDCallInitReturnsArrayModule : Module := {
  name := "w549_3d_call_init_returns_array",
  imports := [],
  globals := [],
  functions := [w549ThreeDCallInitReturnsArrayCube, w549ThreeDCallInitReturnsArrayCheck],
  tests := [
    { name := "three_d_corner_sum", params := [], ret := none, body := [
      .bareCall (.call "assert_eq" [.call "check" [], .intLit 50])
    ]}
  ],
  benches := []
}

/- W550 witness: four-dimensional primitive scalar array initialized from a
   packed-vector function call. -/

/-- W550: helper function that returns a `[2][2][2][2]u8` packed vector. -/
def w550FourDCallInitReturnsArrayHyper : Function := {
  name := "hyper",
  params := [],
  ret := some (.array 2 (.array 2 (.array 2 (.array 2 .u8)))),
  body := [
    .return_ (some (.arrayLit (.array 2 (.array 2 (.array 2 (.array 2 .u8)))) [
      .arrayLit (.array 2 (.array 2 (.array 2 .u8))) [
        .arrayLit (.array 2 (.array 2 .u8)) [
          .arrayLit (.array 2 .u8) [.intLit 1, .intLit 2],
          .arrayLit (.array 2 .u8) [.intLit 3, .intLit 4]
        ],
        .arrayLit (.array 2 (.array 2 .u8)) [
          .arrayLit (.array 2 .u8) [.intLit 5, .intLit 6],
          .arrayLit (.array 2 .u8) [.intLit 7, .intLit 8]
        ]
      ],
      .arrayLit (.array 2 (.array 2 (.array 2 .u8))) [
        .arrayLit (.array 2 (.array 2 .u8)) [
          .arrayLit (.array 2 .u8) [.intLit 9, .intLit 10],
          .arrayLit (.array 2 .u8) [.intLit 11, .intLit 12]
        ],
        .arrayLit (.array 2 (.array 2 .u8)) [
          .arrayLit (.array 2 .u8) [.intLit 13, .intLit 14],
          .arrayLit (.array 2 .u8) [.intLit 15, .intLit 16]
        ]
      ]
    ]))
  ]
}

/-- W550: function that binds the returned 4-D array to a local `let` and returns
    the sum of four corner elements indexed through the full index chain. -/
def w550FourDCallInitReturnsArrayCheck : Function := {
  name := "check",
  params := [],
  ret := some .u32,
  body := [
    .varDecl "m" (.array 2 (.array 2 (.array 2 (.array 2 .u8)))) (some (.call "hyper" [])),
    .return_ (some
      (.binop "+"
        (.binop "+"
          (.index (.index (.index (.index (.identifier "m") (.intLit 0)) (.intLit 0)) (.intLit 0)) (.intLit 0))
          (.index (.index (.index (.index (.identifier "m") (.intLit 0)) (.intLit 1)) (.intLit 1)) (.intLit 1)))
        (.binop "+"
          (.index (.index (.index (.index (.identifier "m") (.intLit 1)) (.intLit 0)) (.intLit 0)) (.intLit 0))
          (.index (.index (.index (.index (.identifier "m") (.intLit 1)) (.intLit 1)) (.intLit 1)) (.intLit 1)))))
  ]
}

def w550FourDCallInitReturnsArrayEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["hyper", "check"]
}

/-- W550: module with a function-local 4-D packed primitive array initializer. -/
def w550FourDCallInitReturnsArrayModule : Module := {
  name := "w550_4d_call_init_returns_array",
  imports := [],
  globals := [],
  functions := [w550FourDCallInitReturnsArrayHyper, w550FourDCallInitReturnsArrayCheck],
  tests := [
    { name := "four_d_corner_sum", params := [], ret := none, body := [
      .bareCall (.call "assert_eq" [.call "check" [], .intLit 34])
    ]}
  ],
  benches := []
}

/- W535 negative witness environments and modules: the tightened predicate rejects
   the exact patterns flagged by the Rust structural classifier. -/

def w535CastToStringEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_label"]
}

def w535CastToStringMakeLabel : Function := {
  name := "make_label",
  params := [("v", .i32)],
  ret := some .string,
  body := [.return_ (some (.unsupportedIcarus "cast to string"))]
}

def w535CastToStringModule : Module := {
  name := "w535_negative_cast_to_string",
  imports := [],
  globals := [],
  functions := [w535CastToStringMakeLabel],
  tests := [],
  benches := []
}

/-- W535-A: a cast to `string` is not Icarus-lowerable. -/
theorem w535_cast_to_string_not_lowerable :
  ¬ Module.isLowerable w535CastToStringEnv w535CastToStringModule := by
  native_decide

/-- W535-B: environment for a struct with an `f32` field. -/
def w535F32FieldEnv : Env := {
  structs := [("Point", [("x", .f32), ("y", .i16)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := [],
  vars := [("src", .struct "Point")]
}

def w535F32FieldConst : Stmt :=
  .constDecl "src" (.struct "Point")
    (some (.structLit "Point" [("x", .f32Lit "1.5"), ("y", .intLit 42)]))

def w535F32FieldModule : Module := {
  name := "w535_negative_f32_field",
  imports := [],
  globals := [w535F32FieldConst],
  functions := [],
  tests := [],
  benches := []
}

/-- W535-B: a struct with an `f32` field is not Icarus-lowerable. -/
theorem w535_f32_field_not_lowerable :
  ¬ Module.isLowerable w535F32FieldEnv w535F32FieldModule := by
  native_decide

/-- W535-C: environment for a call to a host-only helper from synthesizable code. -/
def w535HostOnlyHelperEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := ["print"],
  reachable := ["synth"]
}

def w535HostOnlyHelperPrint : Function := {
  name := "print",
  params := [("msg", .string)],
  ret := none,
  body := [.bareCall (.unsupportedIcarus "host print")]
}

def w535HostOnlyHelperSynth : Function := {
  name := "synth",
  params := [],
  ret := some .i32,
  body := [
    .bareCall (.call "print" [.stringLit "hello"]),
    .return_ (some (.intLit 1))
  ]
}

def w535HostOnlyHelperModule : Module := {
  name := "w535_negative_host_only_helper",
  imports := [],
  globals := [],
  functions := [w535HostOnlyHelperPrint, w535HostOnlyHelperSynth],
  tests := [],
  benches := []
}

/-- W535-C: a call to a host-only helper from synthesizable code is not
    Icarus-lowerable. -/
theorem w535_host_only_helper_not_lowerable :
  ¬ Module.isLowerable w535HostOnlyHelperEnv w535HostOnlyHelperModule := by
  native_decide

/-- W535-D: environment for whole-struct assignment of a struct with a non-lowerable
    field (`String`) at module scope. -/
def w535NonlowerableStructAssignEnv : Env := {
  structs := [("Tagged", [("label", .string), ("value", .i16)])],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := [],
  vars := [("src", .struct "Tagged"), ("dst", .struct "Tagged")]
}

def w535NonlowerableStructAssignSrc : Stmt :=
  .constDecl "src" (.struct "Tagged")
    (some (.structLit "Tagged" [("label", .stringLit "hello"), ("value", .intLit 42)]))

def w535NonlowerableStructAssignDst : Stmt :=
  .varDecl "dst" (.struct "Tagged") none

def w535NonlowerableStructAssignModule : Module := {
  name := "w535_negative_nonlowerable_struct_assign",
  imports := [],
  globals := [w535NonlowerableStructAssignSrc, w535NonlowerableStructAssignDst],
  functions := [],
  tests := [],
  benches := []
}

/-- W535-D: a struct with a `String` field is not Icarus-lowerable. -/
theorem w535_nonlowerable_struct_assign_not_lowerable :
  ¬ Module.isLowerable w535NonlowerableStructAssignEnv w535NonlowerableStructAssignModule := by
  native_decide

/-- W535-E: environment for an unbounded `while (true)` loop. -/
def w535UnboundedWhileEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["run_forever"]
}

def w535UnboundedWhileRunForever : Function := {
  name := "run_forever",
  params := [],
  ret := some .i32,
  body := [
    .varDecl "i" .i32 (some (.intLit 0)),
    .whileLoop (.boolLit true) [
      .assign (.identifier "i") (.binop "+" (.identifier "i") (.intLit 1))
    ],
    .return_ (some (.identifier "i"))
  ]
}

def w535UnboundedWhileModule : Module := {
  name := "w535_negative_unbounded_while",
  imports := [],
  globals := [],
  functions := [w535UnboundedWhileRunForever],
  tests := [],
  benches := []
}

/-- W535-E: `while (true)` is not Icarus-lowerable. -/
theorem w535_unbounded_while_not_lowerable :
  ¬ Module.isLowerable w535UnboundedWhileEnv w535UnboundedWhileModule := by
  native_decide

/-- W535-F: environment for an unresolved imported function call. -/
def w535UnresolvedImportEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [("magic", ("some::external", "magic"))],
  hostOnly := [],
  reachable := ["synth"]
}

def w535UnresolvedImportSynth : Function := {
  name := "synth",
  params := [],
  ret := some .i32,
  body := [.return_ (some (.call "magic" [.intLit 1, .intLit 2]))]
}

def w535UnresolvedImportModule : Module := {
  name := "w535_negative_unresolved_import",
  imports := [{ path := "some::external", items := ["magic"] }],
  globals := [],
  functions := [w535UnresolvedImportSynth],
  tests := [],
  benches := []
}

/-- W535-F: a call to an imported function is not Icarus-lowerable. -/
theorem w535_unresolved_import_not_lowerable :
  ¬ Module.isLowerable w535UnresolvedImportEnv w535UnresolvedImportModule := by
  native_decide

/-- W537: environment for a function returning an undeclared struct type. -/
def w537UndefinedStructEnv : Env := {
  structs := [],
  constructors := [],
  enums := [],
  imports := [],
  hostOnly := [],
  reachable := ["make_pt"]
}

def w537UndefinedStructMakePt : Function := {
  name := "make_pt",
  params := [("x", .u32)],
  ret := some (.struct "Pt"),
  body := [.return_ (some (.structLit "Pt" [("x", .identifier "x")]))]
}

def w537UndefinedStructModule : Module := {
  name := "w537_negative_undefined_struct",
  imports := [],
  globals := [],
  functions := [w537UndefinedStructMakePt],
  tests := [],
  benches := []
}

/-- W537: a function returning an undeclared struct type is not Icarus-lowerable,
    matching the Rust structural classifier. -/
theorem w537_undefined_struct_not_lowerable :
  ¬ Module.isLowerable w537UndefinedStructEnv w537UndefinedStructModule := by
  native_decide

end Trinity.IcarusLowerable