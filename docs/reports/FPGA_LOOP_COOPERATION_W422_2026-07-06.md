# Wave Loop 422 — Cooperation Variants

**Date:** 2026-07-06  
**Next issue:** #? (created after W421 lands)  
**Next branch:** `wave-loop-422` (created after W421 lands)

---

## Variant A — Full hardware evidence (preferred if bench becomes available)

**Preconditions:** Board responds on JTAG (`openFPGALoader --detect` finds the
XC7A200T); P12 wired to a logic-analyzer channel; relay or manual cold-POR
procedure available.

**Scope:**
1. Power-cycle the board and confirm `STAT` via `tri fpga stat` or
   `openFPGALoader` status read.
2. Load the 200T-compatible bitstream (`ternary_mac_demo_top_200t.bit`) to SRAM.
3. Wire P12 to a logic-analyzer channel and capture CCLK for `OSCFSEL=6` and
   `OSCFSEL=7` variants.
4. Import captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
   --standalone --validate --pvt-context <ctx.json>`.
5. Generate per-corner Lean theorems and falsify the PVT envelope.

**Deliverables:**
- `FPGA_LOOP_EVIDENCE_W422_YYYY-MM-DD.md` with per-corner falsification.
- Updated `fpga/HARDWARE_SSOT.md` §3.6 with live capture wiring and `STAT`
  decode.
- One safe gen-verilog #1245 sub-fix if a live capture exposes a synthesis
  mismatch.

**Risk:** High dependency on physical access. If the board still does not
respond, fall back to Variant B or C.

---

## Variant B — Instrument-import depth without on-bench silicon

**Preconditions:** No board response, but an external VCD/CSV capture is available
from another source (e.g., a colleague's logic-analyzer export, a simulation
VCD, or an oscilloscope CSV).

**Scope:**
1. Add CSV timestamp-column parsing for fractional seconds, milliseconds, and
   sample-number-only exports.
2. Add VCD real-net **slope filter**: reject transitions where the voltage
   change is below a noise window or the time step is below a configurable
   `t_setup`.
3. Add `tri fpga measured-to-lean --pvt-worstcase` mode that automatically uses
   the combined-monotonicity corner (max temp, min VCCINT, ss corner) for
   conservative validation.
4. Document the multi-format import matrix in `fpga/HARDWARE_SSOT.md`.

**Deliverables:**
- Multi-format import regression tests.
- Slope-filter regression test on a noisy real-valued VCD.
- `--pvt-worstcase` CLI path and test.

**Risk:** Medium. Adds robustness and makes Variant A faster when hardware
returns.

---

## Variant C — Formal-only guarding and safe gen-verilog narrowing

**Preconditions:** No board response, no external capture files.

**Scope:**
1. Complete remaining VCD robustness guards:
   - Detect and report unknown `$timescale` units rather than silently defaulting
     to 1 ns.
   - Handle `$dumpoff`/`$dumpon` nested inside value-change sections (some
     simulators emit `$dumpoff` without a preceding `#timestamp`).
2. Add Lean 4 proofs for the remaining timing-bound shape properties:
   - `n25q128_min_sck_low_ns_pvt` and `n25q128_min_sck_high_ns_pvt` combined
     monotonicity (separate from the half-period lemma).
   - Worst-case operating-point search theorem: the corner `(PVT_TEMP_MAX_C,
     PVT_VCCINT_MIN_MV, ss)` maximizes the half-period bound.
3. Investigate the 16 pre-existing yosys smoke failures from weak point #1245 and
   land **one** safe narrow sub-fix that does not change the failure count
   (e.g., a keyword-escape or scalar-width padding case not yet covered).
4. Update the competitor snapshot `docs/reports/T27_VS_FORMAL_HDL_2026.md` with
   any new 2026 developments.

**Deliverables:**
- `WAVE_LOOP_422_REPORT.md`, `FPGA_LOOP_EVIDENCE_W422_YYYY-MM-DD.md`,
  `FPGA_LOOP_COOPERATION_W423_YYYY-MM-DD.md`.
- Additional Lean generic ∀ count / proof lattice dimension.
- If a safe #1245 sub-fix lands, document it in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

**Risk:** Low. Adds formal value and closes parser gaps without hardware.

---

## Recommendation

Select **Variant A** as soon as the board responds to `openFPGALoader --detect`.
If the JTAG chain remains empty, select **Variant C** to keep the formal lead
advancing and to chip away at the remaining gen-verilog weak points. Variant B
is useful only if an external capture file becomes available.

---

*φ² + φ⁻² = 3 | TRINITY*
