# NOW -- spec-guards: duplicate test names, and 10 ledgered stale seals cleared

Closes #7383.

- `specs/graph/knowledge_graph.t27`: three tests were declared twice (#6872).
  Dropped the weaker `find_entity_returns_entity_when_exists` (it checked only
  `!= null`) and the identical second copies of
  `find_entity_returns_null_when_not_exists` and `stats_returns_correct_counts`.
- `specs/port/fpga/vivado/gf16_uart_sim_bench.t27`: helpers
  `test_single_element_multiplication` and `test_full_matrix_multiplication`
  renamed to `check_*`, so they no longer collide with the lowered `test`
  blocks (#7234).
- Since #5577, tools/seal_baseline.txt has carried specs ledgered `stale`
  because their tests failed on master. Their fixes already exist on the
  open loop branch of #5084. This PR carries 10 of those files
  byte-identical: huber_loss, kl_divergence, mse_loss, multi_head_attention,
  residual_connection, rmsprop, timing_tb, merge_sort, pattern, template.
  - With master's t27c every one passes, with 0 vacuous passes.
  - Mutation evidence is in the #5084 commits. Three mutants were re-run
    here on timing_tb and pattern, and all were killed.
- Resealed on the Railway t27c lab at master cd6708d32. Their 20 ledger lines
  and the 2 TriBellmanFord lines (that seal already holds) are dropped.
- Left ledgered (6 seal files):
  - html.t27 and xml.t27: the #5084 versions pass, but they copy 11 helper
    bodies between them, so duplicate-bodies goes red. They need a shared
    helper spec first.
  - terminal.t27: its test is right, but t27c lowers `"\x1b"` as a literal
    backslash (#6654).
