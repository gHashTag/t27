# NOW -- the remainder was stated from the top six rows (2026-08-23)

## the claim held, but it was made before it was checked (Refs #2325)

- After the three emitter fixes I wrote that the remaining elaboration errors
  are "two design decisions". That came from the head of the error output.
- Checked exhaustively afterwards: 48 unique unbound names, 68 references --
  56 string-field reads, 12 unsized-array-element reads, ZERO anything else.
  The claim survived, which is luck rather than method.
- ci-gates section 13 gains the rule: a remainder is a claim like any other.
  Enumerate it, or say out loud that you sampled. The full breakdown is posted
  on #2325 so the owner sees the whole distribution, not my summary of it.
