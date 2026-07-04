# Wave Loop 421 — Cooperation Variants

**Date:** 2026-07-06  
**Next issue:** #? (created after W420 lands)  
**Next branch:** `wave-loop-421` (created after W420 lands)

---

## Variant A — Full hardware evidence (preferred if bench becomes available)

**Preconditions:** P12 wired to a real capture point; DLC10 cable present; relay
or USB power switch present.

**Scope:**
1. Wire P12 to the buffered SPI clock net or a debug header.
2. Run cold-POR relay sweeps across the PVT operating rectangle using the W419/W420
   envelope (temp, VCCINT, process corner model).
3. Import captured VCDs with `--measured-to-lean` (no manual threshold needed,
   thanks to W420 auto-threshold).
4. Falsify the W420 PVT half-period bound against the measured CCLK period
   distribution.
5. Generate a Lean 4 theorem statement per captured corner.

**Deliverables:**
- `FPGA_LOOP_EVIDENCE_W421_YYYY-MM-DD.md` with per-corner falsification.
- Updated `fpga/HARDWARE_SSOT.md` §3.6 with capture wiring diagram.
- One safe gen-verilog #1245 sub-fix if the live bitstream exposes a new
  synthesis mismatch.

**Risk:** High dependency on physical access. If hardware not available, fall
back to Variant B or C.

---

## Variant B — Instrument-import depth without silicon

**Preconditions:** No bench, but the user can supply an external VCD/CSV from
another FPGA board or oscilloscope.

**Scope:**
1. Add support for CSV timestamp columns in fractional seconds / milliseconds.
2. Add support for VCD analog/real nets with explicit slope filters (ignore
   transitions with Δt < t_setup or ΔV < threshold_window).
3. Add a `dlc10 capture --stub` dry-run path that records the expected command
   sequence for later replay.
4. Extend the PVT envelope to include `OSCFSEL` derating coefficients.

**Deliverables:**
- Multi-format import docs.
- `dlc10 capture --stub` regression test.
- At least one new Lean 4 PVT shape lemma (e.g., setup-time monotonicity over
  OSCFSEL).

**Risk:** Medium. No silicon proof, but improves the pipeline and makes Variant A
faster when hardware returns.

---

## Variant C — Formal-only guarding and documentation

**Preconditions:** No bench, no external capture files.

**Scope:**
1. Complete the remaining VCD robustness guards:
   - `$timescale` parsing,
   - support for `$dumpoff`/`$dumpon` sections,
   - real-net slope/rise-time rejection.
2. Add Lean 4 proofs for the remaining timing-bound shape properties:
   - `n25q128_max_sck_half_ns_pvt` antitonicity,
   - combined temp + voltage + corner monotonicity,
   - worst-case operating point search lemma.
3. Write a public-facing comparison note: **t27 vs Sparkle/Verilean vs
   Clash/Chisel** highlighting the spec-first `*.t27 → gen/` + Lean proof
   pipeline.
4. (Optional) Land one of the smaller #1245 gen-verilog sub-fixes that is safe
   to apply without changing bitstream semantics.

**Deliverables:**
- `WAVE_LOOP_421_REPORT.md`, `FPGA_LOOP_EVIDENCE_W421_YYYY-MM-DD.md`,
  `FPGA_LOOP_COOPERATION_W422_YYYY-MM-DD.md`.
- Additional Lean generic ∀ count / proof lattice dimension.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` competitor note.

**Risk:** Low. Adds formal value and closes instrumentation gaps without
hardware.

---

## Recommendation

Select **Variant B** if any external capture becomes available before the
on-bench hardware is restored; it is the highest-value fallback. If no capture
is available, select **Variant C** to keep the formal lead and instrument
robustness advancing. Variant A remains the target as soon as P12 + DLC10 +
relay are available.

---

*φ² + φ⁻² = 3 | TRINITY*
