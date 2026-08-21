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
