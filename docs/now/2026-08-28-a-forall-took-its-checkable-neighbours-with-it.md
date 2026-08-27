# NOW -- A forall took its checkable neighbours with it (2026-08-28)

## A forall took its checkable neighbours with it (Refs #2161)

- Refs #2161. An invariant with `assert g(1) == 111`, then a forall, then `assert g(2) == 222` lost BOTH asserts. Skipping an unbounded forall is a defensible language decision -- the compiler says so in its own comment -- but taking the clauses around it is a second loss on top of the defensible one, and nothing recorded that it happened
- Clauses lowered BEFORE the unmodellable one are now kept, and the block is MARKED. The mark matters as much as the keeping: the NOT CHECKED notice keys on children.is_empty(), so a partial block without it would report as fully verified -- the exact claim W635 exists to stop
- Measured: assertions in generated Zig 11704 -> 11712 (+8), NOT CHECKED markers unchanged at 1068, specs that LOST an assertion ZERO, parse 620 -> 620, RATCHET CLEAN. Eight is the honest number -- the census that motivated this estimated ~9400 recoverable tokens, but that counterfactual REWROTE THE SOURCE while this keeps what the existing parser already lowered
- Found with the compiler own instrument: T27_BDD_DEBUG named the fallback site on the first try, after I had guessed the wrong one, edited it, and watched the probe not move. The tool that answers exactly was already in the tree
