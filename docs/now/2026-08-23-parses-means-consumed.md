# NOW -- "parses" meant "a backend exited 0" (2026-08-23)

Refs #2325.

- `check_specs_parse.py` decided that a spec parses by running `gen-c` and
  reading the exit code. Measured: appending `xyzzy plugh plover` to a REQUIRED
  spec leaves **gen-c 0, typecheck 0, and check_specs_generate 0** -- the
  top-level drop-recovery discards what it cannot parse and says nothing.
- The compiler already ships the stronger answer and no gate was calling it.
  `t27c parse-complete` over the tree: 650 specs, 430 consume all, **66
  DISCARD 26,546 tokens**, 154 do not parse.
- Two of the four REQUIRED specs discard TODAY -- `ternary_mac.t27` 1139 and
  `systolic_ternary.t27` 1409, 2,548 tokens between them, including 41 of
  ternary_mac's 137 `invariant` declarations. So this is a ratchet with the
  counts frozen as named debt, not a demand for zero.
- Control, and it took three attempts to build one that exercises the new path
  rather than the old one: a malformed `fn` makes the spec fail to parse
  outright (old branch), and additions after the discard point are re-synced
  and not counted. Three bare words are what raises the count while every
  backend still exits 0: 1139 -> 1144, caught by name; the pre-change gate
  exits 0 on the identical tree.
