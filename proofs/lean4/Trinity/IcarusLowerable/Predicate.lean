/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 491
  Icarus-lowerability predicate over the simplified t27 AST.

  The predicate mirrors the Rust heuristics in bootstrap/src/compiler.rs:
    - fn_body_has_unlowerable_construct (line ~7551)
    - compute_host_only_functions (line ~7622)
    - scalar/array-of-struct call field checks
    - host-only and namespace-call classification

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

import Trinity.IcarusLowerable.Ast

namespace Trinity.IcarusLowerable

/-- Environment needed to judge lowerability. -/
structure Env where
  /-- Struct name → fields (name, type). -/
  structs : List (String × List (String × Ty))
  /-- Constructor function name → struct name. -/
  constructors : List (String × String)
  /-- Declared enum type names. -/
  enums : List String
  /-- Imported item name → (module path, item name). -/
  imports : List (String × (String × String))
  /-- Names of functions classified host-only. -/
  hostOnly : List String
  /-- Names of functions reachable from tests/benches/module logic. -/
  reachable : List String
  /-- Module-level variable / constant name → type. -/
  vars : List (String × Ty) := []
  deriving Repr

def Env.structFields (env : Env) (name : String) : List (String × Ty) :=
  match (env.structs.find? (fun p => p.1 == name)).map (·.2) with
  | some fields => fields
  | none => []

def Env.structForConstructor (env : Env) (ctor : String) : Option String :=
  (env.constructors.find? (fun p => p.1 == ctor)).map (·.2)

def Env.isEnum (env : Env) (name : String) : Bool :=
  env.enums.contains name

def Env.isHostOnly (env : Env) (name : String) : Bool :=
  env.hostOnly.contains name

def Env.isReachable (env : Env) (name : String) : Bool :=
  env.reachable.contains name

def Env.varType (env : Env) (name : String) : Option Ty :=
  (env.vars.find? (fun p => p.1 == name)).map (·.2)

/-- Builtins that the Verilog backend does not yet lower. -/
def unlowerableBuiltins : List String :=
  ["@intCast", "@min", "@mod", "@max", "@abs", "@clz", "@ctz", "@popCount", "@byteSwap", "@bitReverse"]

/-- Numeric types (signed or unsigned). -/
def Ty.isNumeric : Ty → Bool
  | .u8 | .u16 | .u32 | .u64 | .i8 | .i16 | .i32 | .i64 => true
  | _ => false

/-- Types that are lowerable in synthesizable Icarus contexts. -/
def Ty.isLowerable : Ty → Bool
  | .bool | .u8 | .u16 | .u32 | .u64 | .i8 | .i16 | .i32 | .i64 => true
  | .array _ elem => elem.isLowerable
  | .struct _ => true   -- struct lowerability is checked per-field when used
  | .f32 | .string | .enum _ => false

/-- Leaf-field lowerability: used when a struct is actually packed/sliced. -/
def Ty.isLeafLowerable : Ty → Bool
  | .bool | .u8 | .u16 | .u32 | .u64 | .i8 | .i16 | .i32 | .i64 => true
  | .array _ elem => elem.isLeafLowerable
  | .f32 | .string | .enum _ | .struct _ => false

/-- Is the given operator a lowerable numeric/bitwise/boolean operator? -/
def opIsLowerable (op : String) : Bool :=
  ["+", "-", "*", "/", "%", "&", "|", "^", "<<", ">>", "==", "!=", "<", "<=", ">", ">=", "&&", "||", "!", "and", "or"].contains op

set_option linter.unusedVariables false

/-- Fuel budget for the transparent lowerability predicates.  It only needs to be
    larger than the depth/list-length of any lowerable expression encountered in
    the proof-relevant corpus. -/
def predicateFuel : Nat := 1000

mutual

/-- Lowerability of an expression in synthesizable context.  Total and transparent
    by structural recursion on an explicit `fuel` parameter. -/
def Expr.isLowerableFuel (fuel : Nat) (env : Env) (e : Expr) : Bool :=
  match fuel, env, e with
  | 0, _, _ => false
  | fuel+1, env, .boolLit _ => true
  | fuel+1, env, .intLit _ => true
  | fuel+1, env, .f32Lit _ => false
  | fuel+1, env, .stringLit _ => false
  | fuel+1, env, .identifier _ => true
  | fuel+1, env, .binop op lhs rhs =>
      opIsLowerable op && lhs.isLowerableFuel fuel env && rhs.isLowerableFuel fuel env
  | fuel+1, env, .unop op e =>
      opIsLowerable op && e.isLowerableFuel fuel env
  | fuel+1, env, .fieldAccess base field =>
      base.isLowerableFuel fuel env &&
      match base with
      | .call name _ =>
          match env.structForConstructor name with
          | some sname =>
              let fields := env.structFields sname
              let found := fields.find? (fun p => p.1 == field)
              match found with
              | some ty => ty.2.isLeafLowerable
              | none => true
          | none => true
      | _ => true
  | fuel+1, env, .index base idx =>
      base.isLowerableFuel fuel env && idx.isLowerableFuel fuel env &&
      match base with
      | .fieldAccess (.call _ _) _ => true
      | _ => true
  | fuel+1, env, .call name args =>
      (!Env.isHostOnly env name) &&
      ((name.splitOn "::").length == 1) &&
      (!unlowerableBuiltins.contains name) &&
      Expr.isLowerableListFuel fuel env args
  | fuel+1, env, .structLit _ fields => Expr.isLowerableFieldListFuel fuel env fields
  | fuel+1, env, .arrayLit ty elems => ty.isLowerable && Expr.isLowerableListFuel fuel env elems
  | fuel+1, env, .enumVal _ _ => false
  | fuel+1, env, .len base => base.isLowerableFuel fuel env
  | fuel+1, env, .contains base item => base.isLowerableFuel fuel env && item.isLowerableFuel fuel env
  | fuel+1, env, .unsupportedIcarus _ => false

/-- Helper: lowerability of a list of expressions, fuel-threaded. -/
def Expr.isLowerableListFuel (fuel : Nat) (env : Env) (es : List Expr) : Bool :=
  match fuel, env, es with
  | 0, _, _ => false
  | fuel+1, env, [] => true
  | fuel+1, env, e::es => e.isLowerableFuel fuel env && Expr.isLowerableListFuel fuel env es

/-- Helper: lowerability of a list of struct-literal fields, fuel-threaded. -/
def Expr.isLowerableFieldListFuel (fuel : Nat) (env : Env) (fs : List (String × Expr)) : Bool :=
  match fuel, env, fs with
  | 0, _, _ => false
  | fuel+1, env, [] => true
  | fuel+1, env, p::ps => p.2.isLowerableFuel fuel env && Expr.isLowerableFieldListFuel fuel env ps

/-- Lowerability of a statement in synthesizable context.  Total and transparent
    by structural recursion on an explicit `fuel` parameter. -/
def Stmt.isLowerableFuel (fuel : Nat) (env : Env) (s : Stmt) : Bool :=
  match fuel, env, s with
  | 0, _, _ => false
  | fuel+1, env, .assign lhs rhs => lhs.isLowerableFuel fuel env && rhs.isLowerableFuel fuel env
  | fuel+1, env, .varDecl _ ty init => ty.isLowerable && init.all (fun e => e.isLowerableFuel fuel env)
  | fuel+1, env, .constDecl _ ty init => ty.isLowerable && init.all (fun e => e.isLowerableFuel fuel env)
  | fuel+1, env, .ifThenElse cond then_ else_ =>
      cond.isLowerableFuel fuel env &&
      Stmt.isLowerableListFuel fuel env then_ &&
      Stmt.isLowerableListFuel fuel env else_
  | fuel+1, env, .forLoop _ range body =>
      range.isLowerableFuel fuel env && Stmt.isLowerableListFuel fuel env body
  | fuel+1, env, .return_ e => e.all (fun x => x.isLowerableFuel fuel env)
  | fuel+1, env, .bareCall e => e.isLowerableFuel fuel env

/-- Helper: lowerability of a list of statements, fuel-threaded. -/
def Stmt.isLowerableListFuel (fuel : Nat) (env : Env) (ss : List Stmt) : Bool :=
  match fuel, env, ss with
  | 0, _, _ => false
  | fuel+1, env, [] => true
  | fuel+1, env, s::ss => s.isLowerableFuel fuel env && Stmt.isLowerableListFuel fuel env ss

/-- True when an expression is purely combinational (no placeholder nodes).
    Total and transparent by fuel-threaded structural recursion. -/
def Expr.isCombinationalFuel (fuel : Nat) (e : Expr) : Bool :=
  match fuel, e with
  | 0, _ => false
  | fuel+1, .boolLit _ => true
  | fuel+1, .intLit _ => true
  | fuel+1, .identifier _ => true
  | fuel+1, .binop _ lhs rhs => lhs.isCombinationalFuel fuel && rhs.isCombinationalFuel fuel
  | fuel+1, .unop _ e => e.isCombinationalFuel fuel
  | fuel+1, .fieldAccess base _ => base.isCombinationalFuel fuel
  | fuel+1, .index base idx => base.isCombinationalFuel fuel && idx.isCombinationalFuel fuel
  | fuel+1, .call _ args => Expr.isCombinationalListFuel fuel args
  | fuel+1, .structLit _ fields => Expr.isCombinationalFieldListFuel fuel fields
  | fuel+1, .arrayLit _ elems => Expr.isCombinationalListFuel fuel elems
  | fuel+1, .enumVal _ _ => false
  | fuel+1, .len _ => false
  | fuel+1, .contains _ _ => false
  | fuel+1, .unsupportedIcarus _ => false
  | fuel+1, .f32Lit _ => false
  | fuel+1, .stringLit _ => false

/-- Helper: combinationality of a list of expressions, fuel-threaded. -/
def Expr.isCombinationalListFuel (fuel : Nat) (es : List Expr) : Bool :=
  match fuel, es with
  | 0, _ => false
  | fuel+1, [] => true
  | fuel+1, e::es => e.isCombinationalFuel fuel && Expr.isCombinationalListFuel fuel es

/-- Helper: combinationality of a list of struct-literal fields, fuel-threaded. -/
def Expr.isCombinationalFieldListFuel (fuel : Nat) (fs : List (String × Expr)) : Bool :=
  match fuel, fs with
  | 0, _ => false
  | fuel+1, [] => true
  | fuel+1, p::ps => p.2.isCombinationalFuel fuel && Expr.isCombinationalFieldListFuel fuel ps

/-- True when a statement is purely combinational: no conditionals, no loops,
    no uninitialized declarations, and no bare `return_` without a value.
    Total and transparent by fuel-threaded structural recursion. -/
def Stmt.isCombinationalFuel (fuel : Nat) (s : Stmt) : Bool :=
  match fuel, s with
  | 0, _ => false
  | fuel+1, .assign lhs rhs => lhs.isCombinationalFuel fuel && rhs.isCombinationalFuel fuel
  | fuel+1, .varDecl _ _ (some e) => e.isCombinationalFuel fuel
  | fuel+1, .varDecl _ _ none => false
  | fuel+1, .constDecl _ _ (some e) => e.isCombinationalFuel fuel
  | fuel+1, .constDecl _ _ none => false
  | fuel+1, .ifThenElse _ _ _ => false
  | fuel+1, .forLoop _ _ _ => false
  | fuel+1, .return_ (some e) => e.isCombinationalFuel fuel
  | fuel+1, .return_ none => false
  | fuel+1, .bareCall e => e.isCombinationalFuel fuel

/-- Inferred t27 type for the lowerable subset.  Identifiers are typed from the
    environment; function-call types come from the callee's return type or the
    constructor map.  Total and transparent by fuel-threaded recursion. -/
def Expr.typeOfFuel (fuel : Nat) (env : Env) (m : Module) (e : Expr) : Option Ty :=
  match fuel, env, m, e with
  | 0, _, _, _ => none
  | fuel+1, env, m, .boolLit _ => some .bool
  | fuel+1, env, m, .intLit _ => some .u32
  | fuel+1, env, m, .identifier name => env.varType name
  | fuel+1, env, m, .binop op lhs _ =>
      if ["==", "!=", "<", "<=", ">", ">=", "&&", "||"].contains op then some .bool
      else lhs.typeOfFuel fuel env m
  | fuel+1, env, m, .unop _ e => e.typeOfFuel fuel env m
  | fuel+1, env, m, .fieldAccess base field =>
      match base.typeOfFuel fuel env m with
      | some (.struct sname) =>
          let fields := env.structFields sname
          fields.find? (fun p => p.1 == field) |>.map (·.2)
      | _ => none
  | fuel+1, env, m, .index base _ =>
      match base.typeOfFuel fuel env m with
      | some (.array _ elem) => some elem
      | _ => none
  | fuel+1, env, m, .call name _ =>
      match env.structForConstructor name with
      | some sname => some (.struct sname)
      | none => (m.findFunction name).bind (·.ret)
  | fuel+1, env, m, .structLit name _ => some (.struct name)
  | fuel+1, env, m, .arrayLit ty _ => some ty
  | fuel+1, env, m, _ => none

/-- Names of functions called inside an expression (constructor / imported /
    ordinary calls are all treated as calls for resolution purposes).
    Total and transparent by fuel-threaded structural recursion. -/
def Expr.functionNamesFuel (fuel : Nat) (e : Expr) : List String :=
  match fuel, e with
  | 0, _ => []
  | fuel+1, .call name args => name :: Expr.functionNamesListFuel fuel args
  | fuel+1, .binop _ lhs rhs => lhs.functionNamesFuel fuel ++ rhs.functionNamesFuel fuel
  | fuel+1, .unop _ e => e.functionNamesFuel fuel
  | fuel+1, .fieldAccess base _ => base.functionNamesFuel fuel
  | fuel+1, .index base idx => base.functionNamesFuel fuel ++ idx.functionNamesFuel fuel
  | fuel+1, .structLit _ fields => Expr.functionNamesFieldListFuel fuel fields
  | fuel+1, .arrayLit _ elems => Expr.functionNamesListFuel fuel elems
  | fuel+1, .len base => base.functionNamesFuel fuel
  | fuel+1, .contains base item => base.functionNamesFuel fuel ++ item.functionNamesFuel fuel
  | fuel+1, _ => []

/-- Helper: function names in a list of expressions, fuel-threaded. -/
def Expr.functionNamesListFuel (fuel : Nat) (es : List Expr) : List String :=
  match fuel, es with
  | 0, _ => []
  | fuel+1, [] => []
  | fuel+1, e::es => e.functionNamesFuel fuel ++ Expr.functionNamesListFuel fuel es

/-- Helper: function names in a list of struct-literal fields, fuel-threaded. -/
def Expr.functionNamesFieldListFuel (fuel : Nat) (fs : List (String × Expr)) : List String :=
  match fuel, fs with
  | 0, _ => []
  | fuel+1, [] => []
  | fuel+1, p::ps => p.2.functionNamesFuel fuel ++ Expr.functionNamesFieldListFuel fuel ps

/-- Names of functions called inside a statement.  Total and transparent by
    fuel-threaded structural recursion. -/
def Stmt.functionNamesFuel (fuel : Nat) (s : Stmt) : List String :=
  match fuel, s with
  | 0, _ => []
  | fuel+1, .assign _ rhs => rhs.functionNamesFuel fuel
  | fuel+1, .varDecl _ _ (some e) => e.functionNamesFuel fuel
  | fuel+1, .varDecl _ _ none => []
  | fuel+1, .constDecl _ _ (some e) => e.functionNamesFuel fuel
  | fuel+1, .constDecl _ _ none => []
  | fuel+1, .ifThenElse cond then_ else_ =>
      cond.functionNamesFuel fuel ++
      Stmt.functionNamesListFuel fuel then_ ++
      Stmt.functionNamesListFuel fuel else_
  | fuel+1, .forLoop _ range body =>
      range.functionNamesFuel fuel ++ Stmt.functionNamesListFuel fuel body
  | fuel+1, .return_ (some e) => e.functionNamesFuel fuel
  | fuel+1, .return_ none => []
  | fuel+1, .bareCall e => e.functionNamesFuel fuel

/-- Helper: function names in a list of statements, fuel-threaded. -/
def Stmt.functionNamesListFuel (fuel : Nat) (ss : List Stmt) : List String :=
  match fuel, ss with
  | 0, _ => []
  | fuel+1, [] => []
  | fuel+1, s::ss => s.functionNamesFuel fuel ++ Stmt.functionNamesListFuel fuel ss

end

/-- Wrapper: lowerability with the default predicate fuel. -/
def Expr.isLowerable (env : Env) (e : Expr) : Bool := e.isLowerableFuel predicateFuel env

/-- Wrapper: statement lowerability with the default predicate fuel. -/
def Stmt.isLowerable (env : Env) (s : Stmt) : Bool := s.isLowerableFuel predicateFuel env

/-- A function is lowerable when reachable and its body/interface are lowerable. -/
def Function.isLowerable (env : Env) (fn : Function) : Bool :=
  let interfaceOK := fn.params.all (fun p => p.2.isLowerable) && fn.ret.all (·.isLowerable)
  if !Env.isReachable env fn.name then
    true  -- unreachable functions do not need to be lowerable
  else
    interfaceOK && fn.body.all (Stmt.isLowerable env)

/-- A module is lowerable when all reachable functions and global synthesizable parts are lowerable. -/
def Module.isLowerable (env : Env) (m : Module) : Bool :=
  let globalLowerable := m.globals.all (Stmt.isLowerable env)
  let fnsLowerable := m.functions.all (Function.isLowerable env)
  let testsLowerable := m.tests.all (Function.isLowerable env)
  let benchesLowerable := m.benches.all (Function.isLowerable env)
  globalLowerable && fnsLowerable && testsLowerable && benchesLowerable

/-- Standalone verdict for a module under a given environment. -/
def lowerabilityVerdict (env : Env) (m : Module) : String :=
  if Module.isLowerable env m then "lowerable" else "not_lowerable"

/-- Wrapper: expression combinationality with the default predicate fuel. -/
def Expr.isCombinational (e : Expr) : Bool := e.isCombinationalFuel predicateFuel

/-- Wrapper: statement combinationality with the default predicate fuel. -/
def Stmt.isCombinational (s : Stmt) : Bool := s.isCombinationalFuel predicateFuel

/-- True when a function body is purely combinational. -/
def Function.isCombinational (fn : Function) : Bool :=
  fn.body.all Stmt.isCombinational

/-- True when a module is purely combinational. -/
def Module.isCombinational (env : Env) (m : Module) : Bool :=
  m.globals.all Stmt.isCombinational
  && m.functions.all (fun f => if Env.isReachable env f.name then f.isCombinational else true)
  && m.tests.all Function.isCombinational
  && m.benches.all Function.isCombinational

/-- Wrapper: type inference with the default predicate fuel. -/
def Expr.typeOf (env : Env) (m : Module) (e : Expr) : Option Ty := e.typeOfFuel predicateFuel env m

/-- Wrapper: statement function names with the default predicate fuel. -/
def Stmt.functionNames (s : Stmt) : List String := s.functionNamesFuel predicateFuel

def Function.functionNames (fn : Function) : List String :=
  fn.body.flatMap Stmt.functionNames

/-- True when the module contains a function/test/bench with the given name. -/
def Module.hasFunctionNamed (m : Module) (name : String) : Bool :=
  (m.functions ++ m.tests ++ m.benches).any (fun f => f.name == name)

/-- True when every call inside every reachable function resolves to a function
    actually present in the module.  This is a strong but realistic assumption
    for the Icarus-lowerable combinational subset and lets the generic
    equivalence theorem inline function calls on both sides. -/
def Module.callsResolved (env : Env) (m : Module) : Bool :=
  let allFns := m.functions ++ m.tests ++ m.benches
  allFns.all (fun f =>
    if Env.isReachable env f.name then
      f.functionNames.all (fun name => Module.hasFunctionNamed m name)
    else true)

/-- True when every call inside every reachable function is itself marked
    reachable in the environment, so that `emitModule` will include its
    definition in the emitted shallow Verilog. -/
def Module.callsReachable (env : Env) (m : Module) : Bool :=
  let allFns := m.functions ++ m.tests ++ m.benches
  allFns.all (fun f =>
    if Env.isReachable env f.name then
      f.functionNames.all (fun name => Env.isReachable env name)
    else true)

end Trinity.IcarusLowerable
