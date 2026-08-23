# NOW -- Zig-subset campaign stops at twenty-one specs, and the last two fixes moved nothing (2026-08-23)

## Zig-subset campaign stops at twenty-one specs, and the last two fixes moved nothing (Closes #2426)

- Waves 1-3 took 541 generate to 562 and the ledger 171 to 150. Two further gaps were then closed — a call returning a type and initialised on the spot, and defer/errdefer — and the corpus did not move: still 562 and 150. They advance one legacy file along its own ladder and unblock no spec.
- Stopping there is the measurement talking. The campaign's value came from gaps shared across specs; what remains in contrib/backend/zig/legacy/main_zig_handwritten.t27 is a multiline string literal and whatever follows it, in a file whose directory says legacy. cap_test.t27 is not a grammar gap at all — the compiler refuses closures deliberately.
