# NOW -- The fuel floor scales with the lanes (2026-09-20)

## `--runway auto` is twice whatever the swarm reports (Closes #4326)

- A fixed floor is the same mistake the lane count was. 24 was chosen when the swarm ran ten bees; the day it ran twenty, one reading showed **about 12 dispatchable issues for 20 lanes** while the floor said 24 -- barely one round of work.
- The swarm reports its lanes on every reading, so the floor comes from them: `auto` means `2 x capacity`, with a floor of 4 for the case where capacity reads zero. A number still means that number and `0` still means off, so nothing that passed a number changes behaviour.
- Measured immediately after: `runway: about 12 dispatchable issue(s) against a floor of 40 (122 carry a boundary, 68 claimed, 42 completed)` -- and the feeder then had 221 uncovered specs to draw on.
- The 68 claimed are not a bug to fix here. A row that spent its retry ceiling is released once, after an hour, and then stays rejected on purpose: "a third identical failure is evidence about the issue, not about the attempt". They are a person's problem by design, and `work-parked` already names them.
