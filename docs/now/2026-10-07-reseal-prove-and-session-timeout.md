# NOW -- reseal depin/prove and sandbox/session_timeout (2026-10-07)

Closes #7356.

- #6954 and #6820 changed `specs/depin/prove.t27` and
  `specs/sandbox/session_timeout.t27` without resealing; spec-guards' seal
  currency check failed on every PR.
- `session_timeout`: tests run and pass 6/6 (first time; they were BLOCKED on a
  missing `timestamp` module before #6820). Resealed.
- `prove`: the tests #6954 added did not compile (`return false` in a test
  body) and `test_pack_gf16_matrix_identity` expected the wrong bytes. Rewritten
  as asserts; in a scratch spec on the t27c lab the pack tests pass 3/3 and a
  mutant without `<< 4` fails 2 of 3. The whole spec stays BLOCKED on
  `use tri::crypto::sha256::TriSha256`, and the seal says so.
- Resealed on the Railway t27c lab at master 223c28c71; `t27c seal --verify`
  reports all hashes MATCH for both specs.
- Not done here: other seals stale after the same 07:24-07:28 merge wave
  (knowledge_graph, prims_mst, graph_bfs, formats_catalog, pipeline/workflow).
