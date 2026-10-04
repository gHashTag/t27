# NOW -- frames to .bit from the specs, byte-identical to xc7frames2bit (2026-10-03)

## specs/xilinx7/packets.t27 + bitwalk --write (Refs #5608)

- `packets.t27` gains the packet sequence xc7frames2bit writes around the frame
  data: 42 steps (`SEQ`, kind / register / value), from the preamble to DESYNC
  and the 400 trailing NOOPs. Each step cites prjxray c9f02d857
  (`bitstream_writer.cc:27-30`, `configuration.cc:301-470`). The test checks it
  against what one xc7frames2bit bitstream holds: 57 words before the FDRI
  type-1 header, sync at word 12, 524 words after the frame data, and the
  register words for COR0, IDCODE, CMD and FAR. COR0's value is checked as a
  literal. A test that compared it with the named constant passed under a
  mutant that changed the constant; the mutation catalogue found that.
- `bitwalk --write FRAMES OUT --part_file part.yaml --part_name NAME` reads a
  prjxray `.frames` file. It places each frame at `fdri_index` from `far.t27`
  and seals it with `ecc_after` from `frames.t27`. Then it emits `SEQ` around
  the data and the .bit header. Only the header and the file I/O are hand-written
  Rust; every word of the stream comes from a spec.
- `bitwalk --frames BIT OUT` goes the other way: it walks a .bit and writes the
  frames that carry data, at their walked addresses.

## Measured

- 8 of 8 local xc7a100t frames/bit pairs that xc7frames2bit wrote: our .bit is
  **byte-identical**. The header's date and time are taken from the reference.
- 6 Vivado xc7a35t bitstreams and 2 xc7a200t bitstreams: dump the frames, then
  write them with both tools. Our .bit is byte-identical to xc7frames2bit's in
  all 8 cases. Our FDRI payload is also identical to the original bitstream's,
  Vivado's included.
- One input that must be refused. Frames from a xc7a100t design were handed
  over as xc7a200t. 192 of their addresses do not exist on the 200T.
  xc7frames2bit exits 0 and writes 24272 frames instead of 24080; every frame
  after the first foreign address lands one or more frames late. Cause:
  `readFrames` inserts any address into the map without checking it against
  the part (`frames.h:103-106`, unchanged since bb400b4f1 in f4pga and openXC7).
  `bitwalk --write` names each rejected address and exits 1. Reading the
  xc7frames2bit output, `bitwalk` reports "FAR walk 24080 frames (FDRI 24272,
  off by 192)".
- The loop now runs this as an audit gate (`tri x7-audit`, writer sweep
  16/16, wrong-part refused). It is also a mutation gate. 16 of 16 seeded
  defects are caught. Four are new: LFRM NOOP count, COR0 bit, type-2 count,
  and the index starting at part 0. The last is caught only by the 100T/200T
  writer comparisons, because every Vivado reference here is a 35T.

## What this does not show

- No Vivado reference for xc7a100t or xc7a200t is used anywhere here. On those
  parts the claim is "same bytes as xc7frames2bit", not "same as Vivado".
  The xc7a35t claim is: the same FDRI payload as Vivado wrote.
- No FASM to frames step yet. That half is still prjxray's `fasm2frames` (or
  fpga-assembler). This work replaces only the frames to .bit half.
- No board run of a bitstream written by `bitwalk --write`.
