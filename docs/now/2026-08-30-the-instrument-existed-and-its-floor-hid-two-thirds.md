# NOW -- The instrument existed and its floor hid two thirds (2026-08-30)

## The instrument existed and its floor hid two thirds (Refs #2914, Refs #2915)

- `tri gates dead` already asked this question; its shipped `--min-runs 50` reported **2** of the **6** workflows in the tree that have never succeeded
- among the four it hid: `brain-seal-refresh.yml`, whose last step is a `git push` the repository's own ruleset answers with GH013, 8 runs and 8 rejections over five months
- few runs is not few enough to be safe -- a structurally impossible workflow fails every time it runs, and runs rarely
- `state=="active"` is the API's word, not the repository's: **61** registrations against **48** files, so 9 of the 15 it found have no file to fix
- `coq-proofs.yml` has failed 62 of 62 at `opam init` -- step 2 of 5, and step 3 is the one that calls `coqc`; its thirteen files have never been compiled
- 58 Coq files in the tree, **9** compiled by anything; 35 sit outside every root
- read the FIRST failing step before the last: a gate stopped during setup has never reached what it checks, and fixing what it checks repairs nothing
