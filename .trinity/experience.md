# t27 / Trinity Agent Experience Log

## 2026-07-02 — Wave Loop 358 completion

### What worked
- Running `./scripts/tri` (via `t27c suite --repo-root .`) gives a single 546-check conformance gate; after cleaning 54 bare W347 blocks and regenerating seals from the repo root, the suite returned **546/546 PASS**.
- `env -u GH_TOKEN gh ...` is required when `GH_TOKEN` is set to an invalid token; the keyring-stored `gHashTag` account is usable once the env override is removed.
- `lake build Trinity.TernaryInference` isolates the IGLA proof module from pre-existing failures in physics modules (`H4Lagrangian`, `NeutrinoMasses`).

### What changed behavior
- `t27c seal --save` writes seals relative to the current working directory, not the repo root. Regenerating seals must be done from `/Users/playra/t27` or the suite will read stale seals.
- The Verilog backend is critically broken for ternary MAC generation; FPGA evidence sprint is now blocked on either a hand-written synthesis module or a backend fix in `bootstrap/src/compiler.rs`.

### Patterns to reuse
- Before each wave: build `t27c`, run `t27c suite --repo-root .`, inspect `git status`, and address any bare/dangling blocks before adding new wave content.
- For issue-gated commits: if `GH_TOKEN` is invalid, use `env -u GH_TOKEN gh issue create` and reference `Closes #N` in the commit message.
- Keep the Lean proof lattice in `TernaryInference.lean` at 4 new generic ∀ theorems per wave; probe accumulation depth first, with minus-lattice parity as fallback if `omega` saturates.

### Anti-patterns to avoid
- Do not remove bare blocks without immediately regenerating all affected seals; otherwise the conformance gate fails with spec_hash mismatches.
- Do not stage `.claude/settings.json` or session metadata into wave-loop commits; keep those in separate commits or leave them unstaged.

## 2026-07-02 — Wave Loop 359 completion

### What worked
- Forward-appending W359 blocks with `test`/`invariant` keywords, plus 4 new Lean 4 generic ∀ theorems (`AccumulateThirtyFivePlus`, `AccumulateThirtyFourMinus`, `DuodecupleCancellation`, `ZeroWeightReorderingClosure`), kept the suite at **546/546 PASS** and pushed the generic ∀ count to **180**.
- Hand-writing a synthesis-ready ternary MAC in `fpga/verilog/ternary_mac_synth.v` bypassed the broken Verilog backend. A self-checking testbench (`tb_ternary_mac.v`) passed 6/6 vectors and `yosys synth_xilinx` produced metrics: 32 LUT5, 32 FDCE, 11 CARRY4.
- Even-number cancellation depths (12 for W359) collapse cleanly to identity with alternating plus/minus weights; odd depths leave a residual `mac(x,a,.plus)` or `x` mismatch, so always prefer even cancellation depths when targeting identity.

### What changed behavior
- The project now has **FPGA synthesis evidence** documented in `docs/reports/FPGA_EVIDENCE_W359.md`. This is the first measured hardware artifact.
- `iverilog` must be invoked from the directory containing the `.v` files and outputs; the `vvp` file is written to CWD, so `cd fpga/verilog` before running the simulator.
- `yosys` scripting for metrics should not mix `abc -liberty` with custom scripts; `synth_xilinx -top ternary_mac_top; stat` is sufficient for Xilinx resource counts.

### Patterns to reuse
- Structure each wave as: spec blocks → Lean theorems → build & seal → conformance → report → cooperation variants → memory. This cadence allows predictable 24–48 hour turnaround.
- For cancellation theorems, use even-length alternating plus/minus chains to guarantee identity collapse; verify with `lake build Trinity.TernaryInference` before seal regeneration.
- Preserve a hand-written synthesis fallback module (`ternary_mac_synth.v`) whenever the generated Verilog backend is unreliable; it protects the FPGA evidence pipeline.

### Anti-patterns to avoid
- Do not append bare wave blocks without `test`/`invariant`/`bench` keywords; the L4 TESTABILITY law rejects them and the conformance gate fails.
- Do not attempt odd-depth identity cancellation theorems without first checking the expected residual weight; even depths are safer.
- Do not rely on the generated Verilog backend for hardware evidence until it passes `yosys -p 'read_verilog'` cleanly.

## 2026-07-02 — Wave Loop 360 completion

### What worked
- A 36-variable `simp+omega` accumulation theorem (`ternaryMacAccumulateThirtySixPlusGeneric`) built successfully in ~3.1 s, so the omega boundary is still linear at depth 36.
- Forward-appending W360 blocks and regenerating all 27 seals from `/Users/playra/t27` returned **546/546 PASS** immediately after the Lean build.
- Creating a board-ready wrapper (`ternary_mac_demo_top.v`) with a ring-oscillator clock and LED outputs produced a clean `yosys` synthesis result: 34 cells, 12 CARRY4 total, estimated 10 LCs.

### What changed behavior
- The Wukong V1 ternary MAC design is now **ready to route**: RTL, XDC constraints, and yosys JSON netlist are in `fpga/verilog/`.
- `nextpnr-xilinx` is **not installed** on the build host; Homebrew only ships `nextpnr-ice40`. The OpenXC7 toolchain must be built from source per `fpga/HARDWARE_SSOT.md` §8.
- Odd-depth cancellation theorems collapse to a single non-identity MAC (`mac(x,a,.plus)` for depth 13), so the statement must match the residual weight.

### Patterns to reuse
- For deep accumulation proofs, generate the Lean binder list with **space-separated variables**; Lean does not accept comma-separated binders.
- For board-ready wrappers, reuse the `blinky.v` ring-oscillator pattern and the R23/T23 LED pins from existing QMTech designs; pass `--ignore-loops` to nextpnr.
- When the bitstream toolchain is missing, commit the ready-to-route artifacts and the evidence document; do not let the missing tool block the formal wave.

### Anti-patterns to avoid
- Do not generate Lean theorem parameters with Python `", ".join()`; use spaces.
- Do not stage `.claude/scheduled_tasks*` or session metadata into wave commits.
- Do not commit generated simulation artifacts (`.vvp`, intermediate `.json`) unless they are explicitly part of the deliverable.

## 2026-07-02 — Wave Loop 361 completion

### What worked
- `boost-python3` had to be actually installed (`brew install boost-python3`); `brew --prefix boost-python3` existing was not enough for CMake to find `Boost::Python 3.x`.
- Building `nextpnr-xilinx` with `-DARCH=xilinx -DUSE_OPENMP=OFF -DCMAKE_CXX_FLAGS="-I$(brew --prefix eigen)/include/eigen3"` succeeded on macOS arm64 with only deprecation/format warnings.
- `bbaexport.py` + `bbasm` produced a 152 MB `xc7a100tfgg676.bin` chipdb in ~1 minute.
- The full OpenXC7 flow yosys → nextpnr → fasm2frames → xc7frames2bit produced a **valid 3.6 MB Xilinx BIT file** for `ternary_mac_demo_top` on the first attempt.
- `nextpnr-xilinx` reported Fmax **643.92 MHz** for the ring-oscillator clock with 4 warnings and 0 errors.

### What changed behavior
- Trinity now has a **generated bitstream** for a formally-grounded ternary MAC, closing the "no silicon evidence" strategic vulnerability.
- The remaining hardware step is purely mechanical: connect the board + DLC10 cable and run `dlc10 sram ternary_mac_demo_top.bit`.
- The OpenXC7 toolchain is now available under `/tmp/openxc7-build/`; for reproducibility it should be moved to a permanent location (e.g. `~/opt/openxc7` or documented in `fpga/HARDWARE_SSOT.md`).

### Patterns to reuse
- Document the exact toolchain versions and build flags; future waves will need to reproduce this flow.
- When a tool is missing on macOS, check `brew list` and `brew info` before assuming the package is installed; `brew --prefix` can lie by returning a path for an uninstalled formula.
- For board flash attempts, always build `dlc10` first and run `dlc10 idcode` to confirm cable/board presence before claiming silicon validation.

### Anti-patterns to avoid
- Do not claim "silicon verified" without an actual board load and `DONE=HIGH`/LED observation.
- Do not leave the OpenXC7 toolchain only in `/tmp`; either persist it or document how to rebuild it.
- Do not forget to set `PYTHONPATH` when invoking `fasm2frames.py`; otherwise `ModuleNotFoundError: No module named 'prjxray'`.

## 2026-07-01 — Wave Loop 362 completion

### What worked
- Forward-appending W362 blocks to all 27 IGLA specs with `scripts/gen_w362.py` and regenerating all 27 seals from `/Users/playra/t27` returned **546/546 PASS** immediately after the Lean build.
- A 38-variable `simp+omega` accumulation theorem (`ternaryMacAccumulateThirtyEightPlusGeneric`) built successfully in **3.5 s**, so the omega boundary is still linear at depth 38.
- The quindecuple cancellation theorem (depth-15 residual `mac(x,a,.plus)`) and zero-weight quintuple closure theorem both built without new lemmas.
- The `dlc10` driver was rebuilt quickly with `cargo build --release -p dlc10` and is ready for the board flash once the QMTech Wukong V1 / Xilinx Platform Cable USB II is connected.

### What changed behavior
- The generic ∀ count across Trinity Lean modules reached **192** (184 `ternaryMac...Generic` theorems in `TernaryInference.lean` plus 8 generic theorems in `TernaryMac.lean`).
- The bitstream remains ready (`fpga/verilog/ternary_mac_demo_top.bit`, 3.6 MB), but the board flash is **blocked by missing hardware connectivity** (`DLC10 cable not found`).
- The W362 deliverable is therefore "silicon-ready" rather than "silicon-verified".

### Patterns to reuse
- For W363, reuse the same generator pattern and Lean theorem script; only the binder count and cancellation depth change.
- Always run `dlc10 idcode` before attempting `dlc10 sram`; idcode failure is a clear hardware-availability signal that should be documented, not hidden.
- When a wave includes both formal extension and hardware validation, complete and verify the formal work first so the hardware attempt does not compromise the zero-IGLA-failure streak.

### Anti-patterns to avoid
- Do not claim "board flashed" when only the bitstream exists; distinguish "generated", "loaded", and "observed running".
- Do not let a hardware blocker delay the spec/Lean/seal/report cadence; ship the formal deliverables and document the blocker.
- Do not commit generator scripts that are still one-off prototypes as part of the main wave commit unless they have been reviewed as tooling.

## 2026-07-01 — Wave Loop 363 completion

### What worked
- Reused `scripts/gen_w363.py` and `scripts/gen_w363_lean.py` to append W363 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **3.6 s**.
- `ternaryMacAccumulateThirtyNinePlusGeneric` (`a+b+...+am`) pushed the accumulation boundary to **39 variables**, still within the linear `simp+omega` regime.
- `ternaryMacSexdecupleCancellationGeneric` (depth-16 alternating plus/minus) collapsed cleanly to identity, confirming even-depth cancellation remains the safe default.
- `dlc10 idcode` was retried and the failure was documented as a hardware-availability blocker rather than a regression.

### What changed behavior
- Generic ∀ count reached **196** (188 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **97 waves** (twenty-third consecutive zero-failure wave).
- The W363 report and cooperation variants explicitly distinguish "bitstream generated" from "silicon physically observed" to avoid false claims.

### Patterns to reuse
- For cancellation theorems, keep alternating plus/minus weights and even depth to guarantee `= x` collapse without residual-weight adjustments.
- Continue the 4-theorem-per-wave cadence in `TernaryInference.lean`: accumulation probe, minus-lattice parity, cancellation depth, zero-weight closure.
- Document hardware blockers in a dedicated evidence file (`docs/reports/FPGA_EVIDENCE_W<N>.md`) so the load procedure is ready when the cable/board is available.

### Anti-patterns to avoid
- Do not modify a generator script with `sed` shortcuts without running it on a scratch copy first; the first `gen_w363.py` draft corrupted the expected-wave check.
- Do not let a single hardware blocker block the full wave deliverable; finalize the formal path and ship the report.
- Do not claim a theorem reaches identity unless the Lean statement literally ends in `= x` or matches the verified residual.

## 2026-07-01 — Wave Loop 364 completion

### What worked
- Reused `scripts/gen_w364.py` and `scripts/gen_w364_lean.py` to append W364 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **3.8 s**.
- `ternaryMacAccumulateFortyPlusGeneric` pushed the accumulation boundary to **40 variables**, still in the linear `simp+omega` regime.
- `ternaryMacSeptendecupleCancellationGeneric` (depth-17) correctly collapsed to residual `mac(x, a, .plus)`; the Lean statement matched the odd-depth residual exactly.
- A narrow, safe `gen_verilog` fix for binary literals (`0b...` → `N'b...`) landed in `bootstrap/src/compiler.rs` without regressions.

### What changed behavior
- Generic ∀ count reached **200** (192 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **98 waves** (twenty-fourth consecutive zero-failure wave).
- The `gen-verilog` backend now emits sized Verilog for binary literals; four larger lowering defects from #1245 are catalogued in `docs/reports/WAVE_LOOP_364_REPORT.md`.
- Board flash remains blocked by missing DLC10 cable/board; the failure is documented in `docs/reports/FPGA_EVIDENCE_W364.md`.

### Patterns to reuse
- For risky compiler changes, prefer narrow literal/formatting fixes over parser rewrites; parser changes can cause 100+ conformance regressions.
- Probe project weak points (e.g. #1245, #1246) during each wave and either fix, document, or file a reproduction; do not let them age silently.
- Keep the report/cooperation-variants cadence: `WAVE_LOOP_N_REPORT.md` + `WAVE_LOOP_N_COOPERATION.md` before the wave commit.

### Anti-patterns to avoid
- Do not attempt broad `parse_const_decl` / `skip_to_next_top_level` parser fixes without a staged branch and a full 546-spec conformance run.
- Do not delete generator scripts after a single wave if they are parameterized by wave number; they can be copied and updated.
- Do not claim identity cancellation at odd depths without first proving the residual equals the intended right-hand side.

## 2026-07-01 — Wave Loop 365 completion

### What worked
- Reused `scripts/gen_w365.py` and `scripts/gen_w365_lean.py` to append W365 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **3.8 s**.
- `ternaryMacAccumulateFortyOnePlusGeneric` pushed the accumulation boundary to **41 variables**, still in the linear `simp+omega` regime.
- `ternaryMacOctodecupleCancellationGeneric` (depth-18) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- Created `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`, giving every remaining #1245 defect an exact reproduction command and a tentative root-cause note.

### What changed behavior
- Generic ∀ count reached **204** (196 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **99 waves** (twenty-fifth consecutive zero-failure wave).
- IGLA totals: **7,618 tests**, **2,880 invariants**.
- The `dlc10` cable/board were still not detected; the failure is documented in `docs/reports/FPGA_EVIDENCE_W365.md`.

### Patterns to reuse
- For IGLA seal regeneration, map seal file names (hyphenated) to spec file names (underscore) when scripting; `t27c seal --save` normalizes the output file name.
- When a compiler fix is risky, ship a reproduction/roadmap document in the same wave; do not let the inability to fix silently erase the finding.
- Keep even-depth cancellation theorems for identity collapse; use odd-depth theorems only when the residual is explicitly verified.

### Anti-patterns to avoid
- Do not attempt to fix `is_top_level_start()` by adding `KwConst`/`KwVar` without tracking nested-block context; it breaks error recovery inside `test`/`invariant`/`bench` blocks.
- Do not leave `gen-verilog` defects without concrete repro commands; future waves will forget the exact failure mode.
- Do not claim "silicon verified" without `dlc10 idcode` success and a loaded bitstream observation.

## 2026-07-01 — Wave Loop 366 completion

### What worked
- Reused `scripts/gen_w366.py` and `scripts/gen_w366_lean.py` to append W366 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **4.1 s**.
- `ternaryMacAccumulateFortyTwoPlusGeneric` pushed the accumulation boundary to **42 variables**, still in the linear `simp+omega` regime.
- `ternaryMacNovemdecupleCancellationGeneric` (depth-19) correctly collapsed to residual `mac(x, a, .plus)`; the Lean statement matched the odd-depth residual exactly.
- Regenerated all 27 IGLA seals with the hyphen-to-underscore mapping; no manual seal edits were needed.

### What changed behavior
- Generic ∀ count reached **208** (200 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **100 waves** (twenty-sixth consecutive zero-failure wave).
- IGLA totals: **7,880 tests**, **2,950 invariants**.
- The `dlc10` cable/board were still not detected; the failure is documented in `docs/reports/FPGA_EVIDENCE_W366.md`.
- The `gen-verilog` backend remained unchanged; #1245 defects are still reproducible and documented.

### Patterns to reuse
- For 42-variable accumulations, the `simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode] <;> try omega` pattern remains sufficient.
- For odd-depth cancellation theorems, keep the residual explicit in both the Lean theorem name and statement to avoid identity/residual confusion.
- Re-run the full 546-spec conformance suite immediately after seal regeneration; seal mismatches are the only expected failure mode after a wave block append.

### Anti-patterns to avoid
- Do not land a broad `gen-verilog` fix in the same wave as a formal milestone unless it has a narrow, regression-free path; ship the reproduction document instead.
- Do not report the previous wave's generic ∀ count from memory when the Lean file can be grepped directly; exact counts prevent inflated or deflated claims.
- Do not skip `dlc10 idcode` just because earlier waves failed; retry each wave to keep the evidence trail current.

## 2026-07-01 — Wave Loop 367 completion

### What worked
- Reused `scripts/gen_w367.py` and `scripts/gen_w367_lean.py` to append W367 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **546/546 PASS** and `lake build Trinity.TernaryInference` succeeded in **4.4 s**.
- `ternaryMacAccumulateFortyThreePlusGeneric` pushed the accumulation boundary to **43 variables**, still in the linear `simp+omega` regime.
- `ternaryMacVigintupleCancellationGeneric` (depth-20) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- Landed a safe `gen-verilog` sub-fix: positive hex literals in scalar `const` declarations are now padded to the declared type width (e.g. `u16 = 0x1` emits `16'h1`). The fix passed the full 546-spec conformance suite without requiring seal regeneration.

### What changed behavior
- Generic ∀ count reached **212** (204 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **101 waves** (twenty-seventh consecutive zero-failure wave).
- IGLA totals: **7,934 tests**, **2,977 invariants**.
- The `dlc10` cable/board were still not detected; the failure is documented in `docs/reports/FPGA_EVIDENCE_W367.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: defect 2 (`0x` width) is fixed for scalar consts; defects 1/3/4/5 remain.

### Patterns to reuse
- For safe compiler sub-fixes, prefer narrow literal-emission changes over parser rewrites; they are the only kind that can land without mass seal regeneration.
- When a `gen-verilog` fix changes no currently-emitting output, the full conformance suite will stay green without regenerating all seals — but verify this explicitly before claiming the fix is regression-free.
- Keep the 4-theorem cadence: accumulation probe, minus-lattice parity, cancellation depth, zero-weight closure dimension.

### Anti-patterns to avoid
- Do not try to fix `gen-verilog` defect 1 (only first const emits) with a one-line parser change; it requires nested-block context tracking to avoid breaking error recovery.
- Do not omit a scratch-spec test for a compiler fix just because the full suite is green; the suite may not exercise the changed code path.
- Do not let a hardware blocker delay the formal + compiler sub-fix cadence; ship the deliverables and document the blocker.

## 2026-07-01 — Wave Loop 368 completion

### What worked
- Reused the generator pattern (`scripts/gen_w368.py` and `scripts/gen_w368_lean.py`) to append W368 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **547/547 PASS** and `lake build Trinity.TernaryInference` succeeded in **4.5 s**.
- `ternaryMacAccumulateFortyFourPlusGeneric` pushed the accumulation boundary to **44 variables**; build time stayed flat, confirming `simp+omega` still scales linearly.
- `ternaryMacVigintiunupleCancellationGeneric` (depth-21) correctly collapsed to residual `mac(x, a, .plus)`, continuing the odd-depth residual pattern.
- Corrected the `zero_weight_closure` helper: it now counts the plus-weight activation (`total = before + 1 + after`), so `ternaryMacZeroWeightUndecupleClosureGeneric` truly has 10 zero-weight MACs around 1 plus-weight MAC (11 variables).
- Landed a second safe `gen-verilog` sub-fix: positive hex literals are now padded to the declared width in scalar `const`, `var`, `let` (StmtLocal), and `return` contexts. A scratch spec `specs/scratch/w368_hex_width.t27` and `yosys read_verilog` verify the emitted RTL.
- Regenerated all affected seals (27 IGLA + 4 non-IGLA + 1 scratch) and reached 547/547 PASS.

### What changed behavior
- Generic ∀ count reached **216** (208 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **102 waves** (twenty-eighth consecutive zero-failure wave).
- IGLA totals: **7,780 tests**, **2,991 invariants** (direct keyword counts across the 27 core specs).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W368.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: defect 2 (`0x` width) now covers const/var/let/return on `trinity-rust-rings`; defects 1/3/4/5 remain. The full #1245 fix set already exists on `master` (commit `701d79b3b`) but was not merged into the wave-loop branch due to history divergence.

### Patterns to reuse
- When extending a literal-emission fix to new contexts, add the target-type context to the codegen state (e.g., `current_fn_return_type`) rather than changing the global expression emitter signature.
- After any `gen-verilog` change, run `t27c seal --save` for every spec whose `gen_hash_verilog` mismatches; the suite will name them explicitly.
- For zero-weight closure theorems, always verify the generated Lean expression by inspecting the plus-weight index; the helper's `total` must include the plus activation or the advertised depth is off by one.

### Anti-patterns to avoid
- Do not merge `master` into a long-lived wave-loop branch just to grab a backend fix unless you have bandwidth to resolve the diverged history and reseal everything.
- Do not leave scratch regression specs unsealed; either seal them or remove them before the final conformance run.
- Do not skip `dlc10 idcode` even when failure is expected; the evidence document needs the exact stderr each wave.

## 2026-07-02 — Wave Loop 369 completion

### What worked
- Reused `scripts/gen_w369.py` and `scripts/gen_w369_lean.py` to append W369 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **548/548 PASS** and `lake build Trinity.TernaryInference` succeeded in **~5.0 s**.
- `ternaryMacAccumulateFortyFivePlusGeneric` pushed the accumulation boundary to **45 variables**; `simp+omega` remains in the linear regime.
- `ternaryMacDuovigintupleCancellationGeneric` (depth-22) collapsed cleanly to identity `= x`, confirming even-depth cancellation remains the safe default.
- `ternaryMacZeroWeightDuodecupleClosureGeneric` uses 6 zero-weight MACs before and 6 zero-weight MACs after a plus-weight MAC (12 + 1 = 13 variables); the corrected `zero_weight_closure` helper from W368 was preserved.
- Landed the third consecutive safe `gen-verilog` sub-fix: positive binary literals (`0b...`) are now padded to the declared width in scalar `const`, `var`, `let` (StmtLocal), and `return` contexts, mirroring the W368 `0x` fix. A scratch spec `specs/scratch/w369_bin_width.t27` and `yosys read_verilog` verify the emitted RTL.

### What changed behavior
- Generic ∀ count reached **220** (212 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **103 waves** (twenty-ninth consecutive zero-failure wave).
- Conformance suite now evaluates **548 specs** (546 canonical IGLA + 1 non-IGLA + 1 scratch regression spec).
- The `dlc10` cable/board were still not detected; the failure is documented in `docs/reports/FPGA_EVIDENCE_W369.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: defects 2/2b (`0x` and `0b` scalar width padding) are fixed; defects 1/3/4/5 remain.

### Patterns to reuse
- For literal-width guards, use the same shape for `0x` and `0b` with only the bit-scaling changed: `hex.len() * 4` vs `bin.len()`.
- Add scratch regression specs for every `gen-verilog` sub-fix and run `yosys read_verilog` before regenerating all seals; this catches regressions without waiting for the full suite.
- For W370, the recommended cooperation variant is B (formal + board retry + one safe backend sub-fix or CI smoke gate).

### Anti-patterns to avoid
- Do not add a scratch spec without either sealing it or removing it before the final suite run; an unsealed spec will produce a suite failure.
- Do not claim the binary-width fix covers non-scalar contexts (arrays, struct fields) until a dedicated reproduction proves it.
- Do not merge the full `master` #1245 fix set into `trinity-rust-rings` during a wave unless the diverged history and seal set are reconciled first.

## 2026-07-02 — Wave Loop 370 completion

### What worked
- Reused `scripts/gen_w370.py` and `scripts/gen_w370_lean.py` to append W370 blocks and 4 new generic ∀ theorems; `t27c suite --repo-root /Users/playra/t27` returned **549/549 PASS** and `lake build Trinity.TernaryInference` succeeded in **4.8 s**.
- `ternaryMacAccumulateFortySixPlusGeneric` pushed the accumulation boundary to **46 variables**; `simp+omega` remains in the linear regime.
- `ternaryMacTresvigintupleCancellationGeneric` (depth-23) correctly collapsed to residual `mac(x, a, .plus)`, continuing the odd-depth residual pattern.
- `ternaryMacZeroWeightTredecupleClosureGeneric` uses 6 zero-weight MACs before and 7 zero-weight MACs after a plus-weight MAC (13 closure size, 14 variables).
- Fixed `gen-verilog` defect 1 (only first `const` emits) in `bootstrap/src/compiler.rs` by removing the early return in `parse_const_decl`. The fix required **mass seal regeneration (~156 seals)** because many specs now emit more `const` declarations than before.
- Verified the B1 fix with scratch spec `specs/scratch/w370_const_order.t27` and `yosys read_verilog` before running the full suite.

### What changed behavior
- Generic ∀ count reached **224** (216 in `TernaryInference.lean` + 8 in `TernaryMac.lean`).
- The zero-IGLA-failure streak extended to **104 waves** (thirtieth consecutive zero-failure wave).
- IGLA totals: **12,696 tests**, **5,549 invariants** (full repo keyword counts; note that earlier waves reported IGLA-only subsets while W370 reports all specs).
- Conformance suite now evaluates **549 specs** (546 canonical IGLA + 2 non-IGLA + 1 scratch regression spec).
- The `dlc10` cable/board were still not detected; documented in `docs/reports/FPGA_EVIDENCE_W370.md`.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` updated: defect 1 (multiple `const` declarations) is fixed on `trinity-rust-rings`; defects 3/4/5 remain.

### Patterns to reuse
- For parser fixes that change how many top-level declarations are parsed, expect mass seal regeneration; script `t27c seal --save` over every mismatched seal and re-run the full suite before claiming green.
- When generating Lean binder lists beyond 26 variables, skip Lean keywords (`at`, `by`, `do`, `if`, `in`, `or`, `to`) so the 46th+ variables do not produce `unexpected token` errors.
- For W370-level cooperation variants, keep Variant B as the recommended path: formal + one safe backend sub-fix + board retry.

### Anti-patterns to avoid
- Do not try to fix defect 1 by adding `KwConst` to `is_top_level_start()`; that breaks error recovery inside `test`/`invariant`/`bench` blocks. The correct fix is inside `parse_const_decl` itself.
- Do not commit a parser fix without a dedicated scratch spec that exercises the previously broken code path; the full suite may not contain a multi-const module.
- Do not trust repository-wide test/invariant counts from prior-wave memory; run `t27c stats` to get current totals.
