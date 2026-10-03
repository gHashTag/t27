# NOW -- FASM to frames from the specs, byte-identical to fasm2frames (2026-10-03)

## specs/xilinx7/frames.t27 + bitwalk --fasm (Refs #5608)

- `frames.t27` gains where a segbit lands in a frame: `seg_pos`, `seg_below`,
  `pos_in_frame`, `seg_in_window` and `bit_merge` (set / clear / conflict).
  Each cites prjxray c9f02d857 (`tile_segbits.py:161-167`,
  `tile_segbits_alias.py:34-41`, `fasm_assembler.py:79-137`). Two rules are
  fasm2frames behaviour, not device rules, and the spec says so. A bit below
  the frame start wraps to its end, because Python reads index -1 as the last
  word. A bit past word 100 is dropped with a message on stderr, and the exit
  code stays 0. 13/13 tests, including positions on real 100T tiles
  (CLBLM_R_X3Y0, LIOB33_SING_X0Y0 shift 64, LIOB33_SING_X0Y149 offset 99).
- `bitwalk --fasm DIGEST FASM OUT [--strict]` reads FASM and writes prjxray's
  non-sparse `.frames` text. The digest is prjxray-db as prjxray's own
  `Database` renders it: tiles, aliases, segbits, ppips, required features and
  IO banks. The hand-written Rust does the lookup and the text: the FASM
  grammar, canonical features, alias and ppip lookup, required features, the
  STEPDOWN pass and the error order. Every bit position comes from
  `frames.t27`.

## Measured

- 20 of 20 corpus files against prjxray `fasm2frames`:
  - 18 accept files, `.frames` byte-identical: 7 local xc7a100t designs (nextpnr), 6 Vivado
    xc7a35t bitstreams taken back to FASM by `bit2fasm`, and 5 synthetic files. The synthetic
    files cover absent SING sites, STEPDOWN banks, ranges, hex / bin / dec / oct literals,
    comments, annotations and ppips, on 100T and 35T.
  - 2 refuse files, refused for the same reason by both tools: a set/clear conflict on one bit,
    and a feature the database lacks.
- The 5 reference-parity cases of lromor/fpga-assembler#49 (`gold.frames`,
  head 4d75e906). 4 of 4 runnable cases match:
  - the missing feature, with the same error text;
  - zynq7 required features: 3 bits, also byte-identical to `fasm2frames`;
  - the PUDC_B case without injection;
  - the 83-bit `RXCDR_CFG` value.

  Case 06 (kintex7 pseudo PIP) was not run, because no kintex7 database is local.
- Time on this machine, with `fasm2frames` timed when the corpus was built:
  - all 20 files: about 2.0-2.6 s ours against 62 s;
  - one real 100T design: about 0.07-0.11 s against 1.2-1.6 s, of which about 50 ms is reading the
    digest;
  - large synthetic files: about 0.1 s against 6-19 s, where the textX parser dominates.

  The 8-11 s first recorded for `gf16_heartbeat_top` was not reproduced; it was a loaded machine.
- Mutation gate: 9 of 9 seeded defects in `frames.t27` fail both its tests
  and the FASM sweep. Two of them first survived, because the oracle had gaps:
  - The conflict file also held a missing feature, so a mutant that ignored conflicts was still
    refused, for the wrong reason. The sweep now checks the reason.
  - An off-by-one in the tile window changed no file's accept or refuse result. The sweep now
    requires that bits outside the tile (`seg_in_window`) equal bits wrapped plus bits dropped
    (`seg_below`, `pos_in_frame`), for every file.

## Found

`fasm2frames` does not refuse a feature of a site that a `*_SING` tile lacks.

- **Bottom SING tiles, for example LIOB33_SING_X0Y0.** The Y1 site's bits sit below the frame
  start. They wrap silently into words 99-100 of the same frame, and those words belong to
  LIOB33_SING_X0Y49, LIOI3_SING_X0Y49 and INT_L_X0Y49.
- **Top SING tiles.** The Y0 site's bits are dropped with a stderr line, and the exit code
  stays 0.

The synthetic corpus hits this on purpose, with 226-309 wrapped and 272-361 dropped bits per
file. None of the 13 real designs hits it, because nextpnr names SING sites by y % 2.

`bitwalk --fasm` reproduces the Python bytes by default. With `--strict` it refuses the file
and names the first line whose bit falls outside its tile.

## What this does not show

- Required features are tested on zynq7 only, in one gold case. No artix7 part has any.
- No xc7a200t FASM is in the corpus.
- No kintex7 database is local, so the #49 alias / pseudo-PIP case 06 was not run.
- fpga-assembler was not timed on this machine. Its 13-112x figure is its own.
- No board run of a bitstream from `bitwalk --fasm` + `bitwalk --write`.
