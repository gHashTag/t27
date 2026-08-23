# NOW — the honest denominator overclaimed (2026-08-24)

`t27c classify` exists to stop a Markdown document being counted as a failing spec. It closes with a sentence that is measurably false, in the one command whose whole job is to be careful about counting.

- **The claim.** *"Anything outside SOURCE cannot parse, and counting it as a failing spec inflates every corpus ratio in a knowable direction."*

- **Measured against `parse-complete`: 5 of the 28 non-SOURCE files parse.** Three ALT-SYNTAX — `specs/ar/coa_planning.t27`, `proof_trace.t27`, `restraint.t27` — and two UNCLASSIFIED — `specs/physics/formula_registry.t27`, `lqg_entropy.t27`.

  The direction of the correction was right; its stated reason was not. `classify` reads the opening of a file. Parsing is a different question about the rest of it, and a command cannot certify the answer to a question it does not ask.

- **Corrected in place.** The sentence now says what the denominator is for, and says explicitly that it does **not** follow that everything outside SOURCE fails to parse. `parse-complete` is named as the authority on parsing; `classify` on what is code.

- **What the two commands say together, which neither says alone.** Of the 154 specs that do not parse: **131 are SOURCE**, 14 NOT-CODE, 5 ALT-SYNTAX, 3 UNCLASSIFIED, 1 MIXED. So the non-code correction is worth 23, not 28, and the real backlog is 131 genuine source specs the parser rejects.

- **A false finding I nearly filed.** My first pass reported `classify`'s summary saying UNCLASSIFIED 5 while its own detail view listed 7 — a command contradicting itself. It does not. My line filter was picking up the closing prose, which contains the string `.t27`. Reading the actual section showed exactly 5. That is six of my own broken rulers this session, and the only reason this one cost nothing is that I looked at the raw output before writing the issue.

- **The order that worked.** Every number here comes from two commands that already existed, cross-referenced. Nothing new was built. Yesterday's lesson was to grep `--help` for the noun before writing a tool; today the same habit answered the question without one.

Refs #2479
