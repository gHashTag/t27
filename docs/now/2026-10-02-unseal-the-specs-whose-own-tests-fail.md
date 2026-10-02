# NOW -- Unseal the fifteen specs whose own tests fail (2026-10-02)

## A seal no longer certifies a failing contract (Refs #5577)

- The seal triage on a26af0ad5 (#5577) put 30 seals on 15 specs in class (c): `t27c test-report` reports failing tests for each of them, and a seal over a failing spec certifies a broken contract. #5578 resealed 26 of them in its bulk reseal, and #5572 resealed the other 4 (`tri/encoding/html`, `tri/encoding/xml`) after its discard repair.
- Re-measured on d04bf141 with Zig 0.16.0, one spec per run: all 15 still fail, on the same tests #5577 names.
- The 30 seal files go back to their a26af0ad5 state. Only #5572 and #5578 had touched them since. No baseline entry.
- After this, Seal Coverage reports exactly these 30 as stale and seal currency reports `STALE generated-code hash: 30`. Each spec needs its body or its tests fixed, then `t27c seal <spec> --save`. The 542 class (b) seals that #5578 resealed (#5576) are left for the maintainer to decide.
