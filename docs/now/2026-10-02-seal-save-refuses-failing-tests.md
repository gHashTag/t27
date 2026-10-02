# NOW -- seal --save refuses a spec whose own tests fail (2026-10-02)

## The seal step runs the tests the hash gate cannot see (Refs #5577)

- `t27c seal --save` now runs the spec's tests through the `t27c test-report` machinery. On any FAIL it refuses: it names each failing test, exits 1 and writes no seal. BLOCKED (no zig, compile error) is reported and the seal is still saved, because blocked is not failing. `--force` still seals, and writes the failures into the seal's new `tests` object.
- `tools/check_seal_coverage.py` reads that object. A seal with `tests.failed > 0` is the new kind `tests-fail`, so a forced seal is reported instead of counted as holding. The self-check has three new controls.
- The 26 seal files of the 13 class-(c) specs that #5578 resealed are entered in `tools/seal_baseline.txt` as `stale`, which is the state #5580 left them in. Their failing tests are named in each entry. Nothing was resealed. `tri/encoding/html` and `xml` (#5572) are not in the ledger and are still reported.
