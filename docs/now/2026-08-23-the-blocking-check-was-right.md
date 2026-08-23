# NOW — the blocking check was right, and its ledger held 58 paid debts (2026-08-23)

`coverage` — the required check blocking a pull request — is `check_seal_coverage.py`, one of this campaign's own gates. **It is not a broken instrument.** Its negative control passes, and the finding is real: **131 seals are stale**, meaning the spec changed after sealing so the recorded hashes describe something it no longer produces.

- **Not mine to decide:** re-sealing 131 specs is blessing the drift the ratchet exists to prevent; fixing them is 131 separate judgements. Filed as an owner decision.
- **But the gate asked for something free**, every run: 56 baselined seals that now hold, and 2 lines naming seal files that no longer exist. **209 → 151 lines**, the 131 untouched. That *tightens* the ratchet — fifty-six seals that were excused are now held.
- **Removed by hand, line by line — not with `--update-baseline`.** That command rewrites the whole ledger from today's state and would bless all 131 in the same stroke. **Tightening a ratchet and blessing drift use the same file and must not use the same command.**

**The state worth naming:** `coverage` is required, so this blocks merges — and every recent merge went in with it red anyway. A required check that is always red costs the friction of a gate without the protection of one: the same condition this campaign opened on, from the opposite direction. Not a gate that cannot fail, but a gate that cannot pass.
