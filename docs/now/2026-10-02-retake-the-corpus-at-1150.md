# NOW -- Re-take the corpus figures at 1150 specs (2026-10-02)

## The 1146 re-take had moved again, and master passed by coincidence (Refs #5497)

- Four spec PRs merged after the 1146 block (#5597), so master at `748ebda20` has 1150 `.t27` files under `specs/` outside `specs/scratch`.
- `scripts/ci/test_retaken_propositions_still_match.py` still passed on master, but only because the guard checks `str(walked) in doc` over the whole document and "1150" already occurs there as a lesson number (line 23435). No re-take quoted 1150. Any PR that adds one spec moves the count to 1151 and fails Untrusted Input Gate; #5609 is the first one seen.
- New block anchored at `748ebda20`, with `t27c impl-status --specs-dir specs` run on that tree: 740 fully implemented, 350 with no functions, 16 partly written, 17 unwritten, 27 that do not parse; 6764 functions declared, 187 with no body. The compiler is the one built from `56772f5c`; its `bootstrap/` tree (`e8cca94d`) is identical at the anchor.
- Against `4f65684d`: +4 specs, all fully implemented; +43 functions, none without a body.
- The guard is unchanged. Its substring match can be satisfied by an unrelated number; narrowing it to the re-take blocks is a separate change and is not made here.
