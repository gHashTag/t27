# Wave Loop 654 — the harness learned to fail, and then found things

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_653_REPORT.md`](WAVE_LOOP_653_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T78  root cause of T76 fixed.  For the first time in this project's
     history, a CLI-generated Verilog test distinguishes a true
     assertion from a false one.

T79  3B2T is the UNIQUE non-degenerate ternary line code with exactly
     one spare codeword -- and the uniqueness is Mihailescu's theorem.
T80  the spare is also an extremal-disparity word, so the worst DC
     excursion sits outside the data stream.
T81  800 of 849 generated modules (94.2%) have NO data ports.
     The encoder, once given one, is 7 LUT6 on Artix-7.
T83  two runaway processes had burned 33 CPU-hours.  One was mine.
T84  first corpus-wide test measurement: 476 PASSED, 34 FAILED,
     46 NOT CHECKED -- where it would have said 556 PASSED, 0 FAILED.
T85  f32 lowered to an UNSIGNED vector.  Sign fixed; fraction cannot be.
```

---

## 1. T78 — the harness can now fail

```rust
if self.emit_test_assertions {
    StmtAssign => { self.materialize_call_array_tmps_in_expr(node);   // present
                    self.gen_verilog_stmt(node); }
} else {
    StmtAssign => { self.gen_verilog_stmt(node); }                    // ABSENT
}
```

`VerilogCodegen::new()` sets the flag **false** and the CLI calls `new()`, so
`given v = two()` emitted a read of a temporary nothing ever wrote. Every
assertion compared against `x`, and `if (!(x))` is FALSE in Verilog.

> **T78.** A flag named for one concern silently gated a second. **T75 was this
> same defect one arm earlier** — the same flag gating the *declarations* — and
> fixing T75 exposed T78 rather than resolving it. When a boolean gates two
> branches of a `match`, **every such pair is an unaudited difference table**.

The control that had failed three times:

```
[TEST] this_one_is_true                : PASSED
[TEST] this_one_is_deliberately_false  : FAILED
```

---

## 2. T84 — what the working harness immediately found

All 1,065 specs generated, compiled, run:

```
gen_fail       216      no Verilog emitted
iv_error       617      emitted Verilog does NOT compile  (72.7% of emitters)
compiles       231
run_timeout      4      simulations that do not terminate

  PASSED  476      FAILED  34      NOT CHECKED  46
```

**The same command would have said `556 PASSED, 0 FAILED` before this session.**

Nine specs carry a failure. The smallest is the interesting one:
`specs/scratch/w375_early_return.t27` — a **W375 regression test** — passes in
Zig 2/2 and fails in Verilog. Measured with a five-line probe, not inferred:

```
f(-1.0)      = 4294967295      the real -1.0 narrowed to [31:0]
(-1.0 < 0.0) = 1               the real comparison is correct
f(-1.0)<0.0  = 0               after narrowing, the sign is GONE
```

> **T84.** `f32` lowers to an *unsigned* vector while float literals stay
> Verilog reals, so every negative value becomes ≈4.29 × 10⁹ at the function
> boundary and every comparison against zero inverts. **A test can be correct
> about its subject and wrong about its substrate**, and only an oracle that can
> fail distinguishes the two.

---

## 3. T85 — the sign was recoverable, the fraction is not

Cause, two lines: `type_is_signed` listed only `i8|i16|i32|i64`, and `f64` fell
through `type_to_width`'s default — **silently narrowing to half its width**, a
second defect in the same pair, found only because the first was being fixed.

| | before | after |
|---|---:|---:|
| `f(-1.0)` | 4294967295 | **−1** |
| `f(-1.0) < 0.0` | 0 | **1** |
| `f(0.5)` | 1 | **1** — unchanged |

> **T85.** Lowering a float to a signed integer vector fixes the *sign* class and
> **cannot** fix the *fraction* class, because the second is a representability
> failure rather than an encoding one. **A partial fix to a mixed failure class
> silently redefines what the remaining failures mean.**

**Blast radius, measured before choosing the design:** 194 specs mention
`f32`/`f64`; **17 compile** under iverilog (128 do not); of the 17 that run, 4
tests pass and 2 fail. Small enough to evaluate exhaustively — which is the
reason to measure first.

**Left open deliberately.** The remaining options are `real` (correct in
simulation, rejected by synthesis, truthful because `f32` was never
synthesizable) or a diagnostic that refuses `f32`. **What must not continue is
today's third option**: a signed vector that compiles, synthesizes, runs, and
computes the wrong value for every non-integral input. T52's shape at the level
of a type.

---

## 4. T79/T80 — why 3B2T, from number theory

> **T79.** `3^m − 2^n = 1` has exactly two solutions in positive integers:
> `(1,1)` and `(2,3)`. For `m,n > 1` this is **Mihailescu's theorem** (2002;
> Catalan's conjecture, 1844). The rest is forced.

`(1,1)` is degenerate — 63.09% efficiency. **So 3B2T is the only non-degenerate
ternary line code leaving exactly one spare codeword**, at 94.64% efficiency.
Verified by exhaustive integer search over `m ≤ 199`, `n ≤ 319`; the search checks
the statement, the proof is Mihailescu's.

Same-rate alternatives make the point: `(4,6)` and `(6,9)` hold the same
1.5 bit/symbol while the spare grows to **17** and **217**.

> **T79'.** A delimiter that is *unreachable* eliminates a failure class; one that
> is *improbable* bounds it. `bpsk.t27`'s Barker-13 gate is a likelihood
> argument with a data-dependent false-sync rate. **0.085 bit/symbol is the exact
> exchange rate**, and by T79 the cheapest that exists over a ternary alphabet.

> **T80.** The reserved word `(+1,+1)` is one of the two **extremal-disparity**
> words, so the largest DC excursion sits on a symbol whose frequency the
> protocol controls and never inside data — the opposite of a scrambler.
> Available only because the alphabet is balanced.

---

## 5. T81 — 94.2% of modules have no boundary

```
specs with a generated module:                849
  ONLY boilerplate (clk/rst_n/en/ready):      800   (94.2%)
  with REAL data ports:                        49   ( 5.8%)
```

The 49 are **all** `specs/ternary/gft_*`, and the difference is one naming
convention: a function called **`on_comb`**.

> **T81.** Expressibility and synthesisability are independent properties.
> Neither "170+ specs parse" nor "5/5 modules synthesize" measures this, because
> a module with no ports *does* synthesize — to nothing.

This is why every bitstream in the repository came from hand-written Verilog
under `fpga/`. Adding `on_comb` to `ternary_link.t27` gave it
`input [7:0] v` / `output [7:0] result`, and the clean datapath is:

| resource | count |
|---|---:|
| **LUT6** | **7** |
| IBUF | 11 |
| OBUF | 9 |

**Seven LUT6 for a complete 3B2T encoder** — the first silicon figure for a
three-valued object here. The encoder is not the cost of a ternary link; the
receiver's two comparators are.

---

## 6. T83 — 33 CPU-hours, and one was mine

```
PID 3592   01-03:19:09   ~85%   t27c parse .../specs/tri/agent/handoff.t27
PID 9297      05:47:40   ~88%   vvp .../scratchpad/s.out
```

Both terminated. **The `vvp` was mine** — an earlier sweep whose `vvp` call
carried no `timeout=` while the `gen-verilog` and `iverilog` calls in the same
loop did.

> **T83.** A timeout on *some* steps of a pipeline is not a timeout on the
> pipeline. The unbounded step hangs **after** the enclosing job reports
> completion, so the cost is invisible where incurred and surfaces as
> unexplained slowness hours later.

**The 27-hour parse could not be reproduced** — fresh runs of both binaries
complete in seconds, and 13 truncated prefixes all parse. **Recorded as unknown**
rather than given an invented mechanism.

**Consequence:** the 744 s and 923 s ratchet wall-clocks quoted in earlier
reports were measured while at least one runaway was running.

**Still running, not mine, not touched:** `trustd` at 73% CPU for 30 hours — a
macOS trust daemon. Flagged for the user; killing a system daemon is theirs to
decide.

---

## 7. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| false-test control | **true → PASSED, false → FAILED** |
| `ternary_link.t27`, Zig | **36/36** |
| `ternary_link.t27`, iverilog+vvp | **36 PASSED**, 0 FAILED, 0 NOT CHECKED |
| encoder synthesis | **7 LUT6** |
| ratchet after T78 | **CLEAN 326/326**, rc 0 |
| ratchet after T85 | **running at time of writing** |
| working tree | clean; seal matches |

---

## 8. What is NOT done

- **The `f32` fraction class is measured, not fixed** (§3).
- **617 specs emit non-compiling Verilog** — size known, contents not.
- **799 modules still have no data ports** — one `on_comb` each.
- **265 Icarus baselines** record a harness that could not fail; re-blessing them
  is now *more* urgent and still must not be bulk (T31).
- **No board-to-board link exists.** Three boards on one Mac's USB hubs is a star
  through a single host.
- **No web literature.** `WebFetch`/`WebSearch` returned a provider error
  (`glm-4.5-air`) on every attempt across the whole session. T79 is derived and
  computationally checked; Mihailescu's theorem is named as a known result and
  **no citation was fabricated**.

---

## 9. Three ways to continue (pick one for W655)

### Option 1 — **Decide the `f32` representation and implement it**

The blast radius is 17 specs. Emit `real` for `f32`/`f64` in the simulation
path, measure those 17, and keep or revert on the number.

- **Cost:** low-medium; bounded and reversible.
- **Pays off in:** removes the last member of T52's family found so far — a type
  that compiles, runs and lies.
- **Risk:** `real` cannot be bit-selected or concatenated; some of the 17 may use
  `f32` in packed contexts. **Measure that before committing, not after.**
- **Confirming measurement:** `w375_early_return_exp` passes in Verilog, and the
  other 16 do not regress.

### Option 2 — **Give the 799 port-less modules an `on_comb`**

T81's remedy, applied at scale. Most specs have an obvious candidate function.

- **Cost:** high if done by hand; low if the backend picks a default (e.g. the
  last `pub fn`) and the spec can override.
- **Pays off in:** turns 94% of the corpus from "synthesizes to nothing" into
  something that can carry a signal — the precondition for *any* spec-first
  hardware.
- **Risk:** a wrong default silently exposes an internal function as a module
  boundary. Prefer an explicit opt-in with a loud diagnostic for the rest.
- **Confirming measurement:** the 800/49 split moves, and the yosys cell count is
  non-zero for the specs that moved.

### Option 3 — **Attack the 617 non-compiling specs by cause, not by count**

T63's method: group by *cause*, not by message or shape. 617 is a floor (T67), so
the first honest step is a cause histogram.

- **Cost:** medium; the method is established and was used successfully in W650.
- **Pays off in:** the largest single population in the project, and the one
  blocking every downstream simulation claim.
- **Risk:** T63 predicts a long tail; a flat histogram is a legitimate result and
  must be reported as one rather than forced into classes.
- **Confirming measurement:** a cause histogram summing to 617 with named classes.

**Recommendation: Option 1.** It is bounded, reversible, and closes a defect that
is *known to produce wrong values today*. Option 3 is the biggest population but
its first wave produces a histogram, not a repair; Option 2 needs a design
decision about defaults that should not be made at speed.

**φ² + φ⁻² = 3 | TRINITY**
