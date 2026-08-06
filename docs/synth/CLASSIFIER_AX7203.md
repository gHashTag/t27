# Synthesizing & flashing the GF-T classifier on AX7203 (Artix-7 XC7A200T)

A copy-paste runbook to take `specs/ternary/gft_classifier4.t27` — the end-to-end
GF-T classifier (`activations → MLP → GF-T logits → argmax → class`) — from spec to
a bitstream running on the ALINX AX7203 board. The open-source flow (yosys +
nextpnr-xilinx) needs no Vivado; the final **flash** step is owner-gated (it drives
the board over JTAG and needs the user at the hardware).

The module is purely combinational (`on_comb`): 16 trit weights + 4 GF-T16
activations in, one 8-bit class index out. To exercise it on real silicon you wrap
it with a tiny top that presents fixed weights and streams activations (or drives
inputs from UART), same pattern used for `gft_dot2` on AX7203 (see PR #185).

## 0. Prerequisites

- `t27c` built (`cargo build --manifest-path bootstrap/Cargo.toml --bin t27c`)
- `yosys` (≥ 0.36), `nextpnr-xilinx`, `prjxray` db for xc7, `openFPGALoader`
- The AX7203 truth from memory `fpga_fleet_physical_link`: CP2102N UART @115200,
  true 200T IDCODE `0x13636093` (the checked-in `IDCODE.md` is wrong — do not trust it).

## 1. Emit synthesizable Verilog from the spec

```bash
t27c gen-verilog specs/ternary/gft_classifier4.t27 > build/gft_classifier4.v
```

The generated module `GftClassifier4` has ports `clk, rst_n, en, wh0_0..wh1_3,
wo0_0..wo3_1, a0..a3, ready, result[7:0]`. It is combinational — `clk/rst_n/en`
are tie-off compatible (`ready` is constant-high).

## 2. Synthesize (yosys, no Vivado)

```bash
yosys -p "read_verilog -sv build/gft_classifier4.v; \
          hierarchy -top GftClassifier4; \
          synth_xilinx -family xc7 -flatten; \
          write_json build/gft_classifier4.json; stat"
```

`stat` prints the LUT/CARRY4 counts. The arithmetic is deep (four sadd/RNE trees +
argmax), so `-flatten` synthesis is slow on a laptop — budget several minutes, or
drop `-flatten` for a faster hierarchical run.

> Measured area is appended to `docs/SYNTH_REPORT.md` when a run completes; the
> combinational logic is dominated by the GF-T signed-add (magadd/magsub) trees.

## 3. Place & route (nextpnr-xilinx)

```bash
nextpnr-xilinx --chipdb xc7a200t.bin \
  --xdc constr/ax7203_classifier.xdc \
  --json build/gft_classifier4.json \
  --fasm build/gft_classifier4.fasm
xc7frames2bit --part_file .../xc7a200tfbg484-1/part.yaml \
  --frm_file build/gft_classifier4.fasm build/gft_classifier4.bit
```

### Constraints (`constr/ax7203_classifier.xdc`, minimal template)

The classifier is combinational, so you only need a clock for the UART/driver
wrapper and pins for whatever you route out. For a self-checking on-board test that
drives fixed inputs and blinks an LED on the expected class, tie the weights/
activations to constants inside a `top` and map only `clk` + `led`:

```
# AX7203 200 MHz differential sys clock (bank pins per ALINX schematic)
set_property -dict {PACKAGE_PIN <CLK_P> IOSTANDARD LVDS} [get_ports clk_p]
set_property -dict {PACKAGE_PIN <CLK_N> IOSTANDARD LVDS} [get_ports clk_n]
# A user LED to signal PASS
set_property -dict {PACKAGE_PIN <LED0>  IOSTANDARD LVCMOS33} [get_ports led]
```

Fill `<CLK_P/N>` and `<LED0>` from the ALINX AX7203 schematic (bank assignments
differ per board rev — verify against the board you have, per the debugging
doctrine: RTFM/schematic before poking).

## 4. Flash — OWNER-GATED (needs the board)

```bash
openFPGALoader -c ft2232 build/gft_classifier4.bit     # volatile, RAM
# or to the onboard SPI flash (persistent):
openFPGALoader -c ft2232 -f build/gft_classifier4.bit
```

> ⚠️ This step drives the physical board over JTAG. It is **not** run autonomously.
> The claim to verify on silicon is the same as PR #185: stream a batch of the
> committed conformance vectors (`bootstrap/tests/gft_classifier4_vectors.txt`) into
> the fabric and confirm the returned class index is **bit-exact** to the expected
> column — 1500/1500 in simulation, target 1500/1500 on-air.

## 5. On-silicon conformance (what "done" means)

The bring-up mirrors the GF-T dot-product silicon proof:
1. Wrap `GftClassifier4` with a UART loader: host sends 16 weights + 4 activations,
   fabric returns the 1-byte class index.
2. Host replays `gft_classifier4_vectors.txt` and diffs the returned index.
3. **PASS = every vector's class index matches** — the same doubly-grounded oracle
   already green in simulation (`cargo test --test gft_classifier4`).

Until then the classifier is **proven in simulation (iverilog, 1500/1500 bit-exact),
synthesizable (yosys), and awaiting a supervised bitstream run on the board.**
