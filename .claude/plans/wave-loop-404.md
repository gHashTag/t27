# Wave Loop 404 Plan — FPGA: close physical CCLK measurement or extend formal CCLK bounds

**Issue:** [#1309](https://github.com/t27/t27/issues/1309)  
**Branch:** `trinity-rust-rings`  
**Date:** 2026-07-06  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

W403 closed without bench hardware by extending the Lean 4 model with the
`BitstreamConfig` canonical predicate and `ColdPOR` linkage lemmas. At the start
of W404 the attached Digilent FTDI cable and XC7A200T board were found to be
reachable (`openFPGALoader --detect` returned idcode `0x3636093`), so the
wave executed **Variant C** — a cable-connected SRAM smoke load in `tri fpga
smoke-gate --require-cable`. Variant A (P12 CCLK measurement) remains blocked
because no logic analyzer / oscilloscope is available.

---

## Weak points

1. **Physical CCLK measurement still blocked.** A Digilent FTDI cable and the
   XC7A200T board are connected, but no logic analyzer / oscilloscope is
   available for pin P12 capture. The agent can drive JTAG but cannot sample
   the analog CCLK waveform autonomously.
2. **Operator-dependent capture.** A logic-analyzer/oscilloscope capture
   requires human bench time; the agent cannot perform it autonomously.
3. **Cable detection fragility.** The `--require-cable` path depends on
   `openFPGALoader --detect` successfully returning an `idcode` line. If the
   board is powered off or the cable is swapped, the gate fails cleanly but
   cannot self-heal.
4. **Competitor formal-HDL pressure.** Verilean / Sparkle HDL and Aria-HDL can
   produce timing/properties claims; t27 must keep adding traceability layers
   (formal → spec → generated code → physical measurement) to stay ahead.

---

## Competitor scan

- **Verilean / Sparkle HDL:** Lean 4-based HDL with embedded proofs. Closest
  formal competitor. Their strength is correctness of generated hardware;
  t27's differentiation is the spec-first `t27`/`tri` pipeline plus physical
  FPGA evidence.
- **Aria-HDL:** meta-compiler with Lean 4 backend. Focuses on compiling
  existing HDL to verified code, not on the physical bring-up traceability chain.
- **seLe4n:** Lean 4 microkernel. Relevant only to secure-boot claims, not
  directly to FPGA CCLK measurement.
- **USENIX WOOT 2024 Zynq secure-boot paper:** shows that bitstream config
  fields matter for security; t27's formal config audit is a defensive parallel.
- **FIRRTL/Chisel/Clash/Bluespec:** no formal proof of FPGA boot timing or
  STAT-register outcomes in their standard flows.

---

## Cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_2026-07-06.md` for the pre-implementation
variants. The implemented wave chose **Variant C** because the Digilent cable
and XC7A200T board were detected; Variant A remains blocked by the lack of a
logic analyzer / oscilloscope.

### Variant A — Physical CCLK measurement (deferred)

Capture CCLK on P12, run `tri fpga measure-cclk --csv`, and record the
frequency/duty cycle in `fpga/HARDWARE_SSOT.md` §3.5. Deferred to W405 unless
hardware becomes available.

### Variant B — Formal `OSCFSEL`/CCLK bounds (not executed)

Add `OSCFSEL` value constants and a `cclk_within_flash_spec` predicate to
`TernaryFPGABoot.lean`; prove that the canonical config implies the N25Q128
read timing is satisfied under published Artix-7 startup-clock tables. This
remains a candidate for a future no-hardware wave.

### Variant C — `--require-cable` SRAM smoke load (implemented)

Extend `tri fpga smoke-gate` to detect the Digilent cable and, when present,
load the GF16 matrix into SRAM and assert `DONE=HIGH`. Verified on the bench:
- `openFPGALoader --detect -c digilent_hs2` returns idcode `0x3636093`.
- `tri fpga load-sram` completes with `done 1`.
- Post-load STAT = `0x401079FC`, matching `Trinity.StatRegister.boot_success`.

---

## Acceptance criteria

| ID | Criterion | Status |
|----|-----------|--------|
| AC-A1 | Physical CCLK trace captured on P12. | ⏸️ deferred |
| AC-A2 | `fpga/HARDWARE_SSOT.md` §3.5 contains measured value. | ⏸️ deferred |
| AC-B1 | New Lean 4 lemmas link `OSCFSEL`/CCLK bounds to decision trees. | ⏸️ not executed |
| AC-C1 | `tri fpga smoke-gate --require-cable` reaches `DONE=HIGH` on the bench. | ✅ |
| AC-D1 | `./scripts/tri test` passes. | ✅ |
| AC-D2 | W404 report + evidence + W405 cooperation variants committed. | ✅ |

---

## Chosen variant

**Variant C** — because the Digilent FTDI cable and XC7A200T board are
connected and reachable, making the hardware smoke gate the highest-leverage
close-out for W404. Variant A remains the next priority once a logic analyzer
or oscilloscope is available.

---

*φ² + φ⁻² = 3 | TRINITY*
