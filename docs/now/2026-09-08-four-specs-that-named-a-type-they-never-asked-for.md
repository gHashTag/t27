# NOW -- Four specs that named a type they never asked for (2026-09-08)

## Four specs that named a type they never asked for (Refs #3408)

- `Trit` is declared by four specs as `pub const Trit = enum(i8)`. Four OTHER specs used it and carried **zero `use` lines** -- naming a type they never imported. One line each fixes it, and the payoff was measured per file before anything was written:

  | spec | rustc errors before | after | delta |
  |---|---:|---:|---:|
  | ar/coa_planning | 69 | 65 | **-4** |
  | ar/explainability | 34 | 29 | **-5** |
  | ar/proof_trace | 25 | 20 | **-5** |
  | ar/restraint | 30 | 19 | **-11** |

- Corpus: coded diagnostics **2962 -> 2937**, exactly the -25 the per-file measurement predicted; unknown-type warnings **478 -> 460**; rustc acceptance unchanged at 433 of 651, which is expected -- each of these files still carries 19 to 65 other errors. All four backends still generate for all four specs.
- **The checker shipped one pass earlier caught this change immediately.** `tools/check_seal_currency.py` reported 10 stale seals by name the moment the specs were edited, including every duplicate; all 10 refreshed, back to 0. A guard built in one pass catching the very next change is the only evidence that it was worth building.
- Ten seals for four specs, again: `Explainability.json` and `ar_Explainability.json`, `ProofTrace.json` and `ar_ProofTrace.json`, `coa_planning.json` and `ar_coa_planning.json`. The duplicate-naming decision filed in #3415 stays open, and it is why the refresh had to walk every seal rather than call `t27c seal --save`.
- zsh does not word-split an unquoted `$VAR`: `for f in $SPECS` ran the loop once with the whole list as one filename. Caught by `cp` refusing, and the tree was clean afterwards. Fourth time this session; the fix is a literal list or `${=VAR}`.
