# NOW -- One population for the census, walked the other way as a control (2026-08-30)

## One population for the census, walked the other way as a control (Refs #2864)

- `unparsed report` said 5 typecheck failures and `unparsed locate`, in the same binary, said 2. Measured independently: 5 is right.
- Cause 1: `parse_failures` was lifted out with a comment saying disagreement was now structurally impossible, but only `prose` was moved onto it -- `report` and `locate` kept their own corpus walks. A helper is not a cure; the callers reading it are.
- Cause 2: `locate` checked whether the error named a LINE before checking which STAGE refused the file. Three typecheck failures print no line, so they were filed as 'nothing claimed' and its buckets summed to 80 against a population of 76.
- Both commands now read the shared scope: 57 confirmed + 14 refuted + 5 silent = 76 exactly, and 5 typecheck + 4 lex + 1 semantic = the 10 `prose` prints.
- New `tri unparsed agree`: three commands reading one variable agree by construction, so it builds the population the other way -- walking the tree where the census asks git -- and demands the same numbers. Mutation-checked: a silent specs/fpga/ filter turns it red with the row named.
- New test `one_corpus_walk` counts the callers: exactly one corpus walk may exist in cli/tri/src. It fails if a sibling grows its own.
