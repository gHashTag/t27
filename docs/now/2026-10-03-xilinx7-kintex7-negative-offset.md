# NOW -- kintex7 in the FASM sweep, and a tile that starts below its frame (2026-10-03)

## specs/xilinx7/frames.t27 + bitwalk.rs (Refs #5608)

- The kintex7 tilegrid has four tiles with word offset -2: GTX_INT_INTERFACE
  X89Y150, X89Y200, X89Y250 and X89Y300. `bitwalk` read the offset as unsigned
  and panicked. `frames.t27` now says where such a tile's bits go:
  `seg_shift(below_words, shift) = below_words * 32 + shift`, fed into the
  existing `seg_pos` / `seg_below`. New test
  `seg_positions_of_a_tile_below_its_frame`; 14/14.
- No segbits row reaches those tiles, so the corpus cannot exercise the rule.
  It is covered by the spec test only.
- `bitwalk` keeps a second counter, `own_out`: a tile's own bits that still
  leave the frame. It counts only on tiles that start below their frame. The
  sweep then checks `outside the tile + own_out == wrapped + dropped`.

## Measured

- FASM sweep 22/22 byte-identical to `fasm2frames`. The two new files are
  kintex7 xc7k325t synthetic files (tiles, stepdown); the tiles file has 693
  bits outside their tile, 337 wrapped and 356 dropped, the same SING
  behaviour as on 100T (f4pga/prjxray#2574).
- fpga-assembler#49 reference cases 5/5, now including case 06 (kintex7
  pseudo PIP). Case 05, the 83-bit RXCDR_CFG, matches the reference;
  fpga-assembler's own frames for it do not (18 bits missing, 5 extra).
  Emulating its long-literal packing from the most significant end gives
  exactly those 18 and 5.
- Mutation gate: 20/20 caught across the three specs.
- Time, best of 3 on a loaded laptop: all 22 files 3.3-4.4 s ours against
  150.7 s for `fasm2frames` when the corpus was built. The earlier note that
  textX dominates on synthetic files was not profiled; the synthetic files are
  shorter than a real design (about 1,100 lines against 1,785).

## Mistake

The first kintex7 change counted every bit outside the frame as "outside the
tile". That made `outside == wrapped + dropped` true by construction, and the
mutant `seg_window_one_past` survived the sweep. A first split still let
`own_out` absorb that mutant's bit. Counting `own_out` only on tiles that
start below their frame brought it back to caught. A second formula must not
be able to absorb the defect it is there to expose.
