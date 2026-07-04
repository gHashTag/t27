# Wave Loop 411 — real P12 + OSCFSEL 6/7 retry, relay CI gate, or auto-proof tooling

**Issue:** #1329  
**Branch:** `wave-loop-411`  
**Date:** 2026-07-04  
**Milestone:** W410 completed the measured-duty formal link but both physical
paths (real P12 capture and DLC10-based `OSCFSEL=6,7` boot) remain blocked by
missing bench wiring and the missing DLC10 cable. W411 either finally connects
the bench or uses the cycle to build automation/formal tooling.

---

## Goal

1. **Variant A** — Real P12 CCLK capture + physical `OSCFSEL=6,7` cold-POR
   boot. Requires wiring P12 to a logic-analyzer channel and connecting the
   Digilent DLC10 cable to the host.
2. **Variant B** — Relay-controlled cold-POR hardware CI gate. Build the relay
   + tri-stateable JTAG automation infrastructure so the flash-boot gate can
   run unattended.
3. **Variant C** — Auto-proof tooling + PVT margins. Add a subcommand that
   generates a Lean theorem from a `tri fpga measure-cclk --json` measurement,
   and extend the formal model with conservative process/voltage/temperature
   margins.

Default recommendation: **Variant A + C bundle**. However, the 2026-07-04 bench
check confirms both physical blockers persist (P12 not wired, DLC10 not
detected). Therefore the **effective W411 default falls back to Variant C
alone**, with a small relay-trait scaffold from Variant B kept as a documented
optional add-on if time allows.

---

## Weak points investigated

| Weak point | Risk | How this wave addresses it |
|---|---|---|
| Measured CCLK data still requires manual copy-paste into Lean | Human error, not reproducible in CI | Add `tri fpga measured-to-lean` that reads `--json` output and emits a type-correct theorem skeleton |
| Formal model has no PVT margin layer | A single measured frequency does not prove compliance across temp/voltage/process corners | Add conservative temperature/voltage derating to `t_CL`/`t_CH` and prove the margin predicate still implies `transaction_satisfies_flash_spec` |
| `measured_cclk_satisfies_flash_spec` only checks one corner | Silicon CCLK and flash timing both vary with conditions | Introduce `measured_cclk_with_margin_satisfies_flash_spec` parameterized by temp (°C) and voltage (mV) |
| No board-less way to exercise the formal link end-to-end | The theorem is only tested by `decide` on a few examples | Add Rust-driven round-trip test: generate synthetic fixture → JSON → theorem snippet, and verify it type-checks against the Lean model |
| Relay-controlled cold-POR is still undefined | W405's manual power-cycle prevents unattended CI | (Optional) Add a `PowerSwitch` trait + mock implementation behind `--auto-power-cycle`, without requiring real relay hardware |
| Competitors can claim the measured-duty link is ad-hoc | The bridge between Rust and Lean is not principled | Mirror the integer-period/duty conversion exactly in both languages and document the conservative rounding directions in one place (`fpga/HARDWARE_SSOT.md`) |

---

## Competitor scan (2026-07-04)

| Competitor / project | Relevant capability | t27 differentiator after W411 |
|---|---|---|
| [Sparkle HDL / Verilean](https://github.com/Verilean/sparkle) | Lean 4 HDL compiler + cycle-accurate simulation | Sparkle has no public 7-series configuration-engine timing model. t27 adds a *measured-to-formal* pipeline: `sigrok-cli` capture → Rust JSON → Lean theorem. |
| [Kami / Kôika](https://github.com/SteffenReith/Kami) | Coq-based hardware DSL + verified compilation | Kami proves custom processors; t27 proves vendor FPGA configuration timing with external-flash datasheet margins. |
| [Project X-Ray / prjxray](https://github.com/f4pga/prjxray) | Reverse-engineered 7-series bitstream docs | prjxray documents *what* the bits mean; t27 formalizes the *timing consequences* across PVT corners. |
| [OpenTitan](https://opentitan.org/book/doc/security/specs/secure_boot/) | Secure SoC boot / RoT with formal security verification | OpenTitan secures processor boot; t27 secures the FPGA *configuration stage* including external SPI flash PVT margins. |
| [spispy](https://github.com/StackSmashing/spispy) / SPI flash emulators | SPI flash emulation/monitoring | Emulators study protocol semantics; t27 models real N25Q128 timing with conservative margins and validates against live capture. |
| Commercial SPI NOR VIP | Closed simulation reference models | t27 provides an open, machine-checked Lean 4 bound tied to a real Artix-7 board and a `sigrok-cli` measurement gate, plus PVT margin reasoning. |
| Yosys + SymbiYosys / Verilator formal | Open-source RTL formal verification | These verify custom RTL; t27 verifies vendor configuration-engine behavior and external flash interface timing at the system level. |

After W411, the defensive value is a **machine-checked, PVT-margin-aware,
measured-to-formal pipeline** that turns a real CCLK capture into a
`transaction_satisfies_flash_spec` proof. The remaining competitive gap is the
physical P12 measurement, which is blocked only by wiring.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `.claude/plans/wave-loop-411.md` | This plan: weak points, competitor scan, chosen fallback variant |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | `measured_cclk_with_margin_satisfies_flash_spec` predicate and theorem linking it to `transaction_satisfies_flash_spec`; helper lemmas for conservative PVT derating |
| 3 | `cli/tri/src/fpga.rs` | New `tri fpga measured-to-lean` subcommand; `MeasuredCclk` helper to print a Lean theorem snippet; optional `PowerSwitch` trait scaffold for Variant B |
| 4 | `fpga/HARDWARE_SSOT.md` §3.6.10/§3.6.11 | Document the measured-to-Lean pipeline and the PVT margin model; note remaining P12/DLC10 blockers |
| 5 | `cli/tri/src/fpga.rs` tests + integration test | Round-trip test: synthetic fixture → JSON → theorem snippet parseable by Lean |
| 6 | `docs/reports/FPGA_LOOP_EVIDENCE_W411_2026-07-04.md` | Build outputs, `measured-to-lean` example, Lean build, Rust tests, `tri test` summary |
| 7 | `docs/reports/WAVE_LOOP_411_REPORT.md` | Close-out report with AC status |
| 8 | `docs/reports/FPGA_LOOP_COOPERATION_W412_2026-07-04.md` | Three W412 cooperation variants |
| 9 | `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md` | W411 entry, learnings, and updated AC status |
| 10 | git/PR | Commit, push `wave-loop-411`, open PR #1329, create W412 issue/branch |

---

## Acceptance criteria

- [ ] AC-C1: `measured-to-lean` subcommand emits a type-correct Lean theorem
      skeleton from `--json` input.
- [ ] AC-C2: PVT margin predicate exists in Lean 4 and implies
      `measured_cclk_satisfies_flash_spec`.
- [ ] AC-C3: A Lean theorem proves that the PVT margin predicate implies
      `transaction_satisfies_flash_spec`.
- [ ] AC-B1 (optional): `PowerSwitch` trait + mock relay exists behind an
      `--auto-power-cycle` flag scaffold, or explicitly deferred.
- [ ] AC-D1: `lake build Trinity.TernaryFPGABoot` passes with the new lemmas.
- [ ] AC-D2: `cargo test -p tri fpga::tests` passes.
- [ ] AC-D3: `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- [ ] AC-D4: gen-verilog-yosys-smoke failures remain tracked separately.
- [ ] AC-D5: W411 report + evidence + W412 cooperation variants committed.

---

## Chosen variant

**Variant C alone** (auto-proof tooling + PVT margins), with an optional small
Variant B scaffold if time allows. This is the strongest implementable step
while the bench remains blocked.

---

*phi^2 + 1/phi^-2 = 3 | TRINITY*
