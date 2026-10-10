# NOW -- sixteen dead `use` lines go, and `PHI` is taken from the spec that declares it (2026-10-07)

## specs/ (12 files), .trinity/seals/ (17 files) (Refs #7191)

- Fourth slice of #7191. 16 `use` lines in 12 specs are deleted. Each named no spec (the #7176 warning), and the body of each reads nothing from what the line names:
  - 13 name a module the body never reads: `math::igla_primitives` (2), `math::statistics` (2), `math::trigonometry` (1), `base::testing`, `base::benchmarking`, `compiler::runtime`, and `std::process` in `check_conflict_markers`. `check_specs_parse` adds 4 more: `std::fs`, `std::os`, `std::process` and `std::temp`.
  - `use io::file;` in `specs/tools/tri_to_t27_converter.t27`: the `file` its body reads is its own local `var file`.
  - `use base::math::phi_distance;` in `specs/memory/formula_embed.t27`: that spec declares `pub fn phi_distance` itself (line 146).
  - `use base::math::normalize_l2;` in `specs/memory/semantic_search.t27`: the body never calls it.
- The Zig backend's own output confirms each one. Before this slice it lowered all 16 as `// use X: no references in this module`, and those 16 comment lines are what disappears from `gen`.
- Two comments that described a deleted line are corrected, in `specs/numeric/gf8.t27` and `specs/queen/lotus.t27`.
- `use base::constants::PHI;` in the same two `memory` specs now reads `use math::constants::PHI;`. `specs/math/constants.t27` declares `PHI` at line 17, and t27c now splices it.
  - The splice also brings `abs`, which both test bodies call. In `gen` the assert's `@abs(...)` becomes a call to that `abs`, the same function for an `f64`.
  - In formula_embed it also brings `pow` and the three names `pow` reads: `floor`, `exp_approx` and `E`. The body reads `pow` only as the builtin `@pow`, a false reference filed as #7292. Only the other spliced functions call these 4, so they are dead code in `gen`, `gen-c` and `gen-rust`.
  - `cc -fsyntax-only` on the `gen-c` output:
    - `formula_embed` goes from 35 errors to 34 (`'PHI' undeclared` is gone);
    - `semantic_search` goes from 23 errors to 21 (`'PHI' undeclared` and `'expected' undeclared` are gone);
    - each file gains one warning: the spliced `double abs(double)` conflicts with C's builtin `int abs(int)` (`-Wbuiltin-declaration-mismatch`). Before, the assert called the `t27_abs` macro.
- Measured on the Railway lab with the #7242 build, on all 1363 specs, under `gen`, `gen-c`, `gen-rust` and `gen-verilog`:
  - The exit code changes on none of the 4 x 1363 runs.
  - Output changes only in the edited files: gen 12, gen-c 3, gen-rust 2, gen-verilog 1.
  - In `gen-c` and `gen-verilog` the change to `arch` is a temporary name that carries a line number (`__t_c774` becomes `__t_c773`).
  - #7176 warnings drop from 42 to 24 under each backend.
  - Parse and typecheck exit 0 on all 12, before and after.
  - `iverilog` diagnostic counts are unchanged on each of the 12.
- `t27c test-report`: all 12 are blocked before and after, on the same zig error, 1 to 4 lines earlier. None was passing.
- Seals: 10 of the 12 have seals, and the 2 `port/tools` specs have none. The 10 were resealed on the lab, and 17 seal files change.
  - `spec_hash` and `gen_hash_zig` change in all 17.
  - `gen_hash_c` and `gen_hash_verilog` change only for `arch`. No `gen_hash_rust` changes: a seal hashes a spec's own output, before any `use` is spliced.
  - Master's t27c and the #7242 build both print "all hashes MATCH" on all 10.
  - 4 seal files (lotus and tri_to_t27_converter) gain a `tests.blocked` record, written by the #7242 build.
- 24 warnings remain. None is a dead line, and none is fixed by changing its path alone:
  - `std::*` modules (9 lines, 8 of them in `specs/port/tools`);
  - functions that exist in no spec, such as `l2_norm`, `cosine_sim` and `gf16_from_f64`;
  - `vsa::core` in `specs/vsa/sdk.t27` and `specs/server/vm.t27`: its functions are in `specs/vsa/vsa_core.t27`, but the bodies call them as `core::...`;
  - `hybrid_arithmetic` in the same two specs: the bodies read `hybrid_arithmetic::packed_mode`, which `specs/ternary/hybrid_arithmetic.t27` declares as a variant of `StorageMode`, and `vm.t27` calls `hybrid_arithmetic::create_unpacked`, which no spec declares.
- The full list, with a reason for each line, is on #7191.
