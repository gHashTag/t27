# NOW -- An array parameter that cannot be bound is passed by value (2026-08-28)

## An array parameter that cannot be bound is passed by value (Closes #2743)

- three refusals deleted: no call site, disagreeing call sites, non-identifier argument -- none of them a defect
- whole suite with --no-fail-fast: 2419 passed 5 failed -> 2424 passed 0 failed, no test file touched
- earlier 1786/1 figures were from runs that stop at the first failing binary; four of the five failures had never been printed
- blast radius 10 of 650 specs, all one direction: 19 refusals removed, 19 functions gained
