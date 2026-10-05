# NOW -- typecheck: `?T` is an optional and accepts a `T` (2026-10-04)

## Optional prefix and payload coercion (Closes #6186)

- `resolve_type_str` only knew the suffix spelling `T?`. Zig's prefix `?T` resolved to `Custom("?i32")`, so `opt = value` with `value: i32` was a type mismatch, while zig accepts it.
- `types_compatible`: an `Optional(T)` target accepts `T` (and `?T`) under the payload's own rules. Cross-sign and narrowing are still rejected (`u32 -> ?i32`, `i64 -> ?i32`, `f64 -> ?f32`).
- Against a t27c built from exact origin/master, over the 31 specs that use `?T`: 2 false typecheck errors removed (`specs/pins/parser.t27` line 330, `specs/port/scripts/gen_w378.t27`) and 1 false warning removed (`queen-public-leaderboard.t27`, `returns I32 where ?i32 is declared`). No other verdict moved.
- t27b: `gen_w378.t27` was the one spec where t27b's frontend rejected what the reference passes (lab LAB-FRONTEND-DISAGREES). It now reaches a real codegen blocker (string literal).
- Unit test `optional_accepts_its_payload_6186`; FROZEN_HASH resealed (sha256 of compiler.rs).
- Corpus ratchet (`t27c suite --ratchet --corpus-only`) with this compiler: UNEXPECTED PASS for exactly those two specs, 0 unexpected failures. Both ledger entries are removed and `max_entries` follows the count down, 113 -> 111.
