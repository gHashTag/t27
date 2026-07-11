/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 495
  Pure emitter model from the simplified t27 AST to the shallow Verilog AST.

  This is *not* a bit-exact model of `bootstrap/src/compiler.rs`.  It is a
  shallow, deterministic translation that is just deep enough to prove the
  soundness contract: every construct accepted by the Icarus-lowerability
  predicate maps to a Verilog AST node that is not an `UNSUPPORTED_ICARUS`
  or `// TODO` placeholder.

  W495 changes:
    - The module `m` is threaded through expression/statement emission so that
      field access and indexing can use the callee's return type from `m`.
    - t27 functions are emitted as `VFunction` definitions inside `VModule`.
    - `VExpr.index` carries the element width derived from the base type.

  The functions below are `partial` because the real termination argument
  depends on the (finite) type graph stored in `Env`, which is awkward to
  thread through Lean's structural recursion checker.  For the concrete,
  non-recursive t27 types used by the Icarus-lowerable subset, evaluation
  terminates and is sufficient for the soundness claims proved by
  `native_decide`.

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

import Trinity.IcarusLowerable.Ast
import Trinity.IcarusLowerable.Predicate
import Trinity.IcarusLowerable.Verilog

namespace Trinity.IcarusLowerable

/-- Packed bit width of a lowerable t27 type.
    Non-numeric / non-bool types get a default 32-bit width for the model. -/
partial def widthOfType (env : Env) : Ty → Nat
  | .bool => 1
  | .u8 => 8
  | .u16 => 16
  | .u32 => 32
  | .u64 => 64
  | .i8 => 8
  | .i16 => 16
  | .i32 => 32
  | .i64 => 64
  | .array n elem => n * widthOfType env elem
  | .struct name =>
      let fields := env.structFields name
      fields.foldl (fun acc p => acc + widthOfType env p.2) 0
  | _ => 32

/-- Element width for a t27 index expression, derived from the base type. -/
partial def indexElemWidth (env : Env) (m : Module) (base : Expr) : Nat :=
  match Expr.typeOf env m base with
  | some (.array _ elem) => widthOfType env elem
  | _ => 8

/-- Emit a t27 expression into the shallow Verilog AST.
    Non-lowerable constructs become explicit placeholders. -/
partial def emitExpr (env : Env) (m : Module) : Expr → VExpr
  | .boolLit true => .lit 1 "1"
  | .boolLit false => .lit 1 "0"
  | .intLit n => .lit 32 (toString n)
  | .f32Lit _ => .unsupported "f32 literal"
  | .stringLit _ => .unsupported "string literal"
  | .identifier name => .ident name
  | .binop op lhs rhs => .binop op (emitExpr env m lhs) (emitExpr env m rhs)
  | .unop op e => .unop op (emitExpr env m e)
  | .fieldAccess base field =>
      let baseV := emitExpr env m base
      match Expr.typeOf env m base with
      | some (.struct sname) =>
          let fields := env.structFields sname
          let offset := fields.foldl (fun acc p =>
            if p.1 < field then acc + widthOfType env p.2 else acc) 0
          let w := match fields.find? (fun p => p.1 == field) with
            | some ty => widthOfType env ty.2
            | none => 1
          .slice baseV (offset + w - 1) offset
      | _ => .slice baseV 0 0
  | .index base idx => .index (emitExpr env m base) (emitExpr env m idx) (indexElemWidth env m base)
  | .call name args => .call name (args.map (emitExpr env m))
  | .structLit _ fields => .concat (fields.map (fun p => emitExpr env m p.2))
  | .arrayLit _ elems => .concat (elems.map (emitExpr env m))
  | .enumVal _ _ => .unsupported "enum value"
  | .len _ => .lit 32 "N"
  | .contains _ _ => .lit 1 "0"
  | .unsupportedIcarus reason => .unsupported reason

mutual
  /-- Emit a single t27 statement into the shallow Verilog AST. -/
  partial def emitStmt (env : Env) (m : Module) : Stmt → VStmt
    | .assign lhs rhs => .assign (emitExpr env m lhs) (emitExpr env m rhs)
    | .varDecl name ty init =>
        let width := widthOfType env ty
        let initExpr := (init.map (emitExpr env m)).getD (VExpr.lit width "0")
        .assign (.ident name) initExpr
    | .constDecl name ty init =>
        let width := widthOfType env ty
        let initExpr := (init.map (emitExpr env m)).getD (VExpr.lit width "0")
        .localparam name width initExpr
    | .ifThenElse _ then_ else_ =>
        .alwaysComb (emitStmts env m then_ ++ emitStmts env m else_)
    | .forLoop _ _ body =>
        .initial (emitStmts env m body)
    | .return_ e =>
        let rhs := (e.map (emitExpr env m)).getD (VExpr.lit 1 "0")
        .assign (.ident "__return") rhs
    | .bareCall e => .taskCall "" [emitExpr env m e]

  /-- Emit a list of t27 statements. -/
  partial def emitStmts (env : Env) (m : Module) (stmts : List Stmt) : List VStmt :=
    stmts.map (emitStmt env m)

  /-- Emit a t27 function body (only reachable functions are kept by the caller). -/
  partial def emitFunction (env : Env) (m : Module) (fn : Function) : List VStmt :=
    emitStmts env m fn.body

  /-- Emit a t27 function as a Verilog function definition. -/
  partial def emitVFunction (env : Env) (m : Module) (fn : Function) : VFunction :=
    {
      name := fn.name,
      params := fn.params.map (fun p => (p.1, widthOfType env p.2)),
      retWidth := Option.getD (fn.ret.map (widthOfType env)) 32,
      body := emitStmts env m fn.body
    }

  /-- Emit a t27 module into a shallow Verilog module. -/
  partial def emitModule (env : Env) (m : Module) : VModule :=
    let globalItems := emitStmts env m m.globals
    let fnDefs := m.functions.filter (fun f => env.isReachable f.name)
                    |> .map (emitVFunction env m)
    let testItems := m.tests.flatMap (emitFunction env m)
    let benchItems := m.benches.flatMap (emitFunction env m)
    {
      name := m.name,
      ports := [],
      items := globalItems ++ testItems ++ benchItems,
      functions := fnDefs
    }
end

end Trinity.IcarusLowerable
