# NOW -- A guard that ran a different shell from the thing it guarded (2026-09-07)

## A guard that ran a different shell from the thing it guarded (Refs #3396)

- ci-gates gains section 598. The `coq-kernel` Admitted step inherited the container's `sh -e` (dash) and used bash arrays, so it exited **2 before opening any file** -- the same exit clean and with an `Admitted.` planted. CI's own log on `f46050296` reads `61: Syntax error: "(" unexpected` / `exit code 2`, the same line number the local `/bin/dash` reproduction printed.
- The meta-gate written to protect that step extracts the body correctly and then runs it through `bash -c`, a different shell from CI, and passes 13/13 on the body dash refuses. **A guard must invoke its subject the way the runner will.**
- `sh` is a role, not a program: on macOS `/bin/sh` is bash 3.2 and accepts arrays, so it is a false-negative control. Ask `/bin/sh -c 'echo $BASH_VERSION'` before treating `sh` as evidence of POSIX-compatibility.
- A comparison table needs one row where the instrument moves. The first attempt gave four cells of `exit 0` because the extractor silently returned an empty body -- caught by making the extractor refuse, not by reading the table.
- Append-only: the file's first 916,851 characters are byte-identical to master, fenced-block lines unchanged at 347, sections 559 -> 560. Duplicate numbers 546/547/548 are pre-existing and filed as #3399 rather than silently renumbered.
