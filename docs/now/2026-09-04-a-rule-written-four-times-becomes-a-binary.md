# NOW -- A rule written down four times becomes a binary (2026-09-04)

## `tri fmt` runs the formatter and puts back what was not yours

- `cargo fmt --all` on a one-file change leaves **165 files dirty**, 164 of them
  collateral, including `bootstrap/src/compiler.rs` -- M5-frozen, so the freeze
  gate goes red as a side effect of tidying another crate. An earlier run with
  `-p t27c` produced 155.
- Nothing keeps this tree formatted: no workflow invokes `cargo fmt` or
  `rustfmt`, so an unformatted tree is the repository's normal state and running
  the formatter is not a fix.
- `tri fmt` takes the dirty set, runs the formatter, and restores every file that
  was clean before and is dirty after. Clean-before means identical to HEAD, so
  the restore loses nothing. Measured here: 165 dirty, 1 kept, 164 restored,
  `FROZEN_HASH` intact afterwards. Every restored path is printed.
- The reason this is a command and not a note: the skill already recorded it four
  times (sections 72, 381, 407, 447) and the command was run anyway this week.
- The limitation the first use exposed, now stated in both the section and the
  module: it protects every file except the one you edited. Formatting the
  thirteen added lines of `main.rs` also sorted that file's `mod` declarations
  and reported 31 insertions with 18 deletions.
