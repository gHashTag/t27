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
