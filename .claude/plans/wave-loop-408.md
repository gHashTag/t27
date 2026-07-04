# Wave Loop 408 — real P12 CCLK measurement + complete SPI transaction model in Lean 4

**Issue:** #1318  
**Branch:** `wave-loop-408`  
**Date:** 2026-07-04  
**Milestone:** W407 closed the synthetic CCLK fixture and deeper static
N25Q128_3V timing constants. W408 should anchor the model to silicon and add a
complete SPI flash read-transaction proof.

---

## Goal

1. **Variant A** — Real CCLK measurement on pin P12.
2. **Variant B** — Fully automated cold-POR flash-boot smoke gate with a relay
   power switch + isolated/tri-stateable JTAG cable. Deferred to W409 unless
   relay hardware appears on the bench.
3. **Variant C** — Complete SPI flash read-transaction model in Lean 4.

Default recommendation: **Variant A + C bundle**. However, the bench check on
2026-07-04 showed the Digilent FTDI cable is present but **P12 is not wired to
ADBUS4** (live capture returned 100 k all-high samples at 0 MHz). Therefore the
**effective W408 default falls back to Variant C alone**, with a documented real-
capture blocker that can be re-attempted as soon as the wiring is available.

---

## Weak points investigated

| Weak point | Risk | How this wave addresses it |
|---|---|---|
| No real silicon CCLK measurement exists to anchor the 2.5 MHz nominal claim | Competitors can say the timing model is purely axiomatic | Attempt real P12 capture; if wiring is missing, record the blocker and keep the synthetic fixture as the CI anchor |
| `flash_spi_timing_ok` is a static config predicate, not a transaction | It does not model the *sequence* of CS# / SCK / wake-up events the FPGA engine issues | Add `SPIReadTransaction` + `artix7_boot_transaction` + `transaction_satisfies_flash_spec`; prove the canonical config produces a compliant transaction |
| CS# high time and wake-up are constants, not derived from CCLK | The model cannot yet prove the FPGA engine respects these board-level delays between transactions | Add the constants to the transaction spec and trace them to the N25Q128 datasheet; note in SSOT that engine timing is a future extension |
| Duty-cycle guard is a placeholder (25%–75%) | It is not a datasheet limit and may be too loose | Keep the guard as a sensible smoke test; document that it should be tightened once a real capture exists |
| No automated cold-POR (Variant B) | Manual power-cycle remains human-in-the-loop | Keep Variant B on the W409 backlog; no code changes in W408 |
| Competitors (Sparkle HDL, OpenTitan, prjxray) have deeper SoC/full-chip formal stories | t27 needs to keep narrowing to the *FPGA configuration stage* where it has physical evidence | The transaction model is specifically about the Artix-7 ↔ N25Q128 boot read, not a general SoC proof |

---

## Competitor scan (2026-07-04)

| Competitor / project | Relevant capability | t27 differentiator after W408 |
|---|---|---|
| [Sparkle HDL / Verilean](https://github.com/Verilean/sparkle) | Lean 4 HDL compiler + cycle-accurate simulation; verified RISC-V SoC, AXI, etc. | t27 does not design RTL in Lean; it formalizes a *vendor* 7-series boot interface and links it to physical cold-POR evidence. Sparkle has no public Artix-7 configuration-engine timing model. |
| [Kami / Kôika](https://github.com/SteffenReith/Kami) | Coq-based hardware DSL + verified compilation | Kami proves custom processors; t27 proves vendor FPGA configuration engine timing against an external flash datasheet. |
| [Project X-Ray / prjxray](https://github.com/f4pga/prjxray) | Reverse-engineered 7-series bitstream docs | prjxray documents *what* the bits mean; t27 formalizes the *timing consequences* of the CCLK/CS/SCK bits and validates them empirically. |
| [OpenTitan](https://opentitan.org/book/doc/security/specs/secure_boot/) | Secure SoC boot / RoT with formal security verification (Cycuity Radix, Uppaal) | OpenTitan secures a processor boot chain; t27 secures the FPGA *configuration* stage itself, including external SPI flash timing. |
| [spispy](https://github.com/StackSmashing/spispy) / SPI flash emulators | SPI flash emulation/monitoring for boot research | spispy emulates flash to study TOCTOU; t27 models the real on-board N25Q128 timing spec and validates against live capture. |
| Commercial SPI NOR VIP | Closed simulation reference models | t27 provides an open, machine-checked Lean 4 bound tied to a real Artix-7 board and a `sigrok-cli` measurement gate. |
| Yosys + SymbiYosys / Verilator formal | Open-source RTL formal verification | These tools verify custom RTL; t27 verifies vendor configuration-engine behavior and external flash interface timing at the system level. |

The unique position after W408 is a **machine-checked, transaction-level
boot-timing argument** that covers the actual sequence of CS# / SCK / wake-up
events, not just static frequency and duty predicates. Once P12 is wired, the
same model can be compared directly to measured silicon.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `.claude/plans/wave-loop-408.md` | This plan: weak points, competitor scan, chosen fallback variant |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | `SPIReadTransaction` structure; `artix7_boot_transaction`; `transaction_satisfies_flash_spec`; canonical + cold-POR theorems |
| 3 | `fpga/HARDWARE_SSOT.md` §3.6 | Traceability callout for transaction model; note that real P12 capture is blocked by missing wiring |
| 4 | `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-04.md` | Exact live-capture attempt output (0 MHz / all-high) and synthetic fixture rerun |
| 5 | `docs/reports/WAVE_LOOP_408_REPORT.md` | Close-out report with AC status |
| 6 | `docs/reports/FPGA_LOOP_COOPERATION_2026-07-04.md` | Three W409 cooperation variants |
| 7 | `docs/NOW.md`, `.trinity/experience.md` | W408 entry and learnings; ensure `Last updated:` is 2026-07-04 |
| 8 | git/PR | Commit, push `wave-loop-408`, open PR #1318, create W409 issue/branch |

---

## Acceptance criteria

- [ ] AC-A1: real P12 capture attempted; if wiring is missing, the blocker is
      recorded in the evidence file and HARDWARE_SSOT.md.
- [ ] AC-A2: `tri fpga measure-cclk --live ... --validate` is re-run and the
      output is committed as evidence (success or documented blocker).
- [ ] AC-B1: Variant B remains deferred to W409.
- [ ] AC-C1: `TernaryFPGABoot.lean` defines `SPIReadTransaction`,
      `artix7_boot_transaction`, and `transaction_satisfies_flash_spec`.
- [ ] AC-C2: A theorem proves that the canonical bitstream configuration
      produces an N25Q128-compliant boot transaction.
- [ ] AC-C3: A theorem links the cold-POR predicate to the transaction spec.
- [ ] AC-D1: `./scripts/tri test` passes (576/576).
- [ ] AC-D2: `lake build Trinity.TernaryFPGABoot` passes.
- [ ] AC-D3: `cargo test -p tri fpga::tests` passes.
- [ ] AC-D4: W408 report + evidence + W409 cooperation variants committed.

---

## Default variant

**Variant C alone for W408** because the bench lacks a P12 → logic-analyzer
wire. Variant A should be the first priority in the next wave as soon as the
wiring is available; Variant B remains the automation priority for W409.

---

*phi^2 + phi^-2 = 3 | TRINITY*
