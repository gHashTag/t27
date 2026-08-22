# NOW -- I audited my own gate and it lost on four counts (2026-08-23)

Refs #2325. A fan-out audit put five questions to every gate script in
`tools/` -- the questions `check_elab_ratchet.py` failed this morning. 43
findings survived independent refutation; these four are the ones in the file
I wrote today, verified by hand before acting on them.

- **The classification table was wrong when written.** 57+64+21+4+5+2 = 153
  under a headline of 161. Within the hour three rows were also stale (unbound
  72, condition 59, malformed 0 -- that defect closed in the next commit) and a
  seventh class was never listed. Deleted: the docstring now points at
  `tri elab classify`, which measures it.
- **The ratchet looked in one direction.** It iterated `now` only, so a module
  that stops being generated contributed no row, the total fell, and the gate
  printed OK. Now iterates the union; a baseline module absent from the output
  is a GONE failure. Control: a phantom baseline line drops the total by 7 and
  exits 1.
- **`--update-baseline` deleted the hand-written hazard note** explaining that
  removing a syntax error can RAISE a count -- the command the gate itself
  recommends. Notes below a sentinel now survive. The first version of that
  preservation bounded the section by a blank line, which this writer never
  emits: the SECOND consecutive run swallowed the module list into the notes
  and wrote 64 modules with 32 duplicates. Reproduced, then bounded by the
  comment lines themselves; three consecutive runs are now stable.
- **The BETTER branch congratulated without hedging.** A syntax error truncates
  the file and collapses a module's count; the gate said "Modules improved.
  Record it." and obeying it froze the truncated number. It now asks for a
  classification first.
- **`fpga-build.yml` did not trigger on its own gates.** A one-line edit to
  `check_vector_data.py` turning `return 1` into `return 0` lands in a PR that
  never runs it. Five paths added to both blocks.
