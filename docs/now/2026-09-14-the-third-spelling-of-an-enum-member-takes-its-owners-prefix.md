# NOW -- The third spelling of an enum member takes its owner's prefix (2026-09-14)

## What was wrong

- The C backend wrote an inferred enum literal (`.POS`, the member without its type) as the bare constant `POS`. The generated header declares only `TRIT_POS`. A `switch` case label naming a member without its type had the same defect.
- `Type.member` and `Type::member` already resolved to the prefixed constant. The inferred form is the third spelling of the same rule, and it was the one left unrouted.
- Measured on master 2ef6c2a7e (Apple clang 21, `-ferror-limit=0`): 788 `use of undeclared identifier` diagnostics for `POS` / `NEG` / `ZERO` (412 / 319 / 57), concentrated in base/ops, base/types, base/ternary_add and base/seed. Issue #3620.

## What changed

- `c_unique_enum_owner(member)` returns the owning enum only when exactly ONE enum in the module declares that member. Both the inferred literal and the case label use it. When two enums share a member name the owner is ambiguous, so the name is left as it was. It is never guessed.
- A numeric case label is untouched.
- `bootstrap/tests/c_inferred_enum_literal.rs` has 5 tests: the literal, the case label, the ambiguous member (checked on the `return` line only, since the header declares both prefixed names), a known member with nothing invented, and a numeric label.

## Measured, same instrument, same 945 specs, before -> after

- Total errors 12721 -> 11704 (-1017). Clean units 340 -> 341. 877 of 945 specs generate on both sides.
- Bare `POS` / `NEG` / `ZERO`: 788 -> 0.
- Files: 19 better, 0 worse. base/ops 409 -> 58, base/types 323 -> 37, base/ternary_add 133 -> 8, base/seed 24 -> 0.
- Classes that rose, all inside files whose total fell. These are diagnostics clang could not reach while the identifier was undeclared: `member reference base type ... is not a structure or union` +18, `array type ... is not assignable` +6, `incompatible pointer to integer conversion` +2, `undeclared identifier ...; did you mean` +1.

## Not claimed

- A mutant that deletes the numeric-label branch survives the tests. For a decimal label `to_uppercase()` is the identity, so the branch is only observable on a label that contains letters (`0xff` -> `0XFF`, which is the same C value). How many such labels the corpus holds was not measured.

## Bookkeeping

- `bootstrap/stage0/FROZEN_HASH` = sha256 of `bootstrap/src/compiler.rs`, in this commit.
- Seals: `check_seal_currency.py --stale-specs` listed 0 stale specs on master (checked with the master binary via TRI_T27C, in a worktree that had no local build) and 19 with this change. All 19 were resealed with `t27c seal --save`, which rewrote 40 seal files. Afterwards it lists 0.
- `tools/corpus_figures_pin.txt` re-blessed. The pin already disagreed with master before this change (it recorded 651 specs and 10802 errors, against 945 and 12721 measured today), so this bless also absorbs drift that other merges left behind.
- `cargo test --release`: 3573 passed, 0 failed.
