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

/-- Sentinel variable used to signal a `break` inside loop bodies. -/
def breakFlag : String := "__break"

/-- Sentinel variable used to signal a `continue` inside loop bodies. -/
def continueFlag : String := "__continue"

/-- Sentinel variable used to store a function return value. -/
def returnFlag : String := "__return"

/-- True when the valuation has an exit sentinel set (break, continue, or return). -/
def hasExitFlag (val : Valuation) : Bool :=
  (val breakFlag).isSome || (val continueFlag).isSome || (val returnFlag).isSome

/-- True when the break sentinel is set. -/
def isBreakFlagSet (val : Valuation) : Bool := (val breakFlag).isSome

/-- True when the continue sentinel is set. -/
def isContinueFlagSet (val : Valuation) : Bool := (val continueFlag).isSome

/-- True when the return sentinel is set. -/
def isReturnFlagSet (val : Valuation) : Bool := (val returnFlag).isSome

/-- Set a sentinel variable in a valuation to the given value.  For `break` and
    `continue` the payload is a one-bit placeholder; only the presence of a
    `some` value is tested. -/
def setFlag (val : Valuation) (flag : String) (v : Value) : Valuation :=
  fun x => if x == flag then some v else val x

/-- Clear the loop-related sentinels (break and continue) from a valuation.
    The return sentinel is preserved so that a `return` inside a loop still
    propagates to the function caller. -/
def clearLoopFlags (val : Valuation) : Valuation :=
  fun x => if x == breakFlag || x == continueFlag then none else val x

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
      | .enumVal enum variant => do
          let n := env.enumValue enum variant |>.getD 0
          some ⟨32, BitVec.ofInt 32 n⟩
      | .switch disc cases default =>
          evalSwitchCasesTotal (fuel + 1) env m val disc default cases
      | _ => none
  termination_by (fuel, sizeOf e)

  /-- Switch-case walker.  It mirrors the emitted nested ternary `disc == tag ? res : ...`.
      The emitted Verilog consumes one fuel level for the ternary and one for the
      `==` comparison, so each case needs two fuel levels.  The discriminant and tag
      are therefore evaluated at `fuel - 2`, the result and the tail at `fuel - 1`. -/
  def evalSwitchCasesTotal (fuel : Nat) (env : Env) (m : Module) (val : Valuation)
      (disc : Expr) (default : Expr) (cs : List (Expr × Expr)) : Option Value :=
    match cs with
    | [] => evalExprTotal fuel env m val default
    | (tag, res) :: rest =>
        match fuel with
        | 0 => none
        | 1 => none
        | fuel+2 => do
            let d <- evalExprTotal fuel env m val disc
            let t <- evalExprTotal fuel env m val tag
            let eq <- evalBinop "==" d t
            if eq.bits.toNat > 0 then evalExprTotal (fuel+1) env m val res
            else evalSwitchCasesTotal (fuel+1) env m val disc default rest
  termination_by (fuel, sizeOf cs + sizeOf disc + sizeOf default)

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
  termination_by (fuel, 0)

  /-- Helper: execute a t27 for-loop body `n` times, binding `var` to `i`, `i+1`, ...
      W504: each iteration consumes one fuel unit so that the `all_equiv`
      induction hypothesis at the smaller fuel covers the loop body.
      W508: the loop consumes `break`/`continue` sentinels; `break` exits the
      loop and `continue` proceeds to the next iteration.  Loop flags are
      cleared on normal exit. -/
  def evalForLoopTotal (fuel : Nat) (env : Env) (m : Module) (val : Valuation) (var : String) (i : Nat) (n : Nat) (body : List Stmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match n with
      | 0 => some (clearLoopFlags val)
      | n+1 => do
          let loopVal := fun x => if x == var then some ⟨32, BitVec.ofNat 32 i⟩ else val x
          let val' <- evalStmtsTotal fuel env m loopVal body
          if isBreakFlagSet val' || isReturnFlagSet val' then
            some (clearLoopFlags val')
          else
            evalForLoopTotal fuel env m (clearLoopFlags val') var (i + 1) n body
  termination_by (fuel, n)

  /-- Helper: execute a t27 while-loop body until the condition becomes false or
      fuel runs out.  W507: each iteration consumes one fuel unit, the body runs
      at the smaller fuel, and the combinational condition is re-evaluated after
      every iteration.  W508: `break` exits the loop and `continue` re-evaluates
      the condition; loop flags are cleared on normal exit. -/
  def evalWhileLoopTotal (fuel : Nat) (env : Env) (m : Module) (val : Valuation)
      (cond : Expr) (body : List Stmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 => do
        let c <- evalExprTotal fuel env m val cond
        if c.bits.toNat > 0 then
          let val' <- evalStmtsTotal fuel env m val body
          if isBreakFlagSet val' || isReturnFlagSet val' then
            some (clearLoopFlags val')
          else
            evalWhileLoopTotal fuel env m (clearLoopFlags val') cond body
        else
          some (clearLoopFlags val)
  termination_by (fuel, 0)

  /-- Helper: execute a t27 switch statement by matching the discriminant value
      against each case tag and running the first matching body (or the default).
      W506: the discriminant is evaluated once by the caller at the statement fuel;
      tags and chosen bodies are evaluated at the same smaller fuel. -/
  def evalSwitchStmtCasesTotal (fuel : Nat) (env : Env) (m : Module) (val : Valuation)
      (discV : Value) (default : List Stmt) (cs : List (Expr × List Stmt)) : Option Valuation :=
    match cs with
    | [] => evalStmtsTotal fuel env m val default
    | (tag, body) :: rest => do
        let t <- evalExprTotal fuel env m val tag
        let eq <- evalBinop "==" discV t
        if eq.bits.toNat > 0 then evalStmtsTotal fuel env m val body
        else evalSwitchStmtCasesTotal fuel env m val discV default rest
  termination_by (fuel, sizeOf cs + sizeOf default)

  /-- Compute the root identifier name, the bit offset inside that root, and the
      width of the slice addressed by an lvalue expression.  Only identifier-based
      lvalues are supported; this matches the packed scalar-struct local lowering.
      The index expressions are evaluated to determine dynamic offsets. -/
  def assignTargetOffsetWidth (fuel : Nat) (env : Env) (m : Module) (val : Valuation)
      (e : Expr) : Option (String × Nat × Nat) :=
    match e with
    | .identifier name => (val name).map (fun v => (name, 0, v.width))
    | .fieldAccess base field => do
        let (name, off, _) <- assignTargetOffsetWidth fuel env m val base
        match Expr.typeOf env m base with
        | some (.struct sname) =>
            let foff := structFieldOffsetTotal fuel env sname field
            let fw := structFieldWidthTotal fuel env sname field
            some (name, off + foff, fw)
        | _ => none
    | .index base idx => do
        let (name, off, _) <- assignTargetOffsetWidth fuel env m val base
        let i <- evalExprTotal fuel env m val idx
        let n := i.bits.toNat
        let elemW := match Expr.typeOf env m base with
          | some (.array _ elem) => widthOfType fuel env elem
          | _ => 8
        some (name, off + n * elemW, elemW)
    | _ => none
    termination_by (fuel, sizeOf e)

  /-- Total statement evaluator. -/
  def evalStmtTotal (fuel : Nat) (env : Env) (m : Module) (val : Valuation) (stmt : Stmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match stmt with
      | .assign (.identifier name) rhs => do
          let v <- evalExprTotal fuel env m val rhs
          some (fun x => if x == name then some v else val x)
      | .assign lhs rhs => do
          let rv <- evalExprTotal fuel env m val rhs
          match assignTargetOffsetWidth fuel env m val lhs with
          | some (name, off, w) =>
              if _h : rv.width = w then
                let old <- val name
                let newV <- Value.replaceSlice old rv off
                some (fun x => if x == name then some newV else val x)
              else
                none
          | none => none
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
      | .ifThenElse cond then_ else_ => do
          let c <- evalExprTotal fuel env m val cond
          if c.bits.toNat > 0 then evalStmtsTotal fuel env m val then_
          else evalStmtsTotal fuel env m val else_
      | .forLoop var range body => do
          let r <- evalExprTotal fuel env m val range
          evalForLoopTotal fuel env m val var 0 r.bits.toNat body
      | .whileLoop cond body => do
          evalWhileLoopTotal fuel env m val cond body
      | .switch disc cases default => do
          let d <- evalExprTotal fuel env m val disc
          evalSwitchStmtCasesTotal fuel env m val d default cases
      | .return_ (some e) => do
          let v <- evalExprTotal fuel env m val e
          some (setFlag val returnFlag v)
      | .return_ none => some val
      | .break => some (setFlag val breakFlag ⟨1, 1#1⟩)
      | .continue => some (setFlag val continueFlag ⟨1, 1#1⟩)
      | _ => some val
  termination_by (fuel, sizeOf stmt)

  /-- Total statement-list evaluator.  W508: if an exit sentinel (break,
      continue, or return) is already set, the remainder of the list is skipped.
      This scopes early-exit to the current statement sequence. -/
  def evalStmtsTotal (fuel : Nat) (env : Env) (m : Module) (val : Valuation) (stmts : List Stmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      if hasExitFlag val then some val
      else
        match stmts with
        | [] => some val
        | stmt :: rest => do
            let val' <- evalStmtTotal fuel env m val stmt
            evalStmtsTotal fuel env m val' rest
  termination_by (fuel, sizeOf stmts)
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
      | .ternary cond then_ else_ => do
          let c <- evalVExprTotal fuel env vm val cond
          if c.bits.toNat > 0 then evalVExprTotal fuel env vm val then_
          else evalVExprTotal fuel env vm val else_
      | .unsupported _ => none
      | .todo _ => none
  termination_by (fuel, sizeOf e)

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
  termination_by (fuel, 0)

  /-- Helper: execute a shallow-Verilog for-loop body `n` times, binding `var` to `i`, `i+1`, ...
      W504: each iteration consumes one fuel unit so that the `all_equiv`
      induction hypothesis at the smaller fuel covers the loop body.
      W508: mirrors the t27 side: `break` exits and `continue` recurses. -/
  def evalVForLoopTotal (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation) (var : String) (i : Nat) (n : Nat) (body : List VStmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match n with
      | 0 => some (clearLoopFlags val)
      | n+1 => do
          let loopVal := fun x => if x == var then some ⟨32, BitVec.ofNat 32 i⟩ else val x
          let val' <- evalVStmtsTotal fuel env vm loopVal body
          if isBreakFlagSet val' || isReturnFlagSet val' then
            some (clearLoopFlags val')
          else
            evalVForLoopTotal fuel env vm (clearLoopFlags val') var (i + 1) n body
  termination_by (fuel, n)

  /-- Helper: execute a shallow-Verilog while-loop body until the condition becomes
      false or fuel runs out.
      W507: mirrors `evalWhileLoopTotal`.
      W508: consumes and clears break/continue/return flags after each iteration. -/
  def evalVWhileLoopTotal (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation)
      (cond : VExpr) (body : List VStmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 => do
        let c <- evalVExprTotal fuel env vm val cond
        if c.bits.toNat > 0 then
          let val' <- evalVStmtsTotal fuel env vm val body
          if isBreakFlagSet val' || isReturnFlagSet val' then
            some (clearLoopFlags val')
          else
            evalVWhileLoopTotal fuel env vm (clearLoopFlags val') cond body
        else
          some (clearLoopFlags val)
  termination_by (fuel, 0)

  /-- Helper: execute a shallow-Verilog switch statement by matching the
      discriminant value against each case tag and running the first matching body
      (or the default).  Fuel accounting mirrors the t27 side. -/
  def evalVSwitchStmtCasesTotal (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation)
      (discV : Value) (default : List VStmt) (cs : List (VExpr × List VStmt)) : Option Valuation :=
    match cs with
    | [] => evalVStmtsTotal fuel env vm val default
    | (tag, body) :: rest => do
        let t <- evalVExprTotal fuel env vm val tag
        let eq <- evalBinop "==" discV t
        if eq.bits.toNat > 0 then evalVStmtsTotal fuel env vm val body
        else evalVSwitchStmtCasesTotal fuel env vm val discV default rest
  termination_by (fuel, sizeOf cs + sizeOf default)
  decreasing_by all_goals simp_wf <;> simp [sizeOf] <;> try { omega }

  /-- Compute the root identifier name, the bit offset inside that root, and the
      width of the slice addressed by a shallow-Verilog lvalue expression.  The
      emitted LHS for packed scalar-struct array fields is a chain of
      `VExpr.slice` (from `.fieldAccess`) and `VExpr.index` (from `.index`). -/
  def assignVTargetOffsetWidth (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation)
      (e : VExpr) : Option (String × Nat × Nat) :=
    match e with
    | .ident name => (val name).map (fun v => (name, 0, v.width))
    | .slice base hi lo => do
        let (name, off, _) <- assignVTargetOffsetWidth fuel env vm val base
        some (name, off + lo, hi - lo + 1)
    | .index base idx elemW => do
        let (name, off, _) <- assignVTargetOffsetWidth fuel env vm val base
        let i <- evalVExprTotal fuel env vm val idx
        let n := i.bits.toNat
        some (name, off + n * elemW, elemW)
    | _ => none
    termination_by (fuel, sizeOf e)

  /-- Total shallow-Verilog statement evaluator. -/
  def evalVStmtTotal (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation) (stmt : VStmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      match stmt with
      | .assign lhs rhs => do
          let rv <- evalVExprTotal fuel env vm val rhs
          match lhs with
          | .ident name =>
              some (fun x => if x == name then some rv else val x)
          | _ =>
              match assignVTargetOffsetWidth fuel env vm val lhs with
              | some (name, off, w) =>
                  if _h : rv.width = w then
                    let old <- val name
                    let newV <- Value.replaceSlice old rv off
                    some (fun x => if x == name then some newV else val x)
                  else
                    none
              | none => none
      | .localparam name _ init => do
          let v <- evalVExprTotal fuel env vm val init
          some (fun x => if x == name then some v else val x)
      | .wire _ _ => some val
      | .reg _ _ => some val
      | .alwaysComb body => evalVStmtsTotal fuel env vm val body
      | .initial body => evalVStmtsTotal fuel env vm val body
      | .ifThenElse cond then_ else_ => do
          let c <- evalVExprTotal fuel env vm val cond
          if c.bits.toNat > 0 then evalVStmtsTotal fuel env vm val then_
          else evalVStmtsTotal fuel env vm val else_
      | .forLoop var range body => do
          let r <- evalVExprTotal fuel env vm val range
          evalVForLoopTotal fuel env vm val var 0 r.bits.toNat body
      | .whileLoop cond body => do
          evalVWhileLoopTotal fuel env vm val cond body
      | .switch disc cases default => do
          let d <- evalVExprTotal fuel env vm val disc
          evalVSwitchStmtCasesTotal fuel env vm val d default cases
      | .taskCall _ _ => some val
      | .break => some (setFlag val breakFlag ⟨1, 1#1⟩)
      | .continue => some (setFlag val continueFlag ⟨1, 1#1⟩)
  termination_by (fuel, sizeOf stmt)

  /-- Total shallow-Verilog statement-list evaluator. -/
  def evalVStmtsTotal (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation) (stmts : List VStmt) : Option Valuation :=
    match fuel with
    | 0 => none
    | fuel+1 =>
      if hasExitFlag val then some val
      else
        match stmts with
        | [] => some val
        | stmt :: rest => do
            let val' <- evalVStmtTotal fuel env vm val stmt
            evalVStmtsTotal fuel env vm val' rest
  termination_by (fuel, sizeOf stmts)
end

/-- Total shallow-Verilog module evaluator. -/
def evalVModuleTotal (fuel : Nat) (env : Env) (vm : VModule) (fnName : String) (args : List Value) : Option Value :=
  match evalVStmtsTotal fuel env vm (fun _ => none) vm.globals with
  | some initVal =>
      match vm.functions.find? (fun f => f.name == fnName) with
      | some fn => evalVFunctionTotal fuel env vm fn args initVal
      | none => none
  | none => none

/-- Verilog ternary-case walker.  It mirrors the emitted `switch` nested-ternary
    and consumes the same fuel levels as `evalSwitchCasesTotal`: the discriminant
    and tag are evaluated at `fuel - 2`, the result and tail at `fuel - 1`. -/
def evalVTernaryCasesTotal (fuel : Nat) (env : Env) (vm : VModule) (val : Valuation)
    (disc : VExpr) (default : VExpr) (cases : List (VExpr × VExpr)) : Option Value :=
  match cases with
  | [] => evalVExprTotal fuel env vm val default
  | (tag, res) :: rest =>
      match fuel with
      | 0 => none
      | 1 => none
      | fuel+2 => do
          let d <- evalVExprTotal fuel env vm val disc
          let t <- evalVExprTotal fuel env vm val tag
          let eq <- evalBinop "==" d t
          if eq.bits.toNat > 0 then evalVExprTotal (fuel+1) env vm val res
          else evalVTernaryCasesTotal (fuel+1) env vm val disc default rest
  termination_by (fuel, sizeOf cases)

end Trinity.IcarusLowerable
