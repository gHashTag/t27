# FPGA Hardware & Toolchain — Single Source of Truth (SSOT)

> **Status:** authoritative. Last verified 2026-05-31 on the developer Mac
> (Darwin arm64). When any other FPGA doc disagrees with this file, **this file
> wins** — fix the other doc, do not fork the facts here.
>
> Scope: physical board, JTAG cable, host toolchain, and the program/flash path
> for the GoldenFloat (GF16) RTL. Numeric-format truth lives separately in
> `conformance/FORMAT-SPEC-001.json` + `specs/numeric/` (see bottom).

---

## 1. Target board (the one we build & flash for)

| Field | Value |
|-------|-------|
| Board | **QMTech Wukong V1** |
| FPGA | **XC7A100T-FGG676** |
| Vivado part string | **`xc7a100tfgg676-1`** |
| JTAG IDCODE | **`0x13631093`** (XC7A100T) |

`Arty A7-100` (`xc7a100t-csg324`, `specs/boards/arty_a7.t27`) is a **different**
board — not the flash target. Do not mix its `csg324` package into build/flash
flows for the Wukong.

All Vivado TCL (`fpga/vivado/build*.tcl`) and SPI-flash helpers
(`fpga/tools/*_xc7a100t*fgg676*.bit`) already target `fgg676`. Keep it that way.

> Note: an earlier memory/deck claim of "XC7A200T" was wrong — the real board is
> A100T.

---

## 2. What is physically connected (via Terminus USB2.0 hub)

| Device | USB VID:PID | Role |
|--------|-------------|------|
| Xilinx Platform Cable USB II (Digilent DLC10) | `0x03FD:0x0013` (pre-FW), `0x03FD:0x0008` (after firmware load) | JTAG programmer |
| DSLogic Plus (DreamSourceLab) | `0x2A0E:0x0035` | Logic analyzer (JTAG capture) |

There is **no `/dev/cu.usb*` / `/dev/tty.usb*` serial node**, and there should
not be: both devices speak **libusb**, not UART/VCP. Absence of a serial port is
**not** "board not connected." Verify presence with `ioreg -rc IOUSBHostDevice`.

DSLogic capture config: `fpga/diagnostics/dsview_jtag_config.json`.
JTAG header pinout: `fpga/diagnostics/jtag_wiring.md` (pinout table only — its
tooling/IDCODE sections are stale; see §6).

---

## 3. Program / flash path (CANONICAL, local, no Vivado)

The cable is a **native Xilinx cable (`0x03FD`)**. Drive it with the in-repo
pure-Rust driver:

**`cli/dlc10`** — `rusb`/libusb driver for DLC10/DLC9, JTAG + SPI flash via a
7-series proxy. No prebuilt binary; build it:

```bash
cargo build --release -p dlc10        # from repo root
```

Subcommands (`src/bin/dlc10.rs`):

| Command | Purpose |
|---------|---------|
| `dlc10 idcode` | Read JTAG IDCODE → **must be `0x13631093`** (confirms XC7A100T alive) |
| `dlc10 sram <file.bit>` | Program FPGA SRAM (volatile, fast iteration) |
| `dlc10 flash <file.bit>` | Program on-board SPI flash (non-volatile) |
| `dlc10 reload` | JPROGRAM + JSTART (reload FPGA from flash) |
| `dlc10 read-id` | SPI flash JEDEC ID via JTAG→SPI bridge |
| `dlc10 debug` | Decode 7-series config registers |

Typical bring-up: `cargo build --release -p dlc10` → `dlc10 idcode` →
`dlc10 flash <bitstream>` → `dlc10 reload`.

### Do NOT use openFPGALoader for this board
`openFPGALoader` (v1.1.1, installed via brew) **cannot drive the `0x03FD` cable**.
Its cable DB is FTDI (`0x0403:*`) / CMSIS-DAP / J-Link / XVC only — there is no
`0x03FD` entry, so `--detect` fails with "device not found". Dead end; use
`dlc10`.

---

## 4. Synthesis toolchain (how to get a `.bit`)

There is **no native macOS Vivado** (AMD ships Vivado for Linux/Windows only;
`trinity/fpga/install_vivado.sh` claiming "OS: macOS" is wrong). No Vivado, no
yosys/nextpnr is currently on PATH. Docker is available (v29.x).

Two options, but **past experience (`docs/fpga/`, issue #592) makes the choice
clear by design class:**

- **(B) OpenXC7** (`yosys` + `nextpnr-xilinx` + `prjxray`) — native arm64, open,
  no account. **PROVEN to build user-pin designs** on `xc7a100tfgg676`: chipdb
  builds, nextpnr routes to ~254 MHz, and `fpga/openxc7-synth/` already holds
  working `.bit` files (`test_top`, `blink_j26`, `find_led`,
  `phi_temporal/temporal_heartbeat`). **This is the path for the GF16 matrix**
  (a user-pin design — ring osc + LEDs, no STARTUPE2/config pins).
  Per `docs/fpga/OPENXC7_FGG676_STATUS.md`, OpenXC7 **only fails** on designs
  using dedicated config pins (FCS_B=C8/MOSI=B19/MISO=A18) + STARTUPE2 — i.e. the
  SPI-flash *proxy* `bscan_spi_qmtech` (nextpnr `pack_clocking_xc7.cc` aborts with
  `std::out_of_range`). Our matrix does **not** use those, so OpenXC7 works.
  **VERIFIED 2026-05-31: the recipe in §8 built `gf16_matmul4x4_top.bit` and it
  reaches `DONE=HIGH` on the board.**
- **(A) Vivado in Linux Docker** — only needed for the **SPI-flash proxy**
  bitstream (Vivado-only in the OSS ecosystem). Setup exists
  (`docker/Dockerfile.vivado` 2025.2, `tri fpga build-proxy-docker`) but is
  **currently non-functional**: the image was never persisted (no `t27/vivado`
  image present), the Xilinx auth token expired ~2026-05-19, and host disk is
  tight (~24 GiB free vs ~25-30 GiB peak). Avoid unless non-volatile SPI flash is
  truly required.

**Loading without the proxy:** use `dlc10 sram <bit>` (volatile) to run a design
immediately — this bypasses the broken SPI-flash path entirely. The current
`fpga/tools/bscan_spi_xc7a100t.bit` is an OpenXC7 user-pin fallback that loads but
never reaches `DONE=HIGH` (STAT=0x0), so `flash-id` returns `00 00 00` instead of
Micron `20 BA 18` — non-volatile flash is **known-broken** pending a real proxy.

---

## 5. The GoldenFloat matrix design (current FPGA task)

- RTL: `fpga/vivado/gf16_matmul4x4_top.v` → `gf16_matmul4x4` (16× `gf16_dot4`) →
  `gf16_dot4` (4× `gf16_mul` + 3× `gf16_add`).
- Self-check: top computes `A × I` and lights LEDs when result == `A`
  (`diag_ok & off_zero`). LED pins: **R23, T23** (`gf16_matmul4x4_top.xdc`).
- Build flow: `fpga/vivado/build_gf16_matmul4x4.tcl` → `gf16_matmul4x4_top.bit`.
  (`build_gf16.tcl` builds only the single-`gf16_top` design — not the matrix.)

---

## 6. Known-stale docs corrected by this SSOT

- `fpga/diagnostics/jtag_wiring.md`:
  - references `tools/dlc10_jtag.py` — **does not exist**; the driver is now
    Rust at `cli/dlc10/`.
  - lists IDCODE `0x03631093` — the active driver expects **`0x13631093`**.
  - "ESP32 XVC" path is broken and its firmware is absent — ignore.
- Keep the JTAG **pinout table** in `jtag_wiring.md`; everything else there is
  superseded here.

---

## 7. Numeric formats (separate SSOT — pointer only)

Number-format truth is **not** here. See `conformance/FORMAT-SPEC-001.json`
(L6 numeric SSOT) and `specs/numeric/`. Family: GF4/GF8/GF12/**GF16 (primary)**/
GF20/GF24/GF32, plus `GF64`, `GFTernary`, `TF3`, balanced-ternary `BigInt`.
Open gap: `GF64` exists in `specs/numeric/gf64.t27` but is **not** listed in
`FORMAT-SPEC-001.json` / the 7-member family array — reconcile separately.

---

## 8. VERIFIED OpenXC7 recipe (macOS arm64) — built the matrix, DONE=HIGH

End-to-end flow that produced `gf16_matmul4x4_top.bit` and configured the board
on 2026-05-31. The `tri fpga setup-openxc7-chipdb`/`build-proxy` automation does
**not** fit this branch (it targets `nextpnr-himbaechel`; `openXC7/nextpnr-xilinx`
default `stable-backports` is **classic `nextpnr-xilinx`**), so run the stages by
hand. Hard-won macOS arm64 fixes (each was a real failure):

1. **Branch:** clone `openXC7/nextpnr-xilinx` at **`stable-backports`** (no `master`).
2. **`brew install yosys boost boost-python3 eigen cmake`** — Boost.Python is a
   *separate* Homebrew formula (`boost-python3`); without it cmake errors
   "No version of Boost::Python 3.x".
3. **cmake config:** `-DUSE_OPENMP=OFF` (Apple clang rejects bare `-fopenmp`) and
   `-DCMAKE_CXX_FLAGS=-I$(brew --prefix eigen)/include/eigen3` (Eigen 5.0 ships no
   `EIGEN3_INCLUDE_DIRS`, so `#include <Eigen/Core>` is otherwise not found).
4. **Build:** `cmake --build build --target nextpnr-xilinx bbasm -j` (parallel
   build can spuriously fail once on a generated header race — just re-run).
5. **Chipdb:** `PYTHONPATH=xilinx/python python3 xilinx/python/bbaexport.py
   --device xc7a100tfgg676-1 --xray xilinx/external/prjxray-db/artix7
   --bba build/xc7a100tfgg676.bba` (~70s, 464 MB) then
   `build/bbasm --le …bba …bin` (159 MB). bbaexport is stdlib-only (no numpy/prjxray).
6. **prjxray tools (for FASM→bit):** clone `f4pga/prjxray` + `f4pga/prjxray-db`;
   `cmake -B build -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DPRJXRAY_BUILD_TESTING=OFF`
   (cmake 4.x rejects the old min-policy) then `cmake --build build --target xc7frames2bit`.
   `fasm2frames.py` needs a venv with `pip install fasm pyyaml simplejson intervaltree numpy`
   and `PYTHONPATH=<prjxray repo>`.

**Per-design stages** (matrix = user-pin: ring-osc clock + LEDs):

```sh
yosys -p 'read_verilog gf16_add.v gf16_mul.v gf16_dot4.v gf16_matmul4x4.v gf16_matmul4x4_top.v; \
          synth_xilinx -family xc7 -top gf16_matmul4x4_top -flatten; write_json m.json'
nextpnr-xilinx --chipdb xc7a100tfgg676.bin --xdc gf16_matmul4x4_top.xdc \
          --json m.json --fasm m.fasm --ignore-loops   # ring osc => MUST pass --ignore-loops
python fasm2frames.py --db-root prjxray-db/artix7 --part xc7a100tfgg676-1 m.fasm m.frames
xc7frames2bit --frm_file m.frames --output_file m.bit \
          --part_file prjxray-db/artix7/xc7a100tfgg676-1/part.yaml --part_name xc7a100tfgg676-1
dlc10 sram m.bit         # => STAT 0x401079FC, DONE=HIGH (golden), CRC_ERROR=0
```

Result: 70 LUTs, Fmax 322 MHz, `.bit` 3 825 964 B, `STAT=0x401079FC` (matches the
known-good golden value).

**Correctness verified (2026-05-31, iverilog 13):** all four bench files
(`gf16_{add,mul,dot4,matmul4x4}_tb.v`) pass, and a 262 144-point sweep of
`gf16_mul` (exp=31 grid) vs a float reference now shows **0 failures, max rel err
0.097 %** (< half-ulp for 9-mantissa GF16). This sweep first exposed a real bug:
`gf16_mul.v` declared `mant_rounded` as `[8:0]` but tested `mant_rounded[9]`, so the
rounding-overflow carry (product mantissa rounding up to 2.0) was lost — 189/262144
pairs were ~2× wrong (e.g. 1.002×1.996 → 1.0 instead of 2.0). Fixed by widening
`mant_rounded` to `[9:0]`; board re-synthesized + re-flashed (DONE=HIGH). The A×I
identity self-check never triggered the bug (×1.0 doesn't round-overflow), which is
why it stayed hidden. **The same pattern should be audited in the chip RTL repos
(`tt-gf16-euler` etc.) and the wider GF4..GF256 multiplier portfolio.**
