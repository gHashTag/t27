/- Copyright (c) 2026 Trinity S³AI — t27 Wave Loop 491
  Simplified t27 AST for Icarus-lowerability classification.

  This is intentionally a shallow model: it keeps only the constructs needed to
  decide whether a t27 spec can be emitted for Icarus Verilog simulation. It is
  NOT a full t27 semantics.

  Anchor: φ² + φ⁻² = 3 | TRINITY
-/

namespace Trinity.IcarusLowerable

/-- Scalar or composite t27 type relevant to lowerability. -/
inductive Ty
  | bool
  | u8 | u16 | u32 | u64
  | i8 | i16 | i32 | i64
  | f32
  | string
  | enum (name : String)
  | array (size : Nat) (elem : Ty)
  | struct (name : String)
  deriving BEq, Repr

/-- Simplified expression. -/
inductive Expr
  | boolLit (v : Bool)
  | intLit (n : Int)
  | f32Lit (s : String)   -- kept as string; not lowerable anyway
  | stringLit (s : String)
  | identifier (name : String)
  | binop (op : String) (lhs rhs : Expr)
  | unop  (op : String) (e : Expr)
  | fieldAccess (base : Expr) (field : String)
  | index (base : Expr) (idx : Expr)
  | call (name : String) (args : List Expr)
  | structLit (name : String) (fields : List (String × Expr))
  | arrayLit (ty : Ty) (elems : List Expr)
  | enumVal (enum : String) (variant : String)
  | len (base : Expr)
  | contains (base : Expr) (item : Expr)
  | switch (disc : Expr) (cases : List (Expr × Expr)) (default : Expr)
  | unsupportedIcarus (reason : String)
  deriving BEq, Repr

/-- Simplified statement. -/
inductive Stmt
  | assign (lhs : Expr) (rhs : Expr)
  | varDecl (name : String) (ty : Ty) (init : Option Expr)
  | constDecl (name : String) (ty : Ty) (init : Option Expr)
  | ifThenElse (cond : Expr) (then_ else_ : List Stmt)
  | forLoop (var : String) (range : Expr) (body : List Stmt)
  | whileLoop (cond : Expr) (body : List Stmt)
  | break
  | continue
  | switch (disc : Expr) (cases : List (Expr × List Stmt)) (default : List Stmt)
  | return_ (e : Option Expr)
  | bareCall (e : Expr)
  deriving BEq, Repr

/-- Function or test/bench block. -/
structure Function where
  name : String
  params : List (String × Ty)
  ret : Option Ty
  body : List Stmt
  deriving BEq, Repr

/-- Import declaration. -/
structure Import where
  path : String
  items : List String
  deriving BEq, Repr

/-- Simplified module. -/
structure Module where
  name : String
  imports : List Import
  globals : List Stmt
  functions : List Function
  tests : List Function
  benches : List Function
  deriving BEq, Repr

/-- Find a declared function by name in a module.  Tests and benches are not
    function-call targets and are looked up separately. -/
def Module.findFunction (m : Module) (name : String) : Option Function :=
  m.functions.find? (fun f => f.name == name)

end Trinity.IcarusLowerable
