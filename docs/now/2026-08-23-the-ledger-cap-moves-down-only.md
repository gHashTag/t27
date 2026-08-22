# NOW -- the debt ledger grew when you ran the blessing command (2026-08-23)

Refs #2325.

- `check_specs_generate.py --update-baseline` rewrote the ledger
  unconditionally, so the debt list could GROW as a side effect of the very
  command the gate's own failure text tells you to run -- while the docstring
  asserts the number can only go down.
- The repository already had the right rule written down, for the corpus
  ratchet, in `bootstrap/src/suite.rs:2634`: *"raising the cap must be a hand
  edit in the pull request, never a side effect of running the blessing
  command."* Same file format, same intent, now the same behaviour.
- Controls, both directions: blessing a tree with two entries removed refuses
  (`REFUSING to grow the ledger: 169 -> 171`, exit 1, ledger untouched) and
  names the specs it would have admitted; blessing a tree with a fabricated
  extra line still SHRINKS 172 -> 171. Clean tree writes 171 byte-identical.
- `tools/specs_generate_baseline.txt` was also absent from
  `emit-bitexact-gate.yml`'s `paths:` -- so the pull request the gate's NOTE
  asks you to open, the one that edits the ledger, did not run the gate that
  validates it. Added.

## noticed in passing, not fixed

One ledger line's error MESSAGE drifted under a spec that still fails
(`codegen.t27`, "near line 644" -> "near line 647", different token). The
ledger stores messages, so a message that moves rewrites the file on every
blessing without any spec changing. Not touched here.
