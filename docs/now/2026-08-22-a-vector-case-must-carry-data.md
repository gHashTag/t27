# NOW -- a vector case must carry data, or it is documentation (2026-08-22)

## the prose-only class is frozen at 24 and cannot grow (Refs #2241)

- 412 of 512 conformance cases carry nothing but an id and a sentence. They
  accumulated because nothing objected: a file landed in conformance/, the
  summary counted it, and "34 vector files" read as coverage while the number
  actually applied to RTL was zero.
- tools/check_vector_data.py freezes the 24 existing prose-only files as named
  debt (the repository's own baseline pattern) and fails when a NEW one lands
  or when a listed file gains data without the baseline being updated. Wired
  into fpga-conformance ahead of execution.
- Both directions controlled: a planted prose-only file is caught with its
  case count and exits 1; a planted file with one input field passes; removing
  both returns the gate to 0.
