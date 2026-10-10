# NOW -- reseal five specs left stale by the 2026-10-07 merge wave

Closes #7364.

- Resealed on the Railway t27c lab at master 0c637735b: knowledge_graph
  (#6872), prims_mst (#6876), graph_bfs (#7065), formats_catalog (#6930),
  pipeline/workflow (gen-drift from #6994).
- Tests: workflow 3/3 pass, formats_catalog 0/0 (no tests, as before);
  knowledge_graph, graph_bfs and prims_mst are BLOCKED at Zig compile and the
  seals say `blocked`, not passed.
- After the reseal, on the lab: `check_seal_currency.py` exit 0 (0
  unledgered stale), `check_seal_coverage.py` exit 0.
- Not done here: prims_mst's #6876 body is not a working MST and its tests
  are vacuous; 26 seals stay ledgered `stale` in tools/seal_baseline.txt.
