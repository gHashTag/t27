# Wave Loop 422 — Live XC7A200T SRAM boot + gen-verilog keyword escape + PVT worst-case bound (Closes #1365)

**Branch:** `wave-loop-422`  
**Issue:** #1365  
**Date:** 2026-07-06  
**Variant executed:** A-lite hardware evidence + C fallback formal/tooling hardening

---

## Executive summary

Wave 422 re-evaluated the physical bench and discovered that the XC7A200T board
**is reachable** through a Digilent HS2 cable even though the on-board DLC10 is
still missing and the P12 CCLK probe remains unwired. This changed the wave
outcome from a pure Variant-C formal extension to a mixed A-lite/C close-out:

1. Captured first live XC7A200T evidence since W404: SRAM load of
   `ternary_mac_demo_top_200t.bit` succeeded with `done 1` and post-load STAT
   `0x401079FC` (DONE=1, MODE=001, EOS=1, no CRC/ID errors).
2. Recorded real XADC operating context: ≈45.7 °C, VCCINT ≈1.000 V, VCCAUX
   ≈1.807 V — a point inside the PVT envelope used by the flash-timing model.
3. Landed a safe, narrow gen-verilog sub-fix for weak point #1245: Verilog
   keyword collision escape (`\\name `). The fix reduced the yosys smoke failure
   set from 16 to 7 pre-existing failures and added two regression unit tests.
4. Completed the PVT envelope shape theory: separate combined-monotonicity lemmas
   for low/high bounds, a corner-ordering helper, and a worst-case operating-point
   bound theorem in Lean 4, mirrored by a numeric grid-search regression test in
   Rust.

All conformance gates pass: **576/576** spec checks, **0 seal mismatches**,
**7 pre-existing** gen-verilog yosys smoke failures (none new), **0** FPGA smoke
failures, and **52/52** `tri` fpga unit tests.

---

## What changed

### 1. `bootstrap/src/compiler.rs` — Verilog keyword escape (weak point #1245)

- Added a static `verilog_keywords()` list covering the Verilog-2001 reserved
  words.
- Added `verilog_safe_identifier(name: &str) -> String` that emits the escaped
  identifier `\\name ` when a user identifier collides with a keyword.
- Applied escaping consistently across:
  - function/task names and `current_fn_name`;
  - function parameters;
  - module-level `const`/`var` declarations and references;
  - local variable declarations and references;
  - `for` / `for_range` loop variables;
  - `ExprIdentifier`, `ExprCall`, `ExprEnumValue`;
  - `ExprFieldAccess` base identifiers and flattened field names.
- Added two regression tests:
  - `test_verilog_keyword_parameter_escaped` — a parameter named `task` is
    emitted as `\\task ` and referenced as `\\task  == k`.
  - `test_verilog_keyword_local_and_module_escaped` — module-level `wire`/`reg`
    and local `task` are all escaped in declarations and references.

**Impact:** the `benchmark.t27` parameter named `task` (and any other spec that
uses a Verilog keyword as an identifier) now produces yosys-acceptable
Verilog. The failure count for the gen-verilog yosys smoke gate dropped from
**16 to 7**, and no new failures were introduced.

### 2. `proofs/lean4/Trinity/TernaryFPGABoot.lean` — PVT envelope shape completion

- `pvt_low_ns_monotone_combined` — low bound is monotone in the combined ordering
  (temp ↑, VCCINT ↓, corner worse).
- `pvt_high_ns_monotone_combined` — high bound is monotone in the same combined
  ordering.
- `ProcessCorner.any_worse_than_ss` — every process corner is no better than the
  ss corner, which is the corner-ordering fact a worst-case search needs.
- `pvt_half_ns_worst_case_bound` — across the documented operating envelope, the
  PVT-aware half-period bound is maximized at `(max temp, min VCCINT, ss)`.
  This is the theorem a finite grid-search validation relies on.

### 3. `cli/tri/src/fpga.rs` — Rust regression mirroring the worst-case bound

- Added `test_pvt_half_ns_worst_case_bound`, which evaluates
  `n25q128_min_sck_half_ns_pvt` on a grid of temperatures, VCCINT values, and
  process corners and asserts that no sampled point exceeds the worst-case
  corner.

### 4. `fpga/HARDWARE_SSOT.md` — §3.6.19 live XC7A200T SRAM boot and XADC context

Documents the real board response, the exact `openFPGALoader` commands, the
decoded STAT register, XADC readings, and the remaining blockers (P12 probe,
DLC10 cable, SPI flash boot / OSCFSEL sweep).

### 5. Boot-log artifact

`build/fpga/boot-log-archive/boot-log-20260706-130006-w422-sram-load.json`
captures the live STAT read and XADC context. Because `build/` is
`.gitignore`d, the file is retained as a local lab artifact and its contents
are summarized in the evidence report.

### 6. Seal regeneration

The compiler change shifted `gen_hash_verilog` for specs that contain keyword
identifiers. All affected `.trinity/seals/*.json` files were regenerated with
`t27c seal <spec> --save`; no other hashes changed. The suite now reports
**0 seal mismatches**.

---

## Verification

| Gate | Result |
|------|--------|
| `./scripts/tri test` / `t27c suite --repo-root .` | **576 passed**, 0 seal mismatches, 7 pre-existing gen-verilog yosys smoke failures, 0 FPGA smoke failures |
| `cargo test -p tri fpga::tests` | **52 passed** |
| `cargo test -p t27c --bin t27c` | **1493 passed** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |

The 7 remaining yosys smoke failures are pre-existing weak point #1245 defects
and are tracked in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`:

- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`
- `specs/scratch/cordic.t27`
- `specs/scratch/cordic_top.t27`

None of these are keyword-collision failures; they relate to `let`
destructuring syntax, tuple returns, ROM array lowering, and CORDIC-specific
generated constructs.

---

## Acceptance criteria status

From `.trinity/current-issue.md`:

### Bundle A (physical)

- [ ] AC-A1: real CCLK capture for OSCFSEL 6/7 — **blocked** (P12 unwired).
- [ ] AC-A2: `measured-to-lean --standalone` with real capture — **blocked**.
- [ ] AC-A3: PVT-aware flash-spec validation of real capture — **blocked**.

### Bundle B (instrument depth)

- [ ] AC-B1: fractional/millisecond/sample-number CSV timestamp parsing — **deferred**.
- [ ] AC-B2: VCD real-net slope filter — **deferred**.
- [ ] AC-B3: `--pvt-worstcase` mode — **deferred**.

### Bundle C (fallback)

- [x] AC-C1: VCD robustness already addressed in W420/W421; no new W422 work.
- [x] AC-C2: PVT envelope shape lemmas and worst-case bound landed.
- [x] AC-C3: one safe gen-verilog #1245 sub-fix landed (keyword escape); failure
    count reduced from 16 to 7.
- [x] AC-C4: competitor snapshot not updated because no new 2026 developments
    surfaced during W422; `docs/reports/T27_VS_FORMAL_HDL_2026.md` remains the
    current reference.

---

## Weak points investigated

1. **gen-verilog #1245 — keyword collision:** closed the subclass for keyword
   identifiers. Remaining subclasses (let destructuring, tuple returns, ROM
   arrays, CORDIC) are not narrow regression-free fixes and remain tracked.
2. **Physical bench readiness:** the board is powered and reachable via HS2,
   but two prerequisites are still missing for the full Variant A plan:
   - P12 → logic analyzer wiring for CCLK capture;
   - DLC10 cable for the in-repo `dlc10` driver (HS2 works through
     `openFPGALoader` instead).
3. **PVT envelope shape:** fully closed. The worst-case bound theorem gives
   future validation tools a single corner to check.

---

## Competitor note

No new 2026 formal-HDL developments were found during W422. The current snapshot
remains `docs/reports/T27_VS_FORMAL_HDL_2026.md`, with Sparkle/Verilean as the
closest Lean-native competitor and t27's differentiation resting on the ternary
compute layer + spec-first sealed pipeline + physical boot-evidence loop.

---

## Files touched

- `bootstrap/src/compiler.rs`
- `cli/tri/src/fpga.rs`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `fpga/HARDWARE_SSOT.md`
- `.trinity/seals/*.json` (regenerated)

## Close-out artifacts

- `docs/reports/WAVE_LOOP_422_REPORT.md` (this file)
- `docs/reports/FPGA_LOOP_EVIDENCE_W422_2026-07-06.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W423_2026-07-06.md`

---

*φ² + φ⁻² = 3 | TRINITY*
