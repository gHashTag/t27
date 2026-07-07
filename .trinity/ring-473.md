## 2026-07-08 — Wave Loop 473 (gen-verilog aggregate hardening: writable nested struct-array field assignment + higher-dimensional arrays of structs)

**Issue:** #1447  
**Branch:** `wave-loop-473`

### Insight
Module-level arrays of structs with array-typed fields (`[2][3]Shape { pts : [M]Pt }`) are lowered into per-leaf per-element memories with exactly one outer dimension. All outer array coordinates must therefore be linearized into that first memory index before inner field indices and the final scalar bit-slice are applied. Trying to emit nested memory indices for multi-dimensional outer arrays produces illegal Verilog because the memory is only one-dimensional.

The write path must use the same collected (root, indices, fields) tuple as the read path. Using the read path as an assignment target happened to work for one-dimensional outer arrays, but it fails for higher dimensions and is fragile even for the one-dimensional case.

### Pattern
- Store outer array dimensions for every module-level array of structs in a dedicated registry (`module_struct_array_dims`).
- Split collected index nodes into outer indices, inner indices, and the leaf bit slice; linearize only the outer indices into the first memory dimension.
- Reuse the same collector (`collect_field_index_path`) and the same linearization in both read and write paths.
- Add one scratch spec per new aggregate shape and run the yosys smoke gate on it before claiming the feature works.

### Anti-pattern
- Do not rely on a read expression as an assignment target for aggregate paths; emit the full indexed target explicitly.
- Do not assume a memory declared for `[N]Struct` has the same number of dimensions as the source array type; the backend collapses outer arrays into one dimension per leaf field.
- Do not reseal only the specs that visibly changed; any Verilog lowering change can shift `gen_hash_verilog` for unrelated specs.

### Verification
- `cargo test -p t27c`: 1871 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: 633/633 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c, **113/113 yosys smoke**, FPGA smoke gate OK, standalone lake build OK, 0 seal mismatches.
- `./scripts/tri test --fast`: 633/633 non-smoke, **113/113 yosys smoke**, 0 seal mismatches.
