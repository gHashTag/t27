# NOW -- The count went down and nothing printed (2026-08-30)

## The count went down and nothing printed (Closes #2900, Refs #2905)

- `tri elab` has not existed since #2427, a Zig-lexer PR, deleted `mod elab;` and two more lines from main.rs in one hunk and left the 319-line file
- `cargo build` cannot error on a file it does not compile, and the suite fell 358 -> 354 with no gate reading that number: four tests left in silence
- `tools/check_elab_ratchet.py` kept telling its reader to run the command, in its docstring and in its failure output, the whole time
- restored, it reads 176 real diagnostics and 28 iverilog summary lines that are not diagnostics -- the published 162/26 was measured before the tree moved
- `tri mods orphan` is the general question; historical control: the same binary prints elab.rs against origin/master and nothing after the fix
- `mod c;` in `a/b.rs` means `a/b/c.rs`, never `a/c.rs` -- the loose stem-name rule hides an orphan behind a same-named module elsewhere
