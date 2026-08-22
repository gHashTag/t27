# NOW — the DMA write-address claim has an instrument again

Last updated: 2026-08-22

## The differential behind PR #2345 is committed as a testbench (Refs #2348, Refs #2003)

- Branch: `sim/2348-dma-write-address-harness`
- Issue: #2348 (artefact), #2003 (the defect)

### What landed

`sim/tb_bitnet_dma_write_address.v` — a differential harness that elaborates the pre-fix
and post-fix renderings of `dma_controller` in one simulation and drives them from
identical stimulus, following the `sim/tb_bitnet_request_overflow.v` precedent set by
#2351. Issue #2348 proposed `bootstrap/tests/rtl/`; `sim/` is used instead, because that
is where the one committed harness already lives and two conventions would be worse than
either.

PR #2345 claimed "OLD writes beats to addr 1..4, `mem[0]` never written; NEW writes 0..3".
That reproduced on the first run and is now re-runnable from the repository:

```
old wrote: 1(=0),2(=1),3(=2),4(=3)
new wrote: 0(=0),1(=1),2(=2),3(=3)
```

Reconstruction needs no cargo build. `bootstrap/src/bitnet_dma.rs` has no `use`
statements and no `crate::` references, so the emitter compiles standalone under `rustc`
at any revision — the exact commands are in `sim/README.md`.

### Honesty bounds (BINDING)

- **This is reporting, not a gate.** `vvp` exits 0 whether the run prints `RESULT: PASS`
  or `RESULT: FAIL` — measured, both ways. No CI job executes it: `cargo test -p t27c` is
  invoked by no workflow (removed by #2292) and `fpga-build.yml` never calls `vvp`
  (#2241). Neither was changed here; #2348 is explicitly about the artefact existing.
- **Biting was demonstrated**, by two planted mutants on the post-fix rendering. Reverting
  the `READ_DATA` pairing fails case 1 with `new lowest address written: got 1, want 0`.
  Deleting *only* the IDLE re-arm of the index leaves case 1 **passing** and fails case 2
  with `new lowest address on second transfer: got 4, want 0` — so case 2 is the only
  check constraining that hunk, and it is not redundant with case 1.
- The harness found three defects in **itself** before it found anything else: an unsigned
  12-bit port compared against an integer `-1` (so the high-water mark never advanced), a
  `[255:0]` label argument silently truncating a 38-character string, and a `start` pulse
  racing the DUT's own sampling at the same timestep. All three are commented at the point
  of fix. The published numbers reproduced *before* any of them were corrected.
- Scope is one of the four PRs named in #2348. #2337, #2340 and #2344 remain without a
  committed harness; #2348 stays open.
- The harness also passes against current master, whose later waves renamed the fix's
  `beat_index` to `word_index`. Every check is made at the ports, so the rename is
  invisible to it — but that is a property of this harness, not evidence about the rename.
