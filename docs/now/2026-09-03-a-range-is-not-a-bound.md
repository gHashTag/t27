# NOW -- A range is not a bound (2026-09-03)

## gen-verilog wrote the whole iterable where the comparison belongs (Closes #2997)

- `gen_verilog_for_stmt` emitted `for (i = 0; i < (0 .. 1000); i = i + 1)` -- the range expression in the loop's bound -- which `iverilog` answers with `syntax error`; measured by regenerating the corpus with the stock compiler and with this one and counting files whose `for (` line carries a `..`: **36 -> 0** in the simulation path, **5 -> 0** synthesizable
- #2849 fixed exactly this in the C emitter and recorded the trap in its own comment: the range is an `ExprBinary` whose `extra_op` is `".."`, NOT the `ExprRange` variant, which is declared in `NodeKind` and constructed nowhere. The repair did not travel; Rust tests the same shape, and Zig writes the range verbatim and is correct because Zig has ranges
- 581 specs generate before and after; `iverilog -g2012` accepts **380 before and 380 after**; 5 synthesizable and 129 simulation files change; seals for them re-sealed in this commit
- the first version of this change also renamed a `_` capture to `__t27_i`, copying C. C declares its counter in the `for` header; here the declaration is hoisted by `collect_fn_loop_vars`, so the rename produced `register `__t27_i' unknown` -- **caught by the probe and not by the corpus**, because all 36 carriers already failed to elaborate on the very defect being repaired. The rename is gone; the counter is whatever was declared
- mutation-checked 4 of 4 after the third guard clause got its own test: `for (data) |x|` iterates an identifier with zero children, so a mutant that keeps only `children.len() == 2` left it untouched; `for (lo + hi) |x|` is the two-child non-range that tells them apart
- NOT fixed, and #2997 scopes it out: the same literal reaches SLICE positions -- `expr[(0 .. idx)]`, 15 sites in 7 files, unchanged. A range in an index is a part-select and a different repair
