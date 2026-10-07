# NOW -- t27b runs module vars that hold a string or start undefined (2026-10-07)

## cli/t27b/src/lower.rs, specs/tri/t27b/conformance/str_module_var.t27, undefined_module_var.t27 (Closes #7448)

- Lane 5 of #6063. `specs/boards/xc7a100t_minimal.t27` was blocked on `VarDecl(module, pointer/str/slice)` and `specs/vsa/similarity_search.t27` on `VarDecl(module, undefined)`. The reference runs both.
- A module var declared `= undefined` now starts as 0xAA in every byte. That is what Zig's Debug build, used by `t27c test-report`, puts there (zig 0.15, measured on the t27b lab).
- A module var whose type holds a `str` (a `str`, a struct with a `str` field, or an array of those) is written with its initial value at the start of every test, invariant and bench. The reference does the same, because each test runs in a fresh process. A data image cannot hold a string's address before the program is loaded.
- Still refused: a module var of a pointer or slice type, and a string-holding var declared `undefined`.
- New conformance specs, written first. The reference passes both with nonzero asserts in every test:
  - `str_module_var.t27`: 5 of 5 tests;
  - `undefined_module_var.t27`: 4 of 4 tests.
- Seven expected-value mutants of the two specs each fail one test in t27b. One of them checks that a write in one test is gone in the next.
- Ledger moves:
  - `xc7a100t_minimal.t27`: `blocked` -> `pass`, 23 of 23 tests, the same as the reference;
  - `similarity_search.t27`: moves to its next blocker, `ExprCall(@intCast)`, which belongs to lane 4;
  - the two conformance specs get `pass` rows;
  - after the merge with master, `max_not_pass` goes from 24 to 23.
