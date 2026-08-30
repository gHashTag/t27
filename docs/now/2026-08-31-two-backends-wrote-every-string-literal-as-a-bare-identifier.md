# NOW -- Two backends wrote every string literal as a bare identifier (2026-08-31)

## Two backends wrote every string literal as a bare identifier (Closes #2970)

- the lexer tags the node extra_kind=string; Zig, Verilog and the typechecker read it, C and Rust never did
- #define NAME trinity is the worst shape because it is VALID C -- it compiles and fails at the use site, unlike the two honest compile errors
- two call sites in C: c_literal takes a &str and cannot see a tag that lives on the node, so fixing the expression arm alone leaves the #define wrong
- cc 268 -> 290 (+22, 0 regressions), rustc 223 -> 224 (+1, 0); the C number was derived independently twice and both say 22
