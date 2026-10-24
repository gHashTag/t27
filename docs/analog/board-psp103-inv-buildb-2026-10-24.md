# Board run of the PSP103 inverter, Build B, ss ff fs sf corners, 2026-10-24

Epic #10486 (analog emulation v0.1), success criterion SC-003; task #12050. English, ASCII only.

The tt note (`board-psp103-inv-2026-10-20.md`) is the method and the grammar. The corner notes of PR #12031 (`board-psp103-inv-{ss,ff,fs,sf}-2026-10-24.md`) are Build A: the corner's tables with tt samples in the RAM, where the board's PASS/FAIL is the prediction (for ss 77 PASS and 323 FAIL a round). This note is Build B: the corner's tables and the corner's own samples in the RAM, the form of the tt receipt, where the board's own verdict is PASS on every line. Every number below was measured in this run; what was not measured is said so.

What this document does not say: nothing about speed, power or value. Fmax is nextpnr's post-route estimate for one placement of one netlist. A line of the board is agreement with the spec's fixed-point run of the same tables, not with ngspice; the spec's distance to ngspice is what `mos_psp103.t27` and the tables PRs (#11123, #11214) measure.

## 1. Receipt status

| corner | receipt lines (the corner's own design) | PASS | FAIL | S at N = 18F, three rounds | the pin |
|---|---|---|---|---|---|
| ss | 1216 | 1216 | 0 | `0x987F22AC4132345B` x 3 | `EXP_S_SS_Q` |
| ff | 1218 | 1218 | 0 | `0x836220E2C48AB136` x 3 | `EXP_S_FF_Q` |
| fs | 1219 | 1219 | 0 | `0xA45CC48E565AD2EB` x 3 | `EXP_S_FS_Q` |
| sf | 1217 | 1217 | 0 | `0x67F02DF65E28F63E` x 3 | `EXP_S_SF_Q` |

SC-003 ("for each of the five corners the inverter's board receipt reads PASS") is met by these four runs and the tt run of 2026-10-20 (1217 of 1217): the five corners now have a bitstream and a receipt whose every well-formed line reads PASS, and the fold S at the last step of every complete round equals the pinned word of its corner.

## 2. The run

- Source: the two specs as they stand in PR #12031 at head `750b3dc7078511c80a942e2cf85ebbd26c1b81cf` (`specs/analog/mos_psp103_data.t27`, `specs/fpga/psp103_inv_ax7203.t27`). The only change made for a corner is the one literal `pub const BOARD_CORNER : u32 = N;` in the data spec (TT = 0, FF = 1, SS = 2, FS = 3, SF = 4), done on a scratch copy and restored; no spec was edited in a branch. The committed board spec reads `ex = rom_ex(BOARD_CORNER, idx)`, which is why the corner's own samples are the expected word. `top.v` is `t27b verilog` of the board spec and `data-<c>.v` is `t27c gen-verilog` of the data spec, with the master-built t27c; the four `data-<c>.v` files (101674 lines each) differ in exactly the one line `parameter [31:0] BOARD_CORNER = N;`.
- Toolchain on this Mac: yosys 0.67+post (git b8e7da6f40ae8f552c116bf6c359b07c6533e159); nextpnr-xilinx "Version 0.9.2-107-g7037c948"; prjxray c9f02d8576042325425824647ab5555b1bc77833 with prjxray-db-full 517d66a383676cb971177ea92b0ff3b6ea6e8690; chipdb `xc7a200tfbg484-2.bin` md5 `59f828c441e6e6081bfee63ff994660e`; `openFPGALoader v1.1.1`. The yosys script (synth_xilinx `-flatten -abc9 -nodsp -nosrl`, the cmp2lut techmap, block RAM for `$mem_v2`), the nextpnr, fasm2frames and xc7frames2bit commands are the ones of section 2 of the tt note. They ran as shell cells in the T27 OS pane, with the load below 40 and more than 4 GB of disk free checked first.
- Board and cable: ALINX AX7203, `xc7a200tfbg484-2`. `openFPGALoader --scan-usb` listed the HS2 (ft232H, serial 210512180081) at bus 000 device 004 and an unrelated Digilent Adept cable at 000 002; `-c digilent_hs2 --detect` returned idcode 0x3636093 (artix a7 200t). Loads used `--busdev-num 000:004`. UART: CP2102N, 115200 baud 8N1, pin N15, `/dev/cu.usbserial-130`.
- Programming: SRAM only (no `-f`, no `-m`). No flash and no eFUSE was written.

## 3. The board table

| | ss | ff | fs | sf |
|---|---|---|---|---|
| `data-<c>.v` sha256 | `c3bbc5b82b6efc1982236d18a2ec1f179ffdec9e46867f2cf5654d3d65a45e96` | `299e97ede0d81b40f68173c844b53f7ea95c8777287612bbb8572e3172356c18` | `dbd719098cdd590a457534d10da5a9f50f1c2a5c064348f362bfd54f62ed41f7` | `1a6a6ff81336ca39cc1745e3c003cbdbbf39bc0fd4fd81eb7a4e43935407da61` |
| LUT (yosys, LUT2..LUT6) | 1418 (456, 133, 189, 319, 321) | 1420 (465, 172, 184, 312, 287) | 1417 (465, 164, 164, 298, 326) | 1415 (463, 159, 179, 305, 309) |
| INV (yosys) | 170 | 170 | 170 | 170 |
| FF | 1219 (FDRE 1196, FDSE 23) | 1219 | 1219 | 1219 |
| DSP48E1 | 0 | 0 | 0 | 0 |
| BRAM | 4 RAMB36E1 + 1 RAMB18E1 | same | same | same |
| CARRY4 (yosys) | 207 | 207 | 207 | 207 |
| nextpnr SLICE_LUTX / SLICE_FFX | 2765 / 1219 | 2757 / 1219 | 2754 / 1219 | 2762 / 1219 |
| nextpnr seed | default | default | 3 | 2 |
| Fmax before routing | 76.70 MHz | 77.23 MHz | 74.29 MHz | 75.21 MHz |
| Fmax after routing | **101.70 MHz** (PASS at 100) | **108.26 MHz** (PASS at 100) | **104.18 MHz** (PASS at 100) | **116.73 MHz** (PASS at 100) |
| frames file | 20230 lines | 20230 lines | 20230 lines | 20230 lines |
| bitstream sha256 (the header carries a time) | `b064e40ac017fea5da2232029be767a6a5ce8f2e25237f6ce2e72965ed4ccc81` | `403f8269f96b358124cbcda68fa5b7fdddc3f6c366c0dc925ec97dfb9ec69806` | `cf67d542b2d93a68b09bc468cc41767634f53a584df270bfa0514aab5e31c8b6` | `6f57e0caadc3d0a92731a60f6c9109683a3250c8d7b129af7d84a6f4bee371fe` |

`top.v` is the same for the four (sha256 `50289021534ab13d11d068d31bee9b3746f41c1baaa51075c5a04ac2faff1db5`, 1215 lines). The four yosys runs, the four nextpnr runs, fasm2frames and xc7frames2bit all returned 0.

The fs corner needed more than one placement. Seed 2 reached 96.42 MHz after routing and seed 4 reached 93.55 MHz (both FAIL at 100); seed 3 reached 104.18 MHz and is the bitstream that was loaded. The default seed of fs was not run in this build (PR #12031 reports 88.83 MHz FAIL for its Build A netlist, a different netlist). The seed 2 bitstream of fs was built and not loaded. The LUT counts are of the Build B netlist and differ by a few LUTs from the Build A counts of PR #12031 (ss 1414, ff 1420, fs 1403, sf 1411), as the ROM contents differ; the same holds for Fmax (Build A ss 108.85 MHz against 101.70 here). That the nextpnr-xilinx build used here differs from the one named in the Build A notes was not tested as a cause.

## 4. The captures

One reader per corner, started before the load, 7 s after the load returned; the board was left running the previous corner's bitstream while the next one was loaded (order ss, ff, sf, fs). A line is well-formed when it matches `T27 PSP N=<3 hex> Y=<7 hex> E=<7 hex> S=<16 hex> L=<4 hex> (PASS|FAIL)`; only those are counted. The judge starts at the first well-formed `N=000 ... L=0000` line of the new design (the previous design's tail and one line cut at the switch are not counted); the last line is cut where the reader is killed. Line numbers are of the capture with CR and NUL bytes deleted.

| corner | captured at (+07) | bytes | sha256 of the capture | lines that contain `T27 PSP` | previous design's lines before the start | design starts at line |
|---|---|---|---|---|---|---|
| ss | 23:49:41 | 80256 | `7d88dd19b03d409e50e7b7c1f7b2f0714d782a6dfb0435465fcb665589ed63a5` | 1216 | 0 | 1 |
| ff | 23:50:33 | 81801 | `92e32a0742d16d68d9fad7b72644678dfa911e5aa5de2702e7356e771bff0041` | 1238 | 19 (ss) | 21 |
| sf | 23:51:04 | 81232 | `f94357e7601feb5c7589b0b9c6a68650f81ef7a1cd5b9204d640fbc96b5d0acd` | 1230 | 13 (ff) | 15 |
| fs | 23:52:09 | 81223 | `128dfc46e3d24de92cd95c5a5dbc85aa53afab4f2768c4fc983fe78012b4d911` | 1230 | 9 (sf) | 12 |

The first line of each design:

```
ss  T27 PSP N=000 Y=009C1F2 E=009C1F2 S=000000000009C1F2 L=0000 PASS
ff  T27 PSP N=000 Y=00BA810 E=00BA810 S=00000000000BA810 L=0000 PASS
fs  T27 PSP N=000 Y=00A080B E=00A080B S=00000000000A080B L=0000 PASS
sf  T27 PSP N=000 Y=00B35BF E=00B35BF S=00000000000B35BF L=0000 PASS
```

Y is the board's sample and E the expected word of the corner: they are equal on every counted line, which is what PASS means here (in Build A of PR #12031 E is the tt word, so ss shows `E=00A8451` for the same step). No line of the previous design was counted: the previous corner's pinned word does not occur in the counted lines of the ff, sf and fs captures, and the tt pinned word `0x215DC5EAC4C48576` does not occur in the ss capture (what the board ran before the ss load was not recorded).

The judge was `grep` only: the count of well-formed lines, of `PASS` and `FAIL` suffixes, and the `N=18F` lines with their S column. Not checked: that the three rounds are byte-equal to each other column by column (the S fold at N = 18F is equal in all three, which pins the round), and the L column for gaps.

## 5. Not claimed

No speed claim (nothing here is compared with ngspice's wall time). No power. Each corner is one gate waveform (two levels, an ideal step) and one load (500 fF with dt = 100 ps). The post-route Fmax is nextpnr's estimate, not a measurement of the board. The runs were done once each; no repetition, no temperature or voltage variation. The comparator step on the board (#11380) and the four NOT MODELLED blocks are not part of this note.

## 6. Reproduce

In a shell cell of a T27 OS pane (the load below 40 and the disk above 4 GB), with the two specs of PR #12031 at head `750b3dc7078511c80a942e2cf85ebbd26c1b81cf`: change the one literal `BOARD_CORNER` to the corner's number, run `t27c gen-verilog specs/analog/mos_psp103_data.t27` and `t27b verilog specs/fpga/psp103_inv_ax7203.t27`, then the yosys, nextpnr (`--freq 100`, seed as in the table), fasm2frames and xc7frames2bit commands of section 2 of the tt note, then `openFPGALoader -c digilent_hs2 --busdev-num <bus>:<dev> <bit>` after `--detect`, and read the UART at 115200 baud for 7 s.
