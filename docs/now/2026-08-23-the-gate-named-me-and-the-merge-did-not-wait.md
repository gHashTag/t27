# NOW -- the gate named me, on time, and the merge did not wait (2026-08-23)

Refs #2325.

- The note I wrote ABOUT the withdrawal gate stated three of the retracted
  figures verbatim as examples, and `check_withdrawn_live` failed on master
  with my own name on the commit. Removed from the prose here rather than
  added to the exemption baseline: every exemption line is a small hole, and
  a note about withdrawals does not need to restate them.
- The sequel is the part worth keeping. The gate **ran** on the pull request
  that introduced the file -- it has no `paths:` filter, so it triggers on
  every PR to master -- it **failed**, and it **named all three lines** by
  file, line, matched pattern and the retraction covering it. The PR merged
  anyway: `withdrawn-live` is not among the four required checks, so
  auto-merge did not wait. It then stayed red on master through two further
  merges, neither of which was blocked by it.
- That is the audit's class I with a third variant beside "it never runs" and
  "it runs on the wrong diff": **it runs, it fails, it names the exact lines,
  and nothing stops the merge.** The figure never reached a reader -- no post
  or paper carries it and the file is repo-internal -- but the mechanism that
  was meant to stop it stopped nothing, and it was caught only because the
  gates were re-run by hand on master an hour later.
- Whether `withdrawn-live` joins the required set is an owner decision: a
  ruleset is a repository security setting. The same question stands for
  `Seal Coverage` and `Corpus Ratchet`, permanently red on master.
