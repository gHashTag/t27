# NOW -- triage tells blocked from actionable (2026-08-21)

## tri triage: five ordered classes, and blocked is tested before actionable (Closes #2156)

- `tri triage` had three classes and made `actionable` the default, so anything it did not recognise was counted as available work. Three kinds of issue cannot be picked up and finished -- blocked on hardware or a human, research with no checkable end, duplicate or obsolete -- and all three were being advertised. The reported count was an upper bound read as an estimate
- Now five classes matched in order: actionable, research, tracking, blocked, duplicate. `blocked` is tested before `actionable` on purpose, so no blocked item is offered as available. Measured over 241 open issues: 45 actionable, 0 research, 189 tracking, 7 blocked, 0 duplicate. The seven blocked items were previously inside the actionable count
- `research` scoring zero is a limitation of the signal, not a finding: the rule reads title form, and this tracker's titles are declarative sentences rather than questions. Reported as measured rather than tuned until the histogram looked plausible
- The tool prints and exits. Autoclose is forbidden: 80% of the tracker is journal record with no completion condition, and a title regex is not grounds to destroy it. The classification is a composition estimate, never a verdict on one issue, and the output says so
- Loop helpers now dispatch from `scripts/tri_loop/` via `tri <name>`, with `tri loop-help` listing them. They report and never mutate; keeping them out of `t27c` means a broken helper cannot take the compiler CLI down with it
- Entry migrated from `docs/NOW.md` to `docs/now/` (the layout #2298 introduced); the original entry was dated 2026-08-14. The branch's own commit had deleted the heading `# NOW -- BNF: the control that measures what ternary is worth (2026-08-09)` while keeping its body, orphaning it; `docs/NOW.md` is restored to master byte-for-byte, so that heading survives
