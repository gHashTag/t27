# NOW -- a release is an effect in the journal (control.t27 section 7)

## specs/queen/control.t27 section 7 (Closes #7676)

- `EK_RELEASE` (4) is a GitHub release of a tag. Publishing one starts release.yml, which publishes
  the crate. The rules:
  - it can be looked up by its tag;
  - GitHub refuses a second release of the same tag (422 already_exists);
  - so a restarted Queen looks before it publishes, and never publishes twice.
- Tests: 18 of 18. `tri mutate spec` kills every mutant of `receiver_refuses_a_copy` (7 of 7) and
  `effect_can_be_looked_up` (9 of 9). The supervisor's vendored wasm typechecks the card with
  0 errors.
