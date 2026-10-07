# NOW -- spec-guards: duplicate test names in two specs

Closes #7383.

- `specs/graph/knowledge_graph.t27`: three tests were declared twice (#6872).
  Dropped the weaker `find_entity_returns_entity_when_exists` (it checked only
  `!= null`) and the identical second copies of
  `find_entity_returns_null_when_not_exists` and `stats_returns_correct_counts`.
- `specs/port/fpga/vivado/gf16_uart_sim_bench.t27`: helpers
  `test_single_element_multiplication` and `test_full_matrix_multiplication`
  renamed to `check_*`, so they no longer collide with the lowered `test`
  blocks (#7234).
- Resealed on the Railway t27c lab at master cd6708d32: knowledge_graph stays
  BLOCKED at Zig compile (seal says `blocked`, as before); gf16_uart_sim_bench
  5/5 pass (first seal for this spec).
- On the lab after the reseal: `check_seal_currency.py` exit 0 (0 unledgered
  stale), `check_seal_coverage.py` exit 0, `check_duplicate_declarations.py`
  exit 0.
