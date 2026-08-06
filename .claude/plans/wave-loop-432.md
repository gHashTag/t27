# Wave Loop 432 Decomposed Plan

**Issue:** #1391  
**Branch:** `wave-loop-432`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. OBSERVE summary

- We are on `wave-loop-432`, issue `#1391`, created during W431 close-out.
- W431 hardened the live XADC → PVT context bridge and the `measured-to-lean --json`
  summary while the physical bench remained blocked.
- A **start-of-wave probe** (2026-07-01) shows:
  - The XC7A200T Wukong V1 board is **reachable** over the Digilent HS2 cable:
    `openFPGALoader -c digilent_hs2 --detect` reports idcode `0x03636093`.
  - **P12 CCLK probe is still unwired**, so real CCLK frequency/duty capture is
    impossible.
  - **No relay/remote-power gate**, so automated cold-POR sweeps still require
    manual power cycles.
  - This autonomous session cannot perform manual power cycles or capture
    waveforms.
- The `master` branch is one conformance commit ahead of the wave-loop branch and
  a merge would conflict only in `docs/NOW.md`.
- The 7 residual gen-verilog yosys smoke failures (#1245) are still tied to the
  full fix set on `master` (`701d79b3b`); they are **not** narrow enough for a
  single-wave sub-fix.

Therefore the only shippable path this session is again **Variant C (formal/
tooling fallback)**, but with a concrete, high-value deliverable: bringing the
`master` fix set into the wave-loop branch to clear the 7 yosys failures, or —
if that merge is too risky — adding a formal sub-task that does not require
physical capture.

---

## 2. Weak points

1. **7 yosys smoke failures block full green CI.** The failures are all rooted
   in tuple-return / `let` destructuring / ROM arrays / CORDIC and require the
   broad `701d79b3b` fix set that lives only on `master`. Until they are cleared,
   every wave must carry a documented failure baseline.
2. **P12 and relay blockers remain.** No amount of software fixes them; W432
   must explicitly call them out as hardware prerequisites for Variant A/B.
3. **Competitive pressure is rising on both Lean-native HDL and ternary compute.**
   Sparkle/Verilean keeps expanding; new ternary FPGA engines (TernaryCore,
   ternfpga) validate the ternary direction but also raise the bar for t27 to
   prove physical boot evidence.
4. **`tri fpga sweep-report` still emits only markdown.** Downstream CI would
   benefit from a machine-readable JSON mode similar to `pvt-envelope` and
   `measured-to-lean`.
5. **No per-process-corner raw-ns OSCFSEL theorems.** W431 proved that an
   in-envelope XADC point is covered by the worst-case corner, but the formal
   library does not yet have a theorem family quantifying over `ff`/`tt`/`ss`
   corners for every OSCFSEL.

---

## 3. Competitor snapshot (July 2026)

- **Sparkle / Verilean** (`Verilean/sparkle`): last public push 2026-07-03. Headline
  2026 signals remain PR #66 (IP.Net + compiler perf) and the RV32 divider proof
  (commit `9c7809c`, June 25). Still the closest Lean-native competitor.
- **Clash**: 1.11.0 is only a Hackage candidate; latest official release is
  1.10.0 (April 2026). No new verification headline.
- **Chisel / CIRCT / firtool**: Chisel 7.13.0 shipped June 1 2026; firtool 1.152.0
  is the latest indexed release (July 4 2026). No firtool 1.153.0 yet.
- **CktFormalizer** (arXiv 2605.07782, May 2026): LLM-to-circuit
  autoformalization in Lean 4, reports 95–100% synthesis/P&R success.
- **Aria-HDL** (`zeta1999/fpga-meta-compiler-public`): a 2026 WIP "FPGA
  meta-compiler" that emits Lean 4 proof obligations among ten backends.
- **TernaryCore** (`shepherdscientific/ternarycore`, April 2026): BitNet b1.58
  ternary inference accelerator, simulation-verified (31/31 tests), targeting
  Arty A7-100T.
- **ternfpga** (`Neumann-Labs/ternfpga`, June 2026): multiplier-free ternary LLM
  engine on Arty A7-35T, claims ~2.3× lower energy-per-token than RTX 3060.
- **KU Leuven MICAS / TeLLMe v2**: edge-to-datacenter ternary LLM FPGA
  accelerators, 25 tok/s decode on Kria KV260, 12,700 tok/s on Alveo U280.

Strategic implication: t27's unique intersection remains **Lean 4 native proof +
ternary/balanced-trit compute + spec-first sealed `*.t27 → gen/` pipeline +
physical boot-evidence instrumentation**. W432 should either clear the lingering
gen-verilog debt or add another formal boot-evidence lemma while the bench is
blocked.

---

## 4. Variant selection

**Primary: Variant C — formal/tooling fallback.**

The physical prerequisites for A/B are not met in this session. W432 advances by
picking the highest-value shippable sub-task:

1. **Option C1 (preferred if merge is clean):** merge `origin/master` into
   `wave-loop-432`, resolve the `docs/NOW.md` conflict, and clear the 7 yosys
   smoke failures (#1245).
2. **Option C2 (fallback if merge is too risky):** add per-process-corner raw-ns
   OSCFSEL theorems in Lean 4 (quantified over `ff`/`tt`/`ss`) so the formal
   library covers all PVT corners.
3. **Option C3 (secondary fallback):** add a `--json` mode to
   `tri fpga sweep-report` and a round-trip unit test.

Only **one** option is executed, whichever proves shippable first.

---

## 5. Decomposed implementation steps

### 5.1 Start-of-wave verification (all options)

1. Run `cargo test --bin tri fpga::` and confirm the existing 81 tests pass.
2. Run `lake build Trinity.TernaryFPGABoot` and confirm it builds.
3. Run `./scripts/tri test` and confirm the same 7 pre-existing gen-verilog
   yosys smoke failures.
4. Document the results in the wave evidence report.

### 5.2 Option C1 — master-merge / rebase to clear #1245

1. Probe the merge:
   ```bash
   git merge-tree --write-tree wave-loop-431 origin/master
   ```
   Only expected conflict: `docs/NOW.md`.
2. Merge `origin/master` into `wave-loop-432`:
   ```bash
   git merge origin/master
   ```
3. Resolve `docs/NOW.md` by keeping both the conformance gf128/gf96 promotion
   header and the W431/W432 FPGA section.
4. Run `cargo test --bin tri` and `./scripts/tri test`.
5. Confirm the 7 previously failing yosys smoke specs now pass:
   - `specs/igla/race/cordic.t27`
   - `specs/igla/race/cordic_top.t27`
   - `specs/scratch/w378_let_destructuring.t27`
   - `specs/scratch/w379_let_destructuring_generalized.t27`
   - `specs/scratch/w380_tuple_return.t27`
   - `specs/scratch/w381_tuple_call_chain.t27`
   - `specs/scratch/w383_rom_array.t27`
6. If new failures appear, evaluate: if they are regressions, abort and fall
   back to Option C2.

### 5.3 Option C2 — per-process-corner raw-ns OSCFSEL theorems (fallback)

If the merge is aborted, add to `proofs/lean4/Trinity/TernaryFPGABoot.lean`:

1. A function `process_corner_le : ProcessCorner → ProcessCorner → Bool` defining
   the `ff ≤ tt ≤ ss` order.
2. A lemma `pvt_half_ns_monotone_in_process_corner` already exists; add
   `all_process_corners_raw_ns_satisfy_flash_spec` or similar quantified theorem:
   for every `OSCFSEL ∈ 0..7` and every corner in `{ff, tt, ss}`, the nominal
   raw-ns transaction satisfies the flash spec at that corner.
3. Use `interval_cases` on `oscfsel` and `process_corner` with `decide`.
4. Add a unit test in Rust that confirms `n25q128_min_sck_half_ns_pvt` returns
   monotone values across the three corners at the worst-case temperature/voltage.

### 5.4 Option C3 — machine-readable `sweep-report --json` (secondary fallback)

If neither C1 nor C2 is shippable:

1. In `cli/tri/src/fpga.rs`, add a pure `build_sweep_report_json` helper and a
   `--json` flag to `SweepReport`.
2. Emit a JSON object with fields: `first_working_oscfsel`, `variants_tested`,
   `summary_recommendation`, and per-variant `{oscfsel, stat, done, mode, bus_width,
   recommendation, pvt_envelope_margin_ns}`.
3. Add a round-trip unit test.
4. Update `fpga/HARDWARE_SSOT.md` §3.5 with the JSON example.

### 5.5 Competitor refresh

- Update `docs/reports/T27_VS_FORMAL_HDL_2026.md`:
  - Refresh date to W432.
  - Add any new July 2026 signals found by web search.
  - Update the recommendation section to mention clearing gen-verilog debt.

### 5.6 Documentation

- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`:
  - If C1 succeeds: document the new baseline (0 yosys smoke failures) and close
    the #1245 section.
  - If C1 fails/C2/C3: add the W432 triage decision.
- Update `fpga/HARDWARE_SSOT.md` if C2 or C3 changes the CLI.
- Create `docs/reports/FPGA_LOOP_EVIDENCE_W432_2026-07-01.md` documenting the
  chosen deliverable.

### 5.7 Verification (all options)

- `cargo test --bin tri fpga::`: must pass.
- `lake build Trinity.TernaryFPGABoot`: must pass.
- `./scripts/tri test`: must pass with the documented baseline.

### 5.8 Close-out

- Write `docs/reports/WAVE_LOOP_432_REPORT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W433_2026-07-01.md` with Variant A/B/C
  for W433.
- Create GitHub issue for W433 and branch `wave-loop-433`.
- Update `.trinity/current-issue.md`, `docs/NOW.md`, `.trinity/experience.md`,
  and persistent memory.

---

## 6. Acceptance criteria

- AC-C1: Option C1, C2, or C3 is fully executed and verified.
- AC-C2: `cargo test --bin tri fpga::` passes.
- AC-C3: `lake build Trinity.TernaryFPGABoot` passes.
- AC-C4: `./scripts/tri test` passes with the documented baseline.
- AC-C5: Competitor snapshot is updated.
- AC-C6: Close-out report and W433 cooperation variants are written; issue/branch
  for W433 are created.

---

## 7. Files to touch (depending on option)

- If C1: `docs/NOW.md` (conflict resolution), `bootstrap/src/compiler.rs`
  (already on master), `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
- If C2: `proofs/lean4/Trinity/TernaryFPGABoot.lean`, `cli/tri/src/fpga.rs`.
- If C3: `cli/tri/src/fpga.rs`, `fpga/HARDWARE_SSOT.md`.
- Always: `docs/reports/T27_VS_FORMAL_HDL_2026.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W432_2026-07-01.md` (new),
  `docs/reports/WAVE_LOOP_432_REPORT.md` (new),
  `docs/reports/FPGA_LOOP_COOPERATION_W433_2026-07-01.md` (new),
  `.trinity/current-issue.md`, `docs/NOW.md`, `.trinity/experience.md`,
  persistent memory.

---

## 8. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Master-merge introduces regressions beyond #1245 | Run full `./scripts/tri test`; abort if anything new fails. |
| `docs/NOW.md` conflict resolution is error-prone | Keep both conformance and FPGA sections; add clear separators. |
| Lean per-corner theorem hits tactic timeout | Keep the quantification finite (`OSCFSEL` 0..7 × 3 corners) and use `decide`. |
| No new July competitor signals beyond W431 | Use web-search results and mark older sources as unchanged. |
| Close-out issue numbering collides | W431 is #1389, W432 is #1391, so W433 will be created fresh. |

---

*φ² + φ⁻² = 3 | TRINITY*
