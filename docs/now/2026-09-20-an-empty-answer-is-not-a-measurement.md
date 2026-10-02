# NOW -- An empty answer is not a measurement (2026-09-20)

## Eight issues asked for a criterion nobody can satisfy, on work the oracle had passed (Closes #4464)

- Measured in the review log: `#4448 unmet=1 passed=4 failed=1 oracle=pass`. The bee had done the work, the generated Zig compiled and ran, four criteria of five were met -- and the fifth read:

  ```
  - 4. `t27c spec-status specs/fpga/router.t27` prints `` (today: )
  ```

- `spec-status` printed **nothing** at feed time (a stale symlink in the bee-bin directory is enough), and `claims_hold` compared the measured value with the claimed value - both the empty string - and let it through. The check that exists so no number is ever typed rather than measured could not see the one case where there is no number at all.
- Eight issues went out that way. Every one of them came back `sendBack` at four criteria of five, against an oracle that passed. The feeders now refuse an empty answer, in both places, and the eight open issues were repaired in place: every one of them now reads `prints \`IMPLEMENTED\``, re-measured with the compiler.
- The wider lesson is the same one this repository keeps paying for: an instrument that cannot distinguish "nothing" from "zero" will eventually report one as the other. The pusher learned it this morning (`github_readable`), the feeder learned it at noon (`skips_are_fresh`), and this is the third place.
