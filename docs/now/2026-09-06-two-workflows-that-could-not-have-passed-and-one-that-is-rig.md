# NOW -- two workflows that could not have passed, and one that is right to be red (2026-09-06)

## two workflows that could not have passed, and one that is right to be red (Refs #3316)

- coq-proofs has never passed: options --user root makes OPAMROOT default to /root/.opam while the image initialises opam under /home/coq/.opam. The log says it exactly -- Opam has not been initialised, exit code 50. Setting OPAMROOT and re-entering the switch in each step is the whole fix.
- coq-proofs also could not verify its own repair: push: lists the workflow file, pull_request: did not, so a PR that fixes it would not run it. Added, which is what lets this PR check itself.
- brain-seal-refresh fails at Commit brain seals, which runs git push to master and is refused by the ruleset -- GH013, remote rejected, on all three runs since 2026-04-07. The step is removed rather than worked around; the seals were already kept as an artifact, and the schema validation it was burying is the signal worth having.
- lean-proofs is NOT a workflow defect and my earlier diagnosis of it was wrong. 8571 of 8574 targets build; the two failures are marked LEFT FAILING, DELIBERATELY in H4Lagrangian.lean at 73 and 108, because norm_num does not evaluate Real.pi, Real.exp or Real.sqrt. The job is reporting the truth.
