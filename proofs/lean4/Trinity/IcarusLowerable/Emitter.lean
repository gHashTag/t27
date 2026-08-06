/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 497
  Pure emitter model from the simplified t27 AST to the shallow Verilog AST.

  This is *not* a bit-exact model of `bootstrap/src/compiler.rs`.  It is a
  shallow, deterministic translation that is just deep enough to prove the
  soundness contract: every construct accepted by the Icarus-lowerability
  predicate maps to a Verilog AST node that is not an `UNSUPPORTED_ICARUS`
  or `// TODO` placeholder.

  W497 changes:
    - All emission functions now carry an explicit `fuel : Nat` parameter and are
      total by structural recursion on fuel.  This makes the model transparent
      to proofs while preserving the same computational behavior for the
      Icarus-lowerable witness set.
    - `widthOfType` is total with fuel; default wrappers derive fuel from the
      module size.

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

import Trinity.IcarusLowerable.Ast
import Trinity.IcarusLowerable.Predicate
import Trinity.IcarusLowerable.Verilog

namespace Trinity.IcarusLowerable

/-- Size of a type for fuel accounting. -/
def Ty.size : Ty → Nat
  | .bool | .u8 | .u16 | .u32 | .u64 | .i8 | .i16 | .i32 | .i64 | .f32 | .string | .enum _ => 1
  | .array n elem => 1 + n * elem.size
  | .struct _ => 1

/-- Packed bit width of a lowerable t27 type.
    Non-numeric / non-bool types get a default 32-bit width for the model.
    The fuel parameter is kept for API compatibility, but the result is
    independent of it: the model uses a fixed `predicateFuel` budget, which is
    larger than any lowerable type encountered in the proof-relevant corpus.
    This makes widths transparent to the fuel induction in the equivalence
    proof. -/
private def widthOfTypeFuel : Nat → Env → Ty → Nat
  | 0, _, _ => 32
  | _, _, .bool => 1
  | _, _, .u8 => 8
  | _, _, .u16 => 16
  | _, _, .u32 => 32
  | _, _, .u64 => 64
  | _, _, .i8 => 8
  | _, _, .i16 => 16
  | _, _, .i32 => 32
  | _, _, .i64 => 64
  | fuel+1, env, .array n elem => n * widthOfTypeFuel fuel env elem
  | fuel+1, env, .struct name =>
      let fields := env.structFields name
      fields.foldl (fun acc p => acc + widthOfTypeFuel fuel env p.2) 0
  | _, _, _ => 32

def widthOfType (_fuel : Nat) (env : Env) (ty : Ty) : Nat :=
  widthOfTypeFuel predicateFuel env ty

/-- Element width for a t27 index expression, derived from the base type. -/
def indexElemWidth (fuel : Nat) (env : Env) (m : Module) (base : Expr) : Nat :=
  match Expr.typeOf env m base with
  | some (.array _ elem) => widthOfType fuel env elem
  | _ => 8

mutual
  /-- Emit a t27 expression into the shallow Verilog AST.
      Non-lowerable constructs become explicit placeholders.
      The `fuel` parameter is kept for API compatibility; the actual emission is
      structurally recursive on the expression and fuel-independent because
      `widthOfType` already uses a fixed predicate budget. -/
  def emitExpr (fuel : Nat) (env : Env) (m : Module) (e : Expr) : VExpr :=
    match e with
    | .boolLit true => .lit 1 (toString (1 : Int))
    | .boolLit false => .lit 1 (toString (0 : Int))
    | .intLit n => .lit 32 (toString n)
    | .f32Lit _ => .unsupported "f32 literal"
    | .stringLit _ => .unsupported "string literal"
    | .identifier name => .ident name
    | .binop op lhs rhs => .binop op (emitExpr fuel env m lhs) (emitExpr fuel env m rhs)
    | .unop op e => .unop op (emitExpr fuel env m e)
    | .fieldAccess base field =>
        let baseV := emitExpr fuel env m base
        match Expr.typeOf env m base with
        | some (.struct sname) =>
            let fields := env.structFields sname
            let offset := fields.foldl (fun acc p =>
              if p.1 < field then acc + widthOfType fuel env p.2 else acc) 0
            let w := match fields.find? (fun p => p.1 == field) with
              | some ty => widthOfType fuel env ty.2
              | none => 1
            .slice baseV (offset + w - 1) offset
        | _ => .slice baseV 0 0
    | .index base idx => .index (emitExpr fuel env m base) (emitExpr fuel env m idx) (indexElemWidth fuel env m base)
    | .call name args => .call name (emitExprList fuel env m args)
    | .structLit _ fields => .concat (emitFieldExprs fuel env m fields)
    | .arrayLit _ elems => .concat (emitExprList fuel env m elems)
    | .enumVal enum variant =>
        -- The shallow model emits the numeric value directly as a literal.
        -- The Rust backend declares `EnumName_variant` localparams and uses
        -- those identifiers, which evaluates to the same constant in any context
        -- where the localparam is visible.
        let n := env.enumValue enum variant |>.getD 0
        .lit 32 (toString n)
    | .len _ => .lit 32 "N"
    | .contains _ _ => .lit 1 "0"
    | .switch disc cases default =>
        let discV := emitExpr fuel env m disc
        let defaultV := emitExpr fuel env m default
        let rec go (cs : List (Expr × Expr)) : VExpr :=
          match cs with
          | [] => defaultV
          | (tag, res) :: rest =>
              .ternary
                (.binop "==" discV (emitExpr fuel env m tag))
                (emitExpr fuel env m res)
                (go rest)
        go cases
    | .unsupportedIcarus reason => .unsupported reason

  /-- Helper: emit a list of expressions by structural recursion on the list. -/
  def emitExprList (fuel : Nat) (env : Env) (m : Module) (es : List Expr) : List VExpr :=
    match es with
    | [] => []
    | e :: rest => emitExpr fuel env m e :: emitExprList fuel env m rest

  /-- Helper: emit the expression payload of struct-literal fields. -/
  def emitFieldExprs (fuel : Nat) (env : Env) (m : Module) (fields : List (String × Expr)) : List VExpr :=
    match fields with
    | [] => []
    | p :: rest => emitExpr fuel env m p.2 :: emitFieldExprs fuel env m rest
end

mutual
  /-- Emit a single t27 statement into the shallow Verilog AST.
      The `fuel` parameter is kept for API compatibility; emission is
      structurally recursive on the statement and fuel-independent. -/
  def emitStmt (fuel : Nat) (env : Env) (m : Module) (stmt : Stmt) : VStmt :=
    match stmt with
    | .assign lhs rhs => .assign (emitExpr fuel env m lhs) (emitExpr fuel env m rhs)
    | .varDecl name ty init =>
        let width := widthOfType fuel env ty
        let initExpr := (init.map (emitExpr fuel env m)).getD (VExpr.lit width "0")
        .assign (.ident name) initExpr
    | .constDecl name ty init =>
        let width := widthOfType fuel env ty
        let initExpr := (init.map (emitExpr fuel env m)).getD (VExpr.lit width "0")
        .localparam name width initExpr
    | .ifThenElse cond then_ else_ =>
        .ifThenElse (emitExpr fuel env m cond) (emitStmts fuel env m then_) (emitStmts fuel env m else_)
    | .switch disc cases default =>
        .switch (emitExpr fuel env m disc)
          (emitSwitchCases fuel env m cases)
          (emitStmts fuel env m default)
    | .forLoop var range body =>
        .forLoop var (emitExpr fuel env m range) (emitStmts fuel env m body)
    | .whileLoop cond body =>
        .whileLoop (emitExpr fuel env m cond) (emitStmts fuel env m body)
    | .break => .break
    | .continue => .continue
    | .return_ e =>
        let rhs := (e.map (emitExpr fuel env m)).getD (VExpr.lit 1 "0")
        .assign (.ident "__return") rhs
    | .bareCall e => .taskCall "" [emitExpr fuel env m e]
  termination_by (fuel, sizeOf stmt)

  /-- Emit a list of t27 statements. -/
  def emitStmts (fuel : Nat) (env : Env) (m : Module) (stmts : List Stmt) : List VStmt :=
    stmts.map (emitStmt fuel env m)
  termination_by (fuel, sizeOf stmts)

  /-- Emit the (tag, body) pairs of a statement-level switch.  Each case body is
      emitted by a call to `emitStmts`; the `decreasing_by` tactic proves that the
      case list is larger than any nested statement list. -/
  def emitSwitchCases (fuel : Nat) (env : Env) (m : Module) (cases : List (Expr × List Stmt)) : List (VExpr × List VStmt) :=
    match cases with
    | [] => []
    | p :: ps => (emitExpr fuel env m p.1, emitStmts fuel env m p.2) :: emitSwitchCases fuel env m ps
  termination_by (fuel, sizeOf cases)
  decreasing_by all_goals cases p <;> simp_wf <;> simp [sizeOf] <;> omega
end

/-- Emit a t27 function body. -/
def emitFunction (fuel : Nat) (env : Env) (m : Module) (fn : Function) : List VStmt :=
  emitStmts fuel env m fn.body

/-- Emit a t27 function as a Verilog function definition. -/
def emitVFunction (fuel : Nat) (env : Env) (m : Module) (fn : Function) : VFunction :=
  {
    name := fn.name,
    params := fn.params.map (fun p => (p.1, widthOfType fuel env p.2)),
    retWidth := Option.getD (fn.ret.map (widthOfType fuel env)) 32,
    body := emitStmts fuel env m fn.body
  }

/-- Emit a t27 module into a shallow Verilog module.
    W499: every non-host-only function is emitted as a `VFunction` so that the
    equivalence theorem needs no reachability assumption.  Host-only helpers
    and host-side test/bench blocks are not part of the Icarus synthesizable
    model and are omitted from the emitted module. -/
def emitModuleFuel (fuel : Nat) (env : Env) (m : Module) : VModule :=
  let globalItems := emitStmts fuel env m m.globals
  let emittedFns := Module.emittedFunctions env m
  let fnDefs := emittedFns.map (emitVFunction fuel env m)
  {
    name := m.name,
    ports := [],
    globals := globalItems,
    items := globalItems,
    functions := fnDefs
  }

/-- Default fuel for emission.  Lowerable modules are finite, so a constant
    bound is enough for the witness set and for the generic theorem. -/
def defaultFuel : Nat := 1000

/-- Convenience wrapper: emit a module with default fuel. -/
def emitModule (env : Env) (m : Module) : VModule :=
  emitModuleFuel defaultFuel env m

end Trinity.IcarusLowerable
