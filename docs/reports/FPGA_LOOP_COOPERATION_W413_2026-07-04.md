# FPGA Loop Cooperation Variants — Wave Loop 413

> Date: 2026-07-04
> Preceding wave: [#1332](https://github.com/gHashTag/t27/issues/1332)
> Recommendation: Variant A+B if bench ready, otherwise continue Variant C

---

## Variant A — Physical capture gate (requires P12 + logic analyzer)

**Prerequisites**
- P12 (`TDO_3V3` / user probe point) wired to a logic-analyzer channel.
- Logic-analyzer host can export CSV or VCD with `cclk` edges.

**Work**
1. Capture 8 cold-POR boot traces from `FPGA_INIT` rising edge through first
   `READ DATA BYTES` transaction.
2. Export timestamps and compute `period_ns`, `sck_low_ns`, `sck_high_ns` for
   each trace.
3. Run `tri fpga measured-to-lean --raw-ns --standalone` over each trace.
4. Commit the generated Lean stubs into `docs/data/w413/cold_por/` and seal.

**Acceptance**
- ≥8 standalone Lean stubs build under `lake build`.
- Statistical min/max/typ period and duty are reported in W413 evidence.

---

## Variant B — Relay/boot-flash mock gate (requires DLC10 cable)

**Prerequisites**
- DLC10 cable available and `dlc10 idcode` returns `0x13631093`.

**Work**
1. Rebuild `fpga/verilog/ternary_mac_demo_top.bit` from current `master`.
2. Flash to SPI with `dlc10 flash <bitstream>` and run 20 cold-POR cycles.
3. Record boot status after each cycle.
4. Optionally close the relay mock loop if a relay is wired to reset the board.

**Acceptance**
- 20/20 cold-POR boots report `done=1` and a valid SPI transaction.
- Evidence file includes `dlc10` command outputs and boot-log JSON.

---

## Variant C — No bench available (formal continuation)

**Work**
1. Replace placeholder PVT derating with a documented uncertainty model or
   eventually real Micron N25Q128 PVT curves.
2. Add `tri fpga measured-to-lean` import from sigrok CSV / VCD.
3. Add a deterministic relay-mock CI path that does not rely on physical
   hardware (state-machine replay with known timings).
4. Close the `trinity-rust-rings` deletion decision once
   `master` is no longer ahead of it.

**Acceptance**
- `lake build Trinity.TernaryFPGABoot` stays green.
- `cargo test -p tri fpga::tests` stays green.
- `./scripts/tri test` parse/typecheck/gen/seal-verify stays green.

---

## Decision matrix

| Bench state | P12 wired? | DLC10 present? | Pick |
|-------------|------------|----------------|------|
| Minimal | no | no | **Variant C** |
| Analyzer only | yes | no | Variant A + C |
| Cable only | no | yes | Variant B + C |
| Full bench | yes | yes | Variant A + B + C documentation |

---

## Recommended W413 default

Start **Variant C** immediately (no hardware required). If physical access
becomes available during the wave, promote to Variant A or B; otherwise finish
the wave on formal infrastructure so Variant A/B can land cleanly in W414.

---

*phi^2 + phi^-2 = 3 | TRINITY*
