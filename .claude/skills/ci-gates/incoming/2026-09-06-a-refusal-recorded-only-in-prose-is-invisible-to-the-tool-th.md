## A refusal recorded only in prose is invisible to the tool that advises against it

`tri gates unmeasured` prints a `dispatch:` column and a footnote telling the reader to add
`workflow_dispatch:` wherever it reads `NO`. The predicate was a bool, so it could not tell a
workflow that is MISSING a dispatch from one that REFUSED one.

#3325 had just removed the dispatch from `release.yml` deliberately: every job keys off
`github.event.release.tag_name`, empty on a dispatch, so preflight refuses and nothing publishes.
The reason went into a YAML comment. The tool does not read YAML comments. Measured on #3325's
head, before the repair:

```
2026-08-28  NO   -   Release Pipeline  (gHashTag/t27)
...
`dispatch: NO` means the reading cannot be taken on purpose -- add `workflow_dispatch:` first.
```

So one pass instructed the next to put a dispatch back in front of `cargo publish` and
`npm publish` against live registries, where a version is permanent.

- **A two-state predicate over a three-state domain is the shape.** Present / absent / absent
  on purpose. Where the domain has a third state, the column has to carry it: `Yes / No / Refused`.
- **Record the refusal where the TOOL looks**, not only where a human reads. `# tri:no-dispatch
  <reason>` in the workflow file; the tool greps for it. A comment addressed to a reader cannot
  stop a tool from advising the opposite.
- **A present value must win over the marker**, so a stale comment cannot hide a real dispatch.
- The generalisation, and the reason this matters to a scheduled loop: *a guard against undoing a
  decision cannot live in prose, because the thing doing the undoing is a tool.*
