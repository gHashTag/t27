# Wave Loop 652 — the ternary internet is binary, measured three ways

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_651_REPORT.md`](WAVE_LOOP_651_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
A 15-agent reconnaissance was run against the mission "build a ternary
internet on three FPGAs".  All three adversarial critics returned REJECT.

The most useful finding is not a defect.  It is that the PHY is binary:

  specs/fpga/bpsk.t27   SYM_POS : i8 =  1
                        SYM_NEG : i8 = -1
                        (two symbols.  no zero state.  anywhere.)

T68  every binary operator in a const initialiser was DISCARDED, in all
     five backends.  `UART_BIT_PERIOD = CLK / BAUD` emitted `= CLK`.
T69  the blast-radius count moved 893 -> 285 -> 248; each scanner was
     the same defect it was counting.
T70  three toolchain artefacts, no two of them compatible.
T71  `done 1` cannot distinguish my bitstream from the flash boot -- and
     I published it as proof.  T52's shape, committed by me.
```

---

## 1. T71 — I published a non-discriminating signal as evidence

W651's report claimed: *"all three boards configured with a ternary-MAC design
from this repository, `done 1` on each."* An adversarial review of the plan built
on that sentence forced a re-measurement:

```
$ openFPGALoader -c digilent_hs2 --busdev-num {0:4,0:7,0:10} --read-register STAT
Register raw value: 0x401079fc          (identical on all three)
MODE 0x1   EOS 0x1   INIT Complete 0x1   Release Done 0x1   Done 0x1
```

**`0x401079fc` is what the boards already carried.** They boot from Master-SPI
flash and assert DONE by themselves.

> **T71.** `done 1` is true whether or not the load happened. I quoted a signal
> that cannot separate the two cases and called it proof — **T52's shape
> (`R(nothing was done) = R(verified)`), committed by me, in the session whose
> central result is T52.**

What survives: a load ran to 100% and reported completion. What does not: any
claim about *what the boards are running now*. **Nobody read a bitstream back.**

**And the critics went further.** Synthesising `ternary_mac_demo_top` and reading
the cell count indicates the design reduces to a ring oscillator and a counter —
so even a successful load may not put a ternary MAC on the fabric. That is
*not yet independently confirmed here* and is the first item for W653.

---

## 2. T68 — every binary operator in a const initialiser was discarded

Found by generating `specs/fpga/uart.t27` for an unrelated reason and reading it.

```t27
const UART_BIT_PERIOD : u32 = UART_CLOCK_HZ / UART_BAUD_RATE;   // 868
```
```verilog
localparam [31:0] UART_BIT_PERIOD = UART_CLOCK_HZ;              // 100,000,000
```

The divisor was **dropped**. T66 had recorded this for *qualified paths*; the
class is far wider:

```
A / B    -> A        A << 2  -> A        A > 5   -> A
Cfg.width-> Cfg      100 / 7 -> 100
```

silently, in **all five backends**. **The C backend rendered each as
`typedef A DIV;` — a constant became a TYPE.** Two shapes were not truncated but
**erased**: `-A` pushed no child at all (Zig emitted `const NEG: i32;`, not valid
Zig) and `(A + 1) * 2` vanished the same way.

**The control that proves the diagnosis:** `f(A) + 1` was correct all along,
because `(` routed it through `parse_expr`.

> **T68.** When a parser dispatches on the *next token's identity* rather than on
> *whether the expression continues*, the set of correct spellings is exactly the
> set whose second token appears in the dispatch table. **Correctness becomes a
> property of punctuation, not of meaning.**

Fixed with a `token_continues_expr` predicate; three tests pin it, including one
that the bare `-1` spelling keeps its old shape.

**And the same generation exposed a second, independent defect:** the emitted
`ZeroDSP_UART` module's ports are `(clk, rst_n, en, ready)` — **no `tx`, no `rx`**
— and a critic measured `grep -cE 'always @\(posedge|always_ff'` → **0**. A UART
with no serial pins and no sequential logic. **It does not need fixing; it needs
writing.**

---

## 3. The finding that reframes the mission: the PHY has two symbols

Three independent measurements say the "ternary internet" is binary today:

| layer | measured | verdict |
|---|---|---|
| **PHY** | `specs/fpga/bpsk.t27`: `SYM_POS=1`, `SYM_NEG=-1` | two symbols, **no zero state** |
| **wire** | tri-net's header is 11 binary bytes, payloads are hex | binary |
| **compute** | the only scheduled "ternary numerics" spec (`gft_dot2.t27`) contains **no base-3 arithmetic** — two comment lines only | binary float |

A scope critic put it exactly: **nothing ternary ever crosses between two nodes,
in any workstream, at any priority.** Every payload in every network workstream
is a hex byte string over a binary link.

### And the literature says "ternary on the wire" is not novel

Web tools worked for some agents this session (and failed for others with
`glm-4.5-air`), so these are **verified by fetch**, not recalled:

- **Ternary line codes are ~30-year-old shipping technology** — AMI / HDB3 / B8ZS
  put +V/0/−V on T1/E1 (ITU-T G.703).
- **PAM-3 *is* shipping in Ethernet, but not where the brief said.** My own brief
  claimed "PAM-3 in 1000BASE-T" — **factually wrong**; 1000BASE-T is 802.3ab-1999
  and uses 4D-PAM5. The real citation is **IEEE 802.3bp-2016 (1000BASE-T1)**,
  which uses **3B2T / PAM3**, approved 30 June 2016, and **802.3bw-2015
  (100BASE-T1)**.
- **2B1Q is quaternary, not ternary** — the acronym is "2 Binary 1 Quaternary". My
  brief listed it as ternary precedent. Wrong.

> **Two of the three "ternary precedents" I supplied to my own reconnaissance
> were errors.** The agents caught both by fetching the standards. A brief is
> data to be checked, not a premise to be built on — including when I wrote it.

### The two assets that are real

1. **P4's `ternary` match kind**, backed by TCAM — genuine prior art *above* the
   PHY, and almost nobody in this space cites it.
2. **The tri-state I/O buffer.** Every 7-series IOBUF has drive-0, drive-1 and
   **Hi-Z** — a real, natively supported third electrical state on the exact part
   we have three of. **This is the one place a genuinely ternary link could be
   built without inventing silicon.**

---

## 4. The numeric claim does not survive its own table

The competitor agent's verdict on the TNF throughput-per-area result:

> **"Do not submit the throughput-per-area claim. It fails on its own data before
> any reviewer opens a competing paper."**

- The abstract says *20 formats, 8 ours*; the table has **24 data rows, 12 bolded
  as ours**.
- **`int8` beats GFTernary on the same bench** — 0.1736 vs 0.1584 MHz/LUT,
  **1.096×** — and int8 is excluded from the ranking.
- **The metric is confounded with input width, and the article proves it itself.**
  Its own Truncation Proposition says a decoder with `n` input bits has at most
  `2^n` distinct outputs and that "only comparison within one `n` isolates
  design." GFTernary is `n = 2`; every competitor is `n = 8…32`.

**What survives, and it is the valuable half:** the `Z[φ]` closure mathematics is
correct and genuinely distinct from powers of two; the uniqueness argument
(*multiplier-free ⟺ algebraic integer whose companion matrix is `{0,±1}`; degree 2
admits only φ*) is small, sharp and falsifiable; the `r^d = r+1` family is an
apparently unclaimed design axis; and the operation-profile argument names its own
bound.

---

## 5. GitHub issues — the two routes disagree, and that is the finding

| route | open issues |
|---|---:|
| org-wide `gh search issues --owner gHashTag` | **240** |
| per-repo `gh issue list` over 13 repos | **468** |

The per-repo sweep caught what the prescribed command hides: **`--limit 100
--state all` silently returned exactly 100 rows** for `t27`, `trinity`,
`trinity-fpga` and `trios`. The org-wide search under-reports by roughly half.

> **`TNF` has ZERO issues in the entire organisation** — 0 title matches across
> all 26 open+closed sets, 0 org-wide in bodies, 0 `in:comments`. A first-class
> concept with a 2,353-line article and a dedicated skill has **no tracked work
> anywhere.**

`trinity-training` and `trios-t27` have **0 issues total** with issues enabled —
recorded as a finding, not an omission. No repo has issues disabled.

---

## 6. tri-net, measured

Real, and oversold in a specific way. It has a genuine BPSK/Barker-13 modem, a
genuine ETX/WMEWMA metric with RFC-8966 (Babel) feasibility routing, per-hop
X25519 + ChaCha20-Poly1305 with an HKDF ratchet and a 64-frame replay window, and
an 11-byte header whose bytes double as AEAD associated data.

**But `Cargo.toml` sets `autobins = false` with exactly one `[[bin]]` declared.**
Re-enabling autobins in a scratch copy: **25 of 29 binaries fail to compile —
including `trios_meshd`, the mesh daemon itself.** `cargo test` is green (415
passed) because the *library* is exercised; the daemon has been rotting
uncompiled, and the repo's own CI comment admits it.

The PHY is an AD9361 SDR on Zynq-7020, but **no code path binds the modem to the
radio or to TUN** — the only shipping transport is UDP over Ethernet.

### ⚠️ Two credential findings — please rotate

- `tools/ad9361_config.rs:9` carries a hard-coded SSH root password for boards at
  `192.168.1.11-13`.
- `src/bin/trios_meshd.rs:35` derives node secrets as
  `SHA256("trios-mesh/demo/v1/node/" || id)` — **derivable by anyone**, and
  `mac_key = None` at `:301` falls back to a hard-coded HELLO MAC key.

Reported, not modified. `trios-mesh` is private; `tri-net` is public.

---

## 7. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean, after the FROZEN_HASH ceremony |
| new tests | **3 passed** |
| const initialisers, 4 backends | every operand preserved |
| `UART_BIT_PERIOD` | `(UART_CLOCK_HZ / UART_BAUD_RATE)` — was `UART_CLOCK_HZ` |
| `nextpnr-xilinx` | **built** (6 MB, `e4a261c`), after `-DUSE_OPENMP=OFF` |
| chipdb compatibility | **REJECTED** — constids mismatch (784 vs 763 lines) |
| board STAT ×3 | `0x401079fc`, identical, **pre-existing** |
| ratchet | **still running at time of writing** |
| pre/post differential | **still running at time of writing** |

**Two measurements are outstanding and are not being reported as if complete.**

---

## 8. Self-repair events this wave

1. The nextpnr build **reported `rc=0` while failing** — my wrapper took the exit
   code from `tail`, not `cmake`. Re-run with the real code captured.
2. `cargo build` failed after the compiler edit. I first attributed it to the
   Russian TNF article violating LANG-EN; **wrong** — those are `cargo:warning`
   lines and several predate this session. The panic was the FROZEN_HASH seal.
   Ceremony run, build green.
3. My blast-radius scanner was rebuilt three times and is still an upper bound
   (T69). Abandoned in favour of a differential build.

---

## 9. Three ways to continue (pick one for W653)

### Option 1 — **A genuinely ternary link, over the tri-state buffer**

Stop transporting hex over a binary UART. The 7-series IOBUF has drive-0,
drive-1 and Hi-Z natively. Encode `{−φ, 0, +φ}` as `{drive-0, Hi-Z, drive-1}` on
one wire between two boards and measure the received symbol histogram.

- **Cost:** medium. Needs the bitstream flow (Option 2) first, and a physical wire
  between two headers.
- **Pays off in:** it is the **only** proposal on the table where something
  three-valued actually crosses between two nodes. Everything else is a binary
  network with ternary arithmetic inside the nodes — which is a fine thing to
  build, but it is not what the mission is named after.
- **Risk:** requires a human to connect two pins; and Hi-Z is only distinguishable
  with a defined bias network. **Say so before starting, not after.**
- **Confirming measurement:** a three-peak histogram at the receiver, and the
  middle peak surviving when the transmitter is idle.

### Option 2 — **Regenerate the chipdb and produce one new bitstream**

T70's real task. `bbaexport.py` defaults `--constids` to the 763-line file the
binary was compiled with; the 332 MB database was built from a 784-line one.

- **Cost:** medium-high. ~1.3 GB of disk and a long generation.
- **Pays off in:** unblocks *every* hardware claim. Until one new `.bit` exists,
  nothing designed can reach the boards.
- **Risk:** the package (`fgg676` vs `fbg484`) **cannot be verified from
  software** — a critic flagged that if the boards are `fbg484` the output is
  discarded. Resolve the package first or accept the rework.
- **Confirming measurement:** a `.bit` whose SHA differs from all 16 committed
  ones, loaded, and **read back** — not `done 1`.

### Option 3 — **Withdraw the throughput-per-area claim and keep the closure result**

The numeric claim fails on the article's own table (§4). The `Z[φ]` mathematics
does not.

- **Cost:** low. An erratum in the article and the skill, plus re-scoping the
  abstract.
- **Pays off in:** the paper's strongest asset stops being defended by its weakest
  one. The uniqueness and closure theorems are publishable; the ranking is not.
- **Risk:** none technical. It is a retraction, and this project has done nine of
  them well.
- **Confirming measurement:** the abstract's format count matches the table's row
  count, and `int8` appears in the ranking or its exclusion is justified in text.

**Recommendation: Option 3, then Option 2.** Option 3 costs one wave and removes a
claim that would be refuted in a reviewer's first paragraph using only our own
data. Option 2 is the critical path for everything physical. **Option 1 is the
mission's actual name and should not be attempted until Option 2 lands** — it is
the only one that makes the project ternary between nodes, and it deserves to be
built on a flow that works.

---

## Appendix — reproduction

```bash
# the finding that reframes the mission
grep -nE 'SYM_(POS|NEG|ZERO)' specs/fpga/bpsk.t27

# the signal that proves nothing
openFPGALoader -c digilent_hs2 --busdev-num 0:4 --read-register STAT | grep Done

# T68, in one line
printf 'module M\n\nconst A : u32 = 100;\nconst B : u32 = 7;\nconst D : u32 = A / B;\n' > /tmp/t.t27
./target/release/t27c gen-verilog /tmp/t.t27 | grep ' D '
```

**φ² + φ⁻² = 3 | TRINITY**
