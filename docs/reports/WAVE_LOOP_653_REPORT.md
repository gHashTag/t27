# Wave Loop 653 — all three options; the test harness could not fail

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_652_REPORT.md`](WAVE_LOOP_652_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
All three W652 options were taken.

opt 3  the throughput-per-area claim is RETRACTED -- it fails on the
       article's own table, and int8 beats GFTernary on the same bench
opt 2  a bitstream was BUILT LOCALLY and loaded on all three boards,
       with a state transition proving it took effect.  The blocker was
       two appended lines, not the 1.3 GB regeneration the tool advises
opt 1  the first genuinely three-valued object exists: 3B2T, the PAM3
       line code of IEEE 802.3bp-2016.  29 tests pass under `zig test`.

And verifying it found that the verifier had never worked:
NO CLI-GENERATED VERILOG TEST HAS EVER EVALUATED ITS ASSERTION.
```

---

## 1. Option 2 — the flow works, and the blocker was two lines

**Built on this Mac:** `w653_blinky_200t.bit`, 9,730,896 bytes, SHA-256
`4272dd6f…`, matching **none** of the 16 `.bit` files in the tree. Chain:
Verilog → yosys 0.63 → nextpnr-xilinx → FASM → `fasm2frames` → `xc7frames2bit`.
P&R: **22 warnings, 0 errors**, 6.2 s of router time.

**T72 — the diff was two appended lines.** The fork's `constids.inc` has 786
entries; the 332 MB chipdb was generated from a 784-entry one. `constids` are
**ordinal**, so the 784 file is a strict **prefix** and every ID already had the
right value. `X(GE)` is unused; `X(BUFR)` had one use, made dynamic with
`ctx->id("BUFR")`.

> A version assertion reports **that** two artefacts disagree, never **how much**,
> so its recommended remedy is sized for the worst case. **Diff before accepting
> it** — the failure is binary, the disagreement is not.

Also required: `-DUSE_OPENMP=OFF` (Apple clang rejects `-fopenmp`). And `--test`
(archcheck) **still fails** while real P&R succeeds — a self-consistency check is
not a use-case check.

Recipe: [`docs/fpga/LOCAL-BITSTREAM-FLOW.md`](../fpga/LOCAL-BITSTREAM-FLOW.md).

**T73 — the load path checks the envelope, not the contents.**

| loaded | `STAT` |
|---|---|
| nothing (resting) | `0x401079fc`, `Done 0x1` |
| valid 200T bitstream | `0x401079fc`, `Done 0x1` |
| **4 KB of payload XOR-inverted** | **`0x401079fc`, `No CRC error`, `done 1`** |
| bitstream for the wrong part | `0x5000890c`, **`Done 0x0`, `ID Error`** |

A deliberately corrupted payload was **indistinguishable from success**. Only the
wrong-*part* case is caught, by the IDCODE in the header.

**That failure mode is also the fix.** Because a wrong-part load drives `Done`
to `0x0`, it pre-conditions the board into a state where the criterion *can*
fail. Measured on all three:

```
0:4 / 0:7 / 0:10    before Done 0x0  ->  after Done 0x1, No ID error
```

**This is T71's corollary made operational**: when the status quo is already
green, break it deliberately before testing. It still does not identify *which*
design is resident.

---

## 2. Option 3 — the ranking is retracted

None of the headline numbers reproduce from the article's own table:

| claimed | table |
|---|---|
| +10.2% over next | `0.1584/0.1429 = ` **+10.85%** (no row yields 10.2%) |
| 6.1× over posit32 | `0.1584/0.0302 = ` **5.245×** (no row yields 6.1×) |
| 20 formats, 8 ours | **24** rows, **12** ours |

**And `int8` on the same bench is 0.1736 against GFTernary's 0.1584 — it wins by
9.60% — and is excluded.** The metric is confounded with input width, which the
article proves *before* presenting the ranking (its Truncation Proposition:
GFTernary is `n = 2`, competitors `n = 8…32`).

**Replacement claim**, checkable within one `n`, not refuted by int8:

> Among formats of equal input width, GFTernary is the only one whose lattice is
> closed under weight application, and therefore the only one whose linear path
> is exact without a normalisation stage.

The `Z[φ]` closure and uniqueness theorems, the precision law, the taper
diagnostics, the `r^d = r+1` hierarchy and the 28 LUT/weight zero-DSP datapath
measurement are all independent of the ranking and stand. **Eleventh retraction;
the first not found by its authors.**

---

## 3. Option 1 — the first three-valued object

[`specs/fpga/ternary_link.t27`](../../specs/fpga/ternary_link.t27) — **3B2T**,
the PAM3 line code of **IEEE Std 802.3bp-2016** (1000BASE-T1) and **802.3bw-2015**
(100BASE-T1). Adopted, not invented, so the link is comparable to prior art.
Alphabet `{−1, 0, +1}` wired as `2'b01 / 2'b10 / 2'b00`, matching tri-net's
existing `tern_corr8.v`.

**29 tests, all passing under `zig test`.**

**The design result worth keeping.** A ternary symbol carries `log₂3 = 1.585`
bits; 3B2T carries 1.5, an efficiency of 94.6%. **The 0.085 bit/symbol given up
buys the one unused codeword of nine** — so the frame delimiter is unreachable
from data as a *theorem*, where `bpsk.t27`'s Barker-13 preamble is unique only
*statistically*.

**What still requires a human:** two pins wired between two boards, and a
three-level receiver — two comparators, which on 7-series means two VREF
thresholds. Transmit needs nothing external: the IOBUF already has drive-0,
drive-1 and Hi-Z.

---

## 4. The finding that outranks all three — T74/T75/T76

Verifying the ternary link found that the verifier had never worked. **Three
stacked defects, each hiding the next.**

**T74 — the verdict did not depend on the outcome.**

```verilog
if (!(cond)) begin $display("[TEST] x : FAILED"); end
$display("[TEST] x : PASSED");                        // ALWAYS
```

A failing test printed **FAILED and then PASSED**. W640 fixed the empty-body case
(T45) and left this — **T52's shape, third instance in one emitter.**

**T75 — two halves of one feature, two different gates.** The `given`-binding
declaration hoist was gated on `emit_test_assertions`; `VerilogCodegen::new()`
sets it **false**, and `main.rs:4858` — the CLI `gen-verilog` path — calls
`new()`. The assertion *bodies* come from an ungated path. Result: checks reading
names that were never declared — **87 iverilog errors on a 29-test spec.**

**T76 — and the check was against an unknown.** With declarations restored, the
negative control *still* passed a deliberately false test:

```verilog
reg signed [7:0] _t27_call_tmp_..._0;   // declared
v = _t27_call_tmp_..._0;                 // NEVER ASSIGNED -- two() is not called
if (!((v == 99))) begin ... end          // (x == 99) is x; !(x) is x; if(x) is FALSE
```

> **T76.** A test harness written in a logic with an unknown value must use
> **case** equality, or it silently converts *"I could not tell"* into *"it
> passed."*

Changed to `(cond) !== 1'b1`. The control now reports **FAILED for both** tests —
including the true one — **which is correct, because both compare against `x`.**

> **Every `[TEST] … PASSED` line this project has emitted from CLI-generated
> Verilog is uninformative.** The 265 committed Icarus baselines record that
> state, so **T65's staleness problem is larger than measured**: those oracles do
> not merely freeze a formatting bug, they freeze **a harness that could not
> fail**.

**Each defect was exposed only by a negative control run after the previous fix.**
That is T44's discipline applied three times in one sitting, and the only reason
the third was found.

---

## 5. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| `zig test` on the ternary link | **29/29 passed** |
| new bitstream vs 16 committed | **no SHA match** |
| board configuration, ×3 | `Done 0x0 → 0x1` transition |
| corrupted-payload control | **not detected** (T73) |
| false-test control, before fixes | **PASSED** (wrong) |
| false-test control, after fixes | **FAILED** (correct) |
| ratchet | **running at time of writing — not reported as complete** |

---

## 6. Self-repair this wave

1. `iverilog rc=0` from a pipe through `head` — **third instance this session** of
   an exit code taken from the wrong process. The real code was **87**.
2. A python patch double-inserted a struct field; caught by `cargo`, removed.
3. My tail predicate used `emit_test_assertions`, which turned out to be false in
   the CLI path — replaced with a real counter of lowered checks.

---

## 7. Three ways to continue (pick one for W654)

### Option 1 — **Fix the unassigned call temp (T76's root cause)**

The reporting is now truthful; the underlying defect is not fixed. Every
`given v = f()` still lowers to a read of a declared-but-unassigned temp, so
every test genuinely evaluates `x`.

- **Cost:** medium; one materialisation site.
- **Pays off in:** turns the entire Verilog test corpus from *uninformative* to
  *informative*. Nothing downstream of it means anything until this lands.
- **Risk:** once tests really run, an unknown number will genuinely fail. **That
  is the point**, and the ratchet will need a considered bless, not a bulk one.
- **Confirming measurement:** the false-test probe reports `PASSED` for the true
  test and `FAILED` for the false one — **both**, not both-failed.

### Option 2 — **Re-bless the 265 Icarus baselines, after Option 1**

T65 said 45 were stale from a format fix. T74–T76 say **all 265** record a
harness that could not fail.

- **Cost:** high, and it must not be bulk (T31).
- **Pays off in:** the oracles start describing the specification instead of the
  generator's defects.
- **Risk:** doing this *before* Option 1 freezes the current state again.
- **Confirming measurement:** a `generator_sha` stamp present, and a synthetic
  generator change flagging every affected baseline.

### Option 3 — **Synthesize the ternary link and place it on a board**

The flow works now. `ternary_link.t27` → Verilog → yosys → nextpnr → `.bit`,
loaded with the `Done 0→1` criterion.

- **Cost:** low-medium; the recipe is written and proven on the blinky.
- **Pays off in:** the ternary line code stops being a simulation and becomes
  logic on silicon — the first hardware step of the mission's actual name.
- **Risk:** it proves the *encoder* fits, not that anything crosses between
  boards. **Say that up front**; the two-pin link still needs a human.
- **Confirming measurement:** a `.bit` distinct from all committed ones, the
  `Done 0→1` transition, and the yosys cell count showing the encoder present.

**Recommendation: Option 1.** Everything this project believes about its Verilog
rests on a harness that has never been able to report a failure. Option 3 is
attractive and Option 2 is overdue, but both would be built on measurements that
are not yet measurements.

**φ² + φ⁻² = 3 | TRINITY**
