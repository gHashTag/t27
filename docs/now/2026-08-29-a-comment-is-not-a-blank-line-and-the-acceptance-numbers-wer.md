# NOW -- A comment is not a blank line, and the acceptance numbers were flattered (2026-08-29)

## A comment is not a blank line, and the acceptance numbers were flattered (Refs #2754)

- 87 specs discard 33777 tokens -- the bodies of invariants, so the asserts vanish and the phase that would notice sits in BLOCKED
- a comment between a braceless header and its body read as a gap: 33777 to 32728 tokens
- then parsed its body with parse_expr, so then for (...) { } took the asserts inside the loop to the discard with it
- Zig 217 to 215 and cc 157 to 156: those specs were accepted on less code than they contain, and the number was measuring the drop
