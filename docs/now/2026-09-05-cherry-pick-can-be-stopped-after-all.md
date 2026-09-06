# NOW -- cherry-pick can be stopped after all, and my note said it could not (2026-09-05)

## cherry-pick can be stopped after all (Refs #3329)

- Section 594 and its note said `cherry-pick` runs neither commit hook and git offers
  none that could stop it. That was wrong, and the reason is worth more than the fix.
- The probe carried markers for SIX hook names. `cherry-pick` fires
  `prepare-commit-msg`, and `git am` fires `applypatch-msg` and `pre-applypatch` --
  none of which were in the list. **"Nothing fired" can mean "I did not look".**
- Re-measured over the full thirteen. A non-zero exit from `prepare-commit-msg` aborts a
  cherry-pick (128, no commit) and from `applypatch-msg` aborts a `git am` (1, no
  commit). Verified directly, both.
- Three hooks added. `prepare-commit-msg` gates only when `.git/CHERRY_PICK_HEAD`
  exists, so an ordinary commit still pays exactly one barrier run -- verified, and the
  first attempt at that control was void because the commit had been refused by the census
  for an unrelated reason.
- Controls: a cherry-pick of a commit carrying a conflict marker exits 128 with
  `carries a conflict marker` and creates nothing; the same through `git am` exits 1
  with the same sentence, once the index is clean -- the first run of that control was also
  void, refused by git for a dirty index rather than by the hook.
- `rebase` fires nothing at all and remains genuinely uncovered. That gap is real.
- Blessing two census moves that are NOT mine: `quiet` 128 to 127 and `shell` 235 to
  234, from a neighbour repairing two workflows. Confirmed on a clean origin/master
  checkout with the same binary before blessing, rather than assumed.
