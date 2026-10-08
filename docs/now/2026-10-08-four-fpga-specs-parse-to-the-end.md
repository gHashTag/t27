# NOW -- four fpga specs parse to the end, in the idiom each file already speaks (2026-10-08)

## specs/fpga/ (4 files) (Refs #2474, Refs gHashTag/trinity#1484)

- The website's course generator compiles every spec a lesson points at with
  the vendored wasm compiler and refuses any verdict with `discarded > 0`
  (trinity `scripts/course-from-spec.mjs`, behind trinity#1484). Four fpga
  specs failed it: uart (12 discarded), spi (73), top_level (57), and bridge
  (6 wasm typecheck errors). Every construct named below was located by the
  compiler's own token accounting, not by reading: the swallowed-token lines
  and the `stopped mid-clause` lexer discard name the exact lines.
- uart: `test uart_statement_body_with_given` opened its body with a bare
  statement (`tmp = 1`) the clause grammar has no model for. The statement
  becomes a `given` clause; the tested property (`tmp == y`) is unchanged.
- spi: five constructs.
  - Two gherkin `assert` bodies used `implies`; the clause parser discards
    the whole invariant. `A implies B` is now the logically identical
    `A == false or B` in `spi_busy_implies_cs_asserted` and
    `spi_busy_only_in_transfer`.
  - `invariant spi_cs_deasserted_after_transfer` called `spi.transfer(0xAA)`
    -- no such method exists; the real function is `spi_transfer` -- and
    carried an `and` line that was only a `//` comment. Both fixed.
  - `bench spi_cs_assertion_time` measured `nanoseconds for CS to assert`;
    the word `assert` is a clause keyword and the lexer stopped mid-clause
    on the line (the `stopped mid-clause -- :` discard). Now
    `nanoseconds for CS assertion`.
  - `target: < 150ns` carried a trailing `//` comment inside the clause.
    The comment (CS_ASSERT_DELAY plus margin) now sits above the bench
    header, where module-scope comments parse -- the fn-header comments in
    this same file always did.
- top_level: the same `implies` rewrite in three invariants
  (`system_ready_when_not_processing`, `system_error_implies_not_busy`,
  `system_ready_implies_mac_uart_ready`). Both benches carried a `//`
  comment line and a bare call line under `target:`; the comments moved
  above the bench headers, the bare calls are deleted -- uart's benches
  (`measure:` + `target:` only) are the shape that parses, and these now
  match it.
- bridge: six `tx_buffer[bridge.tx_head] = v` element assignments. Native
  `t27c check` passes them; the wasm typechecker reports `cannot assign to
  immutable array element` -- the two compilers disagree on element-assign
  to a `var` global fixed-size array, recorded here as a divergence, not
  resolved here. The writes now go through the file's own
  `buffer_write(tx_buffer, TX_BUFFER_SIZE, head, v)`, the same
  bounds-checked path the rx side and the tests already use. Known corner:
  `buffer_write` refuses `head == size - 1` as "buffer full", so a tx write
  at the last slot that the old direct assignment performed is now refused;
  fixing that refusal condition is rx-visible and stays separate work.
- Measured, before and after:
  - wasm verdicts (trinity `public/t27/t27_compiler.wasm`, the bytes the
    browser runs): uart/spi/top_level `discarded` 12/73/57 -> 0, all
    `typecheckOk, hirOk`, all seven backends ok; bridge `errors` 6 -> 0.
  - native `t27c check` (0.4.0): 0 errors, 0 warnings, before and after,
    all four.
  - native `t27c test-report` status: BLOCKED (comptime), before and after,
    all four -- nothing that ran before stopped running.
  - `t27c gen-verilog` on bridge still emits the module and its ports.
- The braceless-body swallow itself stays open: the parser repair is
  #2474, stage0-frozen, not attempted here. What this does is move four
  specs into the subset the parser already consumes completely, in the
  gherkin idiom the rest of each file uses -- no healthy block was
  converted to braces.
