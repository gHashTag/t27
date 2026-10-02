# NOW -- The corpus ledger says what is true (2026-10-02)

## Shrink the corpus expectations to the failures that exist (Refs #5497)

- The corpus ratchet had not been reached on CI since the earlier Corpus Ratchet steps went red; on master 769f3252 it reports 68 unexpected failures, 57 unexpected passes and 7 worsened discard counts against a ledger of 151 at a cap of 152.
- Remove the 51 entries whose specs now pass every phase; re-key the 6 specs that parse now and fail one phase later (composition, multi_head_attn, pins/parser, packed_trit, jit_semantics, packed_vsa), keeping their old reason; lower the 5 discard pins that improved (uart, coder/benchmark, cordic, jit, lotus); add the 33 failures of ported specs under #5549. 151 -> 133 entries, and the cap follows the count down to 133.
- Not ledgered, on purpose: 29 core specs that began discarding tokens after edits since 2026-09-12 and 7 whose discard counts rose. They are repairs, not records. The ratchet now reports exactly those (29 unexpected failures, 7 worsened, 0 unexpected passes).
