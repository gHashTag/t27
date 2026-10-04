# NOW -- the frame ECC as a spec, checked on every Vivado frame (2026-10-02)

## specs/xilinx7/frames.t27 -- frame ECC (Refs #5608)

- Every 7-series configuration frame is 101 words, and word 50 carries a
  13-bit ECC in its low bits. A writer has to produce it. Until now the only
  statement of the rule we use was prjxray's `ecc.cc`, called by
  xc7frames2bit. `frames.t27` writes it as a spec, citing prjxray c9f02d857 by
  file and line: each set bit of the frame XORs its own 13-bit code into the
  running value (bit offset plus 0x1320 / 0x1340 / 0x1360 by word range),
  word 50's ECC bits are left out, and after word 100 the parity of the low
  12 bits goes into bit 12.
- Oracle, in the spec: three frames copied word for word from a Vivado
  bitstream (`A_direct_top.bit`, FDRI frames 833, 1528 and 1605) and the ECC
  Vivado stored in them. Between them they cover all three code ranges, data
  in word 50 above its ECC bits, and a nonzero parity step. 8/8 tests pass
  under `t27c test-report`.
- For comparison: fpga-assembler (lromor/fpga-assembler, the C++ rewrite of
  this back half) tests the same function with six hand-computed vectors
  (`fpga/xilinx/arch-xc7-frame_test.cc:31-44`); its test file has no frame
  from a vendor bitstream. Those six vectors are now a test here too, so the
  spec is checked against both that implementation and Vivado.
- Oracle, through the driver: `bitwalk` now recomputes the ECC of every FDRI
  frame with the generated module. Over the 6 Vivado bitstreams (2 local, 4
  prjxray-db harness) that is 32,520 frames, bad 0. Most of those are empty;
  the frames with data are 77-244 per file (752 in total), and those are where
  the check has teeth.
- The openXC7 bitstream in the same run (`hb.bit`, 9,464 frames) is also
  bad 0, but that is not independent evidence: its ECC comes from the same
  prjxray code this spec was read from.
- Mutation check. Four deliberate defects, each run through the tests and the
  sweep: code offset off by one, word 50 unmasked, parity step dropped, range
  boundary moved from word 37 to 38. Each fails at least one test and leaves
  the sweep with bad > 0. The first draft had only two Vivado frames, and
  dropping the parity step failed neither of them; frame 1605 was added
  because of that. The moved boundary is caught by the hand test and by one
  of six bitstreams only (basys3) -- word 38 is rarely set in these designs.
- Not done here: writing frames. This is the check a writer will need; the
  writer itself (frames -> .bit, byte-identical to xc7frames2bit) is next.
