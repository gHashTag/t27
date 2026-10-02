# NOW -- Duplicate Body Ratchet green on master (2026-10-02)

## Fix the eleven unledgered duplicate groups (Refs #5497)

- Teach `tools/dupe_scan.py` the lexical forms `t27c` has: a declaration ending in `;` no longer borrows the next body (`digest_for` x8, `githubCiDeps_checkRuns_impl` x5 were false), comments and string literals are masked, `;` at column 1 and `#` are line comments, and a single quote ends at the newline. The repair for prototypes, comments and literals comes from #5473; the quote and `;`-comment rules are new, because an apostrophe in `; the Queen's ...` had opened a "literal" that hid three bodies in specs/queen/views.t27 and specs/tools/catalog.t27. Eleven self-test shapes pass; the #5473 lexer fails the two new ones.
- Share one empty-result literal in `specs/hslm/forward_pass.t27` (from #5473) and reseal both seal files naming it; all four hashes match the #5473 seals made on another machine.
- Replace the two `return undefined;` stubs in `specs/port/trinity/src/tri/gen_fuzz.t27` with a xorshift64 generator and give the port five tests that can fail. Zig: 8 of 8 pass; five mutants (constant byte, `min` only, off-by-one span, the old stubs, frozen state) are each caught.
- Ledger: seven cross-file copies are recorded, as #4497 did, because `use a::b;` still generates no import (#4610): `bytes_eq`, `session_status_str`, `led_config`, `validate_led_config`, `startup_config`, `update_oscillator_chain`, `shift_ir`. Three rows whose groups no longer exist are dropped: `generate_all`, `next`, `is_orphan`.
