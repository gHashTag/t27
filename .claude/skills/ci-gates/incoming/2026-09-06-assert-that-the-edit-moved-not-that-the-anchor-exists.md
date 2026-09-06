## Assert that the edit moved, not that the anchor exists

Twice in one pass a scripted edit reported success and changed nothing.

**First:** patching two footnotes, each guarded by `assert s.count(anchor) == 1`. Both asserted
true. The second replace was still a no-op, because it searched for `anchor` but replaced
`prefix + anchor`, and the prefix did not match. The script printed "both footnotes patched".

Caught by a structural test -- 5 passed, 1 failed. Without it the table would have printed
`refused` under a footnote still saying "add `workflow_dispatch:` first": the repair shipped
half-applied, in the column it was repairing.

**Second, worse:** `i = s.find('        <div class="chips">')` returned **-1** because the real
indentation was four spaces. `s[-1:j]` is the empty string, and `str.replace("", new, 1)` inserts
at position 0. Seven chips landed at the top of the file, above `<title>`, and the "did anything
change" guard passed happily -- something had changed, just not the right thing.

- "The anchor exists" is a claim about the INPUT. "The edit applied" is a claim about the RESULT.
  Assert the second: `before = s.count(needle); ...; assert after == before + 1`.
- Guard every `find()` with `assert i >= 0` before slicing. A negative index makes a silently
  legal, silently wrong slice.
- Same class as a mutation harness that fails on its anchor before writing the file: a rewriter
  that did not rewrite must exit non-zero, not report success.
