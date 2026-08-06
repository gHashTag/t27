# FPGA Loop Cooperation — Wave Loop 424 (2026-07-05)

**Next issue:** #1371  
**Next branch:** `wave-loop-424`.

---

## Context at the end of W423

- The XC7A200T board remains reachable via `openFPGALoader` + Digilent HS2 cable.
- SRAM load of `fpga/verilog/ternary_mac_demo_top_200t.bit` still succeeds and
  produces STAT `0x401079FC`.
- Real XADC context is available (≈45.7 °C, ≈1.00 V VCCINT, ≈1.81 V VCCAUX).
- The instrument-import pipeline in `tri fpga measured-to-lean` now handles:
  - CSV `time_ms`, `time_us`, `time_ns`, and sample-number columns;
  - VCD real-net slope filtering (`--vcd-slope-min-v`, `--vcd-slope-min-s`);
  - VCD unknown `$timescale` fallback and `$dumpoff`/`$dumpon` without timestamp;
  - `--pvt-worstcase` validation against the combined-monotonicity corner.
- The PVT envelope shape theory is complete through W422.
- Two physical blockers remain:
  1. Pin P12 (CFGCLK / CCLK_0) is not wired to a logic analyzer.
  2. The on-board DLC10 / Platform Cable USB II is not connected to the host.
- The gen-verilog weak point #1245 still has 7 pre-existing yosys smoke failures.
  All remaining failures are tied to major features (`let` destructuring, tuple
  returns, ROM arrays, CORDIC) and are not safe branch-local sub-fixes.

W424 must choose the highest-leverage variant given this state. The three
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
| Update `fpga/HARDWARE_SSOT.md` §3.6.21 with measured frequencies/duty cycles | Agent | Documented real timing bounds |

**Acceptance criteria:**

- AC-A1: real CCLK captures for OSCFSEL 6 and 7 exist and pass `--validate`.
- AC-A2: generated Lean files build and the theorems are committed.
- AC-A3: cold-POR flash boot for OSCFSEL 6/7 is documented with STAT reads.

---

## Variant B — Import a real external capture + boot-log dry-run (if hardware is partial)

**Prerequisites:**

- No full P12 wiring, but an external CCLK capture file is available from a
  previous session, another bench, or a synthetic instrument fixture; OR
- The board is reachable for JTAG/SRAM but not for cold-POR automation.

**Work split:**

| Task | Owner | Deliverable |
|------|-------|-------------|
| Import the capture with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone --validate --pvt-worstcase` | Agent | A generated Lean file that builds and a validated raw-ns triple |
| Add any missing unit/noise handling exposed by the real export | Agent | Regression test + parser fix |
| Run a dry-run cold-POR boot-log for OSCFSEL 6/7 variants | Agent | `tri fpga boot-log` JSON outputs and decision-tree conclusions (no physical power-cycle required if relay absent) |
| Update `fpga/HARDWARE_SSOT.md` §3.6.21 with the import recipe and any new fixture constraints | Agent | Documented import checklist |

**Acceptance criteria:**

- AC-B1: at least one real or representative capture is imported end-to-end.
- AC-B2: the import path exposes no unhandled unit or noise cases.
- AC-B3: dry-run boot-log artifacts exist for OSCFSEL 6/7.

---

## Variant C — Continue formal hardening + boot-log/flash tooling (fallback)

**Prerequisites:**

- Bench still has no P12 wiring and no external capture is available.

**Work split:**

| Task | Owner | Deliverable |
|------|-------|-------------|
| Land the next safe gen-verilog #1245 sub-fix from the remaining 7 failures, if one is narrow and regression-free; otherwise explicitly defer | Agent | Reduced yosys smoke failure count, or unchanged count with justification |
| Harden `tri fpga boot-log` / `cclk-sweep` cold-POR artifact capture for manual-power-cycle mode | Agent | Better JSON context, clearer decision-tree output, or board-less dry-run coverage |
| Add `ProcessCorner` totality / decidability helpers if future theorems need them | Agent | Small Lean lemmas in `TernaryFPGABoot.lean` |
| Update `docs/reports/T27_VS_FORMAL_HDL_2026.md` if any new 2026 competitor developments surface | Agent | Refreshed competitor snapshot |

**Acceptance criteria:**

- AC-C1: gen-verilog smoke count does not increase; any deferred fix is explained.
- AC-C2: boot-log/cclk-sweep tooling is measurably more robust or better
  documented.
- AC-C3: competitor snapshot is current.

---

## Default selection rule

1. If P12 is wired and a logic analyzer is ready, execute **Variant A**.
2. Else if an external CCLK/CSV/VCD capture is available or the board is
   reachable for a dry-run boot-log, execute **Variant B**.
3. Else execute **Variant C**.

The agent should probe the bench state at the start of W424 and choose
according to this rule, documenting the chosen variant in `.trinity/current-issue.md`
and `docs/NOW.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
