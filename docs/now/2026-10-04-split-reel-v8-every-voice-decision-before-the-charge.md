# NOW -- split-reel v8: every voice decision before the charge (2026-10-04)

## split-reel v8: every voice decision before the charge (Closes #5885)

- An ordinary caller with no voice_id and no clone is refused before the charge (voice_refused, STOCK_VOICE_FOR_ORDINARY = false); the house keeps its own path.
- A new voice is estimated from its text before it is paid: MS_PER_CHAR = 56, measured 464 chars over 25920 ms on the host's lipsync captions; 214 chars fit, 215 do not.
- The one refusal after the charge -- a voice measured past the ceiling once it exists -- gives that voice back, once (REFUND_TOO_LONG_VOICE, voice_too_long, backstop_refund): the host repo holds one voice with its text, so no slowest-per-language pace can be measured.
- 20/20 tests pass, 0 vacuous; six negative controls each go red; sealed.
