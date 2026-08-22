# NOW -- an input built to be refused, and it was not (2026-08-23)

Refs #2325. Two findings in one file.

- `NEGATIVE_FIXTURES` excluded 29 files under one rationale that enumerated
  **21** -- "twelve deliberately damaged, seven malformed generic-const
  declarations, two truncated-at-EOF hazards". Measured: 21 fail to generate,
  8 do not, and the 8 are **two different kinds of thing**.
  - `terminator/` holds parser CONTROLS with their own assertions in
    `bootstrap/tests/struct_body_terminator.rs`; several are meant to parse.
    Excluding them from a generate census is right.
  - `damage/` and `generic_const/neg_` exist to be REJECTED, and three of them
    generate today. The C they emit does not compile
    (`type name requires a specifier or qualifier`).
- The comment over the list had said it out loud since it was written -- *"the
  day one of them starts generating is the day a parser bug shipped"* -- and
  a repo-wide grep for `damage_class` in tests and tools returns **nothing at
  all**. The alarm had been ringing into an empty room.
- Split into `CONTROL_FIXTURES` and `MUST_NOT_GENERATE`, with the three
  already-leaking files frozen as named debt so master stays green and the
  class cannot grow. Controls: dropping one from the debt list names it and
  exits 1; making a currently-rejected fixture valid names it and exits 1.
- Second finding, same file: `--summary` took path component [1] and glued
  `specs/` back on. `specs/runtime/ 5` was really 3 plus
  `compiler/runtime/{commands,validation}.t27` -- a count true of neither
  directory it named, and 17 of 171 ledger entries are not under `specs/` at
  all. It now prints the directory that exists; all twelve printed rows
  resolve on disk.
