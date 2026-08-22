# NOW — the three remaining #2348 harnesses land, and one of them is a null result

Last updated: 2026-08-22

## Differential harnesses for #1977, #1985 and #2006 (Refs #2348)

- Branch: `sim/2348-harnesses`
- Issues: #2348, #1977 (PR #2337), #1985 (PR #2340), #2006 (PR #2344)

### What landed

Three testbenches in `sim/`, following the shape #2379 set for
`tb_bitnet_dma_write_address.v`, plus the README sections that document them.

- `tb_bitnet_sequencer_zero_count.v` — the `layer_sequencer` that never left
  RUN for a zero count. Reproduces the published numbers exactly: 200,000
  cycles with no `done`, `neuron_id` reaching **50,000**, and six non-zero
  controls byte-identical old vs new.
- `tb_bitnet_prefetch_done_pulse.v` — `prefetch_done` as a level rather than a
  one-cycle pulse. Reproduces `t2 sampled_done` OLD=1 / NEW=0, `done_rises`
  2/2, `we_count` 4/4.
- `tb_bitnet_dma_we_default.v` — **three-way**, because its claim is that the
  #2006 fix changes nothing observable.

Every emitter compiles standalone under `rustc` with a four-line driver — no
`cargo`, no target directory. The only `use` in any of them is `use super::*`
inside `#[cfg(test)]`. The recipe is in `sim/README.md`.

### The null result, and why it needed a third rendering

#2006 defaults `local_we` low ahead of the `case`. Pre-fix and post-fix are
observationally identical: every reachable path already drove the strobe, and
the states that never mention it are never entered with it high. A harness that
passes by finding no difference proves nothing — one wired to the wrong ports
finds no difference too.

So the harness elaborates three renderings: A (pre-#2006), B (#2006), C
(#2006 + #2003). B and C are consecutive revisions — PR #2344's head rendering
is byte-identical to PR #2345's base — so one comparator sees all three. **A vs
B must be identical; B vs C must differ.** Same comparator, same stimulus, same
run, same 293-bit vector. Substituting B into the C slot makes the run fail,
explicitly refusing to certify its own null result.

Measured: A vs B `0` mismatching cycles, B vs C `266`, over 373 cycles and
seven stimulus phases.

### Biting

One mutant per guard, each failing with a quoted message:

- Guard on `num_neurons==0` only → case 1 passes, case 2 fails: the two zero
  compares are independently guarded.
- Guard forced to `1'b1` → cases 1 and 2 pass, **all six** non-zero controls
  fail. The controls are what stop the fix being a licence to retire every
  request.
- `prefetch_done` clear moved back inside the start guard while keeping the new
  `IDLE: begin` syntax → fails. The harness measures ports, not emitter text.
- BRAM address stride broken while the flag stays correct → fails the
  surgical-ness control.
- `READ_DATA`'s `end else local_we <= 1'b0;` deleted from both DMA renderings →
  A emits **22** local writes against B's **18**. #2006 is latent today and a
  real backstop the moment an arm stops clearing the strobe.

Anti-vacuity in every harness: the fixed rendering placed in the *old* slot
makes each one fail, so none can pass against a non-defective "before".

### Honesty bounds (BINDING)

- These are **reporting** instruments, **not gates**. `vvp` exits 0 on
  `RESULT: FAIL` as well as `RESULT: PASS`, and no workflow runs them — `cargo
  test -p t27c` is invoked by none (#2292) and `fpga-build.yml` never calls
  `vvp` (#2241). Wiring `sim/` into CI is tracked separately. No claim is made
  that anything here defends `master`.
- Three published numbers initially failed to reproduce. **All three were
  faults in my instrument, not in the claims**, and each was diagnosed before
  any assertion was adjusted:
  - A posedge-sampled observer read registered outputs before their
    non-blocking update and reported `max_nid=49999` while the port plainly
    read 50000. `200000/4` confirms 50000 independently. Observers moved to the
    negedge.
  - A negedge observer read `d_start` on the same negedge the stimulus assigned
    it — undefined order — and reported `t2 sampled_done=0` for both
    renderings. The sample moved into the driving process.
  - `first_chunk`/`last_chunk` are **absent from the reset block of both
    renderings**, so both sit at X until the first RUN cycle and `X !== X`.
    Those two are now compared only while `valid` is high, with a
    `qual_cycles` counter asserted equal to `num_neurons*num_chunks` so the
    qualification cannot void the check.
- A fourth bug was found only by *running a mutant*: `$display` two-string
  continuation together with format args printed `"expe"` as `1702391909`.
  Failure messages that are never exercised are not known to work.
- The zero-chunk half of #1977 is asserted separately from the zero-neuron
  half. The published claim covered only the zero-neuron case; the second is
  additional, and its numbers (`chunk_id` high-water 255, final 64 = 200000 mod
  256) are measured here, not quoted from the PR.
