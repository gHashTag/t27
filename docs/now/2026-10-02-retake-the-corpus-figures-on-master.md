# NOW -- Re-take the corpus figures on master (2026-10-02)

## The re-take guard quotes the corpus the tree has again (Refs #5497)

- `scripts/ci/test_retaken_propositions_still_match.py` re-counts the `.t27` files under `specs/` outside `specs/scratch` and requires that number to appear in the re-take blocks of `docs/theory/IGLA-FORMAL-RESULTS.md`. Master had moved to 1146 specs, the newest block said 1136 (on a branch) or older figures, and Untrusted Input Gate failed on it.
- New block anchored at `4f65684d`, with `t27c impl-status --specs-dir specs` run on that tree: 736 fully implemented, 350 with no functions, 16 partly written, 17 unwritten, 27 that do not parse; 6721 functions declared, 187 with no body. The compiler is the one built from `d4c4f2f4`; its `bootstrap/` tree is identical at the anchor.
- The guard is unchanged. It fails the next time a bee port moves the corpus, because those PRs do not re-take the figures; the PR that adds specs is expected to, as #5473 did.
