# FPGA Loop Evidence — Wave Loop 433

**Date:** 2026-07-01  
**Issue:** #1393  
**Branch:** `wave-loop-433`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was executed

Wave Loop 433 executed **Variant C3** of the FPGA boot-evidence plan: a board-less
formal fallback because the physical bench is still blocked (P12 CCLK probe
unwired, no relay cold-POR gate, no DLC10 cable connected) and the master-merge
path for the `gen-verilog` fix set remains infeasible.

The wave composed the W431 live-XADC envelope bound with the W432 per-process-corner
raw-ns OSCFSEL theorem, producing a single theorem that says any in-envelope XADC
operating point justifies the nominal raw-ns CCLK capture for any documented
OSCFSEL selection.

No new physical capture was performed; all validation is board-less.

---

## Evidence items

### 1. Live-XADC → OSCFSEL raw-ns PVT bridge theorem

`proofs/lean4/Trinity/TernaryFPGABoot.lean`

- `xadc_envelope_justifies_cclk_variant_raw_ns_pvt`
  - For every `oscfsel ≤ 7` and any `XadcOperatingPoint` inside the documented
    envelope with a process corner at least as slow as `ss`, the ideal raw-ns
    capture whose period equals `cclk_period_ns oscfsel` satisfies the PVT-aware
    raw-ns flash predicate under the measured PVT context.
- `xadc_envelope_justifies_cclk_variant_transaction_ok`
  - The same capture produces a flash-spec-compliant SPI read transaction.
- `xadc_live_example_oscfsel_6_raw_ns_pvt`
  - Concrete example: a representative live readout (≈43 °C, ≈1.00 V VCCINT,
    ≈1.81 V VCCAUX, slow-slow corner) satisfies the predicate for OSCFSEL=6.

These theorems mean a future `tri fpga read-xadc --json` output can be supplied
as `--pvt-context` to `tri fpga measured-to-lean`, and the generated theorem can
reference a single quantified lemma for any documented OSCFSEL.

### 2. Competitor refresh

`docs/reports/T27_VS_FORMAL_HDL_2026.md`

- Noted Sparkle PR #66 remains open as of late July 2026 (last push 2026-07-03),
  adding USB web server, memcached server, networking stack, and compiler perf
  fixes.
- Noted `firtool-1.152.0` was published 2026-07-04 as a maintenance release.
- Noted Clash 1.11.0 remains a Hackage candidate; latest official release is
  still 1.10.0.
- Noted Aria-HDL continued 2026 updates around retiming and PCIe BAR testing.

### 3. Defect baseline

`docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

- W433 triage entry: no compiler work attempted; the 7 residual yosys smoke
  failures remain the documented baseline.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test --bin tri fpga::` | **81 passed, 0 failed** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke | **PASS** |
| `./scripts/tri test` gen-verilog-yosys-smoke | **49 passed, 7 pre-existing failures** (#1245) |

The 7 pre-existing yosys smoke failures are unchanged:
- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

These are covered by the full fix set on `master` (`701d79b3b`), which is on a
divergent lineage relative to `wave-loop-433`.

---

## What is still blocked

- **P12 CCLK probe:** still not wired to a logic-analyzer channel, so real
  CCLK frequency/duty capture for any OSCFSEL variant is not possible.
- **Relay / remote-power cold-POR gate:** still not wired, so automated
  cold-POR SPI flash boot sweeps require manual power cycling.
- **DLC10 cable:** the on-board Xilinx Platform Cable USB II is still not
  connected; the working path remains the Digilent HS2 cable plus
  `openFPGALoader`.
- **Master-merge to clear #1245:** still not safely reachable from the
  wave-loop branch.

---

## Artifacts

- `proofs/lean4/Trinity/TernaryFPGABoot.lean` — XADC-to-OSCFSEL raw-ns PVT bridge
  theorem and example.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — refreshed competitor snapshot.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — updated #1245 triage.
- This evidence note.

*φ² + φ⁻² = 3 | TRINITY*
