# Wave Loop 690 — the verdict left the die

**Date:** 2026-08-14 · **Predecessor:** [`WAVE_LOOP_684_689_REPORT.md`](WAVE_LOOP_684_689_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

One wave, **T167–T172**, lessons 514–523. **The project's oldest open item is
closed.** Six waves chased a machine-readable verdict and four of them refuted
it. It was one parameter.

---

## Summary

```
THE RESULT
  T172   the MVP's answer read off silicon -- A/B/A, three boards, 27 reads
  T172a  the defect: JTAG_CHAIN must equal the SITE nextpnr places the cell at
  T172b  and the first read was exactly the artefact the 28-bit magic exists to catch
  T172c  every tool in the chain returned 0 while the design was wrong

FOUND BY WALKING THE PATH
  T167   87.2% of "synthesizable" generated Verilog contains $display
  T167a  ...so "156 iverilog-clean" measures SIMULATION, not synthesizability
  T168   all six BSCAN routing entries T141 called impossible are in our FASM
  T169   a zero-byte frames file yields a 9,730,899-byte bitstream, rc=0
  T170   t27c fpga-flash omits --busdev-num; CLAUDE.md names a cable we don't own
```

---

## 1. The read (T172)

The acceptance criterion this project set for itself is that `Done 0x1` proves
nothing, because the boards boot from SPI flash and assert DONE unaided. So force
it low first:

```
A1  wrong-part bitstream (xc7a100t)   ->  Done 0x0,  ID Error
B1  ours                              ->  done 1, isc_done 1, init 1
```

Then the read, with a control on both sides:

| | idx0 | idx1 | idx2 |
|---|---|---|---|
| **A** ours | `a5a5a5a7 a5a5a5a7 a5a5a5a5` | `a5a5a5a5 a5a5a5a7 a5a5a5a5` | `a5a5a5a5 a5a5a5a7 a5a5a5a7` |
| **B** no BSCANE2 | `00000000 ×3` | `00000000 ×3` | `00000000 ×3` |
| **A'** ours again | `a5a5a5a7 a5a5a5a7 a5a5a5a5` | `a5a5a5a7 a5a5a5a5 a5a5a5a5` | `a5a5a5a5 a5a5a5a5 a5a5a5a7` |

**Eighteen positive reads, nine negative, three boards, no exception.** The
28-bit magic returns, `const = 01` returns, **`ok = 1` every time**, and `beat`
toggles between reads — the on-chip sweep is running, not frozen.

`ok = 1` means the silicon found **no misclassification across all 256 inputs
since power-up**, re-checked ~250,000 times a second.

---

## 2. The defect was one parameter (T172a)

nextpnr places a lone `BSCANE2` at site **BSCAN3**. The design asked for
`.JTAG_CHAIN(1)`, so the FASM carried:

```
CFG_CENTER_MID_X61Y136.BSCAN.JTAG_CHAIN_1              <- enables chain 1
CFG_CENTER_MID_X61Y136.…CFG_CENTER_BSCAN3_TDI          <- wires site 3
CFG_CENTER_MID_X61Y136.…CFG_CENTER_BSCAN3_TDO
CFG_CENTER_MID_X61Y136.…CFG_CENTER_BSCAN3_DRCK
CFG_CENTER_MID_X61Y136.…CFG_CENTER_BSCAN3_SEL
CFG_CENTER_MID_X61Y136.…CFG_CENTER_BSCAN3_CAPTURE
CFG_CENTER_MID_X61Y136.…CFG_CENTER_BSCAN3_SHIFT
```

**Chain 1 selects a site nothing is wired to; site 3 is wired to a chain nothing
selects.** The tile has four independent chain-enable bits and 44 pseudo-pips
split 11 apiece across BSCAN1..BSCAN4.

| build | USER1 | USER2 | USER3 | USER4 |
|---|---|---|---|---|
| `JTAG_CHAIN(1)`, site BSCAN3 | `ffffffff` | `00000000` | `00000000` | `00000000` |
| **`JTAG_CHAIN(3)`, site BSCAN3** | `00000000` | `00000000` | **`a5a5a5a7`** | `00000000` |

**The BEL cannot be pinned instead.** nextpnr routes `BSCANE2` through the IO
packer and rejects the attribute:
`ERROR: Unexpected IOBUF BEL BSCAN_X0Y0/BSCAN`. Matching the *parameter* to the
placement is the fix that works — and it must be re-checked after every P&R run,
because nothing enforces it.

---

## 3. The magic earned its keep on first use (T172b)

With chain and site mismatched, `USER1` returned `00000007` / `00000005`
alternating. Low nibble `0111` / `0101`:

```
ok = 1      const = 01      beat toggling
```

**A perfect-looking verdict.** The 28 bits above it were **zero**.

W675 widened the magic from 4 bits to 28 because W674's four-bit read could not
be distinguished from a JTAG artefact — ten reads of a bitstream containing *no*
BSCANE2 had returned the same two values in the same proportions (T139). **The
concern was correct, the countermeasure was correct, and it fired on the very
first build that needed it.**

> **T172c.** Every tool in the chain returned 0 while the design was wrong:
> yosys, nextpnr, `fasm2frames`, `xc7frames2bit`, `openFPGALoader`. **The
> mismatch is invisible to the entire toolchain and visible only in the read.**

---

## 4. What walking the path exposed

**T167 — `gen-verilog` puts the spec's tests into the "synthesizable" output.**

| | count | share |
|---|---:|---:|
| specs generating Verilog | 444 | |
| **output contains `$display`** | **387** | **87.2%** |
| free of `$display` and `initial` | **56** | **12.6%** |
| total `$display` calls | **43,053** | |

yosys turns each into a `$print` cell and nextpnr cannot place one. `gen-verilog`
differs from `gen-verilog-for-simulation` — an existing command — by four lines.

> **T167a.** "156 specs are `iverilog`-clean" measures **acceptance of testbench
> code by a simulator**. The count producing Verilog a place-and-route tool would
> accept unaided is **at most 56**.

**Deliberately not repaired this wave.** Changing `gen-verilog` moves what
`corpus` measures, and the forecast for that has to be written first (T44).

**T169 — a bitstream is not evidence.**

```
zero-byte frames  ->  rc 0,  9,730,899-byte .bit
the real build    ->  rc 0,  9,730,898-byte .bit
```

**One byte apart.** Only content discriminates: 9,361 differing bytes (0.10%),
different sha256. **Gate on the frames file, never on the `.bit`.**

**T170 — two tooling defects.** `t27c fpga-flash --dry-run` emits
`openFPGALoader --cable digilent_hs2 <bit>` **with no `--busdev-num`**, so with
three cables sharing serial `210512180081` it programs whichever is enumerated
first. And [`CLAUDE.md`](../../CLAUDE.md) mandates `cli/dlc10` while forbidding
`openFPGALoader` "because it cannot drive the `0x03FD` Xilinx cable" — **this
project has no `0x03FD` cable**, `dlc10` accepts no `--busdev-num` and cannot
address the Digilent `0403:6014` cables at all, and first-party `t27c fpga-flash`
wraps `openFPGALoader`.

---

## 5. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | 0 errors |
| load criterion | `Done 0x0` under wrong part → `done 1` under ours, **all three boards** |
| **USER3 readback** | **magic + `ok=1`, 18/18 positive, 9/9 negative under control** |
| synthesis | 1 BSCANE2, 1 STARTUPE2, 122 LUT, 50 FDRE |
| `fasm2frames` | rc 0, 20,230 frames, **7 BSCAN FASM lines** |
| `t27c boards` | three cables, `idcode 0x3636093`, artix a7 200t |

---

## 6. What is NOT done

- **`gen-verilog` still emits test blocks.** 87.2% of the corpus cannot be placed
  without `delete t:$print`.
- **The `JTAG_CHAIN`/site agreement is not enforced anywhere** — it is a comment
  in one file and a lesson in the tracker.
- **`t27c fpga-flash` still omits `--busdev-num`.**
- **CLAUDE.md's flashing law still names hardware this project does not own.**
- **No `gen/zig` backend**; 184 `.zig` files across two repos remain uncheckable.
- **Three leaked credentials remain unrotated** — a hard gate on any
  history-derived training corpus.
- **The five-level alphabet {0, ±1, ±φ}** (T158a) is still unbuilt, and it is the
  only route to a scientific rather than engineering contribution.

---

## 7. Three ways to continue

### Option 1 — **Make the whole path a first-party command**

Everything in §1–§4 was assembled by hand in a scratch script. The pieces that
made it work — `delete t:$print; delete t:$scopeinfo`, the port-less top, the
frames gate, `--busdev-num`, the `JTAG_CHAIN`/site check, the A/B/A control —
exist only as a shell file outside the repository.

- **Cost:** low. `t27c path` already sequences spec → Zig → Verilog → iverilog;
  this extends it to → FASM → frames → bitstream → **read**.
- **Risk:** low; every stage has now been exercised once and its failure mode
  recorded.
- **Confirming measurement:** `t27c silicon <spec>` returns the verdict word and
  **fails** when the control bitstream is loaded.

### Option 2 — **Fix `gen-verilog` to emit synthesizable Verilog**

T167 says 87.2% of generated Verilog cannot be placed. `gen-verilog-for-simulation`
already exists to carry the tests.

- **Cost:** medium — and it moves what `corpus` measures, so the forecast must be
  written before the change.
- **Risk:** the `iverilog`-clean count will fall, because the testbench is what
  `iverilog` was accepting. **That is a correction, not a regression**, and
  saying so in advance is the whole point of T44.
- **Confirming measurement:** the count of specs whose Verilog nextpnr places
  rises from 56 toward 444; the `iverilog` count changes and the report says by
  how much and why.

### Option 3 — **Build the five-level alphabet {0, ±1, ±φ}**

T158 showed the pure-φ claim is empty; T158a named the non-empty one. The silicon
path is now open end to end, so a new datapath can be *measured on hardware*
rather than argued about.

- **Cost:** medium. One extra add and a two-word register in the contribution
  path; spec, golden and miter already exist for the three-level case.
- **Risk:** it needs an accuracy number on a published benchmark to mean
  anything, and this project has never produced one.
- **Confirming measurement:** the reducibility test **fails** — the five-level
  net's outputs are not a per-layer rescale of any three-level net — and the
  verdict word still reads `ok = 1` off silicon.

**Recommendation: Option 1, then Option 2.** The path exists but is not
reproducible by anyone but the wave that walked it; a result that lives in a
scratch script is one `rm -rf /tmp` from being a claim again. Option 2 is the
largest correctness debt now visible, and Option 3 is worth starting only once
the pipeline it depends on is a command rather than a memory.

**φ² + φ⁻² = 3 | TRINITY**
