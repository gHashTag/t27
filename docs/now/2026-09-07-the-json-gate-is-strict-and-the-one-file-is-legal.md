# NOW -- The JSON gate is strict, and the one file it refused is legal now (2026-09-07)

## The JSON gate is strict (Refs #3388)

- `validate` accepted bare `Infinity`, which CPython documents as "an extension to the JSON
  specification" and RFC 8259 has no literal for. The independent check was another
  language: `node -e JSON.parse(...)` refused the file this gate passed.
- Exactly one tracked file carried it in value position -- 19 others have the word inside
  strings and always parsed.
- **The encoding was not chosen, it was read off the corpus.** 23 conformance files already
  write these values as JSON strings and one did not, and `tools/wp18_selftest_gate.py`
  uses `"inf"` / `"nan"` in its own fixtures. `"inf"` outnumbers `"Infinity"` 37 to 2.
- Nothing is lost: every affected row already carried the same value as `..._hex`
  (`0x7FF0000000000000` IS +infinity), so the authoritative bits were in the file in a
  legal form and the bare literal was redundant.
- The file's sha256 is pinned in `INDEX_all_formats.json` and in `README.md`. Both updated;
  the old hash appears nowhere afterwards, checked.
- The consumer is unchanged in behaviour: `wp18_selftest_gate.py` exits 0 with the **same
  30 PASS** before and after.
- **This pull request is not on auto-merge.** It re-stamps a hash on a conformance artifact
  for the one format that has an FPGA oracle behind it. The encoding question was settled
  by evidence; whether a measurement record may be re-stamped is not mine to settle.
