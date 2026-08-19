# Wave Loops 657–663 — the service, the proof, and the law that governs the backlog

**Date:** 2026-08-14 · **Predecessor:** [`WAVE_LOOP_656_REPORT.md`](WAVE_LOOP_656_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

**Why one report for seven waves.** W657–W662 were reported in conversation and
never written to disk. That is the gap this file closes; it is a consolidation,
not a summary of work done since W662.

---

## Summary

```
THE SERVICE, as commands rather than documents
  tri boards | preflight | path | prove | corpus | backlog

THE PROOF, extended from one MAC to a whole generated datapath
  T110  the classifier == a multiplying golden, for ALL inputs, under induction
  T117  the SAT wall is set by the WEIGHT width -- 64x2 proves, 64x6 does not

THE BACKLOG, measured until the number was honest
  T125  the compiler-defect backlog is 124 specs, not 466 and not 289
  T126  a fix moves the count iff it clears a spec's LAST class -- 3 confirmations
```

---

## 1. The service is five commands (W657, W660, W661)

[`PATH_TO_HARDWARE_RU.md`](../theory/PATH_TO_HARDWARE_RU.md) argues that a recipe
which must be remembered will eventually not be — W657 proved it by applying a
correctly written recipe backwards. So the obligations are in `t27c`, and `tri`
forwards to it (L7: no shell on the critical path).

| command | what it refuses to let you believe |
|---|---|
| `tri boards` | that a `--busdev-num` from yesterday still names the same board |
| `tri preflight` | that a toolchain which builds can place and route |
| `tri path` | that elapsed time is a verdict |
| `tri prove` | that a proof which cannot fail is evidence |
| `tri corpus` | that a diagnostic count measures defects |
| `tri backlog` | that the most frequent cause is the most blocking one |

`tri path` prints the thesis instead of restating it:

```
  OK  spec -> Zig       0.01s  8318 B
  OK  Zig tests         2.12s          All 31 tests passed.
  OK  spec -> Verilog   0.01s 30809 B
  OK  iverilog + vvp    0.09s 64753 B  31 PASSED, 0 FAILED
  OK  yosys             3.03s          166 LUT, 74 CARRY4, 0 DSP48E1
  total 3.41s, of which code generation 0.4%
```

---

## 2. The proof, and the only advantage that survived (W657, W658)

**T110** — the miter now covers the **generated** classifier: 24 weight decodes,
three adder trees, an argmax whose tie rule is part of the spec. 14,050
variables, discharged under `-tempinduct`, so it holds for **all reachable
states** rather than a bounded depth. Two independent perturbations of the golden
make it fail, including one that touches only the tie rule.

**T117** — sweeping the two operand widths separately:

```
weight = 2 bits, activation swept    8x2 0.10s ... 128x2 0.33s   LINEAR
activation = 64, weight swept        64x2 0.16s ... 64x5 119.92s ... 64x6 NOT PROVED
```

> Sixteen-fold growth in **activation** width costs 3.3×. Three extra bits of
> **weight** width costs 750×, and a fourth crosses the wall.

Ternary/binary weights are formally verifiable; int8 weights are not. **It is a
property of narrow weights, not of φ** — BitNet's `{−1,0,+1}` is also two bits
and gets the same benefit — so it does not differentiate this project from
BitNet. What it separates is the whole low-bit family from int8, categorically.

**And it is the only surviving technical advantage.** Area does not separate the
alphabets (T97, T111, T112 — the *multiplying* golden is the larger design).
Power does not: 5 W against Syntiant's sub-milliwatt. Novelty does not:
LogicNets (FPL 2020, **not** FINN) published zero-DSP inference first. Accuracy
is unmeasured, and the MVP does not implement the Fibonacci step at all — its
`contrib` returns `±x`, which is BitNet's alphabet with φ as a common factor
argmax ignores.

---

## 3. The backlog, measured until the number stopped moving (W660–W663)

Three successive corrections, each of a number this project had been quoting:

| claim | reality | inflation |
|---|---|---|
| "466 specs fail" | 173 do not parse, 159 unwritten, 6 partial | — |
| "289 defect specs" | contaminated by unwritten specs | 2.3× |
| **124 defect specs** | populations sum to 617 exactly, two code paths agree | — |

```
iverilog accepts   155 | does not generate 173 | UNWRITTEN 159
PARTIAL 6          | DEFECT 124                     155+173+159+6+124 = 617
```

**T126 — the law.** Four fixes, each correctly diagnosed and verified:

| fix | specs repaired | depth | compiling count |
|---|---:|---|---:|
| escape-last (`\cross _data_width`) | 13 | >1 | 151 → **151** |
| Verilog scaffold `default_input()` | 140 | 94% at 4+ | 151 → **151** |
| **`#` is a comment (phantom fields)** | **4** | **all depth 1** | **151 → 155** |
| Zig builtins leaked to Verilog | 17 | >1 | 155 → **155** |

> The three that moved nothing removed 170 specs' worth of real defects. The one
> that moved the number touched four. **Cause size predicts nothing; depth
> predicts everything, and only depth-1 has any yield.**

Depth-1 is now **zero**. No single compiler fix can raise the count again.

---

## 4. Anomalies found and healed

| anomaly | how it presented | truth |
|---|---|---|
| four orphaned `vvp` at 98% CPU, 38 min | nothing; found by the wave-start check | T83/T98 recurring |
| `nextpnr` "P&R in 0.0 s" | a fast, successful stage | binary deleted with the scratchpad |
| 29 corpus "hangs" | diligent timeout enforcement | **my own** undrained pipe, 64 KiB |
| T113–T116 missing from the tree | commit message described them | concurrent write dropped them |
| `'X' already declared` | a missing dedup in the emitter | `#` not lexed as a comment |
| build "Finished in 0.44s" | a completed rebuild | the *second* cargo call; nothing built |

**The pattern.** Six of six presented as success or as diligence. Not one
announced itself as a failure.

---

## 5. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean, seal matches source |
| MVP, both backends | **31/31** Zig, **31 PASSED** iverilog |
| `prove_ternary_mac.ys` | `Induction step proven: SUCCESS!` |
| `prove_mvp_classifier.ys` | `Induction step proven: SUCCESS!` |
| `tri prove --mutate` | fails on a perturbed golden, as it must |
| `impl-status` | 159 / 6 / 173, unchanged |
| three boards | `Done 0x0 → 0x1` with a wrong-part transition |

---

## 6. What is NOT done

- **The MVP does not implement `Z[φ]`.** `contrib` returns `±x`; there is no
  `(a,b)` pair and no Fibonacci step in silicon.
- **No trained weights in `{−φ,0,+φ}` exist anywhere**, including here.
- **The LED is not read programmatically.** `BSCANE2` is unimplemented.
- **No inter-board link.** Three boards carry the same network: replication.
- **`__mul_noop` is verified on 12 of its 64 bits** (T114) and is emitted by
  130 of 200 specs.
- **Depth is a lower bound** (T127): `syntax error` merges unrelated causes.
- **Two Russian documents violate LANG-EN** and need Architect approval to be
  grandfathered — flagged, not self-granted.

---

## 7. Three ways to continue

### Option 1 — **`Path::Item`, the largest named untranslated construct** (23 specs)

`Severity::Error` reaches Verilog unlowered. It is the biggest of the four
constructs T127 named, and unlike the others it is a single, well-defined
lowering: a namespaced enum path to its integer value.

- **Cost:** low–medium. The enum values are already known to the compiler.
- **Risk:** **T126 predicts +0 compiling specs.** Take it as a correctness fix,
  and say so in advance rather than discovering it afterwards.
- **Confirming measurement:** `Path::Item` occurrences 23 → 0, and the compiling
  count reported honestly whether or not it moves.

### Option 2 — **Generic types** (27 specs, T122)

`pub const Maybe(T) = struct {...}` — the largest single class among the 173
that do not parse, and every member is a container.

- **Cost:** high, and it is a **language decision, not a repair**. Parsing is not
  lowering: a generic type without instantiation has no meaning in Verilog.
- **Risk:** the specs may parse and still not generate, which is a worse position
  than not parsing — it converts a clear failure into an unclear one.
- **Confirming measurement:** 27 parse, and **separately** how many generate.
- **This one needs the user's decision, not mine.**

### Option 3 — **Make the MVP observable over JTAG** (`BSCANE2`)

Everything built on the boards rests on a lamp nobody reads with a machine.

- **Cost:** medium; support in the open flow is unverified.
- **Risk:** may not be reachable through `nextpnr-xilinx` at all.
- **Confirming measurement:** the verdict register read over JTAG agrees with
  the simulation for all ten reference vectors.

**Recommendation: Option 1**, with its expected yield stated as zero. T126 was
confirmed three times this session; the honest way to act on it is to stop
expecting the count to move and start reducing the *named* untranslated
constructs, which is the only thing that can eventually make depth-1 specs exist
again.

**φ² + φ⁻² = 3 | TRINITY**
