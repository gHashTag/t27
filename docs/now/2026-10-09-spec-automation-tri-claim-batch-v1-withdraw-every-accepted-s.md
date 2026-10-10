# NOW -- spec(automation): tri-claim-batch v1 -- withdraw every accepted spec in a few wallet transactions (2026-10-09)

## spec(automation): tri-claim-batch v1 -- withdraw every accepted spec in a few wallet transactions (Closes #8348)

- The server builds at most 16 mint messages per request, each with its own signer quorum, asking signers for 4 earnings at a time; only earnings neither revoked nor sent, oldest first.
- The wallet sends them in one transaction: as many as it declares (TON Connect maxMessages), 4 when it says nothing, never above 255; 874 specs on a W5 wallet are 4 transactions. A cancelled prompt stops the run. The minter is unchanged: each message is op::mint_on_att with 0.1 TON.
