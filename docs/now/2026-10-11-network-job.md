# NOW -- specs/jobs/network_job.t27 + specs/network/job_rules.t27: a network job the Queen runs, from a commit to a test credit (2026-10-11)

## specs/jobs/network_job.t27, specs/network/job_rules.t27 (Closes #8790; gHashTag/trios branch mvp-live-dispatch)

- The card: one commit; CHECK job-commit, EFFECT lab-challenge (a 40-hex challenge into each lab's request queue; t27b-lab-2 has no request path, see #8791), WAIT lab-receipts, CHECK receipts-verified, CHECK quorum, EFFECT settle-credit, WAIT challenge-window; the five registered keys and their one operator.
- The rules, compiled to wasm (`t27c gen` + zig wasm32-freestanding, 15 KB, two builds one hash): `receipt_auth` judges a receipt's real bytes -- corpus message, commit, signed_receipt.t27 order, ed25519.t27 verify, nonce = challenge -- so no signature is ever an input bool; `independent_votes` dedupes by key, then operator; `quorum_verdict` against N4/F1/M3 (TEMP copy of mvp_v2); `settle_outcome` through trinet/ledger.t27 settle(), a replay answering ALREADY_SETTLED; `credit_state` PENDING -> FINAL.
- Real bytes: the tests verify t27b-lab's and t27b-lab-2's signed receipts of master ddb84c5f5 (keys a05db80f53c317f6, fed03daa6459a7fa). The two labs agree and are ONE independent vote: QV_BELOW_QUORUM, a test credit at most.
- `t27c test-report`: 10 tests, 0 FAIL, 0 vacuous; `t27b test`: both pass. 8 negative controls, each failing its test by name.
