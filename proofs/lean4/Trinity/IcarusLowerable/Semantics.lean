/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 495
  Denotational semantics for the simplified t27 AST and the shallow Verilog
  AST, restricted to the Icarus-lowerable scalar subset.

  The semantics is *combinational and finite*: values are `BitVec n` for numeric
  types, structs are concatenations of leaf fields, and arrays are concatenations
  of elements. Function calls are evaluated by inlining the called function's
  body, matching the current t27 → Verilog lowering for pure combinational
  functions.

  Changes in W495:
    - Verilog function definitions (`VFunction`) are stored in `VModule`, and
      `evalVExpr` inlines them for `.call` nodes.
    - t27 field access uses `Expr.typeOf` so it works for general function calls
      returning structs, not only constructor calls.
    - Array indexing derives the element width from the base expression's type.
    - `evalVModule` evaluates module-level items first, then runs a named function.
    - `evalModuleFunction` evaluates module globals before running a t27 function.

  The intended theorem shape:
    Module.isLowerable env m →
      evalModuleFunction env m "main" [] = evalVModule env (emitModule env m) "main"

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

import Trinity.IcarusLowerable.Ast
import Trinity.IcarusLowerable.Predicate
import Trinity.IcarusLowerable.Verilog
import Trinity.IcarusLowerable.Emitter

namespace Trinity.IcarusLowerable

/-- Runtime value: a bit-vector together with its width. -/
structure Value where
  width : Nat
  bits : BitVec width
  deriving BEq, Repr, DecidableEq

/-- Look up a variable in a valuation. -/
abbrev Valuation := String → Option Value

/-- Two valuations are equivalent when they agree on every identifier. -/
def Valuation.equiv (v1 v2 : Valuation) : Prop :=
  ∀ x, v1 x = v2 x

/-- Helper: concatenate two values. -/
def Value.concat (a b : Value) : Value :=
  ⟨a.width + b.width, BitVec.append a.bits b.bits⟩

/-- Helper: concatenate a list of values; empty list yields a 1-bit zero. -/
def Value.concatList (vs : List Value) : Value :=
  match vs with
  | [] => ⟨1, BitVec.ofNat 1 0⟩
  | v :: vs => vs.foldl Value.concat v

/-- Helper: replace the bit slice `[off .. off+new.width-1]` of `old` with `new`,
    leaving all other bits unchanged.  Returns `none` if the slice does not fit. -/
def Value.replaceSlice (old new : Value) (off : Nat) : Option Value :=
  if _h : off + new.width ≤ old.width then
    let highW := old.width - off - new.width
    let highBits := BitVec.extractLsb' (off + new.width) highW old.bits
    let lowBits := BitVec.extractLsb' 0 off old.bits
    let combined := highBits ++ new.bits ++ lowBits
    have h : highW + new.width + off = old.width := by
      simp [highW]
      omega
    some ⟨old.width, BitVec.cast h combined⟩
  else
    none

/-- Fixed fuel budget used by the partial evaluator model.  Lowerable modules
    are finite, so a constant bound is enough for the witness set. -/
def modelFuel : Nat := 1000

/-- Convert a t27 type to its runtime width using the environment.
    Uses the total `widthOfType` from `Emitter.lean` with the model fuel. -/
def widthOfType' (env : Env) (ty : Ty) : Nat :=
  widthOfType modelFuel env ty

/-- Evaluate a lowerable binary operator on two values of equal width. -/
def evalBinop (op : String) (lhs rhs : Value) : Option Value :=
  let w := lhs.width
  if h : rhs.width = w then
    let a := lhs.bits
    let b := rhs.bits
    have : BitVec rhs.width = BitVec w := by rw [h]
    let b' := cast this b
    match op with
    | "+" => some ⟨w, a + b'⟩
    | "-" => some ⟨w, a - b'⟩
    | "*" => some ⟨w, a * b'⟩
    | "/" => some ⟨w, a / b'⟩
    | "%" => some ⟨w, a % b'⟩
    | "&" => some ⟨w, a &&& b'⟩
    | "|" => some ⟨w, a ||| b'⟩
    | "^" => some ⟨w, a ^^^ b'⟩
    | "<<" => some ⟨w, a.shiftLeft b'.toNat⟩
    | ">>" => some ⟨w, a.ushiftRight b'.toNat⟩
    | "==" => some ⟨1, if a == b' then 1#1 else 0#1⟩
    | "!=" => some ⟨1, if a == b' then 0#1 else 1#1⟩
    | "<" => some ⟨1, if a.ult b' then 1#1 else 0#1⟩
    | "<=" => some ⟨1, if a.ule b' then 1#1 else 0#1⟩
    | ">" => some ⟨1, if ¬ a.ule b' then 1#1 else 0#1⟩
    | ">=" => some ⟨1, if ¬ a.ult b' then 1#1 else 0#1⟩
    | "&&" => some ⟨1, if a != 0#w && b' != 0#w then 1#1 else 0#1⟩
    | "||" => some ⟨1, if a != 0#w || b' != 0#w then 1#1 else 0#1⟩
    | _ => none
  else
    none

/-- Evaluate a unary operator. -/
def evalUnop (op : String) (e : Value) : Option Value :=
  let w := e.width
  let a := e.bits
  match op with
  | "!" => some ⟨1, if a == 0#w then 1#1 else 0#1⟩
  | "-" => some ⟨w, 0#w - a⟩
  | "~" => some ⟨w, ~~~a⟩
  | _ => none

/-- Compute the bit offset of a struct field. -/
def structFieldOffset (env : Env) (sname : String) (field : String) : Nat :=
  let fields := env.structFields sname
  fields.foldl (fun acc p => if p.1 < field then acc + widthOfType' env p.2 else acc) 0

/-- Compute the width of a struct field. -/
def structFieldWidth (env : Env) (sname : String) (field : String) : Nat :=
  match (env.structFields sname).find? (fun p => p.1 == field) with
  | some ty => widthOfType' env ty.2
  | none => 0

mutual
  /-- Evaluate a t27 expression under a valuation, environment, and module. -/
  partial def evalExpr (env : Env) (m : Module) (val : Valuation) : Expr → Option Value
    | .boolLit true => some ⟨1, 1#1⟩
    | .boolLit false => some ⟨1, 0#1⟩
    | .intLit n => some ⟨32, BitVec.ofInt 32 n⟩
    | .identifier name => val name
    | .binop op lhs rhs => do
        let l <- evalExpr env m val lhs
        let r <- evalExpr env m val rhs
        evalBinop op l r
    | .unop op e => do
        let v <- evalExpr env m val e
        evalUnop op v
    | .fieldAccess base field =>
        match Expr.typeOf env m base with
        | some (.struct sname) => do
            let v <- evalExpr env m val base
            let off := structFieldOffset env sname field
            let w := structFieldWidth env sname field
            if _h : w > 0 && off + w - 1 < v.width then
              some ⟨w, BitVec.extractLsb' off w v.bits⟩
            else
              none
        | _ => do
            let v <- evalExpr env m val base
            if _h : v.width > 0 then
              some ⟨1, BitVec.extractLsb' 0 1 v.bits⟩
            else
              none
    | .index base idx => do
        let b <- evalExpr env m val base
        let i <- evalExpr env m val idx
        let n := i.bits.toNat
        let elemW := match Expr.typeOf env m base with
          | some (.array _ elem) => widthOfType' env elem
          | _ => 8
        if _h : elemW > 0 && n * elemW + elemW - 1 < b.width then
          some ⟨elemW, BitVec.extractLsb' (n * elemW) elemW b.bits⟩
        else
          none
    | .call name args => evalCall env m val name args
    | .structLit _ fields => do
        let vs <- fields.mapM (fun p => evalExpr env m val p.2)
        some (Value.concatList vs)
    | .arrayLit _ elems => do
        let vs <- elems.mapM (evalExpr env m val)
        some (Value.concatList vs)
    | _ => none

  /-- Evaluate a function call by inlining the function body. -/
  partial def evalCall (env : Env) (m : Module) (val : Valuation) (name : String) (args : List Expr) : Option Value :=
    match m.findFunction name with
    | some fn => do
        let argVals <- args.mapM (evalExpr env m val)
        evalFunction env m fn argVals val
    | none => none

  /-- Evaluate a function body by binding parameters (on top of `base`) and
      returning `__return`. -/
  partial def evalFunction (env : Env) (m : Module) (fn : Function) (argVals : List Value) (base : Valuation) : Option Value :=
    do
      let paramBinds := fn.params.zip argVals
      let init : Valuation := fun name =>
        paramBinds.find? (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name)
      let final <- evalStmts env m init fn.body
      final "__return"

  /-- Helper: execute a t27 switch statement in the partial evaluator. -/
  partial def evalSwitchStmtCases (env : Env) (m : Module) (val : Valuation)
      (discV : Value) (default : List Stmt) (cs : List (Expr × List Stmt)) : Option Valuation :=
    match cs with
    | [] => evalStmts env m val default
    | (tag, body) :: rest => do
        let t <- evalExpr env m val tag
        let eq <- evalBinop "==" discV t
        if eq.bits.toNat > 0 then evalStmts env m val body
        else evalSwitchStmtCases env m val discV default rest

  /-- Evaluate a list of t27 statements. -/
  partial def evalStmts (env : Env) (m : Module) (val : Valuation) (stmts : List Stmt) : Option Valuation :=
    stmts.foldlM (fun acc stmt =>
      match stmt with
      | .assign (.identifier name) rhs => do
          let v <- evalExpr env m acc rhs
          some (fun x => if x == name then some v else acc x)
      | .varDecl name ty init => do
          let v <- match init with
                  | some e => evalExpr env m acc e
                  | none => some ⟨widthOfType' env ty, 0#(widthOfType' env ty)⟩
          some (fun x => if x == name then some v else acc x)
      | .constDecl name ty init => do
          let v <- match init with
                  | some e => evalExpr env m acc e
                  | none => some ⟨widthOfType' env ty, 0#(widthOfType' env ty)⟩
          some (fun x => if x == name then some v else acc x)
      | .return_ (some e) => do
          let v <- evalExpr env m acc e
          some (fun x => if x == "__return" then some v else acc x)
      | .return_ none => some acc
      | .switch disc cases default => do
          let d <- evalExpr env m acc disc
          evalSwitchStmtCases env m acc d default cases
      | _ => some acc) val
end

/-- Evaluate module globals, then the named function, under a module. -/
def evalModuleFunction (env : Env) (m : Module) (fnName : String) (args : List Value) : Option Value :=
  match evalStmts env m (fun _ => none) m.globals with
  | some initVal =>
      match m.findFunction fnName with
      | some fn => evalFunction env m fn args initVal
      | none => none
  | none => none

/-- Evaluate the test named `testName` in a module: bind no arguments, run the
    test body, and return the `__return` value. -/
def evalTest (env : Env) (m : Module) (testName : String) : Option Value :=
  match m.tests.find? (fun t => t.name == testName) with
  | some t => evalFunction env m t [] (fun _ => none)
  | none => none

mutual
  /-- Evaluate a shallow Verilog expression. -/
  partial def evalVExpr (env : Env) (vm : VModule) (val : Valuation) : VExpr → Option Value
    | .lit w s =>
        match s with
        | "1" => some ⟨w, BitVec.ofNat w 1⟩
        | "0" => some ⟨w, BitVec.ofNat w 0⟩
        | _ =>
            match String.toInt? s with
            | some n => some ⟨w, BitVec.ofInt w n⟩
            | none => none
    | .ident name => val name
    | .binop op lhs rhs => do
        let l <- evalVExpr env vm val lhs
        let r <- evalVExpr env vm val rhs
        evalBinop op l r
    | .unop op e => do
        let v <- evalVExpr env vm val e
        evalUnop op v
    | .index base idx elemW => do
        let b <- evalVExpr env vm val base
        let i <- evalVExpr env vm val idx
        let n := i.bits.toNat
        if _h : elemW > 0 && n * elemW + elemW - 1 < b.width then
          some ⟨elemW, BitVec.extractLsb' (n * elemW) elemW b.bits⟩
        else
          none
    | .slice base hi lo => do
        let b <- evalVExpr env vm val base
        if _h : lo ≤ hi && hi < b.width then
          some ⟨hi - lo + 1, BitVec.extractLsb' lo (hi - lo + 1) b.bits⟩
        else
          none
    | .concat parts => do
        let vs <- parts.mapM (evalVExpr env vm val)
        some (Value.concatList vs)
    | .call name args => do
        let argVals <- args.mapM (evalVExpr env vm val)
        match vm.functions.find? (fun f => f.name == name) with
        | some fn => evalVFunction env vm fn argVals val
        | none => none
    | .ternary cond then_ else_ => do
        let c <- evalVExpr env vm val cond
        if c.bits.toNat > 0 then evalVExpr env vm val then_ else evalVExpr env vm val else_
    | .unsupported _ => none
    | .todo _ => none

  /-- Evaluate a shallow Verilog function body by binding parameters (on top of
      `base`) and returning `__return`. -/
  partial def evalVFunction (env : Env) (vm : VModule) (fn : VFunction) (argVals : List Value) (base : Valuation) : Option Value :=
    do
      let paramBinds := fn.params.zip argVals
      let init : Valuation := fun name =>
        paramBinds.find? (fun p => p.1.1 == name) |>.map (·.2) |>.orElse (fun _ => base name)
      let final <- evalVStmts env vm init fn.body
      final "__return"


  /-- Helper: execute a shallow-Verilog for-loop body `n` times, binding `var` to `i`, `i+1`, ... -/
  partial def evalVForLoop (env : Env) (vm : VModule) (val : Valuation) (var : String) (i : Nat) (n : Nat) (body : List VStmt) : Option Valuation :=
    match n with
    | 0 => some val
    | n+1 => do
        let loopVal := fun x => if x == var then some ⟨32, BitVec.ofNat 32 i⟩ else val x
        let val' <- evalVStmts env vm loopVal body
        evalVForLoop env vm val' var (i + 1) n body

  /-- Helper: execute a shallow-Verilog switch statement in the partial evaluator. -/
  partial def evalVSwitchStmtCases (env : Env) (vm : VModule) (val : Valuation)
      (discV : Value) (default : List VStmt) (cs : List (VExpr × List VStmt)) : Option Valuation :=
    match cs with
    | [] => evalVStmts env vm val default
    | (tag, body) :: rest => do
        let t <- evalVExpr env vm val tag
        let eq <- evalBinop "==" discV t
        if eq.bits.toNat > 0 then evalVStmts env vm val body
        else evalVSwitchStmtCases env vm val discV default rest

  /-- Evaluate a shallow Verilog statement. -/
  partial def evalVStmt (env : Env) (vm : VModule) (val : Valuation) (stmt : VStmt) : Option Valuation :=
    match stmt with
    | .assign lhs rhs => do
        let name := match lhs with | .ident n => n | _ => ""
        let v <- evalVExpr env vm val rhs
        some (fun x => if x == name then some v else val x)
    | .localparam name _ init => do
        let v <- evalVExpr env vm val init
        some (fun x => if x == name then some v else val x)
    | .wire _ _ => some val
    | .reg _ _ => some val
    | .alwaysComb body => evalVStmts env vm val body
    | .initial body => evalVStmts env vm val body
    | .ifThenElse cond then_ else_ => do
        let c <- evalVExpr env vm val cond
        if c.bits.toNat > 0 then evalVStmts env vm val then_
        else evalVStmts env vm val else_
    | .forLoop var range body => do
        let r <- evalVExpr env vm val range
        evalVForLoop env vm val var 0 r.bits.toNat body
    | .switch disc cases default => do
        let d <- evalVExpr env vm val disc
        evalVSwitchStmtCases env vm val d default cases
    | .break =>
        -- The partial/combinational model ignores control-flow flags; the
        -- fuel-based total semantics in `SemanticsTotal.lean` is the proof-relevant
        -- model for bounded loops with `break`/`continue`.
        some val
    | .continue =>
        some val
    | .whileLoop _ _ =>
        -- The partial/combinational model does not execute loops; the fuel-based
        -- total semantics in `SemanticsTotal.lean` is the proof-relevant model.
        some val
    | .taskCall _ _ =>
        -- Task calls in the model only occur in test blocks; they are evaluated
        -- by the surrounding statement list, so we leave the valuation unchanged.
        some val

  /-- Evaluate a list of shallow Verilog statements. -/
  partial def evalVStmts (env : Env) (vm : VModule) (val : Valuation) (stmts : List VStmt) : Option Valuation :=
    stmts.foldlM (fun acc s => evalVStmt env vm acc s) val
end

/-- Evaluate a shallow Verilog module by running its globals and then the named
    function. Test/bench items are not evaluated before a function call in this
    model, matching `evalModuleFunction` on the t27 side. -/
def evalVModule (env : Env) (vm : VModule) (fnName : String) : Option Value :=
  match evalVStmts env vm (fun _ => none) vm.globals with
  | some initVal =>
      match vm.functions.find? (fun f => f.name == fnName) with
      | some fn => evalVFunction env vm fn [] initVal
      | none => none
  | none => none

end Trinity.IcarusLowerable
