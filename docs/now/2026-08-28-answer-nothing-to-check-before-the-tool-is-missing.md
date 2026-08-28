# NOW -- Answer nothing-to-check before the-tool-is-missing (2026-08-28)

## Answer nothing-to-check before the-tool-is-missing (Refs #2161)

- #2746 ordered the compiler requirement first, so an empty tree reported the wrong diagnosis
- caught by check_gate_preconditions.py one commit later -- the loud-failure gate doing its job
