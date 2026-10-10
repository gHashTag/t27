# NOW -- specs/hosting: self-hosting slice 1, decided in five cards (2026-10-11)

## specs/hosting/{host,placement,proof,credit,toolchain}.t27 (Closes #8707; gHashTag/trios#1756)

- Users' computers run shards of the corpus (`t27c test-report` on one spec at a pinned commit, read as the t27b lab reads it); the Queen believes a result when k = 2 hosts agree on its normalized output hash (one more host on a tiebreak, up to 3), credits the agreeing side 1 mTRI off-chain (node-work-credit.t27's per-job rate) and strikes a tamperer or a dissenter. Settlement is defined and off: payout.t27 gates any value flow on a device-bound identity no host has.
- host.t27: tiers by the owner's allowlist, never claimed above it; a public host never receives a secret or personal data; the lease is netlink.t27's, imported. placement.t27: one replica per host, one network per job among strangers, the lab's 300 s run bound. proof.t27: tri-net's (model_hash, input_hash, output_hash, ops) in signed_receipt.t27's message layout. toolchain.t27: zig 0.16.0 pins from ziglang.org; t27c pins empty until a release carries the binary.
- `t27c test-report`: 35 tests, 0 FAIL, 0 vacuous, 278 runtime asserts, 11 invariants. 34 negative controls: 31 FAIL by name, 3 BLOCKED by an invariant. Runtime: gHashTag/trios#1757 on branch hosting-mvp, behind TRIOS_HOSTING=on.
