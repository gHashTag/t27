# NOW -- Restore topology and Admitted CI (2026-09-12)

## Two existing CI failures (Refs #3574)

- Classify Gate Topology and Untrusted Input Gate as merge-critical in the topology checker and lower the unclassified ceiling from 27 to 26. This changes checker coverage, not GitHub branch protection.
- Exercise the topology checker against disposable positive and negative workflow trees, including branch filters, missing or malformed classified guards, and growth beyond the unclassified ceiling.
- Extract the named Admitted step structurally from workflow YAML, require its explicit Bash shell and a unique non-empty body, and execute existing file-population fixtures with GitHub's strict Bash flags.
- Declare PyYAML in Untrusted Input Gate and test extraction across comments, key ordering, indentation, and invalid or ambiguous step definitions.
- The explicit PyYAML installation adds one runner-shell step: shell census `run: steps` moves 248 to 249 and `the runner does` moves 227 to 228. Update the shell ledger with this intentional population change; no step is removed or hidden.
- Validation: all local Gate Topology commands and all 11 Untrusted Input Gate check steps pass. Hosted CI is to be verified on the separate PR; no Coq production step, compiler behavior, specification, or branch-protection setting changes.
