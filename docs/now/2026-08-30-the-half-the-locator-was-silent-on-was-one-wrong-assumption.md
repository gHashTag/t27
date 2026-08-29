# NOW -- The half the locator was silent on was one wrong assumption (2026-08-30)

## The half the locator was silent on was one wrong assumption (Refs #2864)

- tri unparsed locate confirmed 37 and refuted 37 -- exactly half. Four iterations of options lists carried that line, and I kept picking something else.
- Cause: split_module assumed a BRACED module. Files declaring 'module NAME;' -- the semicolon form -- have no wrapper, so it took the first 'struct X {' as the header and made the rest of the file a tail. Every truncated prefix then had a large orphan chunk glued to it and failed for the chunk's own reasons.
- Found with the base rate, not without it: tail longer than 10 lines appeared in 32 of 37 REFUTED and 4 of 37 CONFIRMED; a one-line tail in 4 refuted and 33 confirmed. A one-line tail is a real module's closing brace.
- confirmed 37 -> 60, refuted 37 -> 14, all 60 still reproduce alone. The 14 that remain point at lines 5-11: their FIRST item is unsupported -- files opening with 'algorithm NAME {' -- so there is nothing to bisect and the refusal is correct.
