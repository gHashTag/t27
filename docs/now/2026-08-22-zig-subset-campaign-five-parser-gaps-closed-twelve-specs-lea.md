# NOW -- Zig-subset campaign: five parser gaps closed, twelve specs leave the ledger (2026-08-22)

## Zig-subset campaign: five parser gaps closed, twelve specs leave the ledger (Closes #2426)

- Anonymous braced literal, anonymous struct return type, Zig slice spellings including the open-ended form, keyword-named enum values, and a struct-method skip that mistook a return type's brace for the body's. 712 specs: 541 generate to 553, ledger 171 to 159, zero regressions, and the ledger updated with the same compiler that changed it.
- Gap 5 is the one that mattered and it recurred on a documented hazard: W577's comment records methods being skipped brace-balanced, and the skip stopped at the first brace, which for an anonymous struct return type belongs to the type. lexer.t27 lost everything after line 362 while reporting an error 144 lines on. Two of my own earlier reports are corrected in the issue.
