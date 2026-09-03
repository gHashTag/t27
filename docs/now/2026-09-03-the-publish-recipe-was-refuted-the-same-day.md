# NOW -- The publish recipe was refuted the same day it landed (2026-09-03)

## Two complete reads, three refusals, and a mechanism that cannot be inspected (Refs #3023)

- §456 landed a recipe for republishing the shared dashboard: fetch, then every check in one shell call, then an unbroken read, then publish. It had worked once
- it then failed three times in a row on the next publish. `fetch -> merge -> Read all 3061 lines -> publish` was refused as *"not built on it"*; resending was refused as *"identical content already refused"*; fetching again and publishing was refused with the same message
- **two COMPLETE reads of the same file in one turn**, each with nothing between the last Read and the publish, and neither counted
- three candidate mechanisms — a per-path base version recorded by the one successful publish, a refusal invalidating prior reads, a fetch invalidating them — and **no way to separate them from inside the session**
- measured cost: roughly **250k tokens** of context on one artifact in one turn, for a page already merged and verified on disk (3 expected line differences, counts closing at +22 chips / +8 entries / +3 options)
- so §456 is corrected from a recipe into a stopping rule: when a precondition cannot be inspected and its own stated instruction has been followed twice without satisfying it, a third attempt is a state change made while blind. Hand the artefact over and name what refused it
- the durable repair is #3023, not another read
