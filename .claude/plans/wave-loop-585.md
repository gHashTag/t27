# Wave Loop 585 — Decomposed Plan

**Issue:** #1556  
**Branch:** `wave-loop-585`  
**Date:** 2026-07-07  
**Chosen variant:** **C** (recommended) — module-scope 7-D array-of-struct variable initialized from a function call, with multi-site bench CSE.

## 1. Weak spots addressed

1. **Scope boundary for call-return CSE.**  
   Previous loops (W557 → W584) exercised function-local `expected` bindings and call-return values inside functions/benches. Module-scope variables initialized from a function call introduce a new boundary: the CSE key must remain valid when the same call result is reused across module initialization, test blocks, and bench blocks.

2. **Module-scope `var` with computed-field non-literal initializer.**  
   W583 covered module-scope `const` 3-D AoS with computed fields. W585 extends that to a mutable `var` whose initial value comes from a function call, ensuring the generated Verilog correctly instantiates a module-level packed register and assigns it once at elaboration/initialization time.

3. **Multi-site whole-array and indexed usage.**  
   Reading a module-scope 7-D AoS at several whole-array and indexed sites exercises the CSE temporary for the call result and ensures downstream indexing expressions share the same packed register.

4. **Width / simulation budget containment.**  
   7-D `[2]^7 Pt` = 16,384 elements × 32 bits = 524,288 bits. This is large enough to stress the module-scope path while keeping direct simulation under a few minutes, avoiding the 22+ minute wall-clock of W584 rank-17.

## 2. Scientific / engineering background

- **Global common subexpression elimination (CSE).** Classic work by Cocke (1970) extended CSE across basic blocks using available-expression data-flow analysis. Kildall (1973) unified this via global value numbering (GVN). For t27, the relevant insight is that a function-call result bound to a module-scope variable is an available expression for every subsequent use site in the same module.
- **Verified CSE.** CompCert’s `CSE` pass (value numbering over RTL) conservatively resets equations at function calls and memory stores to preserve correctness. Monniaux & Six (LCTES 2021) showed a lightweight, Coq-certified global CSE + loop-invariant code motion pass. t27’s Icarus-lowerable classifier and generated Verilog follow the same conservative principle: call results are materialized once and reused, never re-invoked.
- **Packed-vector module variables in SystemVerilog.** IEEE Std 1800-2017 §7.4 permits packed arrays as module-level variables. A packed vector of 524,288 bits is well below the 4 MiBit stress point of W584 and below the 65,536-bit LRM minimum discussion threshold, so tool compatibility risk is low.

Sources:
- J. Cocke, “Global Common Subexpression Elimination,” *Symposium on Compiler Construction*, 1970. https://doi.org/10.1145/800028.808480
- G. A. Kildall, “A Unified Approach to Global Program Optimization,” *POPL*, 1973. https://doi.org/10.1145/512927.512945
- D. Monniaux & C. Six, “Simple, light, yet formally verified, global common subexpression elimination and loop-invariant code motion,” *LCTES*, 2021. https://doi.org/10.1145/3461648.3463850
- CompCert `backend.CSE`: https://compcert.org/doc/html/compcert.backend.CSE.html

## 3. Implementation steps

1. **Spec witness** `specs/scratch/w585_bench_module_7d_aos_var_call_dedup.t27`
   - Define `pub struct Pt { x: i16, y: i16 }`.
   - Define `pub fn make_week(offset: u16) -> [2][2][2][2][2][2][2]Pt` returning a deterministic 7-D nested literal with computed fields (`offset + N`).
   - Define module-level `pub var dst : [2][2][2][2][2][2][2]Pt = make_week(10)`.
   - `test` block: indexed probes and whole-array assertion against a local `expected` literal.
   - `bench` block: use `dst` in at least two distinct assertion sites (whole-array and indexed) to exercise multi-site CSE.

2. **Integration test** in `bootstrap/tests/icarus_lowerable.rs`
   - Add `accepts_w585_bench_module_7d_aos_var_call_dedup`.

3. **Compiler / reference model**
   - Anticipate **zero compiler changes** if W583’s `emit_packed_scalar_value` width-cast fix and existing module-scope `var` paths already handle call-return initializers.
   - If a new failure appears, fix it and reseal affected specs; update `FROZEN_HASH` if `bootstrap/src/compiler.rs` is touched.

4. **Verification gates**
   - `cargo build --release -p t27c`
   - `cargo test -p t27c --bin t27c`
   - `cargo test -p tri`
   - `cargo test -p t27c --test icarus_lowerable`
   - `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
   - Direct `t27c icarus-simulate` and `t27c icarus-cocotb` on W585.

5. **Seal ceremony**
   - `t27c seal --save` for the new spec.
   - Record seal in `.trinity/seals/`.

6. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W585_2026-07-07.md`.
   - Update `.trinity/experience.md` with W585 learnings.
   - Update `.trinity/current-issue.md` with three W586 cooperation variants.
   - Save persistent memory files.

## 4. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Module-scope `var` initialization from call not lowerable | Keep 7-D width moderate (524 kbit) and rely on W583 width-cast fix. |
| Multi-site CSE creates duplicate temporaries or wrong sharing | Assert equality at distinct whole-array and indexed sites; cocotb reference model catches mismatches. |
| Direct simulation still slow | 7-D is ~1/8 the width of 16-D; expected wall-clock under 2 minutes. |
| Seal mismatches in unrelated specs | Reseal only affected specs; update FROZEN_HASH if compiler changes. |

## 5. Next wave cooperation variants (Wave Loop 586)

1. **Variant A — 18-D array-of-struct return call deduplication.**  
   `[2]^18 Pt` (8,388,608 bits, 262,144 elements). Continues the rank-scaling series. Risk: witness ~44 MB / ~2.4 M lines; direct simulation likely 40+ minutes.

2. **Variant B — 17-D array-of-struct return with non-power-of-two outer dimension.**  
   `[3][2]^17 Pt` (6,291,456 bits, 393,216 elements). Tests product-based width/index arithmetic at the boundary, following W569/W571. Indexed probes must keep `e ≤ 16383`.

3. **Variant C — Large module-scope 8-D array-of-struct variable with indexed field writes (recommended).**  
   Module `var` of type `[2][2][2][2][2][2][2][2]Pt` (1,048,576 bits, 32,768 elements) initialized from a call, then updated at specific indices in a bench block, and read back at multiple sites. Covers module-scope **mutation** + CSE while staying under the 4 MiBit direct-simulation cliff.

## 6. Acceptance criteria

- New witness under `specs/scratch/w585_bench_module_7d_aos_var_call_dedup.t27`.
- Integration test `accepts_w585_bench_module_7d_aos_var_call_dedup` passes.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` reports zero new failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green / unchanged.
- Closeout report and W586 variants recorded.
