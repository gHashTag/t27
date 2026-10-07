# NOW -- packet.t27 serialise takes a slice (2026-10-07)

## t27b lane 4: a buffer with a length (Closes #7413)

- `AnnounceHeader_serialise` in `specs/port/trios/crates/trios-mesh/src/packet.t27` took `buf: [*]u8`; the trios source takes `&mut [u8; MIN_WIRE]`, a buffer whose length is known, so the port now takes `buf: []u8`.
- Spec defect, not a backend gap: no t27b change.
- `t27c test-report` 6/6, 0 vacuous; t27b 6/6, 106 runtime asserts; the ledger row moves blocked -> pass.
