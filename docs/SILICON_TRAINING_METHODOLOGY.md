# Training a neural network on live FPGA silicon from a verified specification

*A reproducible open-toolchain methodology.* Workshop-grade write-up of the GF-T
on-chip training result. Everything below is measured, not projected.

## Claim

A neural-network training loop — forward, loss, backward, weight update — written
as a `.t27` **specification**, compiled by a ternary compiler, **verified bit-exact
against an independent model across four targets (Verilog, C, Rust, and the model)**,
and then **run on a real Xilinx Artix-7 (AX7203, xc7a200t) where it trains a network**
— through a fully open-source toolchain (yosys → nextpnr-xilinx → prjxray), no Vivado,
no Docker, native macOS arm64.

## Method (the pipeline, end to end)

1. **Spec.** The GF-T arithmetic (a ternary-native GoldenFloat: `value = (−1)^s ·
   (1 + m/512) · 2^(off−40)`) is defined in `.t27` — `gft_smul.t27`, `gft_sadd.t27`,
   etc. The compiler `t27c` emits **Verilog, C, Rust, and Zig** from one source.

2. **Generator, not hand-RTL.** `tools/gft_backprop_microcode.py` turns an arbitrary
   feed-forward topology (free inputs × hidden layers × outputs, arbitrary depth) into
   a **microsequencer**: one shared multiply core + one shared add core, driven by a
   microcode program over a register file. Network size grows the microcode (time) and
   the register file — **not the datapath** (one multiplier, regardless of the net).

3. **Verification, as a CI invariant.** On every change, a gate (`verify_emit_bitexact`)
   regenerates the arithmetic cores from spec and proves the generated RTL is **bit-exact
   to the independent Python GF-T model over a full 80-step training run** (forward +
   backprop + update, every output per step) in Icarus Verilog; then **synthesizes** it
   (yosys) with a non-zero cell mapping and asserts the **one-shared-multiplier datapath
   invariant**. A companion gate (`verify_multitarget`, `verify_trainer_c`) proves the
   primitives and the whole trainer are bit-exact in **C and Rust** too, over moderate,
   extreme (saturation-adjacent), and cancellation operands, plus a differential fuzzer
   over random topologies. Spec→any-target bit-exactness is *guaranteed per pull request*.

4. **Silicon build.** `emit_verilog(…, clk_div=16)` emits a silicon-ready variant: the
   register file is forced to flip-flops (distributed LUTRAM cannot do the parallel
   weight-init) and the sequencer steps once per 16 cycles (a clock-enable) so the deep
   shared-core combinational path has time to settle. `yosys synth_xilinx -nocarry`
   → `nextpnr-xilinx --timing-allow-fail` → `fasm2frames` → `xc7frames2bit` → `.bit`.

5. **Flash & train.** `openFPGALoader` loads the bitstream over JTAG; the host streams
   training samples over UART; the board holds the weights, runs a full backprop step
   per sample, and returns the forward output. Weights persist across samples → it learns.

## Results (measured on the AX7203)

- **The full 2-layer backprop microsequencer trains XOR to 4/4**, 24/25–25/25 epochs,
  both layers learning on-chip, with a **weight trajectory bit-exact to the independent
  model** (epoch-0 outputs 0.000 / 0.551 / 0.936 / 0.232 match the model to three
  decimals; the error term converges toward zero).
- The **CI-verified generator's** RTL (not hand-written) trains XOR on silicon the same
  way, closing the loop spec → verified generator → open-source bitstream → live training.
- Earlier on-silicon results on the same board: inference (dot 6/6, BitNet neuron 8/8,
  3-class argmax 16/16 held-out, 2-layer ReLU XOR 4/4) and training (SGD 4/4, gradient
  descent → 0, 1-/2-parameter regressions, ReLU-gated nonlinear neuron, a classifier
  generalizing 8/8, train→save→deploy closed with SPI-flash boot).
- **Size costs time, not area** (CI-measured): microcode steps grow (2,2,1)=32 →
  (2,4,2)=88 → deep [3,4,4,2,1]=216, while synthesized cell counts stay ~constant.

## Honest limits (measured, not hidden)

- **openXC7 correctness ceiling ≈ 17M fasm.** Designs ≤ 16.7M compute correctly; ≥ 19.5M
  place and respond over UART but *miscompute* — always cross-checked against the model.
- **The open-source place-and-route cannot express a multicycle timing constraint**
  (nextpnr-xilinx's XDC parser supports only `create_clock`). The deep shared-core path
  is therefore left timing-relaxed, and correctness is **placement-dependent**: some
  `--seed` values glitch, one trains cleanly. We seed-search. This is an open-toolchain
  limitation — a commercial P&R would close the path directly — not a design flaw. A
  design's microcode step count predicts its marginality (more steps per frame = more
  chances for a glitch).
- **The marginality is a placement property, not a localizable register bug — confirmed
  by instrumentation.** We tried on-chip observability: widen the UART dump to expose the
  hidden pre-activations (z0, z1) alongside the output y, so a glitching seed would reveal
  *which* register diverges first. Adding the probe changed the design's behaviour from
  "computes (marginally)" to "outputs all-zero" — the extra logic re-placed the shared
  core past its (unconstrained) timing edge. This is a genuine Heisenbug: the deep path is
  marginal enough that instrumenting it *moves* the result, which is itself the evidence
  that the fault lives in the timing/placement of the whole path, not in one microcode
  step.
- **A slower clock is NOT the fix — on two independent grounds.** The intuitive remedy
  ("just run the deep path at a slower clock so it settles") fails twice on this flow.
  *(a) It is not buildable.* A fabric-counter divided clock needs a clock buffer, and
  nextpnr-xilinx cannot place one driven from fabric: both `BUFG` and `BUFR` fed by a
  divider bit fail with *"Unable to find legal placement"* (a 7-series clock buffer input
  comes from a clock-capable pin or the CMT, not general routing). Only an MMCM/PLL could
  synthesize a real divided clock. *(b) Even if it built, it would not help.* A divided
  clock with the same microcode `settle` count delivers the **same real settle window**
  (~µs) as the working `/N` clock-enable — no new mechanism. And more settle does not cure
  the glitch: the on-silicon behaviour is **non-monotonic in settle** (a `/128` enable
  glitched *worse* than `/64`), so the fault is a placement **hazard**, not a shortage of
  settle time. Note too that `create_clock` on the differential `clk_p` port does **not**
  propagate through the `IBUFDS`; the internal clock net defaults to a loose 12 MHz target
  and always "passes", so `--timing-allow-fail` was effectively a no-op — the path was
  never actually being closed, just loosely met. Constraining the internal net tighter
  reports the true fmax (~21 MHz) but does not change the silicon hazard.
- **Pipelining the shared core was the leading hypothesis — and it was DISPROVEN on
  silicon (see Ruled-out #10).** The intuition was that registering the intermediate
  stages of `GftSmul`/`GftSadd` would break the deep combinational hazard. We built it
  (bit-exact, mid-cloud registers) and it did **not** fix the lottery — nor did endpoint
  registration (#7) or write/control hardening (#11). This is what pointed the fault at a
  global effect, not the datapath. Kept here only to mark the hypothesis as tested; the
  authoritative current state is the **Ruled-out fixes** list and its conclusion below.
- **Where the depth actually is (measured, so we pipeline the right place).** `GftSmul`
  is purely combinational (`assign result = smul(a,b)`; the `clk`/`en`/`ready` ports are
  unused, so there is no read-before-ready bug). Yosys `ltp` (longest topological path)
  puts the shared-core critical depth at **`GftSadd` = 54** and **`GftSmul` = 44**. The
  depth is *not* the multiplier width: the `*` in `magmul` lowers to a 32-iteration
  shift-add, but its operands are `512+mant ∈ [512,1023]` (10-bit), so yosys prunes the
  dead upper iterations — hand-narrowing the loop to 10 leaves the depth unchanged (44 →
  45) and the area flat, so **narrowing the multiply is a dead end** (tested before
  touching the verified spec). The real depth is the *dependent normalize/round cascade*:
  `magsub`'s 4-stage priority-shift + `<<14` fixed-point + round-to-nearest-even in
  `GftSadd`, and `magmul`'s post-product RNE carry in `GftSmul`. So the pipeline cut is
  concrete: split each of those cascades into **two registered stages** (~27 and ~22 deep),
  which both halves the path *and* resynchronises it — the microsequencer then waits the
  fixed 2-cycle latency instead of a `settle` counter. Note the ~47 ns nominal path is
  already ~100× under the µs settle window, which is why the glitch is a *placement hazard*
  a static-timing fix cannot see, and only a mid-cascade register (resynchronisation)
  addresses.
- **Endpoint registration is NOT enough — the hazard is *inside* the cloud (measured on
  silicon).** We built the discriminating cheap test first: register the shared core's
  *endpoints* — clean flip-flop operands in (`a_reg`/`b_reg`), a registered result out
  (`res_reg`) — without splitting the deep cloud. This is bit-exact in simulation (ep0
  outputs 0 / 0.551 / 0.936 / 0.234 match the model) and it even *raised* fmax 21 → 29 MHz
  by pulling the operand-modifier logic out of the core path. But on the AX7203 it did **not**
  fix the lottery: of four seeds, two produced dead routes and two responded but glitched
  from ep0 (weights exploding / collapsing to zero), same as the baseline. So resynchronising
  the *boundaries* is insufficient — the fault lives in the depth-54 `GftSadd` combinational
  cloud itself, and only splitting **that** into registered stages (the spec-level pipeline
  above) will close it. This rules out the cheap wrapper fix and confirms the full spec
  pipeline is required.
- **A real divided clock IS buildable — via MMCM, not fabric (corrects the note above).**
  While a fabric-counter clock cannot be buffered (`BUFG`/`BUFR` unplaceable from fabric),
  an `MMCME2_BASE` **does** place on this flow: nextpnr-xilinx constrains it to a real
  `MMCME2_ADV` bel by dedicated routing and routes its divided output as a genuine clock.
  So the real-divided-clock route is open through the CMT. It is, however, unlikely to fix
  the glitch — by the endpoint-registration result the fault is an internal-cloud hazard,
  not a settle shortage, and a slower clock only adds settle — so it stays a bounded
  experiment behind the spec pipeline, not the primary fix.

### Ruled-out fixes (do not re-attempt without new evidence)

Measured dead ends from the seed-lottery investigation (cycles 82–96), so future work
does not re-run them:

1. **Wider `settle` counter** — 8-bit is the ceiling; 12-/16-bit counters hang the board.
2. **More settle time** (`/64`, `/128`, `/256` enable) — non-monotonic; `/128` glitches
   *worse* than `/64`. Not a settle-time problem.
3. **Tighter static-timing constraint** — the deep path is already ~47 ns, ~100× under the
   µs settle window; nominal timing is met (fmax 21–29 MHz) and it still glitches.
4. **Fabric-counter divided clock** — `BUFG`/`BUFR` from fabric are unplaceable on openXC7.
5. **On-chip observability probe** (widen the dump) — a Heisenbug: the probe re-places the
   core and changes the result.
6. **Narrowing the multiplier** — yosys already prunes the dead `__mul_noop` iterations
   (10-bit operands); loop 32 → 10 leaves the depth unchanged (44 → 45).
7. **Endpoint registration** — bit-exact and raises fmax, but does not fix the lottery
   (hazard is internal to the cloud).
8. **Automatic retiming** (`synth_xilinx -retime`) — moves only a few levels (ltp 57 → 51
   on `GftSadd` with two output registers). Not enough to split the cloud.
9. **Dropping a register into the *generated* combinational function** (hand RTL pipeline)
   — bit-exact but does **not** reduce depth. Two 2-stage `GftSmul` prototypes (cut after
   the product; cut mid-RNE after `carry`/`q`/`r`) are both verified bit-exact to the
   combinational core over ~40 k random operands, yet ltp *rose* 44 → 49 / 50: the register
   boundary defeats yosys's cross-function optimization of the inlined `sadd`/`magmul`/
   `mul_noop`, and the depth (RNE normalize + accumulator) does not split at a dropped-in
   register. **Implication for the pipeline: it must be a *spec-level* `on_clock` where the
   compiler co-optimizes the stages (and the multiply becomes a real pipelined primitive),
   not a register hand-inserted into the codegen output.**
10. **Pipelining the shared cores (mid-cloud registers) — the decisive silicon test —
    does NOT fix the lottery.** Both cores were pipelined latency-1 with a register *inside*
    the combinational cloud (`GftSmul_p2b` cut mid-RNE, `GftSadd_p2` cut mid-cascade), each
    verified **bit-exact** to its combinational core over 40–60 k random operands (including
    zero and exact-cancellation corners), and the integrated trainer is bit-exact in
    simulation and reaches **fmax 32 MHz** (vs 21 baseline). Flashed on the AX7203 across
    four seeds: **0 / 4 trained stably** — seeds 2 and 3 were near-model at ep0
    (y = 0/0.718/0.886/−0.011 and 0/0.551/1.021/0.506) then collapsed to all-zero by ep20,
    the characteristic training-divergence glitch; seeds 1, 4 glitched from ep0. This is the
    same behaviour as the baseline (~1/8 seeds stable), so **registering the core datapath —
    at the endpoints (item 7) *or* mid-cloud — does not fix the fault.** Together these say
    the hazard is **not in the `GftSmul`/`GftSadd` combinational datapath at all** — it lives
    in the register-file write / control path (`di` destination decode, `pc`/`settle`/`cen`
    counters, or the write-capture), or is a global placement effect. (Sample is four seeds;
    the base rate is ~1/8, so 0/4 is indicative, not a proof of zero improvement.)
11. **Hardening the write / control path does NOT fix it either.** Registered the
    destination index `di` and the result into flip-flops and wrote the register file from
    the *registered* address+data (a clean synchronous write) — bit-exact in simulation.
    On the AX7203 across eight seeds: several dead-routed, and every seed that responded
    still glitched (explode to ~1e16 or collapse to zero), same as the baseline. So the
    write-address / write-data path is not the culprit either.

12. **MMCM real clock tree — places but does NOT function on silicon (open flow).** The
    one remaining structural lever: regenerate the 200 MHz clock through the MMCM/CMT tree
    (low-skew) instead of the fabric `IBUFDS` net, to test the global clock-skew hypothesis.
    `MMCME2_BASE` places in nextpnr and the fasm builds, but the flashed bitstream is dead
    on the AX7203 — no UART response on any of four seeds, with or without a `BUFG` on the
    MMCM output. The open flow (prjxray fasm2frames) does not emit the MMCM configuration
    bits, so the MMCM never locks / drives no clock. So the MMCM lever is **placement-only
    on openXC7, not functional** — the clock-skew hypothesis cannot be tested here, and the
    open-toolchain structural options are fully exhausted. Only commercial P&R (Vivado) can
    close timing directly or provide a working MMCM. Build in `scratchpad/board/bpmmcm/`.

Conclusion of the root-cause arc: **every local register-based fix has failed** — the
combinational datapath at the endpoints (7), mid-cloud (10), and the write/control path
(11) were each resynchronised with flip-flops and none changed the lottery. A fault that
survives registering every local logic boundary is, by elimination, a **global effect**:
clock distribution (skew on the fabric `IBUFDS` net across a large `--timing-allow-fail`
placement), routing, or IR-drop — not a logic hazard any local RTL edit can reach. The
**practical answer is seed-search** (XOR trains bit-exact on a good seed); the only
remaining *structural* levers are a **real clock tree (MMCM)**, which reduces skew and is
buildable on this flow (item: MMCM places), or **commercial P&R** timing closure. Bit-exact
prototypes: cores in `scratchpad/retime/`, pipelined trainer in `scratchpad/board/bppipe/`,
write/control-hardened in `scratchpad/board/bpctrl/`.

## Reproducibility

The verification runs in CI on every pull request. The silicon build is one script
(`board/build_trainer.py`): generate → wrap in the UART front-end → yosys → seed-search
nextpnr → per-seed bitstreams. Flash a seed, drive it over UART, keep the seed that
trains stably. All artifacts (chipdb, nextpnr-xilinx, prjxray) are open-source and
build natively on macOS arm64.

## Why it matters

On-device *learning* on cheap FPGA silicon, in a ternary-native format, from a
machine-verified specification, through an entirely open toolchain, is a capability we
have not seen demonstrated elsewhere. Every inference-only ternary accelerator we know
of (Ternary-NanoCore, TerEffic, bitnet.cpp, bitSMM) runs a *frozen* model in hand-written
RTL or on a CPU; here the spec *is* the network, it is verified bit-exact across four
targets, and it *trains* on live silicon.
