# NOW -- t27b: top-level statements are dropped like the reference (2026-10-05)

## Top-level StmtExpr (Closes #6363)

- t27c's Zig backend (`gen_decl`) emits nothing for a statement at top level, so the reference compiles a file that has one. t27b refused it; it now drops it the same way.
- The common source is a dotted `module a.b;` or `use a.b;`, which the t27c parser reads as `a` plus a stray `.b` (#6102); a test body the parser closed early leaves its tail at top level too.
- A fn named only by a top-level statement is no longer a root of the analyzed-fn set, since the reference never emits that statement.
- `t27b corpus specs --reference t27c` from master bd59d8c73: counted 648/720 -> 653/720, checked pass 370 -> 373, JIT/interpreter mismatch 0 -> 0, files t27b passes but the reference does not 0 -> 0.
- New test `top_level_statements_are_dropped_like_the_reference` in `cli/t27b/tests/source.rs`.
