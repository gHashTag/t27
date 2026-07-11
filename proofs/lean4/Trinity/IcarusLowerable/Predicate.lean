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

mutual
  /-- Lowerability of an expression in synthesizable context. -/
  partial def Expr.isLowerable (env : Env) : Expr → Bool
    | .boolLit _ => true
    | .intLit _ => true
    | .f32Lit _ => false
    | .stringLit _ => false
    | .identifier _ => true
    | .binop op lhs rhs => opIsLowerable op && lhs.isLowerable env && rhs.isLowerable env
    | .unop op e => opIsLowerable op && e.isLowerable env
    | .fieldAccess base field =>
        base.isLowerable env &&
        match base with
        | .call name _ =>
            -- scalar struct-return call: field must be leaf-lowerable
            match env.structForConstructor name with
            | some sname =>
                let fields := env.structFields sname
                let found := fields.find? (fun p => p.1 == field)
                match found with
                | some ty => ty.2.isLeafLowerable
                | none => true   -- unknown field: conservatively allow, backend will reject later
            | none => true       -- not a known constructor: conservatively allow
        | _ => true
    | .index base idx =>
        base.isLowerable env && idx.isLowerable env &&
        match base with
        | .fieldAccess (.call _ _) _ =>
            -- array-typed field on scalar struct-return call: leaf must be array of numeric/bool
            true   -- refinement done in fieldAccess above
        | _ => true
    | .call name args =>
        (!Env.isHostOnly env name) &&
        ((name.splitOn "::").length == 1) &&     -- namespace-qualified calls are not lowerable
        (!unlowerableBuiltins.contains name) &&
        args.all (fun a => a.isLowerable env)
    | .structLit _ fields =>
        fields.all (fun p => p.2.isLowerable env)
    | .arrayLit ty elems =>
        ty.isLowerable && elems.all (fun e => e.isLowerable env)
    | .enumVal _ _ => false
    | .len base => base.isLowerable env   -- static .len() is lowered
    | .contains base item => base.isLowerable env && item.isLowerable env
    | .unsupportedIcarus _ => false

  /-- Lowerability of a statement in synthesizable context. -/
  partial def Stmt.isLowerable (env : Env) : Stmt → Bool
    | .assign lhs rhs => lhs.isLowerable env && rhs.isLowerable env
    | .varDecl _ ty init => ty.isLowerable && init.all (fun e => e.isLowerable env)
    | .constDecl _ ty init => ty.isLowerable && init.all (fun e => e.isLowerable env)
    | .ifThenElse cond then_ else_ =>
        cond.isLowerable env && then_.all (Stmt.isLowerable env) && else_.all (Stmt.isLowerable env)
    | .forLoop _ range body =>
        range.isLowerable env && body.all (Stmt.isLowerable env)
    | .return_ e => e.all (fun x => x.isLowerable env)
    | .bareCall e => e.isLowerable env
end

/-- True when an expression is purely combinational (no placeholder nodes). -/
partial def Expr.isCombinational : Expr → Bool
  | .boolLit _ => true
  | .intLit _ => true
  | .identifier _ => true
  | .binop _ lhs rhs => lhs.isCombinational && rhs.isCombinational
  | .unop _ e => e.isCombinational
  | .fieldAccess base _ => base.isCombinational
  | .index base idx => base.isCombinational && idx.isCombinational
  | .call _ args => args.all (·.isCombinational)
  | .structLit _ fields => fields.all (·.2.isCombinational)
  | .arrayLit _ elems => elems.all (·.isCombinational)
  | _ => false

/-- True when a statement is purely combinational: no conditionals or loops. -/
partial def Stmt.isCombinational : Stmt → Bool
  | .assign lhs rhs => lhs.isCombinational && rhs.isCombinational
  | .varDecl _ _ init => init.all (·.isCombinational)
  | .constDecl _ _ init => init.all (·.isCombinational)
  | .return_ e => e.all (·.isCombinational)
  | .bareCall e => e.isCombinational
  | _ => false

/-- True when a function body is purely combinational. -/
def Function.isCombinational (fn : Function) : Bool :=
  fn.body.all Stmt.isCombinational

/-- True when a module is purely combinational. -/
def Module.isCombinational (env : Env) (m : Module) : Bool :=
  m.globals.all Stmt.isCombinational
  && m.functions.all (fun f => (Env.isReachable env f.name) → f.isCombinational)
  && m.tests.all Function.isCombinational
  && m.benches.all Function.isCombinational

/-- Inferred t27 type for the lowerable subset.  Identifiers are typed from the
    environment; function-call types come from the callee's return type or the
    constructor map. -/
partial def Expr.typeOf (env : Env) (m : Module) : Expr → Option Ty
  | .boolLit _ => some .bool
  | .intLit _ => some .u32
  | .identifier name => env.varType name
  | .binop op lhs _ =>
      if ["==", "!=", "<", "<=", ">", ">=", "&&", "||"].contains op then some .bool
      else Expr.typeOf env m lhs
  | .unop _ e => Expr.typeOf env m e
  | .fieldAccess base field => do
      let ty ← Expr.typeOf env m base
      match ty with
      | .struct sname =>
          let fields := env.structFields sname
          let f ← fields.find? (fun p => p.1 == field)
          some f.2
      | _ => none
  | .index base _ => do
      let ty ← Expr.typeOf env m base
      match ty with
      | .array _ elem => some elem
      | _ => none
  | .call name _ =>
      match env.structForConstructor name with
      | some sname => some (.struct sname)
      | none =>
          match m.findFunction name with
          | some fn => fn.ret
          | none => none
  | .structLit name _ => some (.struct name)
  | .arrayLit ty _ => some ty
  | _ => none

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

end Trinity.IcarusLowerable
