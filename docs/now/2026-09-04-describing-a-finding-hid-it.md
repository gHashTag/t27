# NOW -- Describing a finding hid it from its own detector (2026-09-04)

## The controls caught what a smaller number would have hidden

- `tri deliverables` turns the hand-run measurement behind #3140 into a helper: backticked
  identifiers in `docs/reports/`, minus every one appearing in a **code** file, minus those no
  report presents with a verb of addition or change.
- The two hand-verified instances are wired in as **controls**, and on the first real run the tool
  printed **CONTROL LOST** instead of a number. Cause: the hand-run counted everything outside
  `docs/` as source, and the skill sections written *about* this finding put `ExprAddressOf` and
  `has_cycle_dfs` into `.claude/skills/`. **Describing a finding must not hide it from its own
  detector**, and a tool without controls would have reported a slightly smaller number and been
  believed.
- Source now means code: no `docs/`, `.claude/`, `.trinity/`, and no `.md` or `.txt` anywhere.
  The funnel is **3884 -> 513 -> 117**; the hand-run said 458 -> 105, so #3140's published figure
  was **under** by twelve.
- **It then said CONTROL LOST about itself.** Once tracked, this file's own docstring -- which
  names both controls, as it must to explain them -- counted as source and ate them; the count fell
  by exactly two. `check_documented_commands_exist.py` excludes its own path for the same reason.
  Fifth occurrence of this shape here, and the first where the detector's *docstring* was the
  offending source.
- Not documentation drift. Dredd, Schemathesis, Vale and docs-as-tests all assume the document
  described something real once and the code moved. Here the code never existed -- there is no
  contract to test, only a name, which is why a grep is the whole instrument.
- Also fixed: `tri loop-help` read each helper's description with a sed that did not allow a raw
  docstring, so a helper whose prose carries regex escapes would be listed blank. **Both** copies
  of that line are updated -- the dispatch block is duplicated, see #3055.

Refs #3140
