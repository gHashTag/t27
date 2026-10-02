# M5 freeze: module `var` arrays are writable; qualified calls no longer check against a namesake

Two typecheck defects from the corpus ratchet unexpected-failure list, both fixed in one M5 freeze move of `bootstrap/src/compiler.rs`:

- **#5573** — a module-level `var` array parsed to `ConstDecl` with `extra_mutable` set, but the typechecker registered every `ConstDecl` as `is_mutable: false, is_const: true`, so W456's ROM rule rejected every element write. The registration now reads `extra_mutable`; two tests pin the writable-`var` / ROM-`const` split.
- **#5574** — a qualified call `multi_head_attn::forward(...)` was flattened to the bare `forward` by `use_resolve`, which then bound it to the LOCAL `forward` of a different arity and reported a false "expects 2 args, got 3". The resolver now keeps the qualifier on a qualified CALL whose local namesake disagrees with every imported declaration of that name (normalised text); faithful inline copies (dataset.t27) still flatten.

The two suite_expectations entries (issues 5573, 5574) are removed; `mha_block.t27` and `disjoint_set.t27` now pass typecheck.

Ceremony (FROZEN.md §5): frozen-digest recomputed (`b45a356c2eb651059e73d93e913558131b303d003e80f5d6642d3ddab7dd3526`), cargo build and 1750 tests pass locally.
