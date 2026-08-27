# NOW -- One character emptied a whole test body (2026-08-28)

## One character emptied a whole test body (Refs #2161)

- Refs #2161. A braceless clause may end with a semicolon: `given p = 0;`. Nothing consumed it, so the next loop turn met `;` where it expects a clause head, read that as stopped-mid-clause, and restored the fallback -- discarding the WHOLE block over one character. The identical body without the semicolon lowered fine, which is what kept it invisible: two spellings of one clause, one of them silently emptying every assertion after it
- Measured: discarded tokens 35224 -> 35070, parse 620 -> 620, tests unchanged, RATCHET CLEAN
- A SECOND shape in the same family is not shipped, and the reason is worth keeping. A body OPENING with var/const has no earlier clause to take a column from, so the statement arm is skipped. I wrote that fix and measured it recovering 1914 tokens -- and it regressed specs/memory/notebooklm.t27 from parsing to not parsing. Seeding the column lets an EARLIER clause take the arm, and the parser then reaches `const (notebook, err) = ...` in a state where the old path would have fallen back for the whole block; instead it dies hard
- Isolated by disabling one edit at a time, not by reading: with the semicolon consumption alone the spec parses, with the column seeding alone it does not. parse_bdd_clauses carries the contract "may only ADD assertions, never break a file" in its own doc comment, and that version broke one. Filed as #2735 with the containment fix it actually needs
