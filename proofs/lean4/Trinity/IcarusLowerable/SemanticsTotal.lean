/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 497
  Total, fuel-based denotational semantics for the simplified t27 AST and the
  shallow Verilog AST, restricted to the Icarus-lowerable scalar subset.

  The functions below mirror `Semantics.lean` but are total by structural
  recursion on an explicit `fuel : Nat` parameter.  This makes them transparent
  to proofs and allows a generic structural equivalence theorem.

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

import Trinity.IcarusLowerable.Ast
import Trinity.IcarusLowerable.Predicate
import Trinity.IcarusLowerable.Verilog
import Trinity.IcarusLowerable.Emitter
import Trinity.IcarusLowerable.Semantics

namespace Trinity.IcarusLowerable

/-- Compute the bit offset of a struct field using fuel-accounted widths. -/
def structFieldOffsetTotal (fuel : Nat) (env : Env) (sname : String) (field : String) : Nat :=
  let fields := env.structFields sname
  fields.foldl (fun acc p => if p.1 < field then acc + widthOfType fuel env p.2 else acc) 0

/-- Compute the width of a struct field using fuel-accounted widths. -/
def structFieldWidthTotal (fuel : Nat) (env : Env) (sname : String) (field : String) : Nat :=
  match (env.structFields sname).find? (fun p => p.1 == field) with
  | some ty => widthOfType fuel env ty.2
  | none => 1

mutual
  /-- Total t27 expression evaluator (fuel-based). -/
  def evalExprTotal (fuel : Nat) (env : Env) (m : Module) (val : Valuation) (e : Expr) : Option Value :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match e with
      | .boolLit true => some ⟨1, 1#1⟩
      | .boolLit false => some ⟨1, 0#1⟩
      | .intLit n => some ⟨32, BitVec.ofInt 32 n⟩
      | .identifier name => val name
      | .binop op lhs rhs => do
          let l <- evalExprTotal fuel env m val lhs
          let r <- evalExprTotal fuel env m val rhs
          evalBinop op l r
      | .unop op e => do
          let v <- evalExprTotal fuel env m val e
          evalUnop op v
      | .fieldAccess base field =>
          match Expr.typeOf env m base with
          | some (.struct sname) => do
              let v <- evalExprTotal fuel env m val base
              let off := structFieldOffsetTotal fuel env sname field
              let w := structFieldWidthTotal fuel env sname field
              let hi := off + w - 1
              if _h : off ≤ hi && hi < v.width then
                some ⟨hi - off + 1, BitVec.extractLsb' off (hi - off + 1) v.bits⟩
              else
                none
          | _ => do
              let v <- evalExprTotal fuel env m val base
              if _h : v.width > 0 then
                some ⟨1, BitVec.extractLsb' 0 1 v.bits⟩
              else
                none
      | .index base idx => do
          let b <- evalExprTotal fuel env m val base
          let i <- evalExprTotal fuel env m val idx
          let n := i.bits.toNat
          let elemW := match Expr.typeOf env m base with
            | some (.array _ elem) => widthOfType fuel env elem
            | _ => 8
          if _h : elemW > 0 && n * elemW + elemW - 1 < b.width then
            some ⟨elemW, BitVec.extractLsb' (n * elemW) elemW b.bits⟩
          else
            none
      | .call name args =>
          match m.findFunction name with
          | some fn => do
              let argVals <- args.mapM (evalExprTotal fuel env m val)
              evalFunctionTotal fuel env m fn argVals val
          | none => none
      | .structLit _ fields => do
          let vs <- fields.mapM (fun p => evalExprTotal fuel env m val p.2)
          some (Value.concatList vs)
      | .arrayLit _ elems => do
          let vs <- elems.mapM (evalExprTotal fuel env m val)
          some (Value.concatList vs)
      | _ => none

  /-- Total function-body evaluator. -/
  def evalFunctionTotal (fuel : Nat) (env : Env) (m : Module) (fn : Function) (argVals : List Value) (base : Valuation) : Option Value :=
    match fuel with
    | 0 => none
    | fuel+1 => do
      let paramBinds := fn.params.zip argVals
      let init : Valuation := fun name =>
        paramBinds.find? (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name)
      let final <- evalStmtsTotal fuel env m init fn.body
      final "__return"

  /-- Total statement evaluator. -/
  def evalStmtTotal (fuel : Nat) (env : Env) (m : Module) (val : Valuation) (stmt : Stmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match stmt with
      | .assign (.identifier name) rhs => do
          let v <- evalExprTotal fuel env m val rhs
          some (fun x => if x == name then some v else val x)
      | .varDecl name ty init => do
          let v <- match init with
                  | some e => evalExprTotal fuel env m val e
                  | none => some ⟨widthOfType fuel env ty, 0#(widthOfType fuel env ty)⟩
          some (fun x => if x == name then some v else val x)
      | .constDecl name ty init => do
          let v <- match init with
                  | some e => evalExprTotal fuel env m val e
                  | none => some ⟨widthOfType fuel env ty, 0#(widthOfType fuel env ty)⟩
          some (fun x => if x == name then some v else val x)
      | .return_ (some e) => do
          let v <- evalExprTotal fuel env m val e
          some (fun x => if x == "__return" then some v else val x)
      | .return_ none => some val
      | _ => some val

  /-- Total statement-list evaluator. -/
  def evalStmtsTotal (fuel : Nat) (env : Env) (m : Module) (val : Valuation) (stmts : List Stmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match stmts with
      | [] => some val
      | stmt :: rest => do
          let val' <- evalStmtTotal fuel env m val stmt
          evalStmtsTotal fuel env m val' rest
end

/-- Total module evaluator: globals, then named function. -/
def evalModuleFunctionTotal (fuel : Nat) (env : Env) (m : Module) (fnName : String) (args : List Value) : Option Value :=
  match evalStmtsTotal fuel env m (fun _ => none) m.globals with
  | some initVal =>
      match m.findFunction fnName with
      | some fn => evalFunctionTotal fuel env m fn args initVal
      | none => none
  | none => none

mutual
  /-- Total shallow-Verilog expression evaluator. -/
  def evalVExprTotal (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation) (e : VExpr) : Option Value :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match e with
      | .lit w s =>
          match String.toInt? s with
          | some n => some ⟨w, BitVec.ofInt w n⟩
          | none => none
      | .ident name => val name
      | .binop op lhs rhs => do
          let l <- evalVExprTotal fuel env vm val lhs
          let r <- evalVExprTotal fuel env vm val rhs
          evalBinop op l r
      | .unop op e => do
          let v <- evalVExprTotal fuel env vm val e
          evalUnop op v
      | .index base idx elemW => do
          let b <- evalVExprTotal fuel env vm val base
          let i <- evalVExprTotal fuel env vm val idx
          let n := i.bits.toNat
          if _h : elemW > 0 && n * elemW + elemW - 1 < b.width then
            some ⟨elemW, BitVec.extractLsb' (n * elemW) elemW b.bits⟩
          else
            none
      | .slice base hi lo => do
          let b <- evalVExprTotal fuel env vm val base
          if _h : lo ≤ hi && hi < b.width then
            some ⟨hi - lo + 1, BitVec.extractLsb' lo (hi - lo + 1) b.bits⟩
          else
            none
      | .concat parts => do
          let vs <- parts.mapM (evalVExprTotal fuel env vm val)
          some (Value.concatList vs)
      | .call name args => do
          let argVals <- args.mapM (evalVExprTotal fuel env vm val)
          match vm.functions.find? (fun f => f.name == name) with
          | some fn => evalVFunctionTotal fuel env vm fn argVals val
          | none => none
      | .unsupported _ => none
      | .todo _ => none

  /-- Total shallow-Verilog function-body evaluator. -/
  def evalVFunctionTotal (fuel : Nat) (env : Env) (vm : VModule) (fn : VFunction) (argVals : List Value) (base : Valuation) : Option Value :=
    match fuel with
    | 0 => none
    | fuel+1 => do
      let paramBinds := fn.params.zip argVals
      let init : Valuation := fun name =>
        paramBinds.find? (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name)
      let final <- evalVStmtsTotal fuel env vm init fn.body
      final "__return"

  /-- Total shallow-Verilog statement evaluator. -/
  def evalVStmtTotal (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation) (stmt : VStmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match stmt with
      | .assign lhs rhs => do
          let name := match lhs with | .ident n => n | _ => ""
          let v <- evalVExprTotal fuel env vm val rhs
          some (fun x => if x == name then some v else val x)
      | .localparam name _ init => do
          let v <- evalVExprTotal fuel env vm val init
          some (fun x => if x == name then some v else val x)
      | .wire _ _ => some val
      | .reg _ _ => some val
      | .alwaysComb body => evalVStmtsTotal fuel env vm val body
      | .initial body => evalVStmtsTotal fuel env vm val body
      | .taskCall _ _ => some val

  /-- Total shallow-Verilog statement-list evaluator. -/
  def evalVStmtsTotal (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation) (stmts : List VStmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match stmts with
      | [] => some val
      | stmt :: rest => do
          let val' <- evalVStmtTotal fuel env vm val stmt
          evalVStmtsTotal fuel env vm val' rest
end

/-- Total shallow-Verilog module evaluator. -/
def evalVModuleTotal (fuel : Nat) (env : Env) (vm : VModule) (fnName : String) (args : List Value) : Option Value :=
  match evalVStmtsTotal fuel env vm (fun _ => none) vm.globals with
  | some initVal =>
      match vm.functions.find? (fun f => f.name == fnName) with
      | some fn => evalVFunctionTotal fuel env vm fn args initVal
      | none => none
  | none => none

end Trinity.IcarusLowerable
