# Wave Loop 424 — FPGA tooling hardening: auto-continue boot logs, PVT/XADC context, CSV voltage units, ProcessCorner helpers (Closes #1371)

**Branch:** `wave-loop-424`  
**Issue:** #1371  
**Date:** 2026-07-05  
**Variant executed:** B dry-run sweep + C tooling hardening (physical bench still partially blocked)

---

## Executive summary

Wave 424 continued the Variant B/C plan because the CCLK probe (P12) is still
unwired and no relay/remote-power gate is available. The wave hardened the FPGA
CLI so that future physical captures are easier to run and better annotated:

1. `boot-log`, `cold-por`, and `cclk-sweep` now honor `--wait-seconds` with a
   non-blocking auto-continue and an early ENTER path.
2. All three commands accept an optional `--pvt-context` JSON file and embed both
   the supplied PVT context and an XADC placeholder in their JSON boot logs.
3. `tri fpga measured-to-lean --csv` gained `--csv-voltage-unit mv` so that
   oscilloscope exports in millivolts are parsed correctly.
4. `proofs/lean4/Trinity/TernaryFPGABoot.lean` gained small `ProcessCorner`
   decidability/equality helpers (`eq_decidable`, `worse_than_decidable`,
   `severity`, `worse_than_iff_severity_le`) to support future PVT automation.
5. The formal-HDL competitor snapshot was refreshed for mid-2026.
6. A representative end-to-end dry-run was executed: `cclk-sweep --dry-run`
   across OSCFSEL 0–7 and `measured-to-lean --csv --raw-ns` on a synthetic CSV,
   including a millivolt-scaling test.

The deferred item remains a safe gen-verilog #1245 sub-fix; the 7 pre-existing
yosys smoke failures are still tied to major codegen features.

All conformance gates pass: **60/60** `tri` fpga unit tests, **0** FPGA smoke
failures, **0 seal mismatches**, and `lake build Trinity.TernaryFPGABoot` passes.

---

## What changed

### 1. `cli/tri/src/fpga.rs` — boot/cold-por/cclk-sweep UX and context fields

#### Auto-continue with non-blocking stdin

- Added `wait_for_continue(wait_seconds, label)`. With `wait_seconds == 0` it
  blocks on ENTER; with `wait_seconds > 0` it spawns a background stdin reader
  and auto-continues after the timeout.
- `boot_log` no longer ignores `--wait-seconds`.
- `cold_por --relay-port MOCK` now simulates the operator delay when
  `--wait-seconds` is positive.
- `cclk_sweep` replaced its polling `read_line` loop with the same helper; the
  previous loop was not truly non-blocking because `read_line` itself blocked.

#### PVT/XADC context in boot logs

- Added `--pvt-context` to `BootLog`, `ColdPor`, and `CclkSweep`.
- Added `load_optional_pvt_context`, `xadc_context_json`, and PVT serialization
  helpers.
- `SweepLog` now carries `pvt_context` and `xadc` fields; boot-log and cold-por
  JSON entries include the same fields.
- The XADC object is currently a placeholder (`source: "not_read"`) because real
  XADC readout is reserved for a future wave; when `--pvt-context` is
  supplied its temperature and rail values are copied into the XADC object for
  traceability.

#### CSV voltage-unit support

- Added `CsvVoltageUnit` (`V`, `Mv`) and `parse_csv_voltage_unit`.
- `parse_cclk_csv_reader` scales the voltage column by the selected unit before
  threshold detection.
- `MeasuredToLean` gained `--csv-voltage-unit v|mv`.

#### Default OSCFSEL sweep range

- `cclk_sweep` now defaults to OSCFSEL 0–7 instead of 0–5 so that dry-run/report
  paths exercise a wider range without requiring explicit `--values`.

### 2. `proofs/lean4/Trinity/TernaryFPGABoot.lean` — ProcessCorner helpers

Added small decidability/equality infrastructure:

- `ProcessCorner.eq_decidable`
- `ProcessCorner.worse_than_decidable`
- `ProcessCorner.severity` (ff=0, tt=1, ss=2)
- `ProcessCorner.worse_than_iff_severity_le`

These support future automation that compares PVT contexts and picks the worst
operating point without leaving a `Prop` goal.

### 3. `docs/reports/T27_VS_FORMAL_HDL_2026.md` — competitor refresh

- Updated date to 2026-07-01.
- Added firtool 1.152.0 (June 2026) to the CIRCT/Chisel section.
- Noted W423–W424 physical boot-evidence progress as a t27 differentiator.

### 4. `fpga/HARDWARE_SSOT.md`

No changes were required; the existing §3.6.20 instrument-import documentation
remains accurate. The new `--csv-voltage-unit` and `--pvt-context` behaviors are
CLI conveniences that fall under the documented CSV/VCD import pipeline.

---

## Verification

| Gate | Result |
|------|--------|
| `cargo test -p tri fpga::tests` | **60 passed** |
| `cargo build --release` in `bootstrap/` | **PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `tri fpga cclk-sweep --dry-run` (OSCFSEL 0–7) | **8 variants, first working = 0** |
| `tri fpga measured-to-lean --csv ... --raw-ns --validate` | **PASS** (volts and millivolts) |
| `tri fpga smoke-gate` | **PASS** (board-less) |

The 7 pre-existing yosys smoke failures from weak point #1245 are unchanged and
continue to be tracked in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

---

## Acceptance criteria status

From `.trinity/current-issue.md`:

### Bundle A (physical)

- [ ] AC-A1: real CCLK capture for OSCFSEL 6/7 — **blocked** (P12 unwired).
- [ ] AC-A2: `measured-to-lean --standalone` with real capture — **blocked**.
- [ ] AC-A3: PVT-aware flash-spec validation of real capture — **blocked**.
- [ ] AC-A4: cold-POR SPI flash boot for OSCFSEL 6/7 — **blocked** (no relay gate).

### Bundle B (instrument depth / tooling hardening)

- [x] AC-B1: `boot-log` and `cclk-sweep` honor `--wait-seconds` with
    auto-continue and early ENTER.
- [x] AC-B2: boot-log/cold-por/cclk-sweep JSON includes PVT context and XADC
    placeholder fields.
- [x] AC-B3: `measured-to-lean --csv` supports `--csv-voltage-unit mv`.
- [x] AC-B4: `cclk-sweep` dry-run exercises OSCFSEL 0–7.

### Bundle C (fallback)

- [x] AC-C1: `ProcessCorner` decidability helpers added to
    `TernaryFPGABoot.lean` with a successful `lake build`.
- [x] AC-C2: competitor snapshot refreshed for mid-2026.
- [ ] AC-C3: one safe gen-verilog #1245 sub-fix — **deferred**. All remaining
    failures are tied to major codegen features, not narrow regression-free fixes.

### Invariant checks

- [x] `cargo test -p tri fpga::tests` passes.
- [x] `lake build Trinity.TernaryFPGABoot` passes.
- [x] `tri fpga smoke-gate` passes board-less.

---

## Weak points investigated

1. **Physical bench readiness:** P12 remains unwired and the relay gate is still
   absent. Variant A is on hold. The board is reachable via JTAG/SRAM, and the
   W400 flash-boot signature remains the canonical evidence target.
2. **gen-verilog #1245:** the 7 remaining yosys smoke failures are unchanged.
   They still require a codegen refactor on `master`, so no branch-local sub-fix
   was attempted.
3. **CSV/VCD import coverage:** voltage-unit handling removes one more source of
   silent mis-measurement. The PVT-context embedding prepares boot logs for
   falsifiable corner comparisons once real captures are available.

---

## Competitor note

The formal-HDL snapshot in `docs/reports/T27_VS_FORMAL_HDL_2026.md` was refreshed
for W424:

- **Sparkle / Verilean:** still the closest Lean-native competitor. PR #66 and
  the RV32 divider verification show continued formal + catalog growth.
- **Clash:** CIRCT port and Clash Formal remain the main formal stories, but the
  proof path is still external to the HDL language.
- **CIRCT / firtool:** release 1.152.0 (June 2026) continues LTL/Verif/BTOR2
  work; the gap remains at native dependent-type proof and ternary compute.

t27's differentiation continues to rest on the Lean-native + ternary +
spec-first sealed pipeline triangle plus the physical boot-evidence loop.

---

## Files touched

- `cli/tri/src/fpga.rs`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`

## Close-out artifacts

- `docs/reports/WAVE_LOOP_424_REPORT.md` (this file)
- `docs/reports/FPGA_LOOP_EVIDENCE_W424_2026-07-05.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W425_2026-07-05.md`

---

*φ² + φ⁻² = 3 | TRINITY*
