# Wave Loop 443 — PVT-envelope hardening for the 24-variant theorem matrix + real-capture fallback + gen-verilog debt (Variant B default)

**Issue:** #1417
**Branch:** `wave-loop-443`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 442.

---

## Goal

Wave Loop 442 extended the board-less `tri fpga smoke-gate --theorem-matrix`
across all three documented Artix-7 process corners (`ff`/`tt`/`ss`), producing
24 verified PVT-aware raw-ns theorems. It also hardened the smoke-gate JSON
report with `schema_version: "1.0"` and added Rust unit tests for the theorem
matrix fixture path and the report schema. The bench remains blocked (P12
unwired, no relay gate, no DLC10 cable), the `gen-verilog` fix set
(`701d79b3b`) is still not merged, and the full Trinity `lake build` is still
broken on unrelated physics proofs in `Trinity/NeutrinoMasses.lean` and
`Trinity/H4Lagrangian.lean`.

Wave Loop 443 executes **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W443_2026-07-01.md`:

1. Add explicit PVT-envelope validation to the 24-variant theorem matrix so that
   every synthetic corner point is checked against the documented Artix-7
   operating rectangle before a theorem is generated.
2. Extend the smoke-gate report `theorem_matrix` record with a per-variant
   `envelope_check` field (`ok`/`failed`/`skipped`).
3. Add Rust unit tests for the envelope check and the new report fields.
4. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals that appear after 2026-07-11.
5. Mint the W443 evidence file and cooperation variants for W444.

**Variant A remains preferred** if the bench unblocks during the wave: run a real
`cclk-sweep --xadc --to-pvt-context` and mint `XADC_LIVE_W443_OPERATING_POINT`.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 131+/131 active, 0 ignored, 0 new regressions).
- [ ] `cargo test -p t27c --bin t27c suite::tests` passes.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures; FPGA smoke fails: 0).
- [ ] `./scripts/tri test --json suite-summary.json` produces a parseable
      machine-readable summary and the schema regression tests pass.
- [ ] `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json ...`
      produces a 24-variant `theorem_matrix` with per-variant `envelope_check`
      records and `passed: true`.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 444 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
