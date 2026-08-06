# Decomposed Plan — Wave Loop 412

> Issue: #1332  
> Branch: `wave-loop-412`  
> Continues: W411 close-out (`docs/reports/WAVE_LOOP_411_REPORT.md`)

---

## Weak points inherited from W411

1. **Physical evidence gap.** `measured-to-lean` is implemented, but the input
   has been synthetic because P12 is not wired and the DLC10 cable is not
   detected.
2. **PVT margin is a placeholder.** The 12 ns worst-case constants are a 2×
   derating, not derived from N25Q128 PVT characterization data.
3. **Bench dependency.** All physical progress is gated by hardware access.
4. **Relay CI gate missing.** Cold-POR automation is required before the SPI
   boot gate can be exercised in CI.

---

## Competitor scan (2026 formal-HDL / FPGA-boot space)

Same set as W411, with emphasis on the gap W412 is meant to close:

- **Sparkle HDL / VeriLean** — strong semantics, no public measured-to-formal
  pipeline for SPI flash timing.
- **Kami / Kôika** — Coq hardware DSL; no 7-series configuration timing model.
- **prjxray / OpenXC7** — bitstream-level knowledge; complementary.
- **OpenTitan** — SoC boot security, not FPGA configuration stage.
- **spispy / SPI flash emulators** — protocol analysis, not machine-checked
  timing bounds.
- **Commercial SPI NOR VIP** — closed reference models, not tied to live
  measurement.

W412's relay CI gate + real capture would be hard to reproduce end-to-end.

---

## Chosen variant

**Primary: Bundle A + B** (when the bench becomes available).  
**Fallback: Variant C** (if P12/DLC10 remain blocked).

---

## Decomposed tasks

### A1 — Wire P12 and verify DLC10
- Wire P12 to the nearest logic-analyzer channel.
- Verify `dlc10 idcode` returns `0x13631093`.
- Run `tri fpga measure-cclk --json` and save the JSON under
  `docs/fpga/evidence/w412_oscfsel6_cclk.json`.

### A2 — Real `OSCFSEL=6,7` boot
- Program the bitstream with each `OSCFSEL` value.
- Run cold POR and collect `tri fpga boot-log --json`.
- Convert each measurement to a Lean theorem with
  `tri fpga measured-to-lean --name w412_oscfsel6`.
- Paste the theorem into `proofs/lean4/Trinity/TernaryFPGABoot.lean` and verify
  `lake build`.

### B1 — Relay cold-POR automation
- Add `fpga/src/relay.rs` with `PowerController` trait.
- Implement `MockPowerController` for CI and `HardwareRelay` for a USB relay.
- Add `tri fpga cold-por --oscfsel 6|7 --relay-port PORT`.
- Add mock smoke test in `cargo test -p tri fpga::relay_tests`.
- Document relay wiring in `fpga/HARDWARE_SSOT.md` §3.6.12.

### C1 — PVT refinement (fallback)
- Find or document N25Q128 PVT derating curves.
- If no data, strengthen the falsification plan for the 2× placeholder.

### C2 — Standalone `.lean` output (fallback)
- Extend `measured_to_lean` to emit imports, namespace, and theorem block.

### C3 — Raw-ns input (fallback)
- Add `--raw-ns` mode reading `(period_ns, low_ns, high_ns)`.
- Add `measured_cclk_from_raw_ns_satisfies_flash_spec` and implication theorem.

### D — Close-out
- Write `docs/reports/WAVE_LOOP_412_REPORT.md`.
- Write `docs/reports/FPGA_LOOP_EVIDENCE_W412_2026-07-04.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W413_2026-07-04.md`.
- Update `docs/NOW.md` with W412 close-out / W13 setup.
- Update `.trinity/experience.md`.
- Commit, push `wave-loop-412`, open PR #1332, create issue/branch W13.

---

## Acceptance criteria

See `.trinity/current-issue.md` for the full checklist.

---

*φ² + φ⁻² = 3 | TRINITY*
