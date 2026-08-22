# NOW -- a field of an array element of a struct resolves as ONE part-select (2026-08-22)

## the chain must be resolved whole, and the first version proved why (Refs #2325)

- `mem.ports[i].kind` resolved its base to nothing and flattened to `_kind` -- a
  name with an EMPTY base that can never bind (memory.v, hir.v `_name`, and the
  same shape across the fpga set). The path now lowers to a single cumulative
  part-select: outer[arr_off + i*elem_w + field_off +: field_w].
- The first version handled exactly ONE trailing field and emitted
  `cat[(0 + i*233 + 40) +: 160]_luts` for a two-field tail -- a part-select with
  an identifier glued to it, which is the #2240 defect in a new place. yosys
  caught it inside one smoke run (32 -> 31) and it never left the worktree.
  The landed version walks the whole trailing chain.
- Measured: 228 -> 213 elaboration errors over the 32-module set (on top of
  573 -> 228 from the string-field fix in #2424), yosys smoke 32/32, both
  executed vector modules pass, cargo test unchanged at the 13 failures that
  are red on clean master too (#2292). M5 performed.
