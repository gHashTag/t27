# FPGA Loop Cooperation — Wave Loop 423 (2026-07-06)

**Next issue:** to be created after W422 closes (#1365).  
**Next branch:** `wave-loop-423`.

---

## Context at the end of W422

- The XC7A200T board is reachable via `openFPGALoader` + Digilent HS2 cable.
- SRAM load of `fpga/verilog/ternary_mac_demo_top_200t.bit` succeeds and produces
  STAT `0x401079FC`.
- Real XADC context is available (≈45.7 °C, ≈1.00 V VCCINT, ≈1.81 V VCCAUX).
- Two physical blockers remain:
  1. Pin P12 (CFGCLK / CCLK_0) is not wired to a logic analyzer.
  2. The on-board DLC10 / Platform Cable USB II is not connected to the host.
- The gen-verilog keyword-escape sub-fix reduced yosys smoke failures from 16 to
  7, all pre-existing and unrelated to keyword collisions.
- The PVT envelope shape theory is complete: low/high combined monotonicity and
  worst-case bound theorems are proved in Lean 4 and mirrored in Rust.

W423 must choose the highest-leverage variant given this state. The three
cooperation options are ordered by preference if the physical bench becomes
fully ready; otherwise fall through to Variant C.

---

## Variant A — Full physical CCLK capture and cold-POR flash sweep (preferred)

**Prerequisites:**

1. P12 wired to a logic-analyzer channel.
2. Logic analyzer available and able to export CSV or VCD.
3. Board power-cycling possible (manual is acceptable; relay is not required).

**Work split:**

| Task | Owner | Deliverable |
|------|-------|-------------|
| Wire P12 and capture CCLK for OSCFSEL 6 and 7 | Human operator | Two instrument exports (CSV or VCD) with timestamp, voltage/frequency, and operating context |
| Run `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone --validate --pvt-context ctx.json` | Agent | Generated Lean files that build with `lake build` |
| Prove `measured_cclk_satisfies_flash_spec` / `measured_cclk_with_pvt_satisfies_flash_spec` for the captured points | Agent | New theorems in `proofs/lean4/Trinity/TernaryFPGABoot.lean` |
| Run cold-POR flash sweep for OSCFSEL 6/7 and capture STAT | Human + agent | Two boot-log JSON files showing DONE=1 / any H2 timing failure |
| Update `fpga/HARDWARE_SSOT.md` §3.6.20 with measured frequencies/duty cycles | Agent | Documented real timing bounds |

**Acceptance criteria:**

- AC-A1: real CCLK captures for OSCFSEL 6 and 7 exist and pass `--validate`.
- AC-A2: generated Lean files build and the theorems are committed.
- AC-A3: cold-POR flash boot for OSCFSEL 6/7 is documented with STAT reads.

---

## Variant B — Instrument import depth + `--pvt-worstcase` (if hardware is partial)

**Prerequisites:**

- No full P12 wiring, but an external CCLK capture file is available from a
  previous session or another bench; OR
- No capture available, but the agent can extend the instrument-import pipeline
  so it is ready the moment a capture arrives.

**Work split:**

| Task | Owner | Deliverable |
|------|-------|-------------|
| Add CSV timestamp parsing for fractional seconds, milliseconds, and sample-number-only exports | Agent | Regression tests + updated `parse_csv_to_raw_ns` |
| Add VCD real-net slope filter (reject ΔV below noise window or Δt below configurable `t_setup`) | Agent | Regression test + updated `parse_vcd_to_raw_ns` |
| Add `tri fpga measured-to-lean --pvt-worstcase` mode that validates against the combined-monotonicity corner (max temp, min VCCINT, ss) | Agent | CLI option + regression test |
| Document multi-format import matrix in `fpga/HARDWARE_SSOT.md` | Agent | §3.6.20 or new §3.7 |

**Acceptance criteria:**

- AC-B1: fractional/millisecond/sample-number CSV timestamp columns parse
  correctly with regression tests.
- AC-B2: VCD real-net slope filter rejects noisy transitions with a regression
  test.
- AC-B3: `--pvt-worstcase` validates against the combined-monotonicity corner
  with a regression test.

---

## Variant C — Continue gen-verilog narrowing + remaining formal gaps (fallback)

**Prerequisites:**

- Bench still has no P12 wiring and no external capture is available.

**Work split:**

| Task | Owner | Deliverable |
|------|-------|-------------|
| Land the next safe gen-verilog #1245 sub-fix from the remaining 7 failures, if one is narrow and regression-free | Agent | Reduced yosys smoke failure count or unchanged count with explicit justification |
| Add VCD robustness for unknown `$timescale` units and dumpoff without preceding `#timestamp` | Agent | Unit tests + parser hardening |
| Add `ProcessCorner.worse_than` totality / decidability helpers if needed by future theorems | Agent | Small Lean lemmas in `TernaryFPGABoot.lean` |
| Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any new 2026 competitor developments surface | Agent | Refreshed competitor snapshot |

**Acceptance criteria:**

- AC-C1: VCD parser hardening lands with unit tests.
- AC-C2: gen-verilog sub-fix lands without increasing the 7-failure count, or
  is explicitly deferred if unsafe.
- AC-C3: competitor snapshot is current.

---

## Default selection rule

1. If P12 is wired and a logic analyzer is ready, execute **Variant A**.
2. Else if an external CCLK/CSV/VCD capture is available, execute **Variant B**.
3. Else execute **Variant C**.

The agent should probe the bench state at the start of W423 and choose
according to this rule, documenting the chosen variant in `.trinity/current-issue.md`
and `docs/NOW.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
