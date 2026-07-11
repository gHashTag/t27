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

end Trinity.IcarusLowerable
