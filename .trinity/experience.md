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
