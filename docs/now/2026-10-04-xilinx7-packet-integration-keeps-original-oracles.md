# NOW -- Xilinx7 packet integration keeps original oracles (2026-10-04)

## Restore the sourced packet rules and the corpus ratchet (Closes #5870)

### What was read

- Original PR #5609, head1fc957483e30af028edec0ede84b6d0398f659e8, nine commits by Dmitrii Vasilev; current mastera9e6c7643f0ad4bdfe404dc20e8ee34bac21e974. The original three specs, driver and six receipts remain unchanged.
- The competing master packet skeleton rejects its C-style loop and has wrong opcode/command numbers and a u32-truncated37bitCRC input. The original PR has the sourced packet grammar and reflected CRC32C.
- Original prjxray at c9f02d857: lib/include/prjxray/xilinx/xc7series/crc.h, lib/xilinx/xc7series/ecc.cc and the corresponding ecc.h, retrieved from official f4pga/prjxray and compiled unmodified as an independent oracle.

### What changed

- Merge current master into the original branch and resolve the sole packets.t27 add/add conflict to the exact original PR blob. All three specs and bitwalk.rs remain byte-identical to head1fc957483. Preserve all nine upstream commits/authors, six historical receipts and current-master fixes. Add only this integration receipt.
- No compiler, generated code, classification or failure ledger edit. Three fresh module seals are generated with t27c seal --save and verified: every source/Zig/C/Rust/Verilog hash MATCH, no --force. Generated Rust modules are emitted by t27c, never hand-edited.

### What was verified here

- Three specs fully parse with zero discarded tokens and type errors/warnings. All35existing Zig tests pass: packets12, frames14, far9. Actual generated Rust driver compiles; regenerated modules from the resolved tree match the tested Rust byte for byte. Generated C11 compilation succeeds for all3specs; packets has3parenthesis-style warnings, frames/far none.
- Actual generated Rust agrees with original compiled C++ in37,048observations:11,008CRCwords,25,912ECCwordsteps,128whole101wordframes. Boundary data are0,0xffffffff,0xaaaaaaaa,0x55555555 and each one-hot bit; CRCprevious values0,1,0xffffffff,0x12345678,0x80000000,0x5a5a5a5a at every register0..31. ECCprevious values0,1,0xfff,0x1000,0x1fff,0x1555 at all101indices. Another4096CRC/ECCpairs and128frames use xorshift32 (13,17,5), seeds0x12345678/0x9e3779b9. No generated statements are replaced or removed.
- Three source mutants still compile and are detected against that oracle: CRCpolynomial0x82f63b79 gives10,647mismatches, removedECCwordmask259, idx>=0x25instead of>269.
- Controlled packet file: twoCRCchecks pass; one-bit-corrupted CRC is reported bad. COR0 OSCFSEL0->5 with reseal changes exactly COR0+itsCRC and leaves both checks passing; --no-reseal changes onlyCOR0 and leaves onebadCRC; patching an originally badCRC preserves that badCRC. The independent Vivadotail literal0xe3ad7ea5 remains unchanged. The fulldriver and original C++ validate these five file states.
- Six locally available openXC7/Forge files walk56,784frames total, zeroECCbad and FARsizesagree. None has aCRCwrite, so these files do not proveCRC againstVivado. A seventh path holds a22byteJSONerror and is excluded.
- Types77/77CLEAN/classified77OK; no duplicate group grows (590of4777bodies,169groups). Fast corpus ratchet exits0: primary95/ledger95, no unexpected failures/passes/expiries/discard/gatedrift. Previousmaster was96/95 with packets.parse as the only unexpected failure. No ledger is blessed.

### Not verified here

- Long suite phases are explicitly skipped by --fast. The95known baseline failures and independent seal/ring drift remain visible; this is no all-green full-repository claim.
- The original receipts' six Vivado files,20FASM cases,32,520Vivado frames and writer comparisons were not all replayed here; their historical evidence is preserved, not claimed as new validation.
- No board was configured, no synthesis/timing/RF/power or model inference was measured. cli/tri/src/fpga.rs patch_cor0/bit_config remains unchanged, as in the original issue's stated boundary.
