# NOW -- specs/hosting: slice 1b lane A, a host that does the lab's whole row (2026-10-11)

## specs/hosting/{toolchain,host,proof,placement,row}.t27 (Closes #8753; gHashTag/trios#1761)

- toolchain.t27: t27c and t27b are pinned (URL, SHA-256, size) per platform, all built from master cf42584c9 and served by the gHashTag/trios prerelease trios-host-v0.1.0; the sandbox-exec profile and the bwrap arguments are data. row.t27 (new): a lab row is two halves, the reference and t27b, each its own k-of-n job, complete when both are agreed and assembled the lab's way. placement.t27: the t27b half goes to arm64 hosts only.
- host.t27 section 8: isolation levels none / no-network / job-dir-only / VM; a public job needs job-dir-only wherever it runs; the host measures its level with a planted job and signs it into its registration (proof.t27). proof.t27: the reference half's output v2 carries the lab's blocked reason.
- `t27c test-report`: 45 tests, 0 FAIL, 0 vacuous. 15 negative controls, each failing its test by name. t27b passes all five specs. Runtime: gHashTag/trios#1762 on branch hosting-mvp.
