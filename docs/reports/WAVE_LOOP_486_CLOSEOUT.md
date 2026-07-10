# Wave Loop 486 Close-Out Report

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Branch:** `wave-loop-486`
**Issue:** #1456
**Variant:** B (default)

## Goal

Continue the Icarus/Verilog backend hardening started in W485 by closing the next
set of reachable soft-failure classes:

1. Bench-local fixed-size arrays crossing function boundaries.
2. Imported namespace-qualified helpers used only in host-side contexts.
3. Module-scope wildcard `_` bindings with array-literal initializers.

## What landed

### 1. Bench-local scalar arrays as array-parameter arguments

`bootstrap/src/compiler.rs`

- Pre-collected bench-local array names per bench before the array-parameter
  binding pass.
- Extended the call-site tracking tuple to carry the containing bench name for
  top-level bench-block calls.
- In the signature builder, a bench-local array argument now resolves to the
  shared `__local__` packed-vector signature, just like function-local arrays.
- Fixed a latent emission bug where `emitted_bench_names` was reused between the
  counter-declaration loop and the initial-block loop, causing every bench
  initial block to be skipped. Split into `emitted_counter_names` and
  `emitted_bench_names`.
- Added scalar-array packed-vector packing in
  `gen_verilog_pack_array_of_struct_expr` so bench-local `[N]u32` arguments are
  concatenated element-by-element at the call site.
- Added packed-vector slicing inside the function body for scalar array
  parameters: a parameter declared as `input [N*W-1:0] a` is accessed as
  `a[(idx)*W +: W]` instead of being treated as an unpacked memory.

Witness: `specs/scratch/w486_bench_array_param.t27`

### 2. Namespace-qualified helper erasure

`bootstrap/src/compiler.rs`

- Added `host_only_namespace_calls`: qualified names (e.g. `module::helper`) that
  are dead to synthesizable Verilog contexts.
- `collect_qualified_calls_skipping_wildcards` collects namespace calls while
  treating wildcard `let _ = ...;` subtrees as dead code.
- `compute_host_only_namespace_calls` marks a qualified call as host-only when
  it appears only in invariants, host-only functions, or wildcard statements,
  and never in module-level const/var declarations, bare statements, tests,
  benches, or non-host-only functions.
- `call_is_host_only` now returns true for both unqualified host-only functions
  and host-only namespace calls, giving them the same statement-context
  comment no-op and expression-context sized-zero placeholder behavior.

Witness: `specs/scratch/w486_namespace_helper_erasure.t27`

### 3. Module-scope wildcard array literals

`bootstrap/src/compiler.rs`

- In `gen_verilog_const`, module-scope `let _ = [N]T{...};` no longer discards
  the initializer with a comment. It is re-emitted under an anonymous name
  (`_wildcard_lit_<n>`) with the array type reconstructed from the literal's
  bracket size and element type, so the existing scalar/struct ROM lowering path
  produces a valid anonymous ROM.
- Host-only and namespace-qualified call initializers keep the safe comment-only
  path.
- Module-scope struct-literal wildcards (`let _ = Pt{...};`) remain
  parser-blocked: the parser stops consuming subsequent declarations after such
  a binding. This limitation is documented here and left for a future wave.

Witnesses:
- `specs/scratch/w486_wildcard_module_array.t27` — array-literal wildcard emits
  an anonymous ROM and passes yosys + Icarus smoke.
- `specs/scratch/w486_wildcard_module_array_copy.t27` — alias to a module-level
  array currently degrades to a comment; no named `_` identifier is emitted.
- `specs/scratch/w486_wildcard_module_literal.t27` — struct-literal wildcard is
  parser-blocked and the declarations after it are dropped from the AST.

## Files changed

- `bootstrap/src/compiler.rs`
  - bench-local array-parameter support
  - namespace-qualified helper erasure
  - module-scope wildcard array-literal anonymous ROM emission
  - scalar packed-vector parameter slicing in function bodies
- `specs/scratch/w486_bench_array_param.t27`
- `specs/scratch/w486_helper_module.t27`
- `specs/scratch/w486_namespace_helper_erasure.t27`
- `specs/scratch/w486_wildcard_module_array.t27`
- `specs/scratch/w486_wildcard_module_array_copy.t27`
- `specs/scratch/w486_wildcard_module_literal.t27`
- `.trinity/seals/*.json` — global reseal because generated Verilog changed for
  specs with namespace calls, wildcard arrays, and bench-local array parameters.
- `docs/reports/WAVE_LOOP_486_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W487_2026-07-07.md`
- `.trinity/current-issue.md`
- `.trinity/ring-486.md`
- `.trinity/experience.md`
- `docs/NOW.md`

## Verification

- `cargo build --release`: PASS.
- `cargo test -p t27c --bin t27c`: 1525 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: ALL TESTS PASSED
  - 667 / 667 non-smoke PASS.
  - 147 / 147 yosys smoke PASS, 0 failures.
  - 147 / 147 Icarus smoke PASS, 0 documented baseline failures.
  - 667 / 667 seal matches.
  - 0 fixed-point divergences.
  - FPGA board-less smoke gate: OK.
  - FPGA standalone lake-package build: OK.
  - FPGA smoke gate replay: OK.
- **Total `UNSUPPORTED_ICARUS` placeholders across all 667 specs: 0.**

## Known limitations

- Module-scope struct-literal wildcard bindings (`let _ = Pt{...};`) are not
  handled because the parser stops consuming declarations after them. A future
  wave should fix the parser or add a dedicated recovery path.
- Module-scope wildcard bindings that alias an existing module-level array
  (`let _ = src;`) currently degrade to a comment instead of creating an anonymous
  copy memory. This is safe (no `_` identifier is emitted) but could be extended
  later.

## Next ring

- Branch to create: `wave-loop-487`
- See `docs/reports/FPGA_LOOP_COOPERATION_W487_2026-07-07.md` for variants.

*φ² + φ⁻² = 3 | TRINITY*
