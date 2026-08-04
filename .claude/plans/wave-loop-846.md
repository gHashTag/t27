# Wave Loop 846 — Decomposed Cooperation Plan (2026-08-04)

## Proposed issue/PR

- **Issue:** #1632 (expected)
- **PR:** #1633 (expected)
- **Branch:** `wave-loop-846`
- **Parent branch:** `wave-loop-845` HEAD because earlier wave PRs remain open
- **Goal:** mechanical increment of the non-power-of-two packed array-of-struct ladder

---

## 1. Weak-points audit

### Current branch state (`wave-loop-846`)
- Clean working tree after W845 closeout.
- `icarus_lowerable` suite: **305/0**.
- `FROZEN_HASH`: unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- Pre-existing `verilog_array_literal_expr` regression still fails (`r_ca_2_synthetic_no_comment_only_call_argument`); unrelated to the AoS ladder and tracked for a dedicated ring.
- 626 release warnings / ~780 clippy warnings remain unchanged.

### Recurring process hazards
1. **Generator copy hazard.** Copying `scripts/gen_w{N-1}.py` → `scripts/gen_w{N}.py` requires fixing three text locations:
   - destination path,
   - module header f-string,
   - `MID_IDX` comment.
   A missed location produces a stale witness that still parses but references the previous wave.
2. **Silent width-cliff risk.** The ladder is approaching ~1 MiBit packed vector per module variable. Icarus and Vivado both have practical width limits; W846 (~0.998 MiBit) is still safe but is the closest rung to the 4-MiBit soft ceiling yet.
3. **Manual plan update drift.** Each wave the same set of state files must be updated; missing one breaks autopilot traceability.

### Mitigations in this plan
- Use a single `sed` command for all three copy-hazard locations, then `grep` verify.
- Run every gate (parse → lowerable → simulate → cocotb → seal) before committing.
- Update all state files in one staged commit with a checklist.

---

## 2. Scientific / engineering background

### 2.1 Verified high-level synthesis: Vericert
- **Herklotz, Y., Wickerson, J., & Constantinides, G. A. “Formal Verification of High-Level Synthesis.” *Proceedings of the ACM on Programming Languages* (OOPSLA), 2021.** DOI: [10.1145/3485494](https://doi.org/10.1145/3485494). GitHub: [ymherklotz/vericert](https://github.com/ymherklotz/vericert).
- Vericert is a formally verified HLS tool from C to Verilog, built on CompCert and written in Coq. Its correctness theorem guarantees that generated hardware simulates the source C semantics. While it does not target SystemVerilog packed arrays directly, its memory model and value normalization inform how t27 should preserve array-of-struct semantics through lowering — especially the bit-exact mapping from source indices to Verilog vector slices.

### 2.2 FPGA Roofline and memory coalescing
- **Siracusa, M. et al. “A Comprehensive Methodology to Optimize FPGA Designs via the Roofline Model.” *IEEE Transactions on Computers*, 2021.** [PDF](https://re.public.polimi.it/bitstream/11311/1207688/1/A_Comprehensive_Methodology_to_Optimize_FPGA_Designs_via_the_Roofline_Model.pdf).
- The Hierarchical Roofline model for FPGAs uses a **memory quanta `Q`** to capture how many bytes an AXI master can transfer per cycle. Packing struct fields into a single wide vector (AoS in packed form) increases `Q` and can improve operational intensity, provided the access pattern is coalesced. The Wave Loop ladder is therefore probing the width/complexity ceiling where `Q` gains are still cheaply routable.

### 2.3 Vendor HLS struct/array layout
- **AMD/Xilinx Vitis HLS Memory Model and Struct Handling (UG1399).** [Memory Model](https://docs.amd.com/r/2023.2-English/ug1399-vitis-hls/Memory-Model), [Structs](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Structs), [pragma HLS aggregate](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-aggregate).
- Vitis HLS disaggregates internal arrays-of-structs into struct-of-arrays (SoA) by default, but interface structs can be **aggregated** with `compact=bit` to pack members into a single wide vector. This is the industrial analog of t27’s packed-vector AoS: a struct of two `i16` fields packed into one 32-bit element, then repeated across a non-power-of-two outer dimension.

### 2.4 Icarus Verilog packed-array behavior
- **Icarus Verilog issue #521 / #995:** variable indices on outer packed dimensions are not supported in plain `for` loops (only `generate` / `genvar`). t27c lowers the wave witnesses in a way that avoids this limitation, which is why the static `icarus-lowerable` gate and the dynamic `icarus-simulate` gate both pass.
- **Icarus commit `128c621` (June 2026):** “Fix width calculation for packed array bounds.” Relevant because non-power-of-two bounds used to be widened incorrectly; t27c’s current lowering is stable on modern Icarus builds.

---

## 3. Decomposed implementation plan

### Phase A — Generator preparation (5 min)
- Copy `scripts/gen_w845.py` → `scripts/gen_w846.py`.
- Fix copy hazard in three locations with `sed`:
  - `w845` → `w846`
  - `509` → `511`
  - `254` → `255`
- Verify with `grep -n "module w846\|w845\|OUTER = \|MID_IDX" scripts/gen_w846.py`.

### Phase B — Spec generation & direct gates (10 min)
- Run `python3 scripts/gen_w846.py`.
- Run `./target/release/t27c parse specs/scratch/w846_bench_module_511x2p6_aos_var_call_write.t27`.
- Run `./target/release/t27c icarus-lowerable --json ...`.
- Run `./target/release/t27c icarus-simulate ...` (expect 17 cycles, PASSED).
- Run `./target/release/t27c icarus-cocotb ...` (expect reference-model OK).
- Run `./target/release/t27c seal --save ...`.

### Phase C — Rust integration test (10 min)
- Add `accepts_w846_bench_module_511x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- Run targeted test: `cargo test --release --test icarus_lowerable accepts_w846_bench_module_511x2p6_aos_var_call_write`.
- Run full suite: `cargo test --release --test icarus_lowerable` (expect **306/0**).

### Phase D — Closeout artifacts (15 min)
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W846_2026-08-04.md`.
- Write `.claude/plans/wave-loop-847.md` with variants A/B/C.
- Update `.trinity/current-issue.md` for W847.
- Update `.trinity/experience.md` with W846 entry.
- Update `docs/NOW.md`.
- Update `.claude/skills/t27-wave-loop.md` (bump tracker to 847, add worked example).
- Update `.claude/skills/wave-loop-autopilot.md` run-list.
- Write persistent memory `wave-loop-846.md` and prepend to `MEMORY.md`.

### Phase E — Commit & publish (5 min)
- Stage all W846 artifacts (excluding `.claude/scheduled_tasks.lock`).
- Commit with `Closes #1632`.
- Push `wave-loop-846`.
- Open PR #1633 to `master`.
- Create GitHub issue #1634 for Wave Loop 847.

---

## 4. Cooperation variants for Wave Loop 847

- **A (recommended):** `[513][2]^6 Pt`, outer += 2, `MID_IDX = 256`. Continues the mechanical ladder; expected ~1.002 MiBit, just past the 1-MiBit psychological line but still far from the 4-MiBit cliff.
- **B:** `[511][3]^6 Pt` — grow the second inner dimension to stress stride scaling. Expected 97,920 elements, 3,133,440-bit packed vector (~2.986 MiBit). This is a deliberate boundary probe; if the backend rejects it, convert to a negative witness or fall back to Variant A.
- **C:** `[511][2]^6 Pt` with negative-index writes to exercise wrap-around / signed-index lowering in the packed variable. Keeps the W846 outer dimension but shifts the stress from width to index semantics.

---

## 5. Risk and mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Generator copy hazard | Medium | Stale wave references in generated spec | `sed` all three locations + `grep` verify |
| Width cliff at ~1 MiBit | Low | Simulation/elaboration failure or timeout | Run all gates; fall back to Variant C if Variant A fails |
| Icarus outer-dimension variable-index limitation | Low | `icarus-lowerable` false negative | t27c lowering already avoids this; monitor simulator output |
| Missing state-file update | Low | Broken autopilot / tracker | Use Phase D checklist |
| Pre-existing `verilog_array_literal_expr` failure | Confirmed | Noise in full `cargo test` | Scope W846 validation to `icarus_lowerable`; do not regress |

---

## 6. Close-out checklist

- [ ] `scripts/gen_w846.py` created and copy-hazard-free
- [ ] Spec generated and gates pass (parse/lowerable/simulate/cocotb/seal)
- [ ] Integration test added to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Full `icarus_lowerable` suite green (expected 306/0)
- [ ] `FROZEN_HASH` unchanged
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W846_2026-08-04.md`
- [ ] Next-wave plan `.claude/plans/wave-loop-847.md` with variants A/B/C
- [ ] State files updated: `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- [ ] Skill trackers updated
- [ ] Persistent memory `wave-loop-846.md` + `MEMORY.md` index
- [ ] Commit with `Closes #1632`, push branch `wave-loop-846`, open PR #1633
