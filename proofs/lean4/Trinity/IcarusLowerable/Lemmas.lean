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

end Trinity.IcarusLowerable
