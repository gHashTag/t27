# NOW -- A theorem about an empty module is held by a second ratchet (2026-08-28)

## A theorem about an empty module is held by a second ratchet (Refs #2747)

- 40 of the 73 recorded Rust/Lean disagreements are theorems about a module with no functions, globals or tests
- max_vacuous moves down only; both directions checked by breaking them
- nothing in the repository builds the Lean proofs -- 45 workflows, no lake build -- filed as #2747
