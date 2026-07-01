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
