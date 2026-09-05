# NOW -- The conflict gate read a different operand than the commit (2026-09-05)

## The conflict gate read a different operand than the commit (Refs #3301)

- Staging a file with a marker and then cleaning the working copy: `tri hooks pre-commit`
  exits 0, prints `PASSED`, says nothing about conflicts, and the commit carries the
  marker. Reproduced independently before acting on the report.
- The checker read the WORKING TREE; `git commit` takes the INDEX. A gate that reads a
  different operand than the one being committed is not a barrier.
- `--staged` mode added: the population is `git diff --cached -z --name-only` and the
  bytes come from `git show :<path>`. `-z` because `--name-only` C-quotes non-ASCII
  paths and a quoted path leaves the population silently -- the same defect found in a
  different reader earlier today.
- `.lock` left `SKIP_SUFFIX`. It is TEXT, and 55 tracked files were leaving the
  population, 6 of them lock files. An exclusion is only as wide as its reason, and the
  reason here is "cannot be read as text".
- The caller discarded the checker's stdout on success, so the population was invisible to
  the only person who could act on it. It prints now, pass or fail.
- Exit 2 was reported as `a tracked file carries a conflict marker`. Could-not-run and a
  finding are different facts; conflating them is a false accusation that also hides a
  broken gate.
- Found by an adversarial workflow over the four remaining pre-commit gates: 8 candidates,
  8 survived refutation, 0 refuted. Every one of the eight is the same structural error --
  the gate reads a different operand than the thing it gates.
