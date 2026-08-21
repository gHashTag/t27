# NOW -- maximum-sized requests are clamped and the clamp is reported (2026-08-21)

## fix(bitnet): clamp oversized transfers to the address space and raise `overflow` (Closes #2002)

- **Both defects were live on master verbatim, and re-verified there before editing.**
  #2002 says "Both modules now clamp to the address space and raise a new `overflow`
  output" -- that fix landed on a branch that never merged. On `origin/master`
  `6b19d8beb`, `bootstrap/src/bitnet_dma.rs:124` still emitted
  `output reg  [11:0] local_addr,` fed from `input  wire [31:0] length`, and
  `bootstrap/src/bitnet_buffers.rs:160` still emitted `input  wire [15:0] num_words,`
  driving `output reg  [11:0] bram_addr,`. Neither module clamped and neither had an
  `overflow` port. Triage's line numbers had moved (`bitnet_buffers.rs:152` -> `:160`),
  as expected after #2003 landed earlier the same day
- **Measured on the unmodified master emitters before any edit**, request = capacity + 1
  (DMA `length=32776` = 4097 beats; prefetch `num_words=4097`; capacity 4096 both):
  `DMA writes=4097 max_writes_to_one_addr=2 first_dup_addr=0 done=1` and
  `PREFETCH writes=4097 max_writes_to_one_addr=2 first_dup_addr=1`. The DMA reports
  **`done=1` while having overwritten address 0** -- the corruption is silent
- **Clamp *and* report, not clamp alone.** A bare clamp converts silent corruption into
  a silent *short transfer*: the host is still told `done` and still cannot learn that
  the tail of its request was dropped. Rejection was the other option and was not taken:
  neither module has a handshake on `start`/`start_prefetch` that could carry a refusal,
  so rejecting would mean inventing a protocol and changing behaviour for in-range
  requests. Clamping leaves every in-range request bit-identical and adds one output
- One clamp point per module. `bytes_remaining` is the only thing bounding the DMA beat
  counter, so clamping it in `IDLE` bounds the read path (`beat_index`) and the write
  path (`local_addr + 1`) at once. Likewise `words_remaining` bounds `bram_addr`
- **Differential proof, RTL level (icarus), both renderings in one testbench** --
  committed as `sim/tb_bitnet_request_overflow.v`, with `sim/README.md` giving the exact
  emit-and-run sequence. Pre-fix and post-fix modules differ only in `--module-name`, so
  the comparison is two renderings of one design. Case 1, capacity + 1:
  `old: writes=4097 max_writes_per_addr=2 first_dup_addr=0 done=1` versus
  `new: writes=4096 max_writes_per_addr=1 first_dup_addr=-1 done=1 overflow=1`
- **The harness asserts the OLD rendering still wraps.** Without that it could pass while
  never reaching the wrap and would prove nothing
- **Control, in range (32 beats / 32 words):** `old writes=32` / `new writes=32`,
  `overflow=0`, and the per-address write count of all 4096 addresses compared equal
  old-vs-new. Case 3, exactly capacity: 4096 writes, no duplicate, `overflow=0` -- the
  clamp must not fire one early. `RESULT: PASS (0 errors)`
- **Nine mutants, one per guard**, because assertions abort in order and a mutant that
  trips the first leaves the rest unproven. Each fails **exactly one** guard:
  M1 clamp reverted (`22 passed; 1 failed`), M2 overflow assignment deleted, M3 overflow
  not cleared at reset, M4 port removed; M5-M8 the same four on the prefetch controller
  (`26 passed; 1 failed`). Example message, quoted because the message is what proves the
  guard read the right span: ``` IDLE must not load `bytes_remaining` from `length`
  unclamped: `length` can name 2**29 beats against a 2**12-entry address space, so
  `local_addr` wraps and the transfer overwrites data it already wrote, then reports
  done=1. IDLE arm was: ... ``` followed by the offending arm
- **The anchor is load-bearing, and it was measured rather than asserted.** M9 relocates
  the clamp out of the `IDLE` start branch into the reset block, where it can never act
  on a request. The clamp *text* is still in the module, so an unanchored guard is
  satisfied: `unanchored v.contains(clamp) = true`, `anchored IDLE arm contains clamp =
  false`. Elaborating that same M9 module: `writes=4097 max_writes_to_one_addr=2
  first_dup_addr=0 done=1` -- **the unanchored guard passes on an artefact that still
  corrupts.** `bytes_remaining` is also assigned in the reset block and decremented in
  both data states, which is exactly why the guard slices the `IDLE` arm first
- Clean tree: **23 passed, 0 failed** (`bitnet_dma.rs`) and **27 passed, 0 failed**
  (`bitnet_buffers.rs`). Both modules are self-contained, so
  `rustc --test bootstrap/src/bitnet_dma.rs` runs them with no cargo and no crates
- **These tests are still not executed by CI**, unchanged from #2003's record: no
  workflow runs `cargo test -p t27c`, the step having been removed by #2292 after going
  red on master. Recorded, not papered over
- **Two adjacent defects found and deliberately NOT fixed here**, filed separately:
  `weight_prefetch_ctrl` still writes word N at address N+1 (the #2003 shape, fixed in
  the DMA only), and still hangs on `num_words == 0` (`words_remaining` underflows to
  65535). The clamp is a no-op for `num_words == 0`, so behaviour there is unchanged
- `bitnet_top.rs` connects both modules by name and is untouched; an unconnected new
  output is legal. Wiring `overflow` into the tied-off `.error(1'b0)` IRQ is a separate
  change against a separate module
