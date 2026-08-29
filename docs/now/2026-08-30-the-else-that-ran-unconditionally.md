# NOW -- The else that ran unconditionally (2026-08-30)

## A braceless `else` was consumed and its branch dropped (Refs #2880)

- `parse_if_stmt` handles `else if` and `else {` and has NO third arm, so on any other token it eats `else` and returns
- the body is then parsed as the next statement of the ENCLOSING block and runs unconditionally
- decisive control: the emitted output is BYTE-IDENTICAL to the same spec with the `else` keyword deleted, and different from the braced form -- in all four backends
- `pick(20)` returned 2 where the spec says 1
- `parse-complete`, this project's own silent-truncation detector, prints "nothing discarded" for a file whose `else` has vanished
- the then-branch has had the single-statement arm all along, twenty lines above: `// single statement: if (cond) return expr;`
- 4 sites, all in gf16.t27 and tf3.t27, all `if (a >= b) return a else return b` -- early return accidentally preserves their semantics, so the corruption is latent there and fires on the first non-return body
- an earlier audit reported this same shape with "0 corpus sites"; the count was 4
- two more specs changed output with zero restored `else`: sorted content identical, only the splice ORDER moved. Checked rather than assumed
