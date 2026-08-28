# NOW -- The three specs the discard was flattering, named (2026-08-29)

## The three specs the discard was flattering, named (Refs #2754)

- corpus --per-spec writes the binary outcomes behind every number, sorted, so two binaries can be diffed: three specs moved and all three are now named
- base/ternary_encoding and vsa/similarity_search: a blank line ended the block, the head lowered inside the invariant comptime and the tail hoisted outside it, referencing names that no longer existed
- isa/ternary_gates: gen-c read the array literal's DIMENSION as its element list -- int32_t a[3] = { .v = { _ } } -- and 0 of the 156 C-accepted specs contained that .v wrapper
- per-spec table is now IDENTICAL to a520590e across all 650 specs, with 1292 fewer discarded tokens: same columns, more code behind them
