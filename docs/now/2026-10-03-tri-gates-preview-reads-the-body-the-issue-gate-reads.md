# NOW -- tri gates preview reads the body the issue gate reads (2026-10-03)

## cli/tri/src/gates.rs -- check-linked-issue asks what the gate asks (Closes #5744)

- The gate (`issue-gate.yml`, step "Check for linked issues in PR") greps the
  title as written and only the prose of the body: an awk program, added under
  #3388, drops every line inside a three-backtick fence and every line that
  opens with `>`. The preview matched the gate's pattern against the whole
  body, so a pull request whose only `Closes #N` sat in a fence or a quote read
  PASS here and was refused there -- a false PASS on a required context.
- The awk program is now read out of issue-gate.yml on every run
  (`prose_filter`) and run with the system `awk` on the body, fed as the gate
  feeds it. The shape around it is read too: one `BODY_PROSE` assignment, the
  body piped straight into awk, the substitution closing after the program,
  and the grep reading the title and the filtered body. Anything else reads
  UNAVAILABLE, never PASS.
- The pattern is now matched one line at a time, as grep matches it. Over the
  whole text `\s*` crossed a newline, so `refs` ending one line and `#5`
  opening the next read PASS here and failed the gate. The commit proxy uses
  the same line rule, which is how l1-traceability.yml greps commits.
- The title is still matched as written: the gate filters only the body.

## Tests

- Seven new tests in `preview_tests`: a reference only inside a fence and only
  in a quote do not pass; plain prose, prose after a fence and a quote-shaped
  title still pass; a keyword and its number on two lines are no reference;
  the program is read out of the real issue-gate.yml and strips what the gate
  strips; five altered gates (a pipe stage before or after awk, a grep of the
  raw body, another job, a second assignment) read None.
- A thirteen-case table runs the gate's own step under bash and the row side
  by side, and checks both against the stated answer. It keeps the gate's
  quirks: a `~~~` fence, a four-space indent and `prefs #1` all count for the
  gate, so they count for the preview. Whether the gate should strip those is
  not decided here.
- Mutation check: grepping the raw body again fails 3 tests, matching across
  newlines fails 2, filtering the title fails 2. `cargo test -p tri`: 841
  passed, 0 failed. No pinned census moved.
