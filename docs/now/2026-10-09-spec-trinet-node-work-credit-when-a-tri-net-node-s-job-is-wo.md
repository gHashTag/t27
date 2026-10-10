# NOW -- spec(trinet): node-work-credit -- when a TRI-NET node's job is worth a TRI mint (2026-10-09)

## spec(trinet): node-work-credit -- when a TRI-NET node's job is worth a TRI mint (Closes #8426)

- Record levels: none, self-reported (the node's owner ran the coordinator), verified (an independent coordinator issued and recomputed the job), attested (a quorum signed it). Only attested mints; a self-reported record never does, and a signer's signature never counts for its own earning.
- Once across restarts: one credit per (node, nonce); the coordinator writes a nonce mark before it issues past it and resumes at that mark, so a restart never reissues a nonce (mesh.zig starts next_nonce at 1 and ledger.zig keeps the paid set in memory today); a credit counts only once its journal line is durable; an unreadable journal stops crediting.
- The rate is settlement_law.t27's 1 mTRI per job, not chosen here; a closed run is attested once as one sum to a bound testnet wallet; the tag does not prove the board; production payout waits for an agreed trust model, an independent coordinator and live signers. DESIGN, test contour only.
