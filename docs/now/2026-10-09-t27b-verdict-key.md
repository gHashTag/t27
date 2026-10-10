# NOW -- t27b verdict key: a rebuilt t27c reuses every unchanged verdict (2026-10-09)

## the t27b reference cache is keyed by the generated code, not the t27c binary (Closes #8176)

- `specs/tri/t27b/verdict_key.t27` (gen-rust, 7 tests, 1 invariant) defines the key: FNV-1a 64 over
  schema 2, the spec and its `use` closure, the stdout of `t27c gen <spec>`, `bootstrap/src/test_report.rs`,
  `zig version`, and the arch-os from `zig env`. A part that cannot be computed falls back to the binary hash.
- The host is in the key, so a host-dependent verdict (#8051) never crosses hosts. Per-test timeouts and
  zig process faults (`error: SystemResources`, `error: OutOfMemory`) are never cached, like reference timeouts.
- `cli/t27b/src/blockers.rs` computes the parts and calls the generated functions (38 added foreign lines,
  of the 80-line budget). The t27b-native.yml header comment still describes the binary stamp; the push
  token has no `workflow` scope, so that comment is left to a follow-up.
- Lab, full corpus, master t27c then master + #8110 t27c (a bootstrap/ change): old key 0 of 1723 reused,
  new key 1718 of 1723 reused. The 5 misses are the new seal_identity.t27 and 4 testbench timeouts.
- Before, in CI: a bootstrap/ pull request spent 1406 s in the corpus step (run 37911771594), a
  spec-only one 432 s (run 37911494033).
