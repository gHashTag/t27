# NOW -- spec-guards: duplicate test names, and 12 ledgered stale seals cleared

Closes #7383.

- `specs/graph/knowledge_graph.t27`: three tests were declared twice (#6872).
  Dropped the weaker `find_entity_returns_entity_when_exists` (it checked only
  `!= null`) and the identical second copies of
  `find_entity_returns_null_when_not_exists` and `stats_returns_correct_counts`.
- `specs/port/fpga/vivado/gf16_uart_sim_bench.t27`: helpers
  `test_single_element_multiplication` and `test_full_matrix_multiplication`
  renamed to `check_*`, so they no longer collide with the lowered `test`
  blocks (#7234).
- 12 specs ledgered `stale` in tools/seal_baseline.txt since #5577 had failing
  tests on master. Their fixes already exist on the open loop branch of #5084;
  this PR carries those 12 files byte-identical (huber_loss, kl_divergence,
  mse_loss, multi_head_attention, residual_connection, rmsprop, timing_tb,
  html, merge_sort, pattern, template, xml). With master's t27c every one
  passes, 0 vacuous passes; mutation evidence is in the #5084 commits, plus
  three mutants re-run here on timing_tb and pattern, all killed.
- Resealed on the Railway t27c lab at master cd6708d32. Their 24 ledger lines
  and the 2 TriBellmanFord lines (seal already holds) are dropped.
- Left ledgered: terminal.t27. Its `reset_returns_ansi_sequence` test is right;
  t27c lowers `"\x1b"` as a literal backslash (#6654).
- On the lab after the reseal: `check_seal_currency.py` exit 0 (0 unledgered,
  2 ledgered stale), `check_seal_coverage.py` exit 0 (1365 hold),
  `check_duplicate_declarations.py` exit 0.
