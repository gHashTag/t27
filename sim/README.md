# `sim/` — RTL testbenches for emitted BitNet modules

Testbenches here elaborate the Verilog that `t27c` emits, so they exercise the
emitter output rather than a hand-written copy of it. Nothing in this directory
is generated; the *inputs* to it are.

## `tb_bitnet_request_overflow.v` — issue #2002

Differential harness proving that a maximum-sized request no longer silently
truncates. It instantiates the **pre-fix** and **post-fix** renderings of both
`dma_controller` and `weight_prefetch_ctrl` in one simulation and drives them
from identical stimulus.

The two renderings differ only in the `--module-name` passed to the emitter, so
the comparison is between two renderings of the same design.

### Reproducing

Build `t27c` once (it lands in the *workspace* `target/release/`, not
`bootstrap/target/release/`):

```
cargo build --release -p t27c
T27C=target/release/t27c
```

Emit the post-fix modules from the working tree:

```
$T27C gen-dma-controller        --module-name dma_new > /tmp/dma_new.v
$T27C gen-weight-prefetch-ctrl  --module-name pf_new  > /tmp/pf_new.v
```

Emit the pre-fix modules from the commit before the fix. Build that revision in
a throwaway worktree so the current tree is untouched:

```
git worktree add --detach /tmp/pre <this-commit>~1
cargo build --release -p t27c --manifest-path /tmp/pre/Cargo.toml
/tmp/pre/target/release/t27c gen-dma-controller       --module-name dma_old > /tmp/dma_old.v
/tmp/pre/target/release/t27c gen-weight-prefetch-ctrl --module-name pf_old  > /tmp/pf_old.v
git worktree remove /tmp/pre
```

Run:

```
iverilog -g2005 -o /tmp/tb.vvp sim/tb_bitnet_request_overflow.v \
         /tmp/dma_old.v /tmp/dma_new.v /tmp/pf_old.v /tmp/pf_new.v
vvp /tmp/tb.vvp
```

### What it asserts

The measured property is **no local address is written twice within one
transfer**. That is deliberately independent of the separate off-by-one in
`weight_prefetch_ctrl` (word N lands at address N+1), which shifts the address
set without duplicating it and which this change does not touch.

| case | request | expectation |
|---|---|---|
| 1 | capacity + 1 | old wraps (an address written twice, `done=1` anyway); new writes exactly 4096 distinct addresses and raises `overflow` |
| 2 | in range (32 beats / 32 words) | old and new agree on the write count of every one of the 4096 addresses; `overflow` low |
| 3 | exactly capacity | full transfer, no duplicate, `overflow` low — the clamp must not fire one early |

Case 1 also asserts that the **old** rendering still wraps. Without that the
harness could pass while never reaching the wrap, and would prove nothing.

The AXI read slave holds `rlast` low and streams beats indefinitely, so
termination is decided purely by the DUT's own byte/word counter. That isolates
the counter-to-address path under test; `arlen` being hardwired to `8'hFF`
regardless of `length` is issue #1970 and is not exercised here.

## `tb_bitnet_dma_write_address.v` — issue #2003

Differential harness for the DMA local-write off-by-one: the pre-fix
`dma_controller` raised `local_we` and post-incremented `local_addr` in the same
non-blocking group, so beat N was written at address N+1, address 0 was never
written, and the slot one past the transfer was clobbered.

It instantiates the **pre-fix** and **post-fix** renderings of `dma_controller`
in one simulation and drives them from identical stimulus. The two renderings
differ only in the `--module-name` passed to the emitter.

Every check is made at the module **ports**, so the harness is independent of
the internal register name. A later wave renamed the fix's `beat_index` to
`word_index`; the harness runs unmodified against both.

### Reproducing

`bitnet_dma.rs` has no `use` statements and no `crate::` references, so the
emitter compiles standalone — no cargo build and no target directory are
required:

```
for ref in <base-sha> <head-sha> master; do
  gh api "repos/gHashTag/t27/contents/bootstrap/src/bitnet_dma.rs?ref=$ref" \
     --jq .content | base64 -d > dma_$ref.rs
  printf '#[path = "dma_%s.rs"]\nmod e;\nfn main(){let a:Vec<String>=std::env::args().collect();print!("{}",e::build_dma_controller(&a[1]));}\n' "$ref" > drv_$ref.rs
  rustc -O --edition 2021 -o drv_$ref drv_$ref.rs
done
./drv_<base-sha> dma_old > dma_old.v
./drv_<head-sha> dma_new > dma_new.v
```

where `<base-sha>`/`<head-sha>` are the base and head of PR #2345
(`cb1f0d4eb980` and `4db5729b1817`). Then:

```
iverilog -g2005 -o tb.vvp sim/tb_bitnet_dma_write_address.v dma_old.v dma_new.v
vvp tb.vvp
```

Substituting current master's rendering for `dma_new.v` also passes; master adds
an `overflow` output (issue #2002), which this harness leaves unconnected.

### What it asserts

The measured property is **beat N is written at local address N, and no address
outside `0..N-1` is written**. Address *and* payload are checked together: an
address-only check passes for a controller that writes the right slots in the
wrong order.

| case | stimulus | expectation |
|---|---|---|
| 1 | 4-beat read transfer | old writes 1..4 and never writes address 0; new writes 0..3, each exactly once, address K holding beat K |
| 2 | a second transfer, no reset between | new again writes 0..3 — the fix clears its index in the IDLE start branch, not only at reset |
| 3 | control: write direction (`direction=1`) | `READ_DATA` is never entered, so old and new must agree on AXI write beats, local writes and `done` |

Case 1 also asserts that the **old** rendering still misses address 0 and still
clobbers address 4, and case 3 asserts the control actually produced AXI write
beats. Without those the harness could pass while never reaching the behaviour
it exists to demonstrate.

Case 2 is the only case that constrains the IDLE re-arm: deleting just that line
from the post-fix rendering leaves case 1 passing and fails case 2 with
`addr_range=4..7`.

### Status

This is a **reporting** instrument, not a gate. Nothing in CI runs it: `cargo
test -p t27c` is invoked by no workflow (removed by #2292) and `fpga-build.yml`
never calls `vvp` (#2241). It prints `RESULT: PASS`/`RESULT: FAIL` and exits 0
either way, matching `tb_bitnet_request_overflow.v`. Wiring `sim/` into CI is
tracked separately.

## Emitting the pre-fix and post-fix Verilog

All four BitNet emitters compile **standalone under `rustc`** — no `cargo`, no
target directory, seconds per revision. The only `use` in any of them is
`use super::*` inside a `#[cfg(test)]` module, which `rustc` never reaches
without `--test`.

```
# $1 = bootstrap/src file, $2 = commit sha, $3 = emitter fn, $4 = module name
gh api "repos/gHashTag/t27/contents/bootstrap/src/$1?ref=$2" --jq .content \
  | base64 -d > src_$2.rs
printf '#[path = "src_%s.rs"]\nmod e;\nfn main(){let a:Vec<String>=std::env::args().collect();print!("{}",e::%s(&a[1]));}\n' "$2" "$3" > drv_$2.rs
rustc -O --edition 2021 -o drv_$2 drv_$2.rs
./drv_$2 "$4" > "$4.v"
```

Base and head shas come from `gh api repos/gHashTag/t27/pulls/N --jq
'.base.sha, .head.sha'`. The per-harness tables below give the ones used.

## `tb_bitnet_sequencer_zero_count.v` — issue #1977

Differential harness for the `layer_sequencer` that never terminated when
asked for zero work. Both terminators are `index == count-1` compares against
an unsigned port, and the bare literal `1` widens each subtraction to 32 bits,
so a zero count borrows to `32'hFFFFFFFF` while the index zero-extends. No
value a 16-bit `neuron_id` or an 8-bit `chunk_id` can hold ever matches, the
FSM never reaches `DONE_ST`, and `valid` is asserted forever for work nobody
requested.

| emitter | `bootstrap/src/bitnet_pipeline.rs`, `build_layer_sequencer` |
|---|---|
| pre-fix | `4ea72c322fa5572fc0c33fb5deedb739b7ad6c6a` (PR #2337 base) |
| post-fix | `e3c2d655fbcae69dd41c4a050d3feb801ac2d129` (PR #2337 head) |

```
iverilog -g2005 -o tb.vvp sim/tb_bitnet_sequencer_zero_count.v seq_old.v seq_new.v
vvp tb.vvp
```

### What it asserts

| case | stimulus | expectation |
|---|---|---|
| 1 | `num_neurons=0`, `num_chunks=4`, 200,000 cycles | old never pulses `done` and is *still counting* — `neuron_id` reaches exactly 50,000 (200000/4); new retires with `valid` never raised |
| 2 | `num_neurons=8`, `num_chunks=0`, 200,000 cycles | the other borrow: old's `chunk_id` free-runs and wraps (high-water 255, final 64 = 200000 mod 256) while `neuron_id` never advances |
| 3–8 | six non-zero requests | old and new identical on **every output, every cycle** |

Case 1 asserts the old rendering still hangs *and* still counts: a rendering
frozen with all outputs static would also show `done=0`, and that is a
different defect. The six non-zero controls are what stop the guard being a
licence to retire everything — a mutant guarded on `1'b1` passes cases 1 and 2
and fails all six.

`first_chunk`/`last_chunk` are compared only while `valid` is high. **Neither
rendering resets them** — the emitted reset block is `state<=IDLE;
neuron_id<=0; chunk_id<=0; valid<=0; done<=0;` — so both sit at X until the
first RUN cycle and `X !== X`. That is a property of the design, identical
either side of the fix. `qual_cycles` counts how often the qualified
comparison actually ran and is asserted equal to `num_neurons*num_chunks`, so
the qualification cannot silently void that half of the vector.

## `tb_bitnet_prefetch_done_pulse.v` — issue #1985

Differential harness for `weight_prefetch_ctrl`'s `prefetch_done`, documented
as a one-cycle pulse but implemented as a level. The pre-fix emitter cleared
the flag only *inside* the start guard, so it stayed asserted for the whole
idle gap. A requester that samples it in the same cycle it raises
`start_prefetch` — exactly what `multilayer_sequencer`'s `WAIT_PF` state does —
reads the previous transaction's completion and skips its own prefetch.

| emitter | `bootstrap/src/bitnet_buffers.rs`, `build_weight_prefetch_ctrl` |
|---|---|
| pre-fix | `e058a03ea20397ae4f066a57ad12ad25e01d78df` (PR #2340 base) |
| post-fix | `851dc6d99ed6f88b6bb5fd02cbf93a89ec71900a` (PR #2340 head) |

```
iverilog -g2005 -o tb.vvp sim/tb_bitnet_prefetch_done_pulse.v pf_old.v pf_new.v
vvp tb.vvp
```

### What it asserts

| case | stimulus | expectation |
|---|---|---|
| 1 | two 2-word transactions, 8-cycle gap | at request 2 old reads the stale `prefetch_done=1`, new reads `0`; both show `done_rises=2` and `we_count=4` |
| 2 | the same with a 40-cycle gap | old's high-time grows by exactly the extra 32 cycles — it is a **level**; new's stays at 2, one cycle per completion — it is a **pulse** |

Case 2 is what distinguishes the two readings of case 1: a single sample can
be explained by phase, but only a level stretches with the gap. Both cases
also assert that at the *first* request both renderings read the flag low,
which is what makes the request-2 reading a difference in staleness rather
than a constant offset.

Every output **except** `prefetch_done` is compared cycle by cycle and must be
identical (the fix is surgical), while `prefetch_done` itself must differ on at
least one cycle (the harness is observing something). Those two assertions are
anti-vacuity controls pointing in opposite directions: a mutant that fixes the
flag but breaks the BRAM address stride fails the first, and a mutant that
changes nothing fails the second.

## `tb_bitnet_dma_we_default.v` — issue #2006

**Three-way** differential harness for the `local_we` default-low. Its result
is a **null** one, and that is the whole difficulty: the pre-fix and post-fix
renderings are observationally *identical*. Reading the pre-fix output, every
reachable path already drives the strobe — reset clears it, `READ_DATA` sets
it and has an explicit `else` clear, `DONE_ST` clears it — and the states that
never mention it are never entered with it high, because `READ_DATA`'s only
exit is `DONE_ST`. The fix is defence in depth against a future arm, not a
repair of an observable defect.

A harness that "passes" by finding no difference proves nothing on its own:
one wired to the wrong ports finds no difference either. So this harness
elaborates **three** renderings and runs one comparator over two pairs:

| slot | rendering | sha |
|---|---|---|
| A | pre-#2006 | `bd2d25df4b2e5bcc4cca61eb3da3b3505c73df7d` (PR #2344 base) |
| B | #2006 applied | `4f07aa84acdac656977ea45b284288f2b6d2ba69` (PR #2344 head) |
| C | #2006 + #2003 | `4db5729b1817a2d0f0d453e34c707ab425956934` (PR #2345 head) |

B and C are consecutive revisions of the same file — PR #2344's head rendering
is byte-identical to PR #2345's base rendering — so A, B and C form a linear
chain and one comparator sees all three.

```
iverilog -g2005 -o tb.vvp sim/tb_bitnet_dma_we_default.v dma_a.v dma_b.v dma_c.v
vvp tb.vvp
```

### What it asserts

* **A vs B must be identical** — the null result under test.
* **B vs C must differ** — the anti-vacuity control. #2003 changed the
  `READ_DATA` address arm, so a working comparator has to see it.

The control is not a separate test. It is the same comparator, on the same
stimulus, in the same simulation, over the same 293-bit vector of every output
port. If the comparator is blind — misconnected ports, an enable left off, a
vector omitting `local_we` — then B vs C reports "identical" too and the run
**fails**, explicitly refusing to certify the A/B null result. Only once the
instrument has shown on C that it *can* see a difference does "A equals B"
carry information.

Seven stimulus phases cover read, back-to-back read, stretched `READ_ADDR`
(`arready` held low), write, throttled `WRITE_DATA`, throttled `READ_DATA`
(`rvalid` toggled), and a long idle tail. Phase 6 is load-bearing: throttling
`rvalid` is the only condition under which the pre-fix `end else local_we <=
1'b0;` does any work, and hence the only place the #2006 default has a live
competitor. Deleting that `else` clear from both renderings makes A emit **22**
local writes against B's 18 — four spurious strobes — with the first
divergence inside phase 6. The fix is latent today and a real backstop the
moment an arm stops clearing the strobe.

### Status

Like the harnesses above, all three are **reporting** instruments, not gates.
`vvp` exits 0 on `RESULT: FAIL` as well as `RESULT: PASS`, and no workflow runs
them.
