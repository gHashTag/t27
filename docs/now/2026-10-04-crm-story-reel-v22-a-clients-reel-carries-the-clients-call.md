# NOW -- crm-story-reel v22: a client's reel carries the client's call (2026-10-04)

## crm-story-reel v22: a client's reel carries the client's call (Closes #5802)

- HOUSE_CALL_ON_CLIENT_REEL and HOUSE_BUTTON_ON_CLIENT_REEL false: the template's call and button stay on the house's own reel
- call_from and profile_call_from: the seller's word for this reel, else hers -- story_reel.cta, approved_cta, the default
- button_from: the seller's, her kept card, her profile, or none; never the template's on her reel
- word_taken and house_word_replaced: a given call or button naming the house (HOUSE_MARKS) is replaced by hers and said in the answer
- last_line_way: the template's last line ends with her call when it fits
- line_way and line_names_house: no line of a client's reel names the house -- template lines, lines the agent, the writer or a seller wrote or edited, captions and the end card -- unless it is the person's own words typed this turn; every template line on her reel is its house-free CLIENT VERSION, the first as much as the last (TEMPLATE_LINES_KEPT is gone: it left three lines unchecked and a client's reel said "This is Trinity" in her voice)
- CLIENT_LIST_HOUSE_FREE: a client's template list shows only the client versions
- charged and REFUSED_BEFORE_SPEND: any other line naming the house is refused before anything is spent, as split-reel v7 refuses it; control "template line naming the house kept on a client reel"
- own_words and line_from: "the person's own words" is judged LINE BY LINE -- a line naming the house is hers only when the whole line, or each house name in it with up to OWN_WORDS_REACH of the line's words either side, is in what she typed this turn (case, punctuation and spaces aside); NAMING_THE_BOT_LENDS false: sellers address the bot by name, so "Trinity, make a reel" lends "This is Trinity" nothing, while "at the end let it say: Write to Trinity" makes "Write to Trinity" hers; control "message-level own-words"
- charged is the same body as split-reel v7's, copied on purpose and blessed in tools/duplicate_bodies_baseline.txt (`charged 2`): a `use a::b;` still generates no import (#5718), so no spec can reuse another's function
- voice_of: a client turn speaks her voice; a client turn naming another client is refused before any spend
