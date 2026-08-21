# NOW -- dma_controller pairs the local write address with its own beat (2026-08-21)

## fix(bitnet): present local address, data and enable from one stage (Closes #2003)

- **The defect was live on master verbatim.** #2003 reports itself as fixed, but its fixes
  landed on a branch that never merged (no PR, in any state). On `origin/master`,
  `bootstrap/src/bitnet_dma.rs:189-190` still emitted, inside one non-blocking group:
  `local_we        <= 1'b1;` followed by `local_addr      <= local_addr + 12'd1;`
- **Why that loses slot 0.** Both are non-blocking, so both land on the same edge: the
  cycle the strobe goes high, the address has *already* advanced. Beat 0 is captured while
  `local_addr` is 0, but the memory sees `we=1` with `addr=1`. Every beat is written one
  slot high and address 0 is never written at all
- Fix: add a `beat_index` counter. `READ_DATA` drives `local_addr <= beat_index;` and
  advances `beat_index <= beat_index + 12'd1;`, so address, data and enable are all
  registered from the same stage and describe the same beat. `beat_index` is cleared at
  reset *and* re-armed in the `IDLE` start branch
- **Mutant proof, RTL level (icarus), old and new emitter in one testbench.** 4-beat read,
  identical well-behaved AXI slave driving both. Defective emitter:
  `write[0] addr=1 ... write[3] addr=4`, `mem[0] = ffffffffffffffff` -- slot 0 untouched.
  Fixed emitter: `write[0] addr=0 ... write[3] addr=3`, `mem[0] = d0d0000000000000`.
  Control, unchanged in both: `old AXI w-beats = 3   new AXI w-beats = 3`
- **Mutant proof, guard 1 (`read_data_pairs_address_with_data_and_enable`).** Reverting the
  READ_DATA arm to master's shape fails it with
  ``` READ_DATA must not post-increment `local_addr` in the same non-blocking group that
  raises `local_we`: the increment lands in the same cycle as the strobe, so beat 0's data
  is written to address 1 and slot 0 is never written. READ_DATA arm was: ... ```
  and the message prints the offending arm. 18 passed, 1 failed
- **The anchor is load-bearing, and here an unanchored guard would have been WRONG, not
  merely vacuous -- measured.** `WRITE_DATA` legitimately keeps its own
  `local_addr <= local_addr + 12'd1;` (there the address is a read pointer). A naive
  `!v.contains("local_addr + 12'd1")` therefore **fails on the CORRECT emitter**:
  `test tests::unanchored_guard_ON_THE_CORRECT_EMITTER ... FAILED`. The guard slices the
  `READ_DATA` arm out first, so it cannot be satisfied -- or broken -- by a different state
- **Mutant proof, guard 2 (`beat_index_rearmed_in_idle_not_only_at_reset`), planted
  separately.** One mutant per guard, not one per file. Deleting only the `IDLE` re-arm
  fails it with ``` `beat_index` must be cleared in the IDLE start branch alongside
  `local_addr`, not only in the reset block: without it the second transfer after power-on
  starts writing at a stale index ``` while guard 1 still passes. The reset block *does*
  still contain `beat_index <= 12'd0;` under this mutant, so the guard is provably not
  satisfied by the reset line
- Reverting every mutant: **19 passed, 0 failed**. The emitted Verilog parses under
  `iverilog -g2005`
- **These tests are not executed by CI.** No workflow runs `cargo test -p t27c`;
  `corpus-ratchet.yml` records that the step was removed by #2292 after going red on
  master. Run with `rustc --test` on the module, which is self-contained. Pre-existing gap,
  recorded rather than papered over
- **#1970 is not touched here.** `arlen`/`awlen` are still hardwired to `8'hFF` and
  `READ_DATA` still exits on `m_axi_rlast || bytes_remaining <= 32'd8`. Separate change
