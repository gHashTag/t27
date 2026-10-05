# NOW -- split-reel v10: a client's call is drawn in the client's colour (2026-10-04)

## split-reel v10: a client's call is drawn in the client's colour (Closes #6149)

- call_colour_from: the call's colour comes from where its words do -- the order's, else hers, else NEUTRAL_CALL_COLOR #FFFFFF on CALL_GROUND #000000; HOUSE_CALL_COLOR #FFD700 only on the house's own reel
- call_drawn: a colour that reads is drawn as it is; an unreadable colour the order gave is refused before the charge (GIVEN_CALL_COLOR_UNREADABLE_REFUSED), hers is drawn in its nearest shade that reads
- host: gHashTag/999-multibots-telegraf, branch fix/split-reel-v10-client-call-colour
