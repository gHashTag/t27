# NOW -- gen-zig module-scoped type, landed as an owner exception (2026-10-06)

## bootstrap/src/compiler.rs gen-zig fix (Closes #6533, debt #5980)

- A spliced struct kept `gf16::GF16` as `gf16.GF16`, a name the generated Zig never declares. The Zig type mapper now maps a module-scoped type like its bare name (`gf16::GF16` -> `u16`).
- `specs/compiler/zig_scoped_type.t27` is the regression spec.
- `bootstrap/src/compiler.rs` is hand-written Rust, landed under the owner exception from #6579 (label `owner-approved-foreign`); the debt is #5980.
- Reference pass count on the t27c lab, same base: 776 -> 777 (the new spec), 0 regressions. specs/ml/transformer/multi_head_attention.t27 gets past this error and stops at its next one.
