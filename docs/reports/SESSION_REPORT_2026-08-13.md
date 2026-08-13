# Session report — 2026-08-12/13 · Wave Loops 623–651 + hardware bring-up

**Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959) · **Branch:** `claude/igla-fpga-improvements-3f5e1a`
**Span:** `fee990965` → `6a43d94f4` · **66 commits** · **Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 0. What this session was

It started as a compiler-quality session and ended as a hardware session. Two
distinct bodies of work, one continuous method.

```
W623 .. W651   29 wave loops, 66 commits
               theorems T15 -> T67 (66 in docs/theory/IGLA-FORMAL-RESULTS.md)
               524 wave reports on disk, max index 651

then           3 physical FPGAs appeared on the USB bus
               -> inventory, disambiguation, and all three configured
```

---

## 1. The problem the session opened with

`t27c suite` printed `TOTAL FAILURES: 2614` and **could not distinguish "nothing
changed" from "you broke the compiler."** A number that large is not a signal; it
is a wall. Every wave had to re-derive whether it had regressed.

**What was built, end to end:**

| piece | what it does |
|---|---|
| **Attribution** (`PhaseAttribution::attribute`) | splits PRIMARY from BLOCKED so one defect is not counted once per gated phase |
| **Expectations ledger** | identity-keyed amnesty on `(path, phase)`, XPASS-strict, mandatory `expires`, monotone-downward `max_entries` |
| **`--corpus-only`** | 4057 s → ~314 s with a **bit-identical verdict** |
| **`.github/workflows/corpus-ratchet.yml`** | blocking per-PR gate |
| **`docs/CORPUS-RATCHET.md`** | the bless procedure, with an explicit "what this does NOT cover" section |

The suite now exits **0 while its own total is non-zero**, because the verdict is
observed-versus-expected per identity, not a threshold on a sum.

**Current state, measured:**

```
ledger:              332 / 332 cap
observed (primary):  332
UNEXPECTED FAILURES: 0     UNEXPECTED PASSES: 0     EXPIRED: 0
RATCHET: CLEAN                            rc=0, 744 s
```

Ledger composition: `parse` 173, `parse-no-discard` 132, `no-vacuous-verilog-test` 27.

---

## 2. Corpus repairs, measured

| metric | before | after |
|---|---:|---:|
| parse failures | 206 | **173** |
| vacuous Verilog test blocks | 3,429 | **754** |
| keyword-declaration violations | 171 | **0** |
| iverilog rejections | 10 | **4** |
| corpus `[BENCH]` specs compiling | 3 | **19** |

---

## 3. The unifying result — T52

In **five independent artefacts**, `R(nothing was done) = R(verified)`:

- an `invariant` block with no statements printed `verified`
- an empty Verilog test block printed `PASSED`
- the C runner counted empty tests among the passes
- a backend that emitted nothing scored as covered
- a truncated list of failures read as the complete list

> **Success vocabularies are absorbing.** The empty case is the fixed point of the
> success encoding. The remedy is a **reserved symbol** (`NOT CHECKED`), not more
> care — because care is exactly what the absorbing case defeats.

---

## 4. The recurring meta-defect, and the one thing that catches it

**A syntactic selector standing in for a semantic one.** Recorded **nine-plus
times** this session:

- grouping failures by diagnostic message (1 class, useless) or by normalised
  source shape (55 classes, 27% coverage) instead of by cause (5 classes, 100%) — **T63**
- a keyword table complete for Verilog-2001 while every invocation passes `-g2012` — **T64**
- `grep '::'` finding zero hits in Zig because Zig spells the same dangling
  reference with a dot — **T67**
- `parse_const_decl` taking the correct branch only when a `(` or `{` followed — **T66**

**Not one instance was prevented by having written the previous one down.** What
catches them is **re-measurement by a different route** — and that is a method,
not a memo.

It happened again tonight, in this very session, and is recorded in §7.

---

## 5. T66 — the wave's sharpest result

```t27
pub const A : u8 = constants::COMPLEXITY_HIGH;
```

emitted, in **all four backends**:

```
gen (Zig)    pub const A: u8 = constants;
gen-rust     pub const A: u8 = constants;
gen-c        static const uint8_t A = constants;
gen-verilog  parameter [7:0] A = constants;
```

**Four backends, four silently wrong *values*, no error, no warning. 98
initialisers across 29 specs.** The same path *inside a function body* kept both
segments; `constants::make(5)` already worked because `(` routed it through
`parse_expr`.

> A defect that produces a **wrong value** is invisible to every check that asks
> whether the artefact is **well formed** — and `A = constants` is well formed in
> all four target languages. **Nine gates were built this session; none could see it.**

The repair makes the naive metric *worse*: C and Verilog now emit a visible error
where they emitted a silent falsehood.

**T67** pre-registered the `::` yield at **0 of 24** — measured by *simulating* the
fix — and found `::` is the outermost of **four to six stacked defects**.

---

## 6. Hardware — three FPGAs, found and brought up

Three boards appeared on the bus mid-session. Established by measurement:

| # | JTAG cable (FTDI) | UART | IDCODE | part |
|---|---|---|---|---|
| 1 | `@0x110000` · `0:4` | `/dev/cu.usbserial-130` | `0x3636093` | XC7A200T |
| 2 | `@0x122000` · `0:7` | `/dev/cu.usbserial-1230` | `0x3636093` | XC7A200T |
| 3 | `@0x140000` · `0:10` | `/dev/cu.usbserial-1240` | `0x3636093` | XC7A200T |

`irlength 6`, `artix a7 200t`. SPI flash: **Micron N25Q128 3V, 128 Mbit** (JEDEC `0x20ba18`).

### Two traps, both checked rather than assumed

**Count disagreement.** A first ioreg snapshot showed two cables; `--scan-usb`
showed three. The third (`@0x110000`) enumerated between the two snapshots. **The
population changed — the parser was not wrong.** Reporting either number without
resolving the disagreement would have been reporting a guess.

**Three identical IDCODEs prove nothing** if `--busdev-num` is silently ignored —
then it is one cable read three times. Negative control:

```
--busdev-num 0:99  ->  -3 (device not found)
--busdev-num 0:5   ->  -7 (set baudrate failed)   # landed on a CP2102N, not an FTDI
```

Two distinct failures at two invalid addresses. **The flag is honoured; the three
detections are three boards.**

### ⚠️ All three cables share one serial number

```
000 010 / 000 007 / 000 004   0x0403:0x6014  Digilent  210512180081
```

`--ftdi-serial 210512180081` matches all three and silently takes whichever
enumerates first. **The only working selector is `--busdev-num`, and it is not
stable across replug.** Stable identity is the ioreg `locationID`, which is bound
to the physical hub port.

This is the hardware instance of §4's meta-defect: **a selector that looks
discriminating and is not.**

### First provable milestone — reached

```
$ openFPGALoader -c digilent_hs2 --busdev-num {0:4,0:7,0:10} \
      fpga/verilog/ternary_mac_demo_top_200t.bit

Load SRAM: [====...====] 100%   Done
ir: 1 isc_done 1 isc_ena 0 init 1 done 1        (x3)
```

**All three boards configured with a ternary-MAC design from this repository.**

**What this does and does not prove.** It proves configuration: the bitstream is
valid for the part and the chain accepts it, `done 1` on all three. It does **not**
prove the ternary MAC computes correctly — `ternary_mac_demo_top.v` drives two
LEDs from a ring oscillator and has **no readback path**. Functional verification
needs a design with a UART, and there is not one yet.

---

## 7. Anomaly found in my own work tonight — and corrected

I measured the local synthesis toolchain, found `/opt/homebrew/share/himbaechel/
xilinx/` contained only `chipdb-xc7a100t.bin`, and committed the claim that **no
200T device database existed on this machine, so no bitstream could be built.**

**A single-route measurement published as a totality claim** — the ninth instance
of §4's pattern, committed by me, tonight, in a session whose central finding is
that pattern.

Second route — the repo's own build tree:

```
build/fpga/openxc7/xc7a200tfbg676-1.bin     332 MB   built 2026-08-09
build/fpga/openxc7/xc7a200tfbg676-1.bba     980 MB
build/fpga/openxc7/prjxray-db/artix7/xc7a200tfbg676-1/
```

It exists. It is in the **old `nextpnr-xilinx` format**, and the installed engine
rejects it:

```
$ nextpnr-himbaechel --device xc7a200tfbg676-1 --chipdb <that> --test
ERROR: chipdb ... does not look like a valid himbächel database!
```

**So the blocker is the *binary*, not the database** — and that inverts the plan:

- **(a) install `nextpnr-xilinx` (openXC7)** → consumes the existing 332 MB database. **Cheap.**
- **(b) regenerate a himbaechel chipdb** → repeats the generation that produced a 980 MB `.bba`. **Expensive.**

A plan built on my first measurement would have spent a wave regenerating
something that already exists. Corrected in `6a43d94f4`.

### Related documentation anomalies, unresolved

| doc | claim | measured |
|---|---|---|
| `CLAUDE.md` | XC7A100T, IDCODE `0x13631093` | **XC7A200T**, `0x3636093` |
| `CLAUDE.md` | "flash via `dlc10`, **do not use** openFPGALoader" | `dlc10` drives only `0x03FD`; our cables are `0403:6014`. openFPGALoader is what works |
| `README.md:61` | `FPGA \| E2E bitstream \| GREEN` | **not reproducible from a clean PATH** |
| `README.md` | board row: QMTECH XC7A100T | XC7A200T |
| `fpga/HARDWARE_SSOT.md` §2 | 1 Digilent + DSLogic Plus | **3 Digilent + 3 CP2102N, no DSLogic** |

`fpga/HARDWARE_SSOT.md` §1 is **correct** on part and tooling — `CLAUDE.md` is the
stale doc, and by the repo's own rule ("*if any other FPGA doc contradicts the
SSOT, the SSOT wins — fix the other doc*") `CLAUDE.md` is what must change. §2 of
the SSOT is itself now stale because the hardware changed tonight.

**Left unfixed pending the user's answer** on whether the three-board setup is
permanent — writing the wrong permanence into an authoritative file is worse than
leaving it visibly stale.

---

## 8. Science ingested this session

`docs/theory/TNF_ARTICLE_RU.md` (2,353 lines) and its distillation
`.claude/skills/tnf-gfternary.md`.

**Why it matters here:** the article's entire hardware campaign was measured on
**XC7A200T** — the exact part on all three boards — on a fully open flow
(Yosys 0.65, nextpnr-xilinx 1743d0f, Icarus 13.0, Python 3.14).

**The result that decides IGLA RACE's datapath:**

> **Theorem (the golden alphabet is unique).** For a weight alphabet `{−r,0,+r}`
> whose products must lie in the lattice the datapath already sums in — i.e.
> `r² = r + 1` — then `r = φ`, uniquely.

> **Theorem (multiplier-free path is exact).** `Z[φ]` is a ring containing
> `{−φ,0,+φ}`, so the **entire linear path** — every weight application and every
> accumulation, at any fan-in and any depth — is computed with **zero rounding error**.

Applying a weight is `(a,b) ↦ (b, a+b)`: **one integer addition, no shift.**
Depth costs nothing — `φ^k = F_k·φ + F_{k−1}`, a pair of integers.

**This is what separates φ from the `{−1,0,+1}` alphabet the whole ternary
literature uses:** with unit weights the layer gain is 1 and carries no
information, so every published method hangs a learned real scale `α_ℓ` on each
layer — **and multiplying by `α_ℓ` puts the multiplier back.** Same 2 bits, same 3
symbols; the φ alphabet *carries* the scale the unit alphabet must *learn and then
pay for*.

Measured: **28 LUT per weight, zero DSPs at any fan-in**, on our part.

The skill also carries the article's **ten retracted claims in place** — notably
that the `φ^k` scale grid **loses to APoT-2 by 15×** once compared against the
deployed baseline instead of bare powers of two — so no future wave re-quotes a
withdrawn number.

---

## 9. What is NOT done

- **No functional verification on hardware.** Configuration only. No design with a
  readback path exists.
- **No board-to-board link.** Three boards on one Mac's USB hubs is a **star through
  a single host**, not a mesh. No board-to-board wiring is documented anywhere.
- **`nextpnr-xilinx` not installed**, so no *new* bitstream can be built yet.
- **The 45 stale Icarus baselines** (T65) are not re-blessed.
- **The 98 preserved initialisers** are not audited for *resolution* — T66 stopped
  the truncation; whether each reference resolves is a separate question.
- **`use_resolve` not wired** into the two Verilog entry points (T67 forecasts yield 0).
- **Four gates unaudited** for their totality claims. Two of two audited so far
  were found wrong.
- **Still no web literature.** `WebSearch`/`WebFetch` failed with a provider error
  for the entire session. Everything named is from general knowledge, and **no
  citation was fabricated**.

---

## 10. The method, stated plainly

Five things earned their keep, in order of how much they caught:

1. **Re-measure by a different route.** Every meta-defect this session was caught
   this way and none was caught by having documented the previous one.
2. **Run the negative control.** Three identical IDCODEs meant nothing until
   `0:99` and `0:5` failed differently.
3. **Pre-register the forecast.** T67 said 0 of 24 before the work, by *simulating*
   the fix rather than reasoning about it.
4. **Reserve a symbol for "nothing happened."** Success vocabularies absorb the
   empty case (T52); no amount of care fixes an encoding.
5. **Resolve disagreements instead of picking a number.** Two cables versus three
   was a real event, not a bad parse.

**φ² + φ⁻² = 3 | TRINITY**
