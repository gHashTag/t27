# Wave Loop 409 — real P12 CCLK retry + per-OSCFSEL SPI transaction lookup

**Issue:** #1323  
**Branch:** `wave-loop-409`  
**Date:** 2026-07-04  
**Milestone:** W408 delivered the canonical `SPIReadTransaction` proof and
attempted a real P12 capture; the FTDI cable is present but P12 is not wired to
ADBUS4. W409 retries the measurement and extends the formal model to every
documented `OSCFSEL` value.

---

## Goal

1. **Variant A** — Real CCLK measurement on pin P12. Requires a physical wire
   from P12 to ADBUS4 (or DSLogic/oscilloscope channel) plus the canonical cold-POR
   protocol.
2. **Variant B** — Fully automated cold-POR flash-boot smoke gate with a relay
   power switch + isolated/tri-stateable JTAG cable. Deferred to W410 unless relay
   hardware appears on the bench.
3. **Variant C** — Per-OSCFSEL SPI transaction lookup in Lean 4 and a tighter
   duty-cycle validation derived from the transaction model.

Default recommendation: **Variant A + C bundle**. However, the 2026-07-04 bench
check again shows P12 is not wired to ADBUS4 (live capture returns 100 k all-high
samples at 0 MHz). Therefore the **effective W409 default falls back to Variant C
alone**, with a documented real-capture blocker that can be re-attempted as soon as
the wiring is available.

---

## Weak points investigated

| Weak point | Risk | How this wave addresses it |
|---|---|---|
| No real silicon CCLK measurement exists to anchor the 2.5 MHz nominal claim | Competitors can say the timing model is purely axiomatic | Re-attempt real P12 capture; record the persistent wiring blocker and keep the synthetic fixture as the CI anchor |
| Transaction model only covers the canonical `OSCFSEL=0` configuration | W400 sweep showed `OSCFSEL=0..5` all boot; the formal claim should cover the documented range | Add `artix7_boot_transaction_for_oscfsel` and prove every `OSCFSEL ∈ {0..7}` produces a flash-spec-compliant transaction |
| Duty-cycle guard is a placeholder (25%–75%) | It is not a datasheet limit; at high CCLK it may be too loose | Derive the duty bound from the N25Q128 `t_CL` / `t_CH` limits and the measured frequency: `duty ∈ [t_CL·f, 1 - t_CH·f]` |
| W400 sweep only verified `OSCFSEL=0..5` on real hardware | OSCFSEL 6 and 7 are model-only | Document that 6 and 7 are predicted by the UG470 lookup and have not been physically booted yet |
| OSCFSEL-to-MHz table is an engineering approximation | Actual silicon CCLK can vary with voltage/temperature/process | Treat the lookup as *nominal*; note that the margin to the 50 MHz flash limit absorbs moderate variation for `OSCFSEL ≤ 5` |
| No automated cold-POR (Variant B) | Manual power-cycle remains human-in-the-loop | Keep Variant B on the W410 backlog; no code changes in W409 |
| Competitors can reproduce a discrete lookup table in any theorem prover | The hard part is the silicon anchor and the continuous process-variation argument | Keep the focus on the Artix-7 ↔ N25Q128 configuration-stage timing, and document the missing physical measurement as the remaining gap |

---

## Competitor scan (2026-07-04)

| Competitor / project | Relevant capability | t27 differentiator after W409 |
|---|---|---|
| [Sparkle HDL / Verilean](https://github.com/Verilean/sparkle) | Lean 4 HDL compiler + cycle-accurate simulation; verified RISC-V SoC, AXI, etc. | Sparkle has no public 7-series configuration-engine timing model. t27 formalizes the vendor boot interface and, once P12 is wired, compares it to live silicon. |
| [Kami / Kôika](https://github.com/SteffenReith/Kami) | Coq-based hardware DSL + verified compilation | Kami proves custom processors; t27 proves vendor FPGA configuration-engine timing against an external flash datasheet. |
| [Project X-Ray / prjxray](https://github.com/f4pga/prjxray) | Reverse-engineered 7-series bitstream docs | prjxray documents *what* the bits mean; t27 formalizes the *timing consequences* of the CCLK/CS/SCK bits. |
| [OpenTitan](https://opentitan.org/book/doc/security/specs/secure_boot/) | Secure SoC boot / RoT with formal security verification | OpenTitan secures a processor boot chain; t27 secures the FPGA *configuration* stage itself, including external SPI flash timing. |
| [spispy](https://github.com/StackSmashing/spispy) / SPI flash emulators | SPI flash emulation/monitoring for boot research | spispy emulates flash to study TOCTOU; t27 models the real on-board N25Q128 timing spec and validates against live capture. |
| Commercial SPI NOR VIP | Closed simulation reference models | t27 provides an open, machine-checked Lean 4 bound tied to a real Artix-7 board and a `sigrok-cli` measurement gate. |
| Yosys + SymbiYosys / Verilator formal | Open-source RTL formal verification | These tools verify custom RTL; t27 verifies vendor configuration-engine behavior and external flash interface timing at the system level. |

After W409, the defensive value is a **machine-checked, per-OSCFSEL transaction
lookup** plus a frequency-derived duty bound. The remaining competitive gap is the
physical P12 measurement, which is blocked only by wiring.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `.claude/plans/wave-loop-409.md` | This plan: weak points, competitor scan, chosen fallback variant |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | `artix7_boot_transaction_for_oscfsel`; theorem proving every `OSCFSEL ∈ {0..7}` satisfies the flash spec; link to `artix7_boot_transaction` |
| 3 | `cli/tri/src/fpga.rs` | Replace placeholder 25%–75% duty guard with `duty ∈ [t_CL·f, 1 - t_CH·f]` computed from `N25Q128_MIN_SCK_LOW_NS` / `N25Q128_MIN_SCK_HIGH_NS` and the measured frequency |
| 4 | `fpga/HARDWARE_SSOT.md` §3.6 | Per-OSCFSEL transaction table; note that 6 and 7 are model-only and that P12 capture is still blocked |
| 5 | `docs/reports/FPGA_LOOP_EVIDENCE_W409_2026-07-04.md` | Exact re-run of live capture (0 MHz / all-high), synthetic fixture, Lean build, Rust tests, and `tri test` summary |
| 6 | `docs/reports/WAVE_LOOP_409_REPORT.md` | Close-out report with AC status |
| 7 | `docs/reports/FPGA_LOOP_COOPERATION_W410_2026-07-04.md` | Three W410 cooperation variants |
| 8 | `docs/NOW.md`, `.trinity/experience.md` | W409 entry and learnings; ensure `Last updated:` is 2026-07-04 |
| 9 | git/PR | Commit, push `wave-loop-409`, open PR #1323, create W410 issue/branch |

---

## Acceptance criteria

- [x] AC-A1: real P12 capture re-attempted; persistent wiring blocker recorded
      in evidence file and `fpga/HARDWARE_SSOT.md`.
- [ ] AC-A2: `tri fpga measure-cclk --live ... --validate` succeeds on real
      hardware (blocked by missing P12 wire).
- [x] AC-B1: Variant B remains deferred to W410.
- [ ] AC-C1: `TernaryFPGABoot.lean` defines `artix7_boot_transaction_for_oscfsel`
      and proves every `OSCFSEL ∈ {0..7}` produces a flash-spec-compliant
      transaction.
- [ ] AC-C2: `cli/tri/src/fpga.rs` replaces the placeholder duty guard with a
      frequency-derived bound from the N25Q128 SCK low/high limits.
- [ ] AC-D1: `lake build Trinity.TernaryFPGABoot` passes.
- [ ] AC-D2: `cargo test -p tri fpga::tests` passes.
- [ ] AC-D3: `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass
      (576/576).
- [ ] AC-D4: `./scripts/tri test` gen-verilog-yosys-smoke failures are explicitly
      tracked and remain out of scope for this wave.
- [ ] AC-D5: W409 report + evidence + W410 cooperation variants committed.

---

## Default variant

**Variant C alone for W409** because the bench still lacks a P12 → logic-analyzer
wire. Variant A should be the first priority in the next wave as soon as the
wiring is available; Variant B remains the automation priority for W410.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
