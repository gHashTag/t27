# FPGA Loop Cooperation — Wave Loop 425 (2026-07-05)

**Issue:** #1374  
**Branch:** `wave-loop-425`  
**Context:** Continuation of the FPGA boot-evidence line from W424.

---

## State at the start of W425

- **Branch:** `wave-loop-425` created from `master`, with `docs/NOW.md` and
  `.trinity/current-issue.md` already set up for #1374.
- **Board / cable:** the physical QMTech Wukong V1 / XC7A200T board is reachable
  through the Digilent FTDI cable (`0x0403:0x6014`) via `openFPGALoader`.
- **SRAM load:** still works, still produces `STAT=0x401079FC`.
- **Flash boot:** canonical W400 evidence (cold-POR, `OSCFSEL=0`, `DONE=1`)
  exists in `docs/reports/FPGA_EVIDENCE_W400.md` and is the historical baseline.
- **CCLK capture:** pin P12 (CFGCLK / CCLK_0) is **not wired** to the logic
  analyzer; real CCLK capture for `OSCFSEL=6/7` remains blocked.
- **Relay / power-cycle gate:** still absent; automated cold-POR is blocked.
- **Instrument import:** hardened in W424 (CSV voltage units, PVT context,
  VCD robustness, non-blocking waits).
- **Formal model:** `TernaryFPGABoot.lean` already proves every OSCFSEL 0..7
  rate satisfies the N25Q128 timing spec nominally and under the placeholder
  worst-case PVT envelope.
- **gen-verilog #1245:** 7 pre-existing yosys smoke failures remain. They are
  tied to major codegen features and are **not** safe branch-local sub-fixes.

W425 must therefore start as **Variant C** (formal / tooling hardening) and keep
Variants A/B ready so that if the bench state changes mid-loop, the agent can
pivot immediately.

---

## Variant A — Real P12 CCLK capture + cold-POR flash sweep (preferred if bench ready)

**Prerequisites:**

1. P12 wired to a logic-analyzer channel.
2. Logic analyzer (DSLogic Plus) available and able to export CSV or VCD.
3. Board power-cycling possible (manual is acceptable; relay is not required).

**Work split:**

| Task | Owner | Deliverable |
|------|-------|-------------|
| Wire P12 and capture CCLK for OSCFSEL 6 and 7 | Human operator | Two instrument exports (CSV or VCD) with timestamp, voltage/frequency, and operating context |
| Patch the W400 bitstream or regenerate with `OSCFSEL=6/7` | Agent / human | Two `.bit` files programmed to SPI flash |
| Run `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone --validate --pvt-context ctx.json` | Agent | Generated `.lean` files that build with `lake build` |
| Prove / commit the generated theorems | Agent | New theorems in `proofs/lean4/Trinity/TernaryFPGABoot.lean` or standalone files |
| Run cold-POR for each OSCFSEL and capture STAT | Human + agent | Boot-log JSON files showing `DONE=1` or explicit H2 diagnosis |
| Update `fpga/HARDWARE_SSOT.md` §3.6.21 | Agent | Documented real frequencies, duty cycles, PVT context |

**Acceptance criteria:**

- AC-A1: real captures for OSCFSEL 6 and 7 exist and pass `--validate`.
- AC-A2: generated Lean files build and are committed.
- AC-A3: cold-POR flash boot for OSCFSEL 6/7 is documented with STAT reads.

---

## Variant B — Import a real or representative capture + PVT dry-run (if partial bench)

**Prerequisites:**

- No full P12 wiring, but an external CCLK capture file is available from a
  previous session, another bench, or a verified synthetic instrument fixture; OR
- The board is reachable for JTAG/SRAM but not for cold-POR automation.

**Work split:**

| Task | Owner | Deliverable |
|------|-------|-------------|
| Import the capture with `tri fpga measured-to-lean --csv/--vcd --raw-ns --standalone --validate --pvt-worstcase` | Agent | A generated Lean file that builds and a validated raw-ns triple |
| Add any missing unit/noise handling exposed by the real export | Agent | Regression test + parser fix |
| Run `tri fpga boot-log --dry-run` or `cclk-sweep --dry-run` for OSCFSEL 6/7 with `--pvt-context` | Agent | JSON outputs containing PVT/XADC context fields |
| Probe current board state with `openFPGALoader --detect` and `tri fpga stat` | Human + agent | Fresh IDCODE / STAT evidence in the report |
| Update `fpga/HARDWARE_SSOT.md` §3.6.21 with the import recipe and PVT-context checklist | Agent | Documented import + context checklist |

**Acceptance criteria:**

- AC-B1: at least one real or representative capture is imported end-to-end.
- AC-B2: the import path exposes no unhandled unit or noise cases.
- AC-B3: dry-run boot-log artifacts include PVT/XADC context fields for OSCFSEL 6/7.

---

## Variant C — Formal hardening + PVT falsification + safe gen-verilog deferral (default)

**Prerequisites:**

- Bench still has no P12 wiring and no external capture is available.

**Work split:**

| Task | Owner | Deliverable |
|------|-------|-------------|
| Add PVT operating-rectangle theorems that prove the *combined* worst case (max temp, min vccint, ss corner) is the actual upper envelope | Agent | New Lean lemmas in `TernaryFPGABoot.lean` and matching Rust unit tests |
| Land the next safe gen-verilog #1245 sub-fix if one is narrow and regression-free; otherwise explicitly defer | Agent | Reduced yosys smoke failure count, or unchanged count with justification |
| Harden `tri fpga boot-log` / `cold-por` / `cclk-sweep` JSON schema (operator checklist version, wait-seconds audit, XADC placeholder note) | Agent | Better JSON context or clearer decision-tree output |
| Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new 2026 competitor developments | Agent | Refreshed competitor snapshot |
| Document why real XADC readout is still deferred (or implement a narrow `xadc` subcommand if scope allows) | Agent | Decision note in `fpga/HARDWARE_SSOT.md` or working `xadc.source: "xadc"` |

**Acceptance criteria:**

- AC-C1: gen-verilog smoke count does not increase; any deferred fix is explained.
- AC-C2: at least one new PVT monotonicity / combined-worst-case theorem + test lands.
- AC-C3: boot-log/cclk-sweep tooling is measurably more robust or better documented.

---

## Default selection rule for W425

1. If P12 is wired and a logic analyzer is ready, execute **Variant A**.
2. Else if an external CCLK/CSV/VCD capture is available or the board is
   reachable for a dry-run boot-log with PVT context, execute **Variant B**.
3. Else execute **Variant C**.

**Initial choice for this loop:** **Variant C**, because P12 is unwired and no
external capture is available. The agent should keep Variants A/B ready and
probe the bench state at every turn.

---

*φ² + φ⁻² = 3 | TRINITY*
