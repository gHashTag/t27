# NOW -- spec(automation): tri-wallet-chains v3 -- every wallet write is all or nothing (2026-10-09)

## spec(automation): tri-wallet-chains v3 -- every wallet write is all or nothing (Closes #8419)

- The open-nonce count is read and the nonce written under one lock per person, so parallel requests from one process or several never pass five (20 had issued 7 on a real Postgres).
- bind_commits: the spent nonce, the address and the event commit together or roll back; unbind_commits likewise for the removed row and its event. From the Codex audit of 999 at 0dc5424e.
