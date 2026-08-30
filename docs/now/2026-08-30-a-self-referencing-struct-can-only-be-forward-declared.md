# NOW -- A self-referencing struct can only be forward-declared (2026-08-30)

## A self-referencing struct can only be forward-declared (Closes #2948)

- C had no name in scope for a struct naming itself; a cycle of length one that no topological sort can order
- measured per spec, both directions: cc accepts 264 -> 268, four newly accepted, zero regressions
- four of 24 carrying specs -- the other 20 are blocked further down, b_tree on a generic type name reaching C
- a mutant the M5 freeze rejects is not a measurement: no build, no test, and no error line either
