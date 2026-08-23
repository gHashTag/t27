# NOW -- a withdrawal could be reversed by deleting one line (2026-08-23)

Refs #2325.

- `check_withdrawn_live.py` asked one question -- is a retracted number stated
  in a live document -- and guarded only against the registry being emptied
  ENTIRELY. Deleting a single row was invisible.
- Measured by removing each of the 7 rows in turn: **6 of 7 vanished with the
  gate printing OK and exiting 0**. The seventh, the founding frequency
  claim, is pinned only
  because `self_check()` happens to hardcode that string. Coverage genuinely
  lapses -- planting the throughput figure scores 1 hit with the rule and 0
  without it.
- The reverse direction now runs before the forward one. No new data file: the
  baseline already records WHICH pattern excused each accepted occurrence, so a
  pattern present there and absent from the registry is a deleted rule whose
  exemptions stayed behind.
- After: **0 of 7 delete silently.** Four controls held -- `--update-baseline`
  writes the same 20 entries byte for byte, `--self-check` still passes, a new
  rule matching nothing keeps the tree green (a rule with zero hits is the
  healthy end state, so "every rule must have a hit" would be the wrong gate),
  and a planted live occurrence of a retracted figure is still caught by name.

## the gate caught ME, an hour later

This very file stated three of the retracted figures verbatim as examples, so
`check_withdrawn_live` failed on master with my own name on the commit. The
numbers are gone from the prose above. Adding them to the exemption baseline
was the other option and the gate offers it, but every exemption line is a
small hole, and a note about withdrawals does not need to restate them.

And the sequel is worse than the slip. `withdrawn-live` FAILED on the pull
request that introduced this file, named all three lines, and the PR merged
anyway -- it is not among the four required checks, so auto-merge did not wait
for it. It then stayed red on master through two further merges. The gate was
right, on time, by name, and the merge machinery did not care. That is the
audit's own class I with a third variant: not "it never runs" and not "it runs
on the wrong diff", but "it runs, it fails, and nothing stops the merge".
