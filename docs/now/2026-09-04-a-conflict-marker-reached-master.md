# NOW -- A conflict marker reached master (2026-09-04)

## Three markers in SKILL.md, put there by my own PR

- `.claude/skills/ci-gates/SKILL.md` carried `<<<<<<< HEAD`, `=======` and
  `>>>>>>> origin/master` at lines 12730, 12785 and 12829 on master, merged in
  #3072.
- Cause: an automated conflict resolution of mine handled
  `.github/workflows/untrusted-input-gate.yml` and then ran `git add -A` and
  committed. The conflict that time was in SKILL.md, and the markers went with
  it. Both sides were my own sections, both numbered 479.
- Repaired by keeping both and renumbering the second to 480. Asserted: zero
  anchored markers, unique and ascending numbers, no duplicate titles.
- `tools/check_conflict_markers.py` DID see it -- "conflict marker on line
  12730, 12829" -- so the instrument was right and the merge happened anyway.
- The guard now in the procedure, and it stopped the next commit an hour later:
  `git diff --cached --name-only | xargs grep -l '^<<<<<<<'`, anchored at line
  start because this file legitimately quotes the marker in prose. A substring
  test refused a clean file over its own documentation on the first attempt.
