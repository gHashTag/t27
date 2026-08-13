# Local bitstream flow — Verilog to `.bit` on this Mac, no Vivado, no Docker

**Status:** working, verified end to end 2026-08-13 (W653). Produced
`w653_blinky_200t.bit`, SHA-256 `4272dd6f…`, which matches **none** of the 16
`.bit` files in the tree, and configured all three XC7A200T boards with a
**state transition** proving it took effect.

This file exists because the flow was blocked by a two-line mismatch that no
inventory of installed tools could reveal. Read §1 before concluding the
toolchain is missing.

---

## 1. The blocker, and why "install the missing tool" was the wrong diagnosis

Three artefacts were present on this machine and **no two of them were
compatible** (T70):

| artefact | where | constids |
|---|---|---|
| `nextpnr-himbaechel` + `chipdb-xc7a100t.bin` | `/opt/homebrew/` | wrong **part** |
| `xc7a200tfbg676-1.bin`, 332 MB | `build/fpga/openxc7/` | **784** lines, md5 `774077db…` |
| `nextpnr-xilinx` sources (openXC7 fork, `stable-backports`) | cloned fresh | **786** lines, md5 `537d8427…` |

A binary built from the fork's HEAD rejects the 332 MB database:

```
Assertion failure: The internal IDs of nextpnr are inconsistent with the
supplied chip database ... We recommend regenerating the chip database.
```

**The tool's own advice — regenerate — costs ~1.3 GB on a disk that was 98%
full.** It was not needed.

**The diff between the two constids files is two appended lines:**

```
785,786d784
< X(GE)
< X(BUFR)
```

`constids` are **ordinal**: each `X(name)` takes the next integer. The 784-line
file is therefore a strict **prefix** of the 786-line one, and every ID in the
old database already has the correct numeric value. The assertion fires only
because the chipdb's own extra-constids block starts at index 784 while the
binary has 786 baked in.

### The fix — two lines, one rebuild, zero disk

```bash
# 1. use the constids the database was generated with
cp build/fpga/openxc7/constids.inc <nextpnr-xilinx>/xilinx/constids.inc

# 2. X(GE) is unused; X(BUFR) has exactly one use -- make it dynamic
#    xilinx/pack_clocking_xc7.cc
-  } else if (ci->type == id_BUFR) {
+  } else if (ci->type == ctx->id("BUFR")) {
```

`ctx->id("BUFR")` interns the string at runtime and compares identically.

### Building on Apple silicon

```bash
cmake -S . -B build -DARCH=xilinx -DBUILD_GUI=OFF -DBUILD_TESTS=OFF \
      -DUSE_OPENMP=OFF -DCMAKE_BUILD_TYPE=Release
cmake --build build -j 8
```

**`-DUSE_OPENMP=OFF` is required.** `USE_OPENMP` defaults to `ON` and hardcodes
`-fopenmp` into `CMAKE_CXX_FLAGS_RELEASE`; Apple clang rejects it outright. It
accelerates only the analytic placer.

> **`--test` still fails** (`Assert bel == bel2` in `archcheck.cc:41`) while real
> place-and-route succeeds. The archcheck is stricter than the P&R path. **Do not
> use `--test` as the gate** — use a routed design.

---

## 2. The flow

```bash
REPO=/path/to/t27-worktree
PNR=<nextpnr-xilinx>/build/nextpnr-xilinx
DB=$REPO/build/fpga/openxc7/prjxray-db/artix7
XR=/Users/playom/t27/build/fpga/openxc7/prjxray
VENV=/Users/playom/t27/build/fpga/openxc7/venv

# 1. synthesise (yosys 0.63 is present and works)
yosys -p "read_verilog design.v; synth_xilinx -family xc7 -top top -flatten; \
          write_json design.json"

# 2. place and route
$PNR --chipdb $REPO/build/fpga/openxc7/xc7a200tfbg676-1.bin \
     --xdc design.xdc --json design.json \
     --write routed.json --fasm design.fasm

# 3. FASM -> frames   (NOTE: the venv wrapper is broken -- 'No module named utils')
PYTHONPATH=$XR $VENV/bin/python $XR/utils/fasm2frames.py \
     --db-root $DB --part xc7a200tfbg676-1 design.fasm > design.frames

# 4. frames -> bitstream
xc7frames2bit --part_file $DB/xc7a200tfbg676-1/part.yaml \
              --part_name xc7a200tfbg676-1 \
              --frm_file design.frames --output_file design.bit
```

Measured on the blinky: P&R 6.2 s of router time, 623 FASM lines, 20,230 frames,
9,730,896-byte bitstream.

---

## 3. Loading, and what the load actually proves

```bash
openFPGALoader -c digilent_hs2 --busdev-num 0:4 design.bit
```

**All three cables share serial `210512180081`.** `--ftdi-serial` matches all
three and silently takes the first. Use `--busdev-num` (`0:4`, `0:7`, `0:10`), or
the ioreg `locationID` (`0x110000`, `0x122000`, `0x140000`) for an identity that
survives replug.

### ⚠ `done 1` is not evidence — measured

The boards boot from **Master-SPI flash** and assert DONE unaided. Their resting
`STAT` is `0x401079fc` with `Done 0x1`. So:

| what was loaded | loader says | `STAT` |
|---|---|---|
| nothing (resting) | — | `0x401079fc`, `Done 0x1` |
| a valid 200T bitstream | `done 1` | `0x401079fc`, `Done 0x1` |
| **a bitstream with 4 KB of its payload XOR-inverted** | **`done 1`** | **`0x401079fc`, `No CRC error`** |
| a bitstream for the wrong part (xc7a100t) | — | `0x5000890c`, **`Done 0x0`, `ID Error`** |

**A deliberately corrupted payload produced signals indistinguishable from
success.** Only the wrong-*part* case is caught, by the IDCODE check in the
bitstream header — a check on the envelope, not the contents.

### The acceptance criterion that does discriminate

Make it **falsifiable by the status quo**: force `Done` low first, then load.

```bash
# 1. drive the board to a known-bad state (wrong-part bitstream)
openFPGALoader -c digilent_hs2 --busdev-num $BD fpga/openxc7-synth/blink_j26.bit
openFPGALoader -c digilent_hs2 --busdev-num $BD --read-register STAT | grep '^Done'
#   -> Done 0x0        <-- the criterion can fail

# 2. load the artefact under test
openFPGALoader -c digilent_hs2 --busdev-num $BD design.bit
openFPGALoader -c digilent_hs2 --busdev-num $BD --read-register STAT | grep -E '^Done|ID Error'
#   -> Done 0x1 / No ID error      <-- a TRANSITION caused by this artefact
```

Measured on all three boards, 2026-08-13:

```
0:4    before Done 0x0  ->  after Done 0x1, No ID error
0:7    before Done 0x0  ->  after Done 0x1, No ID error
0:10   before Done 0x0  ->  after Done 0x1, No ID error
```

**This still does not prove which design is resident** — only that *a valid
bitstream for this part* configured the device. Establishing the design's
identity needs a readback, or a design that reports itself over an observable
channel. The blinky has neither.

---

## 4. What is still not covered

- **The package is unverified.** `idcode 0x3636093` identifies the *die*
  (XC7A200T), not the package. `fgg676` vs `fbg484` **cannot be determined from
  software**; the SSOT reasons that `fbg676` and `fgg676` share die and pinout,
  so the prjxray `fbg676` entry is pinout-correct — but if the boards are
  `fbg484`, every pin constraint is wrong. **A human must read the part marking.**
- **No UART pin assignment exists for this package** in any `.xdc` in the tree.
- **Nothing establishes that the three CP2102N bridges are electrically wired to
  any FPGA pin.** `fpga/HARDWARE_SSOT.md` §2 states the opposite verbatim for the
  older single-board setup ("both devices speak libusb, not UART/VCP"). Until
  that is settled by schematic or continuity test, every UART-based plan is
  unfounded.
- **`--test` (archcheck) fails** and is not a valid gate; see §1.

**φ² + φ⁻² = 3 | TRINITY**
