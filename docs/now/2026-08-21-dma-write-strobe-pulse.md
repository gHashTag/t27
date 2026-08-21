# NOW -- the DMA write strobe becomes a pulse (2026-08-21)

## fix(bitnet): default `local_we` low ahead of the state dispatch (Closes #2006)

- **The defect is live on master, verbatim.** #2006 reports itself as fixed; the
  branch that carried the fix never merged, in any state. On `4ea72c322`,
  `bootstrap/src/bitnet_dma.rs` still emits `end else case (state)` with the only
  `local_we` clears inside `READ_DATA`'s else-arm and inside `DONE_ST`. No state
  arm clears it on entry, so the strobe is a level that happens to fall rather
  than a pulse that is driven low
- The fix is the standard shape: `local_we <= 1'b0;` once, between the reset arm
  and `case (state)`, so every arm inherits a low strobe and only the arm that
  actually presents data raises it. `READ_ADDR` deliberately does **not**
  hand-clear it -- a per-arm patch-up would miss the next state somebody adds

### Honest scope: this is latent today, and that is the reason to land it first

- Measured, not assumed. A differential Icarus bench (`iverilog` 13.0) ran the
  old and new emitter side by side against private AXI slaves and private local
  memories, and instrumented every cycle where `local_we` was high outside
  `READ_DATA`:

  ```
  --- OLD emitter ---
    writes=4  we_outside_READ_DATA=1  we_in_READ_ADDR=0
  --- NEW emitter ---
    writes=4  we_outside_READ_DATA=1  we_in_READ_ADDR=0
  EQUIV: old and new local memories are identical (4 writes each)
  ```

- `we_in_READ_ADDR=0` on **both** sides is the honest result: on master's
  single-burst FSM, `READ_DATA` only ever exits to `DONE_ST`, so the strobe never
  actually latches into `READ_ADDR`. The one `we_outside_READ_DATA` cycle is the
  final beat's legitimate write retiring in `DONE_ST`, on old and new alike
- So this change is **behaviour-preserving today** and carries no simulation
  evidence of a behavioural fix. It is not decoration: #1970 derives a real burst
  length and returns `READ_DATA -> READ_ADDR` between bursts, and at that moment
  the un-cleared strobe becomes a spurious write at the wrong address on every
  burst boundary. Landing this first means #1970 cannot introduce that
- No adjacent problem was fixed in passing. The `local_addr` off-by-one that the
  bench also shows (`mem[0]` never written, every word shifted up one slot) is
  #2003 and is left untouched here

### The guard bites

- Two new unit tests. `write_strobe_defaults_low_ahead_of_case_dispatch` is
  anchored to the span between the reset arm's 8-space `end else begin` and
  `case (state)`. The anchoring is the point: the emitted module contains three
  other `local_we <= 1'b0;` sites, so a bare `contains(...)` would be satisfied by
  an unrelated line and prove nothing
- Against the **unfixed** master emitter the guard fails:
  `reset arm must be closed by an 8-space `end else begin``
- Three mutants planted in the fixed emitter, one per guard behaviour, each
  leaving `local_we <= 1'b0;` present somewhere in the emitted Verilog so that a
  `contains`-only guard would pass all three. All three bite:
  - `M1_delete_default` -> `write strobe must default low between the reset arm
    and `case (state)`, otherwise it latches high across states that never clear
    it`
  - `M2_relocate_default_into_IDLE` (string still emitted, just moved after the
    dispatch) -> same assertion fires. This is the anchoring proof
  - `M3_read_addr_hand_clears` -> `READ_ADDR must inherit the low strobe from the
    default, not assign it locally`
- Unit suite: 17 passed, 0 failed (15 pre-existing + 2 new). No existing
  assertion was relaxed, and nothing pinned `end else case (state)`
