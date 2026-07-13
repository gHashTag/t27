/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 492
  Shallow Verilog AST with explicit placeholder constructors.

  This AST is just deep enough to express the constructs that the current
  t27 → Icarus backend emits, and to detect the two placeholder fallbacks
  that violate the Icarus-lowerability contract:
    - `UNSUPPORTED_ICARUS` comments (expression-level zero placeholders)
    - `// TODO: implement` stubs (aggregate / unlowered initializer stubs)

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

namespace Trinity.IcarusLowerable

/-- Bit-vector / identifier / operator expression in emitted Verilog. -/
inductive VExpr
  | lit (width : Nat) (value : String)
  | ident (name : String)
  | binop (op : String) (lhs rhs : VExpr)
  | unop (op : String) (e : VExpr)
  | index (base : VExpr) (idx : VExpr) (elemWidth : Nat)
  | slice (base : VExpr) (hi lo : Nat)
  | concat (parts : List VExpr)
  | call (name : String) (args : List VExpr)
  | ternary (cond then_ else_ : VExpr)
  | unsupported (reason : String)
  | todo (stub : String)
  deriving Repr, BEq, Nonempty

/-- Module-level or procedural statement in emitted Verilog. -/
inductive VStmt
  | assign (lhs : VExpr) (rhs : VExpr)
  | localparam (name : String) (width : Nat) (init : VExpr)
  | wire (name : String) (width : Nat)
  | reg (name : String) (width : Nat)
  | alwaysComb (body : List VStmt)
  | initial (body : List VStmt)
  | ifThenElse (cond : VExpr) (then_ else_ : List VStmt)
  | switch (disc : VExpr) (cases : List (VExpr × List VStmt)) (default : List VStmt)
  | forLoop (var : String) (range : VExpr) (body : List VStmt)
  | whileLoop (cond : VExpr) (body : List VStmt)
  | break
  | continue
  | taskCall (name : String) (args : List VExpr)
  deriving Repr, BEq, Nonempty

/-- A Verilog function definition. -/
structure VFunction where
  name : String
  params : List (String × Nat)
  retWidth : Nat
  body : List VStmt
  deriving Repr, BEq, Nonempty

/-- A Verilog module. Ports are (name, width, direction).
    `globals` are module-level declarations that are evaluated before any named
    function is called; `items` are test/bench/initial blocks that do not affect
    function evaluation in this shallow model. -/
structure VModule where
  name : String
  ports : List (String × Nat × String)
  globals : List VStmt
  items : List VStmt
  functions : List VFunction
  deriving Repr, BEq, Nonempty

/-- True when the expression contains an unsupported or todo placeholder. -/
def VExpr.hasPlaceholder : VExpr → Bool
  | .unsupported _ => true
  | .todo _ => true
  | .binop _ lhs rhs => lhs.hasPlaceholder || rhs.hasPlaceholder
  | .unop _ e => e.hasPlaceholder
  | .index base idx _ => base.hasPlaceholder || idx.hasPlaceholder
  | .slice base _ _ => base.hasPlaceholder
  | .concat parts => exprListHasPlaceholder parts
  | .call _ args => exprListHasPlaceholder args
  | .ternary cond then_ else_ =>
      cond.hasPlaceholder || then_.hasPlaceholder || else_.hasPlaceholder
  | _ => false
where
  exprListHasPlaceholder : List VExpr → Bool
    | [] => false
    | p :: ps => p.hasPlaceholder || exprListHasPlaceholder ps

/-- True when the statement contains an unsupported or todo placeholder. -/
def VStmt.hasPlaceholder : VStmt → Bool
  | .assign lhs rhs => lhs.hasPlaceholder || rhs.hasPlaceholder
  | .localparam _ _ init => init.hasPlaceholder
  | .alwaysComb body => stmtListHasPlaceholder body
  | .initial body => stmtListHasPlaceholder body
  | .ifThenElse cond then_ else_ =>
      cond.hasPlaceholder || stmtListHasPlaceholder then_ || stmtListHasPlaceholder else_
  | .switch disc cases default =>
      disc.hasPlaceholder ||
      switchCaseListHasPlaceholder cases ||
      stmtListHasPlaceholder default
  | .forLoop _ range body => range.hasPlaceholder || stmtListHasPlaceholder body
  | .whileLoop cond body => cond.hasPlaceholder || stmtListHasPlaceholder body
  | .taskCall _ args => VExpr.hasPlaceholder.exprListHasPlaceholder args
  | _ => false
where
  stmtListHasPlaceholder : List VStmt → Bool
    | [] => false
    | s :: ss => s.hasPlaceholder || stmtListHasPlaceholder ss
  switchCaseListHasPlaceholder : List (VExpr × List VStmt) → Bool
    | [] => false
    | p :: ps =>
        p.1.hasPlaceholder || stmtListHasPlaceholder p.2 ||
        switchCaseListHasPlaceholder ps

/-- True when the function body contains a placeholder. -/
def VFunction.hasPlaceholder (f : VFunction) : Bool :=
  VStmt.hasPlaceholder.stmtListHasPlaceholder f.body

/-- True when the module contains an unsupported or todo placeholder. -/
def VModule.hasPlaceholder (v : VModule) : Bool :=
  VStmt.hasPlaceholder.stmtListHasPlaceholder v.globals
  || VStmt.hasPlaceholder.stmtListHasPlaceholder v.items
  || v.functions.any (fun f => f.hasPlaceholder)

end Trinity.IcarusLowerable
