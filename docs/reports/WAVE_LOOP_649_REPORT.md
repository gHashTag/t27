# Wave Loop 649 — the port was the error

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_648_REPORT.md`](WAVE_LOOP_648_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
24 corpus specs:  'clk' has already been declared in this scope

  module APB_Bridge_Testbench (
      input  wire clk,     <- line 11
  );
      reg clk;             <- line 24

The obvious fix -- drop the reg -- would have made a DRIVEN signal
undrivable, and the testbench would still compile and run with a clock
that never toggles.

T62  a duplicate-definition error names the SECOND declaration, which
     is not evidence about which one is wrong.

corpus [BENCH] specs compiling:  3 -> 19      printing:  3 -> 15
the output stratum's reach:      2% -> 13%    from one guard
```

---

## 1. Reading the emitter before editing

W648's recommendation carried a stated risk: *"the testbench needs to drive
`clk`, and a port cannot be driven by an `initial` block — determine which
before editing."* That turned out to be the whole wave.

**The spec's intent is unambiguous:**

```t27
var clk : bool = false;
…
clk = false;
clk = true;
```

**It declares the signal and drives it.** So the `reg` is right, and dropping it
— the reading the error invites — would convert a driven signal into an
undrivable input. **The artefact would still compile. The simulation would still
run. The clock would never toggle**, and no gate in this repository can see
that.

**The port was the error.** `gen_verilog` emitted a boilerplate
`(clk, rst_n, en)` header **unconditionally for every module**, including the
144 specs whose purpose is to declare and drive exactly those signals.

> **T62 — a duplicate-definition diagnostic names the second declaration,
> because that is where the checker noticed. That is a report about *position*,
> not about *authorship*.** Deciding which is wrong requires the intent, which
> lives in the source the generator consumed — not in either emitted
> declaration.

The fix is a guard: skip a boilerplate port the spec itself declares.

---

## 2. T59's back-loading, tested where it applies

T61 corrected T59 by measuring that scratch repairs do not move the corpus
figure. **W649 is the same prediction tested on the population it is about:**

| | before | after |
|---|---:|---:|
| corpus specs emitting `[BENCH]` | 144 | 144 |
| of those, compiling under `iverilog` | **3** | **19** |
| of those, printing a `[BENCH]` line | **3** | **15** |

**The output stratum's reach: 2% → 13%. 6.3× from a single guard.**

That is what "back-loaded value" means concretely — the dynamic stratum's
coverage is a function of the build rate, and one repair to the build rate moved
it more than any amount of work on the stratum itself would have.

---

## 3. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| `apb_bridge_tb` under `iverilog` | **compiles** (warnings only, no errors) |
| corpus `[BENCH]` compiling | **3 → 19** |
| corpus `[BENCH]` printing | **3 → 15** |
| ratchet | **CLEAN**, 332/332, rc 0, 949 s |

---

## 4. What was NOT done

- **The remaining ~122 corpus `[BENCH]` specs still do not compile** — 62 of the
  original 141 were unread `syntax error`s and remain unread.
- **Four iverilog rejections remain** in the scratch set: 2 × missing function,
  1 × empty-identifier declaration, 1 × array-return cast.
- **Four gates remain unaudited** for their totality claims.
- **The falsification condition is untested**: a module that legitimately needs
  the boilerplate `clk` port *and* declares `var clk`. None was found; none was
  searched for exhaustively.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 5. Three ways to continue (pick one for W650)

### Option 1 — **Read the 62 corpus `syntax error`s**

Now the largest remaining blocker of the corpus build rate, and therefore of the
output stratum. `syntax error` is iverilog's least informative message, which is
exactly why T37 says to read the offending line rather than group the message.

- **Cost:** low to characterise; unknown to fix.
- **Pays off in:** the build rate is now a demonstrated multiplier — W649 turned
  one guard into 6.3× stratum coverage. The 62 are the next such lever, and
  nothing can be planned about them until they are grouped by construct.
- **Risk:** T37 measured 147 source-shape classes over 178 parse failures, so
  expect a long tail rather than one cause. Report the histogram honestly even
  if it is flat.
- **Confirming measurement:** a class histogram over the 62 grouped by rejected
  source line, summing to 62, with the top classes' exemplars quoted.

### Option 2 — **Wire the output stratum into the phase list now that it reaches 13%**

`cmd_simulated_output_wellformed` exists and is unwired: at 2% coverage it
bought little. At 13% and rising it is worth its runtime, and it is the only
stratum that can catch a wrong *value*.

- **Cost:** low; the function exists. Scope it to the specs that compile.
- **Pays off in:** the first automatic check in this repository on what the
  generated hardware actually *prints*.
- **Risk:** simulation is slow and can hang — needs a per-spec timeout, which
  the measurement script already needed (two runs exceeded 600 s without one).
- **Confirming measurement:** re-introduce `%%0d`, confirm the phase fails; and
  the phase's wall-time contribution stated explicitly.

### Option 3 — **Finish the gate audit: the remaining four**

`no-vacuous-invariant` (Zig only), `no-vacuous-verilog-test`,
`backends-declare-omissions` (3 of 5 backends), and the ratchet. T56 found the
first audited gate understating by 22%.

- **Cost:** medium; four enumerations.
- **Pays off in:** every figure those gates produce is a lower bound of unknown
  tightness, and this document quotes several as measurements.
- **Risk:** each widening reddens a gate and may need a bless.
- **Confirming measurement:** per gate, the T55 table.

**Recommendation: Option 1.** W649 established the build rate as a measured
multiplier on gate coverage, and the 62 are the largest remaining term in it.
Option 2 is cheap but its value grows with Option 1; Option 3 improves the
accuracy of numbers rather than the state of the repository.

---

## Appendix — reproduction

```bash
./target/release/t27c gen-verilog-for-simulation specs/fpga/testbench/apb_bridge_tb.t27 \
  | head -16
```

For the stratum: count corpus specs whose generated Verilog contains `[BENCH]`,
then how many `iverilog -g2012` accepts, then how many print under `vvp`.
**Use a per-spec timeout** — two measurement runs exceeded 600 s without one.

**φ² + φ⁻² = 3 | TRINITY**
