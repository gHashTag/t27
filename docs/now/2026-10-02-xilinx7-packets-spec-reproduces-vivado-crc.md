# NOW -- the 7-series packet stream as one spec, and the CRC Vivado writes (2026-10-02)

## specs/xilinx7/packets.t27 -- packet grammar, registers, FAR, CRC, COR0 (Closes #5608)

- A `.bit` after its sync word is a stream of Type-1 / Type-2 packets. Until now
  this repo read it in two hand-written places: `patch_cor0` in
  cli/tri/src/fpga.rs, and scripts/dump_bit_config.py behind `bit_config`. The
  spec puts the rules in one module: header decode, opcodes, Series-7 register
  and command numbers, FAR fields, the running CRC, the COR0 field table and a
  `resealed` rule. Each section cites prjxray c9f02d857 by file and line.
- The CRC rule is CRC32C (0x82F63B78), fed 32 data bits and then the 5-bit
  register address, LSB first. A write to CRC compares and resets to 0; CMD
  RCRC resets to 0; reads are not folded in. The test oracle is a value Vivado
  wrote (0xE3AD7EA5), not one the spec computed. 12/12 tests pass under
  `t27c test-report`.
- Driver: specs/xilinx7/bitwalk.rs, built from `t27c gen-rust`. It does the
  I/O and the loop; every decision is a spec function. Over 6 Vivado
  bitstreams (2 local designs and the 4 prjxray-db harness bitstreams) it
  reproduces all 12 CRC checks; bad 0.
- Two openXC7 (xc7frames2bit) bitstreams carry no CRC write at all, so there
  is nothing to check in them.
- `patch_cor0`'s approach, rewriting the COR0 word only, leaves a Vivado
  bitstream with 1 of 2 CRC checks wrong (`bitwalk --cor0 5 --no-reseal`).
  With `resealed` it is 2 words changed and bad 0. This is measured on the
  file only; no board was configured with either output.
- COR0 fields decoded from the table match what dump_bit_config.py printed
  for the same files.
- Not replaced yet: the CTL0 / COR1 field names (no cited source yet), and
  the call sites in fpga.rs, which are a follow-up.
- Speed, corrected. The first draft of this entry said 0.3-0.9 s per 2 MB
  bitstream; that was measured on a machine at load ~45 and is withdrawn.
  With the CRC disabled the walk is ~3 ms, and the bitwise CRC (37 bit steps
  per word) was ~90% of the rest. The spec now carries the table form:
  `CRC32C_TABLE`, `crc_byte`, `crc_word` (4 lookups + 5 bit steps). The
  table is not trusted -- `crc_table_is_derived` recomputes all 256 entries
  from `crc_bit`, and `crc_word_equals_crc_step` compares both forms over
  4096 xorshift pairs; a one-bit change to one entry fails both. One Vivado
  file walks in ~17-22 ms against ~82 ms before, both at load1 ~10-13 on
  8 cores; these are noisy single-machine numbers, not a benchmark.
- Two t27c defects hit while writing it, both worked around in the spec and
  filed with repros: a `var` reassigned in a `test` block is emitted as a
  second `var` (Refs #5606); `gen-rust` emits `if (b) != 0` for a local bound
  to a bool-returning call (Refs #5607).
