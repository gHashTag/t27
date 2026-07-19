# Wave Loop 596 — Decomposed Plan

## Issue

**#1567** — Module-scope `[11][2]^12 Pt` array-of-struct variable with a
non-power-of-two outer dimension (11), initialized from a function call, with
indexed signed field writes.

**Branch:** `wave-loop-596`  
**Previous:** Wave Loop 595 (#1566, branch `wave-loop-595`)

## Chosen cooperation variant

**Variant A — `[11][2]^12 Pt` module-scope mutable array-of-struct initialized
from a function call, with indexed signed field writes and read-back.**

- Total width: `11 × 4096 × 32 = 1,441,792` bits (~1.37 MiBit)
- Elements: `11 × 4096 = 45,056`
- Outer dimension 11 is the next odd non-power-of-two step in the ladder
  `3 → 5 → 7 → 9 → 11`.
- Stays well under the 4-MiBit interactive cliff.
- Expected zero compiler / reference-model changes.

## Rationale

Agent E weak-point analysis and literature survey confirmed:
- t27 compiler and cocotb reference model use actual dimension sizes for strides,
  not power-of-two rounding.
- Signed i16 leaf-value schedule `(2*e + offset) % 32768` stays in range for
  45,056 elements (`max raw = 90111`, `90111 % 32768 = 24574`).
- Multi-line W584 brace style is mandatory; rank 13 is lower than W595 rank 14,
  so parser risk is equal or lower.
- Simulator capacity at ~1.4 MiBit is safe; W595 at ~2.36 MiBit passed.
- Variant B (`[2]^18 Pt`) crosses the 4-MiBit cliff and is deferred.
- Variant C (`[9][2]^13 Pt` + `if` reassignment) adds only control-flow coverage
  and is deferred to W597.

## Decomposition

### Phase 1 — Witness generation (Spec/TDD)
- [ ] Adapt `/tmp/gen_w595.py` to `DIMS = [11] + [2] * 12`.
- [ ] Emit `specs/scratch/w596_bench_module_11x2p12_aos_var_call_write.t27`.
- [ ] Ensure multi-line brace style and balanced brackets/braces.
- [ ] Include `test` and `bench` blocks per L4 TESTABILITY.

### Phase 2 — Integration test (TDD)
- [ ] Add `accepts_w596_bench_module_11x2p12_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs` after the W595 test.

### Phase 3 — Build & structural gate
- [ ] `cargo build --release -p t27c`
- [ ] `cargo test -p t27c --bin t27c` → expect 1494/0/2
- [ ] `cargo test -p tri` → expect 78/0
- [ ] `cargo test -p t27c --test icarus_lowerable` → expect 56/0

### Phase 4 — Seal & baseline
- [ ] `t27c seal --save specs/scratch/w596_bench_module_11x2p12_aos_var_call_write.t27`
- [ ] Create empty Icarus baseline
  `.trinity/icarus-baselines/specs/scratch/w596_bench_module_11x2p12_aos_var_call_write.json`
  so the suite records actual output on first run.

### Phase 5 — Direct simulation & cocotb
- [ ] `t27c icarus-simulate` W596 → expect PASS
- [ ] `t27c icarus-cocotb` W596 → expect reference-model OK
- [ ] Clean `/tmp/claude-501/t27c_cocotb_*` before final gate.

### Phase 6 — Repository sweep
- [ ] `./scripts/tri test --fast` → expect non-smoke phases pass, 0 seal
      mismatches, 24 pre-existing yosys smoke failures unchanged.
- [ ] `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
      → expect all Icarus/cocotb PASS.

### Phase 7 — Documentation & closeout
- [ ] Update `.trinity/current-issue.md` to W596 details + W597 variants.
- [ ] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W596_2026-07-07.md`.
- [ ] Append W596 learnings to `.trinity/experience.md`.

### Phase 8 — Persistent memory & commit
- [ ] Create `memory/wave-loop-596.md` and add pointer to `MEMORY.md`.
- [ ] Commit on `wave-loop-596` with L1-compliant message referencing #1567.

## Risk mitigations

| Risk | Mitigation |
|------|------------|
| Parser silently truncates single-line mega-literal | Use multi-line W584 brace style; validate AST completeness |
| Signed i16 overflow in witness values | Keep `(2*e + offset) % 32768` schedule |
| cocotb disk exhaustion | Clean `/tmp/claude-501/t27c_cocotb_*` before gate |
| Outer dimension 11 stride math | Rely on prior W592-W595 dimension-agnostic paths |

## Next Wave Loop 597 cooperation variants (to finalize in closeout)

1. **Variant A — `[13][2]^11 Pt`:** next odd non-p2 outer dimension 13,
   1,114,112 bits, 34,816 elements. Continues the ladder.
2. **Variant B — `[2]^18 Pt`:** 8,388,608 bits, crosses 4-MiBit cliff;
   needs chunked-literal design.
3. **Variant C — `[11][2]^12 Pt` + conditional `if` reassignment:** same
   width, adds control-flow guarded whole-array write coverage.
