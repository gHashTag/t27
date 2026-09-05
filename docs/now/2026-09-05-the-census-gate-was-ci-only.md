# NOW -- The census gate was CI-only and I moved one without blessing it (2026-09-05)

## The census gate was CI-only and I moved one without blessing it (Refs #3294)

- Twice in one day. `fetches` went red on master at 10:46Z and stayed red about an hour;
  then the session that repaired it moved `shell` two steps in #3293 and pushed without
  blessing -- having written the "re-bless in the SAME commit" rule that same morning.
- `tri census pin --gate` takes 133 ms. It was a CI-only reading for no reason.
- `pre_commit` did not call it. The one `census` match in `hooks.rs` was a comment,
  which is the third time today a mention was nearly read as a call.
- Wired in now, so the question is asked where the move happens rather than an hour later
  on master.
- Blessed in this commit, named: `fetches` moved four `red.rs` line numbers by +5 --
  a neighbour's #3291, positional only, no new site -- and `shell` moved
  `run: steps` 233 to 235 and `the runner does` 212 to 214, which is #3293, mine.
- The coupling is deliberate and worth stating: if a neighbour moves a census and does not
  bless it, this hook now blocks MY commits too. That is correct -- the ledger disagrees
  with the tree for everyone -- and the message names what moved so the blessing is
  informed rather than reflexive.
