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
  /-- Enum localparam name → integer value.  The name is emitted as
      `EnumName_variant` by the Verilog backend, so the model stores the same
      qualified key to evaluate `Expr.enumVal`. -/
  enumValues : List (String × Int) := []
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

def Env.enumValue (env : Env) (enum : String) (variant : String) : Option Int :=
  (env.enumValues.find? (fun p => p.1 == enum ++ "_" ++ variant)).map (·.2)

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

/-- Types that are lowerable in synthesizable Icarus contexts, fuel-threaded
    so struct recursion is transparent to the Lean kernel.
    W535: a struct type is lowerable only when it exists in the environment and
    every field is itself lowerable (recursively). -/
def Ty.isLowerableFuel (fuel : Nat) (env : Env) (ty : Ty) : Bool :=
  match fuel, ty with
  | 0, _ => false
  | _+1, .bool | _+1, .u8 | _+1, .u16 | _+1, .u32 | _+1, .u64
  | _+1, .i8 | _+1, .i16 | _+1, .i32 | _+1, .i64 => true
  | fuel+1, .array size elem => size > 0 && Ty.isLowerableFuel fuel env elem
  | fuel+1, .struct name =>
      let fields := env.structFields name
      -- W537: a struct is lowerable only when it is declared in the environment
      -- and every field is itself lowerable (recursively).  Undefined struct
      -- names are rejected, matching the Rust structural classifier.
      !fields.isEmpty && fields.all (fun p => Ty.isLowerableFuel fuel env p.2)
  | _+1, .f32 | _+1, .string | _+1, .enum _ => false

/-- Leaf-field lowerability: used when a struct is actually packed/sliced. -/
def Ty.isLeafLowerable : Ty → Bool
  | .bool | .u8 | .u16 | .u32 | .u64 | .i8 | .i16 | .i32 | .i64 => true
  | .array n elem => n > 0 && elem.isLeafLowerable
  | .f32 | .string | .enum _ | .struct _ => false

/-- W544: primitive scalar types that are bit-lowerable as single Verilog regs. -/
def Ty.isPrimitiveScalar : Ty → Bool
  | .bool | .u8 | .u16 | .u32 | .u64 | .i8 | .i16 | .i32 | .i64 => true
  | _ => false

/-- W544/W545/W547: a primitive scalar array (e.g. [3]u8 or [3]i8).  Function
    returns of this shape are lowerable once the backend stores the packed
    function result in a packed-vector module const/var or local reg and indexes
    it by bit-slices. -/
def Ty.isPrimitiveScalarArray : Ty → Bool
  | .array n elem => n > 0 && elem.isPrimitiveScalar
  | _ => false

/-- Is the given operator a lowerable numeric/bitwise/boolean operator? -/
def opIsLowerable (op : String) : Bool :=
  ["+", "-", "*", "/", "%", "&", "|", "^", "<<", ">>", "==", "!=", "<", "<=", ">", ">=", "&&", "||", "!", "and", "or"].contains op

set_option linter.unusedVariables false

/-- Fuel budget for the transparent lowerability predicates.  It only needs to be
    larger than the depth/list-length of any lowerable expression encountered in
    the proof-relevant corpus. -/
def predicateFuel : Nat := 1000

/-- Wrapper: type lowerability with the default predicate fuel. -/
def Ty.isLowerable (env : Env) (ty : Ty) : Bool := Ty.isLowerableFuel predicateFuel env ty

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
      -- W535: reject calls to names that are imported from another module; the
      -- Icarus backend cannot resolve cross-module imports in synthesizable code.
      (!env.imports.any (fun p => p.1 == name)) &&
      Expr.isLowerableListFuel fuel env args
  | fuel+1, env, .structLit _ fields => Expr.isLowerableFieldListFuel fuel env fields
  | fuel+1, env, .arrayLit ty elems => ty.isLowerable env && Expr.isLowerableListFuel fuel env elems
  | fuel+1, env, .enumVal enum variant =>
      env.enums.contains enum && (env.enumValue enum variant).isSome
  | fuel+1, env, .len base => base.isLowerableFuel fuel env
  | fuel+1, env, .contains base item => base.isLowerableFuel fuel env && item.isLowerableFuel fuel env
  | fuel+1, env, .switch disc cases default =>
      disc.isLowerableFuel fuel env &&
      Expr.isLowerableSwitchCaseListFuel fuel env cases &&
      default.isLowerableFuel fuel env
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

/-- Helper: lowerability of a switch case list, fuel-threaded. -/
def Expr.isLowerableSwitchCaseListFuel (fuel : Nat) (env : Env) (cs : List (Expr × Expr)) : Bool :=
  match fuel, env, cs with
  | 0, _, _ => false
  | fuel+1, env, [] => true
  | fuel+1, env, (tag, res)::cs =>
      tag.isLowerableFuel fuel env && res.isLowerableFuel fuel env &&
      Expr.isLowerableSwitchCaseListFuel fuel env cs

/-- Helper: lowerability of a statement switch case list, fuel-threaded. -/
def Stmt.isLowerableSwitchCaseListFuel (fuel : Nat) (env : Env) (cs : List (Expr × List Stmt)) : Bool :=
  match fuel, env, cs with
  | 0, _, _ => false
  | fuel+1, env, [] => true
  | fuel+1, env, (tag, body)::cs =>
      tag.isLowerableFuel fuel env &&
      Stmt.isLowerableListFuel fuel env body &&
      Stmt.isLowerableSwitchCaseListFuel fuel env cs

/-- Lowerability of a statement in synthesizable context.  Total and transparent
    by structural recursion on an explicit `fuel` parameter. -/
def Stmt.isLowerableFuel (fuel : Nat) (env : Env) (s : Stmt) : Bool :=
  match fuel, env, s with
  | 0, _, _ => false
  | fuel+1, env, .assign lhs rhs => lhs.isLowerableFuel fuel env && rhs.isLowerableFuel fuel env
  | fuel+1, env, .varDecl _ ty init => ty.isLowerable env && init.all (fun e => e.isLowerableFuel fuel env)
  | fuel+1, env, .constDecl _ ty init => ty.isLowerable env && init.all (fun e => e.isLowerableFuel fuel env)
  | fuel+1, env, .ifThenElse cond then_ else_ =>
      cond.isLowerableFuel fuel env &&
      Stmt.isLowerableListFuel fuel env then_ &&
      Stmt.isLowerableListFuel fuel env else_
  | fuel+1, env, .switch disc cases default =>
      disc.isLowerableFuel fuel env &&
      Stmt.isLowerableSwitchCaseListFuel fuel env cases &&
      Stmt.isLowerableListFuel fuel env default
  | fuel+1, env, .forLoop _ range body =>
      range.isLowerableFuel fuel env && Stmt.isLowerableListFuel fuel env body
  | fuel+1, env, .whileLoop cond body =>
      -- W535: reject unbounded `while (true)`.  Bounded loops are accepted
      -- structurally; termination is handled by the fuel-bounded semantics
      -- and the soundness layer.
      !(cond == .boolLit true) &&
      cond.isLowerableFuel fuel env &&
      Stmt.isLowerableListFuel fuel env body
  | fuel+1, env, .break => true
  | fuel+1, env, .continue => true
  | fuel+1, env, .return_ e => e.all (fun x => x.isLowerableFuel fuel env)
  | fuel+1, env, .bareCall e => e.isLowerableFuel fuel env

/-- Helper: lowerability of a list of statements, fuel-threaded. -/
def Stmt.isLowerableListFuel (fuel : Nat) (env : Env) (ss : List Stmt) : Bool :=
  match fuel, env, ss with
  | 0, _, _ => false
  | fuel+1, env, [] => true
  | fuel+1, env, s::ss => s.isLowerableFuel fuel env && Stmt.isLowerableListFuel fuel env ss

/-- Contextual validity of `break`/`continue`: they may only appear inside a loop
    body.  The predicate is fuel-threaded for transparency and mutually recursive
    over statement lists and switch-case lists.  A `break`/`continue` outside any
    loop makes the whole statement invalid. -/
  def Stmt.hasValidLoopControlFuel (fuel : Nat) (inLoop : Bool) (s : Stmt) : Bool :=
    match fuel, s with
    | 0, _ => false
    | fuel+1, .break => inLoop
    | fuel+1, .continue => inLoop
    | fuel+1, .ifThenElse _ then_ else_ =>
        Stmt.hasValidLoopControlListFuel fuel inLoop then_ &&
        Stmt.hasValidLoopControlListFuel fuel inLoop else_
    | fuel+1, .switch _ cases default =>
        Stmt.hasValidLoopControlSwitchCaseListFuel fuel inLoop cases &&
        Stmt.hasValidLoopControlListFuel fuel inLoop default
    | fuel+1, .forLoop _ _ body =>
        Stmt.hasValidLoopControlListFuel fuel true body
    | fuel+1, .whileLoop _ body =>
        Stmt.hasValidLoopControlListFuel fuel true body
    | fuel+1, _ => true

  def Stmt.hasValidLoopControlListFuel (fuel : Nat) (inLoop : Bool) (ss : List Stmt) : Bool :=
    match fuel, ss with
    | 0, _ => false
    | fuel+1, [] => true
    | fuel+1, s::ss =>
        s.hasValidLoopControlFuel fuel inLoop &&
        Stmt.hasValidLoopControlListFuel fuel inLoop ss

  def Stmt.hasValidLoopControlSwitchCaseListFuel (fuel : Nat) (inLoop : Bool)
      (cs : List (Expr × List Stmt)) : Bool :=
    match fuel, cs with
    | 0, _ => false
    | fuel+1, [] => true
    | fuel+1, (_, body)::cs =>
        Stmt.hasValidLoopControlListFuel fuel inLoop body &&
        Stmt.hasValidLoopControlSwitchCaseListFuel fuel inLoop cs

/-- Wrapper: loop-control validity with the default predicate fuel and no
    surrounding loop. -/
def Stmt.hasValidLoopControl (s : Stmt) : Bool := s.hasValidLoopControlFuel predicateFuel false

/-- True when a function body only uses `break`/`continue` inside loops. -/
def Function.hasValidLoopControl (env : Env) (fn : Function) : Bool :=
  if Env.isHostOnly env fn.name then true else fn.body.all Stmt.hasValidLoopControl

/-- True when an entire module satisfies the loop-control invariant. -/
def Module.hasValidLoopControl (env : Env) (m : Module) : Bool :=
  m.globals.all Stmt.hasValidLoopControl
  && m.functions.all (Function.hasValidLoopControl env)

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
  | fuel+1, .enumVal _ _ => true
  | fuel+1, .len _ => false
  | fuel+1, .contains _ _ => false
  | fuel+1, .switch disc cases default =>
      disc.isCombinationalFuel fuel &&
      Expr.isCombinationalSwitchCaseListFuel fuel cases &&
      default.isCombinationalFuel fuel
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

/-- Helper: combinationality of a switch case list, fuel-threaded. -/
def Expr.isCombinationalSwitchCaseListFuel (fuel : Nat) (cs : List (Expr × Expr)) : Bool :=
  match fuel, cs with
  | 0, _ => false
  | fuel+1, [] => true
  | fuel+1, (tag, res)::cs =>
      tag.isCombinationalFuel fuel && res.isCombinationalFuel fuel &&
      Expr.isCombinationalSwitchCaseListFuel fuel cs

/-- Helper: combinationality of a statement switch case list, fuel-threaded. -/
def Stmt.isCombinationalSwitchCaseListFuel (fuel : Nat) (cs : List (Expr × List Stmt)) : Bool :=
  match fuel, cs with
  | 0, _ => false
  | fuel+1, [] => true
  | fuel+1, (tag, body)::cs =>
      tag.isCombinationalFuel fuel &&
      Stmt.isCombinationalListFuel fuel body &&
      Stmt.isCombinationalSwitchCaseListFuel fuel cs

/-- True when a statement is purely combinational: no conditionals, no loops,
    no uninitialized declarations, and no bare `return_` without a value.
    Total and transparent by fuel-threaded structural recursion. -/
def Stmt.isCombinationalFuel (fuel : Nat) (s : Stmt) : Bool :=
  match fuel, s with
  | 0, _ => false
  | fuel+1, .assign (.identifier _) rhs => rhs.isCombinationalFuel fuel
  | fuel+1, .assign _ _ => false
  | fuel+1, .varDecl _ _ (some e) => e.isCombinationalFuel fuel
  | fuel+1, .varDecl _ _ none => false
  | fuel+1, .constDecl _ _ (some e) => e.isCombinationalFuel fuel
  | fuel+1, .constDecl _ _ none => false
  | fuel+1, .ifThenElse cond then_ else_ =>
      cond.isCombinationalFuel fuel && Stmt.isCombinationalListFuel fuel then_ && Stmt.isCombinationalListFuel fuel else_
  | fuel+1, .switch disc cases default =>
      disc.isCombinationalFuel fuel &&
      Stmt.isCombinationalSwitchCaseListFuel fuel cases &&
      Stmt.isCombinationalListFuel fuel default
  | fuel+1, .forLoop _ _ _ => false
  | fuel+1, .whileLoop _ _ => false
  | fuel+1, .break => false
  | fuel+1, .continue => false
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
  | fuel+1, env, m, .enumVal _ _ => some .u32
  | fuel+1, env, m, .switch _ _ default => default.typeOfFuel fuel env m
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
  | fuel+1, .enumVal _ _ => []
  | fuel+1, .switch disc cases default =>
      disc.functionNamesFuel fuel ++
      Expr.functionNamesSwitchCaseListFuel fuel cases ++
      default.functionNamesFuel fuel
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

/-- Helper: function names in a switch case list, fuel-threaded. -/
def Expr.functionNamesSwitchCaseListFuel (fuel : Nat) (cs : List (Expr × Expr)) : List String :=
  match fuel, cs with
  | 0, _ => []
  | fuel+1, [] => []
  | fuel+1, (tag, res)::cs =>
      tag.functionNamesFuel fuel ++ res.functionNamesFuel fuel ++
      Expr.functionNamesSwitchCaseListFuel fuel cs

/-- Helper: function names in a statement switch case list, fuel-threaded. -/
def Stmt.functionNamesSwitchCaseListFuel (fuel : Nat) (cs : List (Expr × List Stmt)) : List String :=
  match fuel, cs with
  | 0, _ => []
  | fuel+1, [] => []
  | fuel+1, (tag, body)::cs =>
      tag.functionNamesFuel fuel ++
      Stmt.functionNamesListFuel fuel body ++
      Stmt.functionNamesSwitchCaseListFuel fuel cs

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
  | fuel+1, .switch disc cases default =>
      disc.functionNamesFuel fuel ++
      Stmt.functionNamesSwitchCaseListFuel fuel cases ++
      Stmt.functionNamesListFuel fuel default
  | fuel+1, .forLoop _ range body =>
      range.functionNamesFuel fuel ++ Stmt.functionNamesListFuel fuel body
  | fuel+1, .whileLoop cond body =>
      cond.functionNamesFuel fuel ++ Stmt.functionNamesListFuel fuel body
  | fuel+1, .break => []
  | fuel+1, .continue => []
  | fuel+1, .return_ (some e) => e.functionNamesFuel fuel
  | fuel+1, .return_ none => []
  | fuel+1, .bareCall e => e.functionNamesFuel fuel

/-- Helper: function names in a list of statements, fuel-threaded. -/
def Stmt.functionNamesListFuel (fuel : Nat) (ss : List Stmt) : List String :=
  match fuel, ss with
  | 0, _ => []
  | fuel+1, [] => []
  | fuel+1, s::ss => s.functionNamesFuel fuel ++ Stmt.functionNamesListFuel fuel ss

/-- Helper: combinationality of a list of statements, fuel-threaded. -/
def Stmt.isCombinationalListFuel (fuel : Nat) (ss : List Stmt) : Bool :=
  match fuel, ss with
  | 0, _ => false
  | fuel+1, [] => true
  | fuel+1, s::ss => s.isCombinationalFuel fuel && Stmt.isCombinationalListFuel fuel ss

end

-- Structural (fuel-independent) call-name extraction.  This is the model used by
-- the generic equivalence proof; the fuel-threaded variants above remain the
-- implementation of the Rust lowerability classifier.

mutual
  /-- Structural function names in an expression. -/
  @[simp]
  def Expr.functionNames' : Expr → List String
    | .call name args => name :: Expr.functionNamesList' args
    | .binop _ lhs rhs => lhs.functionNames' ++ rhs.functionNames'
    | .unop _ e => e.functionNames'
    | .fieldAccess base _ => base.functionNames'
    | .index base idx => base.functionNames' ++ idx.functionNames'
    | .structLit _ fields => Expr.functionNamesFieldList' fields
    | .arrayLit _ elems => Expr.functionNamesList' elems
    | .enumVal _ _ => []
    | .switch disc cases default =>
        disc.functionNames' ++ Expr.functionNamesSwitchCaseList' cases ++ default.functionNames'
    | .len base => base.functionNames'
    | .contains base item => base.functionNames' ++ item.functionNames'
    | _ => []

  /-- Structural function names in a list of expressions. -/
  @[simp]
  def Expr.functionNamesList' (es : List Expr) : List String :=
    match es with
    | [] => []
    | e :: es => e.functionNames' ++ Expr.functionNamesList' es

  /-- Structural function names in a list of struct-literal fields. -/
  @[simp]
  def Expr.functionNamesFieldList' (fs : List (String × Expr)) : List String :=
    match fs with
    | [] => []
    | f :: fs => f.2.functionNames' ++ Expr.functionNamesFieldList' fs

  /-- Structural function names in a switch case list. -/
  @[simp]
  def Expr.functionNamesSwitchCaseList' (cs : List (Expr × Expr)) : List String :=
    match cs with
    | [] => []
    | (tag, res) :: cs => tag.functionNames' ++ res.functionNames' ++ Expr.functionNamesSwitchCaseList' cs
end

-- Structural function names in a statement.
mutual
  /-- Structural function names in a statement. -/
  @[simp]
  def Stmt.functionNames' : Stmt → List String
    | .assign _ rhs => rhs.functionNames'
    | .varDecl _ _ (some e) => e.functionNames'
    | .varDecl _ _ none => []
    | .constDecl _ _ (some e) => e.functionNames'
    | .constDecl _ _ none => []
    | .ifThenElse cond then_ else_ =>
        cond.functionNames' ++ Stmt.functionNamesList' then_ ++ Stmt.functionNamesList' else_
    | .switch disc cases default =>
        disc.functionNames' ++
        Stmt.functionNamesSwitchCaseList' cases ++
        Stmt.functionNamesList' default
    | .forLoop _ range body =>
        range.functionNames' ++ Stmt.functionNamesList' body
    | .whileLoop cond body =>
        cond.functionNames' ++ Stmt.functionNamesList' body
    | .break => []
    | .continue => []
    | .return_ (some e) => e.functionNames'
    | .return_ none => []
    | .bareCall e => e.functionNames'

  @[simp]
  def Stmt.functionNamesList' (ss : List Stmt) : List String :=
    match ss with
    | [] => []
    | s :: ss => s.functionNames' ++ Stmt.functionNamesList' ss

  @[simp]
  def Stmt.functionNamesSwitchCaseList' (cs : List (Expr × List Stmt)) : List String :=
    match cs with
    | [] => []
    | (tag, body) :: cs => tag.functionNames' ++ Stmt.functionNamesList' body ++ Stmt.functionNamesSwitchCaseList' cs
end

/-- Wrapper: expression lowerability with the default predicate fuel. -/
def Expr.isLowerable (env : Env) (e : Expr) : Bool := e.isLowerableFuel predicateFuel env

/-- Wrapper: structural expression function names. -/
@[simp]
def Expr.functionNames (e : Expr) : List String := e.functionNames'

/-- Wrapper: statement lowerability with the default predicate fuel. -/
def Stmt.isLowerable (env : Env) (s : Stmt) : Bool := s.isLowerableFuel predicateFuel env

/-- Wrapper: structural statement function names. -/
@[simp]
def Stmt.functionNames (s : Stmt) : List String := s.functionNames'

/-- A function is lowerable when its interface and body are lowerable.
    W499: every non-host-only function is checked.  Host-only helpers are not
    part of the emitted Icarus Verilog, so they do not need to be lowerable.
    The obsolete reachability shortcut has been removed. -/
def Function.isLowerable (env : Env) (fn : Function) : Bool :=
  if Env.isHostOnly env fn.name then true
  else
    let interfaceOK := fn.params.all (fun p => p.2.isLowerable env) && fn.ret.all (·.isLowerable env)
    -- W545: primitive scalar array return types are now lowerable.  The Rust
    -- backend stores the packed function result in a packed-vector module
    -- const/var and indexes it by bit-slices.
    interfaceOK && fn.body.all (Stmt.isLowerable env)

/-- A module is lowerable when all globals and emitted functions are lowerable
    and `break`/`continue` only occur inside loop bodies.  W499: reachability no
    longer gates lowerability because every non-host-only function is emitted
    unconditionally.  Tests and benches are not part of the Icarus synthesizable
    model, so they are not checked here. -/
def Module.isLowerable (env : Env) (m : Module) : Bool :=
  let globalLowerable := m.globals.all (Stmt.isLowerable env)
  let fnsLowerable := m.functions.all (Function.isLowerable env)
  let globalsValid := m.globals.all Stmt.hasValidLoopControl
  let fnsValid := m.functions.all (Function.hasValidLoopControl env)
  globalLowerable && fnsLowerable && globalsValid && fnsValid

/-- Standalone verdict for a module under a given environment. -/
def lowerabilityVerdict (env : Env) (m : Module) : String :=
  if Module.isLowerable env m then "lowerable" else "not_lowerable"

/-- W499: the generic equivalence theorem needs function names to be unique so
    that `List.find?` on both the source module and the emitted Verilog module
    resolves to the same function.  Duplicates would still map together, but
    uniqueness is part of the well-formedness contract. -/
def Module.hasUniqueFunctionNames (m : Module) : Prop :=
  List.Nodup ((m.functions ++ m.tests ++ m.benches).map Function.name)

-- Structural (fuel-independent) combinationality predicates.  These are used
-- as the static invariant in the generic equivalence proof; the fuel-based
-- predicates below remain the total semantic model.

mutual
  /-- Structural combinationality for expressions. -/
  @[simp]
  def Expr.isCombinational' : Expr → Bool
    | .boolLit _ => true
    | .intLit _ => true
    | .identifier _ => true
    | .binop _ lhs rhs => lhs.isCombinational' && rhs.isCombinational'
    | .unop _ e => e.isCombinational'
    | .fieldAccess base _ => base.isCombinational'
    | .index base idx => base.isCombinational' && idx.isCombinational'
    | .call _ args => Expr.isCombinationalList' args
    | .structLit _ fields => Expr.isCombinationalFieldList' fields
    | .arrayLit _ elems => Expr.isCombinationalList' elems
    | .enumVal _ _ => true
    | .switch disc cases default =>
        disc.isCombinational' && Expr.isCombinationalSwitchCaseList' cases && default.isCombinational'
    | _ => false

  /-- Structural combinationality for a list of expressions. -/
  @[simp]
  def Expr.isCombinationalList' : List Expr → Bool
    | [] => true
    | e :: es => e.isCombinational' && Expr.isCombinationalList' es

  /-- Structural combinationality for a list of struct-literal fields. -/
  @[simp]
  def Expr.isCombinationalFieldList' : List (String × Expr) → Bool
    | [] => true
    | f :: fs => f.2.isCombinational' && Expr.isCombinationalFieldList' fs

  /-- Structural combinationality for a switch case list. -/
  @[simp]
  def Expr.isCombinationalSwitchCaseList' : List (Expr × Expr) → Bool
    | [] => true
    | (tag, res) :: cs =>
        tag.isCombinational' && res.isCombinational' &&
        Expr.isCombinationalSwitchCaseList' cs
end

mutual
  /-- Structural combinationality for statements. -/
  @[simp]
  def Stmt.isCombinational' : Stmt → Bool
    | .assign (.identifier _) rhs => rhs.isCombinational'
    | .varDecl _ _ (some e) => e.isCombinational'
    | .constDecl _ _ (some e) => e.isCombinational'
    | .ifThenElse cond then_ else_ =>
        cond.isCombinational' && Stmt.isCombinationalList' then_ && Stmt.isCombinationalList' else_
    | .switch disc cases default =>
        disc.isCombinational' &&
        Stmt.isCombinationalSwitchCaseList' cases &&
        Stmt.isCombinationalList' default
    | .forLoop _ _ _ => false
    | .whileLoop _ _ => false
    | .return_ (some e) => e.isCombinational'
    | .bareCall e => e.isCombinational'
    | _ => false

  /-- Structural combinationality for a statement switch case list. -/
  @[simp]
  def Stmt.isCombinationalSwitchCaseList' : List (Expr × List Stmt) → Bool
    | [] => true
    | (tag, body) :: cs =>
        tag.isCombinational' && Stmt.isCombinationalList' body &&
        Stmt.isCombinationalSwitchCaseList' cs

  /-- Structural combinationality for a list of statements. -/
  @[simp]
  def Stmt.isCombinationalList' : List Stmt → Bool
    | [] => true
    | s :: ss => s.isCombinational' && Stmt.isCombinationalList' ss
end

/-- Wrapper: structural expression combinationality. -/
@[simp]
def Expr.isCombinational (e : Expr) : Bool := e.isCombinational'

/-- Wrapper: combinationality of an expression list. -/
@[simp]
def Expr.isCombinationalList (es : List Expr) : Bool := Expr.isCombinationalList' es

/-- Wrapper: combinationality of a struct-literal field list. -/
@[simp]
def Expr.isCombinationalFieldList (fs : List (String × Expr)) : Bool :=
  Expr.isCombinationalFieldList' fs

/-- Wrapper: structural statement combinationality. -/
@[simp]
def Stmt.isCombinational (s : Stmt) : Bool := s.isCombinational'

/-- Wrapper: combinationality of a statement list. -/
@[simp]
def Stmt.isCombinationalList (ss : List Stmt) : Bool := ss.all Stmt.isCombinational'

/-- True when a function body is purely combinational. -/
def Function.isCombinational (env : Env) (fn : Function) : Bool :=
  if Env.isHostOnly env fn.name then true
  else fn.body.all Stmt.isCombinational

/-- True when a module is purely combinational.  Only globals and emitted
    functions matter for the synthesizable equivalence theorem; tests/benches
    are handled by the host-side harness and are not part of the Icarus model. -/
def Module.isCombinational (env : Env) (m : Module) : Bool :=
  m.globals.all Stmt.isCombinational
  && m.functions.all (Function.isCombinational env)

-- Structural sequential predicates: same as combinational, but bounded
-- `forLoop` is allowed when its range and body are sequential.

mutual
  /-- Structural sequentiality for statements. -/
  @[simp]
  def Stmt.isSequential' : Stmt → Bool
    | .assign (.identifier _) rhs => rhs.isCombinational'
    | .varDecl _ _ (some e) => e.isCombinational'
    | .constDecl _ _ (some e) => e.isCombinational'
    | .ifThenElse cond then_ else_ =>
        cond.isCombinational' && Stmt.isSequentialList' then_ && Stmt.isSequentialList' else_
    | .switch disc cases default =>
        disc.isCombinational' &&
        Stmt.isSequentialSwitchCaseList' cases &&
        Stmt.isSequentialList' default
    | .forLoop _ range body =>
        range.isCombinational' && Stmt.isSequentialList' body
    | .whileLoop cond body =>
        cond.isCombinational' && Stmt.isSequentialList' body
    | .break => true
    | .continue => true
    | .return_ (some e) => e.isCombinational'
    | .bareCall e => e.isCombinational'
    | _ => false

  /-- Structural sequentiality for a statement switch case list. -/
  @[simp]
  def Stmt.isSequentialSwitchCaseList' : List (Expr × List Stmt) → Bool
    | [] => true
    | (tag, body) :: cs =>
        tag.isCombinational' && Stmt.isSequentialList' body &&
        Stmt.isSequentialSwitchCaseList' cs

  /-- Structural sequentiality for a list of statements. -/
  @[simp]
  def Stmt.isSequentialList' : List Stmt → Bool
    | [] => true
    | s :: ss => s.isSequential' && Stmt.isSequentialList' ss
end

/-- Wrapper: structural statement sequentiality. -/
@[simp]
def Stmt.isSequential (s : Stmt) : Bool := s.isSequential'

/-- Wrapper: sequentiality of a statement list. -/
@[simp]
def Stmt.isSequentialList (ss : List Stmt) : Bool := ss.all Stmt.isSequential'

/-- True when a function body is sequential (combinational or bounded-loop). -/
def Function.isSequential (env : Env) (fn : Function) : Bool :=
  if Env.isHostOnly env fn.name then true
  else fn.body.all Stmt.isSequential

/-- True when a module is sequential: globals and emitted functions contain only
    combinational or bounded-loop statements. -/
def Module.isSequential (env : Env) (m : Module) : Bool :=
  m.globals.all Stmt.isSequential
  && m.functions.all (Function.isSequential env)

-- Combinational statement lists (primed form) and switch case lists are also
-- sequential.  The two theorems are mutually recursive because a switch case
-- body is a statement list and a statement list may contain a switch.
mutual
@[simp]
theorem Stmt.isCombinationalSwitchCaseList_implies_isSequentialSwitchCaseList' :
    ∀ (cs : List (Expr × List Stmt)),
      Stmt.isCombinationalSwitchCaseList' cs = true →
      Stmt.isSequentialSwitchCaseList' cs = true
  | [], _ => rfl
  | (tag, body) :: cs, h => by
      have h1 : tag.isCombinational' = true := by simp at h; exact h.1.1
      have h2 : Stmt.isCombinationalList' body = true := by simp at h; exact h.1.2
      have h3 : Stmt.isCombinationalSwitchCaseList' cs = true := by simp at h; exact h.2
      simp [h1, Stmt.isCombinationalList_implies_isSequentialList' body h2,
        Stmt.isCombinationalSwitchCaseList_implies_isSequentialSwitchCaseList' cs h3]
  termination_by cs => sizeOf cs

@[simp]
theorem Stmt.isCombinationalList_implies_isSequentialList' :
    ∀ (ss : List Stmt), Stmt.isCombinationalList' ss = true → Stmt.isSequentialList' ss = true
  | [], _ => rfl
  | s :: ss, h => by
      have h1 : s.isCombinational' = true := by simp at h; exact h.1
      have h2 : Stmt.isCombinationalList' ss = true := by simp at h; exact h.2
      have hs : s.isSequential' = true := by
        cases s with
        | assign lhs rhs =>
            cases lhs <;> simp at h1 ⊢ <;> try { exact h1 } <;> contradiction
        | varDecl _ _ init =>
            cases init <;> simp at h1 ⊢ <;> try { exact h1 } <;> contradiction
        | constDecl _ _ init =>
            cases init <;> simp at h1 ⊢ <;> try { exact h1 } <;> contradiction
        | ifThenElse cond then_ else_ =>
            simp at h1 ⊢
            have h1_1 := h1.1.1
            have h1_2 := h1.1.2
            have h1_3 := h1.2
            constructor
            · constructor
              · exact h1_1
              · exact Stmt.isCombinationalList_implies_isSequentialList' then_ h1_2
            · exact Stmt.isCombinationalList_implies_isSequentialList' else_ h1_3
        | switch disc cases default =>
            simp at h1 ⊢
            have h1_1 := h1.1.1
            have h1_2 := h1.1.2
            have h1_3 := h1.2
            constructor
            · constructor
              · exact h1_1
              · exact Stmt.isCombinationalSwitchCaseList_implies_isSequentialSwitchCaseList' cases h1_2
            · exact Stmt.isCombinationalList_implies_isSequentialList' default h1_3
        | «break» =>
            simp at h1
            all_goals contradiction
        | «continue» =>
            simp at h1
            all_goals contradiction
        | forLoop _ _ _ =>
            simp at h1
            all_goals contradiction
        | whileLoop _ _ =>
            simp at h1
            all_goals contradiction
        | return_ e =>
            cases e <;> simp at h1 ⊢ <;> try { exact h1 } <;> contradiction
        | bareCall e =>
            simp at h1 ⊢
            exact h1
      simp [hs, Stmt.isCombinationalList_implies_isSequentialList' ss h2]
  termination_by ss => sizeOf ss
end

/-- Combinational statements are also sequential. -/
@[simp]
theorem Stmt.isCombinational_implies_isSequential (s : Stmt)
    (h : Stmt.isCombinational s = true) : Stmt.isSequential s = true := by
  have h_list : Stmt.isSequentialList' [s] = true := by
    apply Stmt.isCombinationalList_implies_isSequentialList'
    simp [Stmt.isCombinational] at h ⊢
    exact h
  simp at h_list ⊢
  exact h_list

/-- Combinational statement lists (wrapper form) are also sequential. -/
@[simp]
theorem Stmt.isCombinationalList_implies_isSequentialList (ss : List Stmt)
    (h : Stmt.isCombinationalList ss = true) : Stmt.isSequentialList ss = true := by
  simp only [Stmt.isCombinationalList, Stmt.isSequentialList, List.all_eq_true] at h ⊢
  intro s hs
  exact Stmt.isCombinational_implies_isSequential s (h s hs)

/-- Combinational functions are also sequential. -/
@[simp]
theorem Function.isCombinational_implies_isSequential (env : Env) (fn : Function)
    (h : Function.isCombinational env fn = true) : Function.isSequential env fn = true := by
  simp only [Function.isCombinational, Function.isSequential] at h ⊢
  by_cases hhost : Env.isHostOnly env fn.name = true
  · simp only [if_pos hhost] at h ⊢
  · simp only [if_neg hhost] at h ⊢
    simp only [List.all_eq_true] at h ⊢
    intro s hs
    apply Stmt.isCombinational_implies_isSequential
    exact h s hs

/-- Combinational modules are also sequential. -/
@[simp]
theorem Module.isCombinational_implies_isSequential (env : Env) (m : Module)
    (h : Module.isCombinational env m = true) : Module.isSequential env m = true := by
  simp only [Module.isCombinational, Module.isSequential, Bool.and_eq_true,
    List.all_eq_true] at h ⊢
  constructor
  · intro s hs
    apply Stmt.isCombinational_implies_isSequential
    exact h.1 s hs
  · intro fn hfn
    apply Function.isCombinational_implies_isSequential
    exact h.2 fn hfn

/-- Wrapper: type inference with the default predicate fuel. -/
def Expr.typeOf (env : Env) (m : Module) (e : Expr) : Option Ty := e.typeOfFuel predicateFuel env m

def Function.functionNames (fn : Function) : List String :=
  fn.body.flatMap Stmt.functionNames

/-- True when the module contains a function/test/bench with the given name. -/
def Module.hasFunctionNamed (m : Module) (name : String) : Bool :=
  (m.functions ++ m.tests ++ m.benches).any (fun f => f.name == name)

/-- The functions that are actually emitted into shallow Verilog. -/
def Module.emittedFunctions (env : Env) (m : Module) : List Function :=
  m.functions.filter (fun f => !Env.isHostOnly env f.name)

/-- True when the module contains an emitted (non-host-only) function with the
    given name. -/
def Module.hasEmittedFunctionNamed (env : Env) (m : Module) (name : String) : Bool :=
  (Module.emittedFunctions env m).any (fun f => f.name == name)

/-- Context predicate: every function name occurring in an expression is
    reachable in the environment, not host-only, and resolvable to an emitted
    function. -/
def Expr.callContext (env : Env) (m : Module) (e : Expr) : Prop :=
  ∀ x ∈ e.functionNames,
    Env.isReachable env x
    ∧ ¬ Env.isHostOnly env x
    ∧ Module.hasEmittedFunctionNamed env m x

def Stmt.callContext (env : Env) (m : Module) (s : Stmt) : Prop :=
  ∀ x ∈ s.functionNames,
    Env.isReachable env x
    ∧ ¬ Env.isHostOnly env x
    ∧ Module.hasEmittedFunctionNamed env m x

def Stmt.callContextList (env : Env) (m : Module) (ss : List Stmt) : Prop :=
  ∀ s ∈ ss, Stmt.callContext env m s

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

/-- W499 replacement for the reachability bookkeeping: the module's globals
    and every emitted function body satisfy the call-context invariant, i.e.
    every function name that appears is actually present in the module and is
    reachable in the environment.  Host-only helpers and host-side tests/benches
    are not part of the synthesizable model and are skipped. -/
def Module.callContext (env : Env) (m : Module) : Prop :=
  Stmt.callContextList env m m.globals
  ∧ ∀ fn ∈ m.functions, ¬ Env.isHostOnly env fn.name → Stmt.callContextList env m fn.body

end Trinity.IcarusLowerable
