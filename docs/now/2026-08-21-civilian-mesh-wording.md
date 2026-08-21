# NOW -- chore: civilian mesh wording (2026-08-21)

## chore: civilian mesh positioning (Refs #1873)

- `specs/fpga/bpsk.t27` header comment: drop 'drone-mesh' -> 'mesh' (civilian 5.8 GHz mesh radio PHY). Comment-only, no spec logic change
- Refs #1873
- **Known cost, stated rather than hidden: this stales one seal.** `.trinity/seals/fpga_ZeroDSP_BPSK.json` records `spec_hash` `sha256:c03511ce...` which is exactly the hash of `specs/fpga/bpsk.t27` on master today -- the seal is VALID before this change and goes `[stale]` after it, taking `check_seal_coverage.py` from 131 stale to 132
- The seal was **not** repaired by editing `spec_hash` by hand. That field travels with four `gen_hash_{c,rust,verilog,zig}` values that certify what the compiler actually emitted; rewriting the spec hash alone would assert those four still describe the output without anything having re-run codegen to check. A comment-only edit very likely leaves the generated targets identical, but "very likely" is not what a seal records. Re-seal with `tri seal` on a machine with a built `t27c` to clear it
- Entry migrated from `docs/NOW.md` to `docs/now/` (the layout #2298 introduced); the original entry was dated 2026-08-07. The branch's own commit had deleted the heading `# NOW — feat: arbitrary-DEPTH trainer (>2 layers), proven bit-exact + synth (2026-08-07)` while keeping its body, orphaning it; `docs/NOW.md` is restored to master byte-for-byte, so that heading survives
