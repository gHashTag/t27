# NOW -- Four required-context defects closed, the fifth priced (2026-09-06)

## Four required-context defects closed (Refs #3388)

- `Closes #0` passed `check-linked-issue`, and `gh issue view 0` answers "Could not
  resolve to an issue". The number must not be zero now.
- A reference inside a fenced code block or a `>` quote counted as traceability. Fences
  and quoted lines are stripped from the body first; the title is not markdown and is
  matched as written. This repository already carries the same lesson about its own skill
  file, where a parser counted headings quoted inside code blocks.
- `check` missed a rename: `--diff-filter=A` reports an R entry, so a docs/now path
  whose content newly lands by rename never reached the shape reader. `--no-renames`.
- `validate` decoded with `errors="replace"`, turning invalid bytes into U+FFFD and
  parsing the repaired text. Measured first: **0 of 2086** tracked files are non-UTF-8, so
  the strictness costs nothing today and closes the hole anyway.
- Six controls on the issue-gate logic, including the one that matters: a real reference
  AFTER a fence is still found, so the stripping does not swallow what follows it.
- The fifth is left: bare `Infinity` is accepted by CPython and refused by
  `node -e JSON.parse`. Exactly one tracked file carries it -- my first count said 20,
  and 19 of those have it inside strings. The file is GENERATED and READ by two tools, so
  the encoding is a conformance-data decision; shipping the stricter parse without it
  would leave a REQUIRED context red and block every merge.
