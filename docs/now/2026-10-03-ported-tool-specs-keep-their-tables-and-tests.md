# NOW -- ported tool specs keep their tables and tests (2026-10-03)

## What was read

- `t27c parse-complete` on master 8fdf7155: `check_conflict_markers` discarded 15 tokens, `jtag/read_user1` 25 and `jtag/read_verdict` 55.
- `check_conflict_markers` wrote `SKIP_SUFFIX : [&str; 7] = .{ ... }`, and the generated Zig declared the constant with no value. `read_verdict` wrote `USER_OPCODE` as a Python dict. Both jtag specs wrote braceless tests full of Zig statements and `expect`, so every test body reached Zig empty.

## What changed

- `SKIP_SUFFIX : [7]str = [...]` and `USER_OPCODE : [5]u8 = [0x00, 0x02, 0x03, 0x22, 0x23]`. Index 0 is unused, so `USER_OPCODE[chain]` keeps its 1..4 chain numbering.
- The five jtag tests are braced t27 tests with `assert`. `read_user1`'s `result == result` held for any value. It is now `result < 16`, because `user1` masks its read to `nbits = 4` bits, and a comment says so.
- Ledger: `read_user1` and `read_verdict` leave it. `check_conflict_markers` now parses whole, and that exposes a typecheck failure that was always there: `staged_files` calls the four-argument `_git` stub with seven arguments. Its entry is re-keyed to `typecheck` under #5549 instead of being removed, so `max_entries` drops by two (to 128 on top of #5702, which had already lowered it to 130). A local `t27c suite --ratchet --corpus-only` reports RATCHET: CLEAN.
- Typecheck is unchanged on all three (1 error in `check_conflict_markers`, 0 elsewhere). `zig ast-check` errors: 1→1, 0→0, 4→3.

## Not verified

- `t27c test-report` is BLOCKED on all three, on master and here alike. The cause is an existing problem in each spec: a `&str` field type, `undefined;` stub bodies, and the undeclared type `Mpsse` in `read_verdict`. The new checks appear in the Zig output, but none of them ran.
- The jtag tests need a JTAG cable. Even once they compile, `read_word` and `report` only return non-null with hardware attached.

Closes #5735
