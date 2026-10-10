# NOW -- specs/hosting: the network MVP hardening lane named in two TODOs (2026-10-11)

## specs/hosting/{sybil,statement}.t27 (Closes #8774; gHashTag/trios#1761)

- The hardening lane's trusted N/F/M configuration (one vote per operator) and base VERIFIER_FEE are not on master yet; sybil.t27 and statement.t27 name it in a header TODO so the cards import it when it lands instead of forking it. The leaf encoding and kind words stay stable for other sources' rows.
- Comments only: generated C byte-identical; `t27c test-report` 13 and 10 tests, 0 FAIL, 0 vacuous.
