# NOW -- seven more `use` lines name a spec's path, and one that names nothing goes (2026-10-07)

## specs/ (8 files), .trinity/seals/ (17 files) (Refs #7191)

- Third slice of #7191. These lines named something that is not a path:
  - 6 named a module name (`tritype-base`, `tritype`) or a bare file name (`core`). They now name the path: `base::types::Trit`, `base::types::trit_negate`, `base::types::trit_multiply` and `test_framework::core::{...}`.
  - 2 brace lists took `GF16` and `GF32` from `numeric::golden_float`, which is no spec. A brace list takes its items from one module, as Rust's `use a::{b, c}` takes them from `a`, and the two types live in two specs. Each list is now two lines: `use numeric::gf16::GF16;` and `use numeric::gf32::GF32;`.
  - `use tritype::base;` in `specs/portable/relay_observer.t27` is deleted: its body reads no `Trit` and no `base::` name.
- `specs/neural/forward_pass.t27` had the same `use tritype::Trit;` line. It now reads `use base::types::Trit;`. This edit was held back until #7242 merged, because #7242 rewrote that spec's calls and resealed it.
  - The #7176 warning on that line goes away under all 4 backends.
  - The output of `gen`, `gen-c`, `gen-rust` and `gen-verilog` does not change.
  - `test-report` blocks before and after on the same zig error, `expected ']', found ';'` at the same line.
  - Its two seal files change only in `spec_hash` and in the zig temp-dir name inside `tests.blocked`.
- Measured on the Railway lab with the #7176 build (PR #7242), on all 1356 specs, under `gen`, `gen-c`, `gen-rust` and `gen-verilog`:
  - The exit code changes on none of the 4 x 1356 runs.
  - Output changes only in the edited files: gen 7, gen-c 6, gen-rust 6, gen-verilog 1.
  - #7176 warnings drop from 52 to 43, and to 42 with `forward_pass` below.
  - Parse and typecheck exit codes are unchanged on every edited file.
  - `iverilog` diagnostic counts are unchanged on each edited file.
- `t27c test-report`: all 7 are blocked before and after; none was passing.
  - `relay_observer`, `property_test_template` and `verilog_bench_harness` keep their zig error.
  - `vsa_core` moves from undeclared `Trit` to undeclared `TRIT_ZERO`: `Trit` now arrives from `specs/base/types.t27`, and `TRIT_ZERO` is declared in no spec it imports.
  - `bigint`, `hybrid_bigint` and `runner` move from `unable to load 'Trit.zig'` (or `'EngineeringStatus.zig'`): `FileNotFound` to `duplicate struct member name`. The splice now declares the type, and the Zig backend still writes `const Trit = @import("Trit.zig");` for a `use` whose item the body reads as `Trit.neg`.
- That collision is an old defect a correct `use` exposes, filed as #7281. Across the corpus it hits 0 specs before this slice and these 3 specs (5 names) after.
- Seals: the 7 specs resealed on the lab; 15 seal files change (17 with `forward_pass`).
  - `spec_hash` changes on all 7, and `gen_hash_zig` only on `relay_observer`, the one with a deleted line.
  - Master's t27c and the #7176 build both print "all hashes MATCH" on all 7, and on `forward_pass` after its reseal.
  - The `tests.blocked` text is written by the #7176 build, so this slice lands after #7242.
