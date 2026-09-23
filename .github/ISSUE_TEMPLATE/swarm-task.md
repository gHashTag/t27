---
name: Task for the swarm
about: Work a bee can pick up. The Boundary section is what makes that possible.
title: ''
labels: ''
assignees: ''
---

<!--
THE BOUNDARY SECTION IS NOT PAPERWORK. IT IS WHAT LETS A BEE TAKE THIS ISSUE.

The Queen reserves files before she dispatches, so that two workers never edit
the same path at once. An issue that does not say which paths it touches cannot
have anything reserved for it, so it is skipped -- silently, forever.

Measured 2026-09-23: 563 of 654 open issues had no Boundary section, and the
Queen's own tick reported `missingBoundary` for 565 candidates while the swarm
sat at 2 of 20 lanes. The backlog was not empty. It was unreachable.

Delete these comments before submitting.
-->

## Summary

What is wrong, missing, or hand-written where a spec should be. One paragraph.

## Boundary

<!--
One path per line. A bee may create or edit files under these paths and nowhere
else; anything it changes outside them is named in the review rather than
discarded. Directories are fine, and so is a single file.
-->

- `specs/port/tools/example.t27`
- `tools/example.py`

## Acceptance

<!--
How the work is judged. Prefer something that can be run over something that
has to be believed: a test that passes, a command that exits zero, a generated
file that matches.
-->

- [ ] The spec compiles and its own `test` blocks pass
- [ ] `Closes #N` is in the pull request (law L1)
