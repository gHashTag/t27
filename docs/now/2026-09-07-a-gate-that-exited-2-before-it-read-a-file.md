# NOW -- A gate that exited 2 before it read a file (2026-09-07)

## A gate that exited 2 before it read a file (Closes #3396)

- The Kernel `Admitted` step declared no shell key and neither does its job, so it inherited the `coqorg/coq` container default `sh -e` -- dash -- while its body uses `VFILES=()`, `VFILES+=(...)`, `${#VFILES[@]}` and `done < <(...)`. Dash rejects the first with a syntax error before opening any file.
- Measured on the body extracted verbatim from the workflow, against the nine `.v` files `coq/_CoqProject` names: dash exits **2 on a clean tree and 2 with an `Admitted.` planted** -- identical, no discrimination -- while bash exits **0 and 1**. The bash rows are the positive control; without them the dash rows would only mean a broken probe.
- macOS `/bin/sh` is bash 3.2 and **accepts arrays**, so it cannot serve as the control here. `/bin/dash` was used, and the trap is recorded in the workflow comment so the next reader does not repeat it.
- Class enumerated, not assumed: of 50 workflow files 4 use a container, and this is the only step in any of them combining a container with a bash-only construct. The `<<<` in `rings-rust.yml` sits in the `discover` job, which has no container and already gets bash.
