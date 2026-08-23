# NOW -- Ten gates had never been run in an empty tree (2026-08-24)

## Ten gates had never been run in an empty tree (Closes #2161)

- check_gate_preconditions ends with '0 known-uncovered', which counts rows it chose to write and not gates. Six of sixteen gate scripts are exercised; ten had never been run in an empty tree.
- Running them: two skip by design and fail under --require, five crash, two refuse clearly, and one PASSED — check_json_parses printed 'OK: 0 tracked JSON files' and returned 0. Zero tracked JSON is not a clean repository, it is not this repository, so it is broken() and not skip().
- The meta-gate now prints 'coverage: 6 of 16' and names the ten. Reported, not enforced: a gate that is red on the day it lands teaches people to ignore red.
- Adding the import broke four of the gate's own controls at once, all with stdout empty — the planted copy died on ImportError because the plant copied the gate and not its dependencies. A plant that copies the subject but not what it imports runs a different program.
