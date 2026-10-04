# NOW -- crm-story-reel v23: house marks read as a reader reads them (2026-10-04)

## crm-story-reel v23: house marks read as a reader reads them (Closes #5897)

- fold: the text and every house mark are folded the same way first -- NFKC (S3AI written with a superscript is s3ai), invisible characters dropped (INVISIBLES_DROPPED), one case, and each Cyrillic look-alike read as its Latin letter (FOLD_LOOKALIKES, LOOKALIKE_LATIN), so a Latin T in the Russian name and a Cyrillic Te in the English one both read as the house
- code_mark: the brand T27 is the house when it starts a word (CODE_MARK_PREFIX: t27, t27ai, t27_ai, T27.ai), not after a letter or digit (at27, 5t27) nor before a digit (T270); write to t27ai is now flagged
- product_mark: the product is ONE code token (PRODUCT_ONE_TOKEN): S3AI and S3AIs are the house, our S3 AI pipeline is S3 the storage and the word AI and names no house
- name_mark: a house name is the house inside a longer word (NAME_INSIDE_WORD: TrinityBot) and before any word (NAME_IN_ANY_CONTEXT): Trinity College stays flagged on purpose -- no rule on the next word clears the college without clearing Trinity AI or Write To Trinity AI, and a refused agent line is rewritten for free while a missed one is on a client's reel; her own typed line is still hers (own_words)
- test: no rule that clears a false positive clears a mark -- Trinity, Trinity S3AI, T27 and @t27ai_bot stay flagged; 60/60 spec tests, controls in the 999 binding
