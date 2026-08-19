# Wave Loops 684–689 — the wave the literature answered, and three of our own claims fell

**Date:** 2026-08-14 · **Predecessor:** [`WAVE_LOOP_677_683_REPORT.md`](WAVE_LOOP_677_683_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Six waves, **T148–T166**, lessons 478–513. Twelve agents, 1.04 M tokens, one
compiler repair. **The most valuable output of this period is the list of things
this project believed that are not true.**

---

## Summary

```
WHAT THE LITERATURE SAID
  T158  a pure {-phi,0,+phi} net IS a ternary net times phi^k -- phi factors out
  T159  the growth bound is TIGHT, not worst-case; and it is a conservation law
  T160  phi^-1 is free too, and we never used it
  T161  83 LUT has no denominator: 3.46 LUT/MAC vs FINN's 3.66, in 2017
  T162  what we do is TRANSLATION VALIDATION; XLS ships a stronger version

WHAT FELL
  T163  T141 RETRACTED -- BSCANE2 works on silicon; the "missing" bits are
        pseudo-pips that produce zero bits BY DESIGN
  T164  the 80%-off-mission claim refuted: 27.2%, and one cron is 35% of all issues
  T165  four arithmetic errors, all found by re-adding our own published tables

WHAT WAS REPAIRED
  T157  a spec with NO functions was counted FULLY IMPLEMENTED -- 279 -> 218
  ----  the Markdown language check wrote to stderr and reported to nobody
```

---

## 1. The claim at the centre of the project is empty as stated (T158)

Every layer computes `Σᵢ(±φ)xᵢ = φ·Σᵢ(±1)xᵢ`. **φ factors out.** A depth-*k*
pure-φ network is exactly `φᵏ` times the corresponding Ternary Weight Network —
verified by exact simulation at depths 1, 2, 3, 5.

So the golden ratio adds **nothing** over `{−1, 0, +1}`. And BitNet b1.58's
effective alphabet is already `{−γ, 0, +γ}` with `γ = mean|W|` **fitted from
data**; ours is that with γ pinned to 1.618. TWN had the scaling factor in
**2016**; TTQ learned *asymmetric* levels in **2017**.

**The relief and the cost arrive together.** The MVP's `contrib` returning `±x`
has been recorded for many waves as "the MVP does not implement Z[φ]". **It is
not a shortfall** — for the pure-φ alphabet, `±x` *is* correct up to the global
`φᵏ`. But the framing that made it look like a shortfall was itself the error.

> **The defensible claim is the five-level alphabet {0, ±1, ±φ}**, whose outputs
> have irrational ratios and which does *not* factor. One add, one two-word
> register. Nobody has published it. **That is the paper.**

**And the null result is real:** exact-phrase searches across arXiv, DBLP,
OpenAlex and Semantic Scholar return **nothing** on golden-ratio weight
alphabets; a GitHub issue sweep for "Fibonacci number system hardware" returns
**literally zero**. Recorded as *measured-zero*, so no later wave re-spends it.

---

## 2. The growth bound, upgraded to a conservation law (T159, T160)

`width = input_bits + 0.6942·k`. The project had the formula. **It had the word
"worst case" wrong**: growth is `Θ(φᵏ)` for *every* nonzero integer start,
because the contracting eigendirection has irrational slope and no integer point
lies on it. Depth growth is deterministic; the `√N` corollary applies to
**fan-in** only.

**And it cannot be escaped by a better encoding.** Dirichlet's unit theorem plus
Kronecker: in a real quadratic field the only roots of unity are ±1, so every
unit `u ≠ ±1` has `|u| ≠ 1`. **A free add-only multiplier of magnitude ≠ 1
necessarily grows coefficients geometrically.** Imaginary quadratic rings give
growth-free multipliers — and no useful scaling. The trade is structural.

**What *does* escape it: `φ⁻¹ = φ − 1`**, the step `(a,b) ↦ (b−a, a)` — one
subtraction, exact in Z. Renormalise once per layer and width is constant
forever. **T99 presents depth growth as unavoidable; it is unavoidable only if
you decline to divide.**

---

## 3. T141 is retracted, and the retraction was already ours (T163)

W676 fed `fasm2frames` one FASM line at a time, saw `NO BITS` for all six BSCAN
routing entries, and concluded the open flow could not express them.

```
$ grep -n CFG_CENTER_LOGIC_OUTS_B22_2 build/fpga/openxc7/prjxray-db/artix7/ppips_cfg_center_mid.db
1:CFG_CENTER_MID.CFG_CENTER_LOGIC_OUTS_B22_2.CFG_CENTER_BSCAN1_CAPTURE always
```

285 entries, 44 BSCAN, all type `always`. **They are pseudo-pips — always-on
routing that requires no configuration bits by construction.** Zero bits is what
a *working* pseudo-pip looks like. The measurement was correct and the reading
was inverted.

**This project had already established that, on 2026-08-13**, and withdrawn its
own upstream issue with an A/B and a hardware A/B/A:

```
control (no BSCANE2)   drscan -> 00000000  00000000
BSCANE2 design         drscan -> a5a51234  a5a51234  a5a51234
control reloaded       drscan -> 00000000  00000000
BSCANE2 reloaded       drscan -> a5a51234  a5a51234
```

Nine reads, a magic constant, both directions — on **xc7z020**. The artix7
database carries the same file, so the path is open on our part; **that is
support, not yet a read.**

> **Four waves refuted the readback. The fifth located the root cause upstream.
> The sixth found the root cause was a submodule in our own tree — and that the
> missing bits were never supposed to exist.**

---

## 4. The compiler repair (T157)

```rust
if fns.is_empty() { r.implemented += 1; continue; }   // before W689
```

**A spec that declares no function was counted as fully implemented.** Sound
arithmetic — it has no *missing* bodies — attached to a false label.

```
before   implemented 279 | partial 6 | unwritten 159 | noparse 173
after    implemented 218 | NO-FN 61 | partial 6 | unwritten 159 | noparse 173 | total 617
```

Sixty-one specs, among them `sacred/dark_matter.t27`, `tri/math/math.t27` and
`tri/agent/memory.t27` — **47 characters each after comments are stripped, and
twenty-five of them byte-identical** (T154).

**The module already knew.** `spec_status`, twenty lines below, has returned
`NOFN` since W665, and a previous wave recorded 218 + 61 = 279. **Knowledge was
never the missing piece; the repair was.**

**Forecast registered before the fix** (T44), all four terms confirmed — including
the refutation condition, *the corpus must not move*: **156 / 196 / 64 / 444,
unchanged**.

---

## 5. Where the corpus actually stands

| population | count | share |
|---|---:|---:|
| every `fn` has a body | **218** | 35.3% |
| **no `fn` at all** | **61** | 9.9% |
| some bodies empty | 6 | 1.0% |
| every body empty | 159 | 25.8% |
| does not parse | 173 | 28.0% |
| **total** | **617** | |

**Twenty-two directories contain not one implemented spec** — all of `tri/` and
all of `ml/`, 219 files. And a glob measures something else entirely: `find specs
-name '*.t27'` returns **1,072** files and **585 MiB**, because `specs/scratch`
holds 455 generated benchmark specs — up to 36.77 MiB each — **tracked by git**,
carrying **99.0% of all bytes**.

---

## 6. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | 0 errors (639 warnings — "clean" is true for errors only) |
| `impl_status` tests | **4/4**, including two new guards |
| corpus | 444 generate · **156** iverilog-clean · 196 Zig-clean · 64 both — **unchanged** |
| `impl-status` | 218 / 61 / 6 / 159 / 173 = **617** |
| `FROZEN_HASH` | matches `compiler.rs` byte for byte |
| three boards | enumerate at 1:4, 1:6, 1:8, `idcode 0x3636093`, artix a7 **200t** |
| MVP, both backends | 31/31 Zig, 31 PASSED iverilog |
| language check | **10 warnings now visible** where 0 were before |

---

## 7. What is NOT done

- **The verdict has still never been machine-read on OUR part.** T163 removes the
  belief that it *cannot* be; it does not supply the read.
- **`gen/zig` does not exist.** `gen/` holds c, numeric, rust, verilog. Until it
  does, 184 `.zig` files across two repos cannot be checked against their specs.
- **Three leaked credentials remain unrotated** (`trinity#601`,
  `trios-dwagent#1`, `trios-railway#124`). This is a hard gate on any
  history-derived corpus.
- **One broken cron has opened 314 issues** — 35% of the organisation's backlog —
  and fires ~4/day. Nobody has stopped it.
- **`trinity-fpga` is a 95% byte-identical manual clone of `trinity`** (11,989 of
  12,567 blobs, zero shared commits). Two live heads, still unresolved.
- **Type aliases (T133)** — now **39%** of struct fields, not 51%. A language
  decision that remains the user's.
- **10 files fail the language check**, 8 beyond the two known.

---

## 8. Three ways to continue

### Option 1 — **Read the verdict off our own silicon**

T163 reopened the channel this project has chased for six waves and closed four
times. The design exists ([`mvp_ternary_classifier_jtag.v`](../../fpga/verilog/mvp_ternary_classifier_jtag.v)),
the transport exists (proven by IDCODE), the artix7 pseudo-pips are present, and
the method is recorded in our own withdrawn upstream issue.

- **Cost:** low-to-medium. One synthesis run and one JTAG read; every component
  has been exercised separately.
- **Risk:** the hardware proof is on **xc7z020**, ours is **XC7A200T**. The
  database supports it; nothing has been read on our part. **Assume it may fail
  and treat a failure as informative.**
- **Confirming measurement:** `0xA5A5A5A` returns from USER1 on a board, with the
  control bitstream reading zeroes on either side of it. **A/B/A or it does not
  count.**

### Option 2 — **Build the five-level alphabet {0, ±1, ±φ}**

T158 says the current claim is empty and names the non-empty one. This is the
only route by which this project has a *scientific* contribution rather than an
engineering one.

- **Cost:** medium. The datapath is one add and one two-word register wider; the
  spec, golden and miter all already exist for the three-level case.
- **Risk:** it may not help accuracy — and the honest version of this option
  requires an accuracy number on a **published benchmark** (MNIST,
  jet-substructure, UNSW-NB15), which this project has never produced.
- **Confirming measurement:** a five-level net whose outputs are **not** a
  per-layer rescale of any three-level net — the reducibility test, run and
  failed.

### Option 3 — **Emit the golden model from the spec, not by hand**

T162b is the one criticism of the proof story that lands. A miter proves
`DUT == GOLDEN`; our golden shares an author with the spec, and Knight & Leveson
(1986) measured what that does to independence.

- **Cost:** medium. A second, deliberately naive lowering in `t27c` — real `*`,
  signed compares, no ternary tricks — reusing the existing miter harness.
- **Risk:** low, and it *strengthens every existing theorem* rather than adding
  a new one. The two paths must share no author, or it buys nothing.
- **Confirming measurement:** `tri prove` passes with a golden nobody wrote, and
  `tri prove --mutate` still fails on a perturbed spec.

**Recommendation: Option 1, immediately, and Option 3 next.** Option 1 is the
cheapest remaining action with the largest outstanding payoff — the project's
entire silicon story rests on a lamp, and the reason it rested there turned out
to be a stale submodule. Option 3 is the one that makes the proofs mean what they
are already being quoted as meaning. **Option 2 is the science, and it should be
started only once there is a benchmark number to move.**

**φ² + φ⁻² = 3 | TRINITY**
