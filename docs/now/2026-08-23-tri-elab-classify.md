# NOW -- `tri elab` and what it found on its first run (2026-08-23)

Refs #2325, #2433.

- The hand-typed pass that caught the ratchet's phantom count is now a
  command. `tri elab classify` prints the distribution of iverilog's
  diagnostics BY MESSAGE SHAPE, with the summary lines separated out instead
  of counted. `tri elab secondary` separates diagnostics that sit on the same
  source line as their own cause from the ones that are independent work.
- Numbers it gives today: 162 real diagnostics, 26 summary lines excluded,
  and of the 162 only **22 are independent** -- 82 are the two design
  decisions and 58 vanish with them. "68 of 161" was the previous framing;
  22 is the number that describes remaining work.
- The command found a defect in ITSELF on the first run. iverilog writes one
  diagnostic with doubled delimiters, ``Enable of unknown task ``x''.``, and
  the shape collapser toggled on either quote character -- so one row became
  three, in the command whose job is to stop a distribution from lying. Fixed,
  with a test that fails when the old logic is restored.
- `--names` surfaced `_name`, an identifier with an EMPTY base. Traced:
  `mod.ports[i].name == mod.ports[j].name` compiles to `(_name == _name)`.
  A string field has no offset, so the field access falls past the
  part-select branch into the flatten path, the base flattens to nothing, and
  both indices are lost. Two such comparisons exist, both in hir, both in
  duplicate-detection loops. Filed on #2433: this is not only an unbound name,
  it is a comparison that became self-comparison, and giving strings a width
  later would turn it into a silent `always true`.
