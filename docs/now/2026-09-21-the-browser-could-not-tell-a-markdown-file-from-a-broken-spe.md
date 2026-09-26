# NOW -- The browser could not tell a Markdown file from a broken spec (2026-09-21)

## The browser could not tell a Markdown file from a broken spec

- t27c classify's rule moved into bootstrap/src/source_kind.rs; t27c and the wasm bridge now share one copy, and analyze_source returns sourceKind
- 81 of the 338 files the Spec Explorer called broken are not modules at all; 31 other non-modules compile cleanly, so this is a separate axis and not a fourth health state
- classify --verbose now says when its 40-file-per-class listing is truncating
