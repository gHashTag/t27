# NOW -- spec(trinet): tri-bridge v2 -- quorum against a fault bound, finality as a fact, whole-lock mint (2026-10-09)

## spec(trinet): tri-bridge v2 -- quorum against a fault bound, finality as a fact, whole-lock mint (Closes #8423)

- The quorum is judged against a fault bound F: safety 2M > N + F, liveness M <= N - F. v1's 'M > N/2 leaves an honest signer in every overlap' was false (N 3, M 2, A faulty: {A,B} and {A,C} meet only in A); that counterexample is now a test.
- A signer keeps a durable record per event and never signs a second envelope for one lock, across restart and rotation; an unreadable record refuses. An attestor signs only on a proven finality fact; unknown finality refuses and a count alone is never finality.
- Supply counts pending mint and pending release; a lock mints its whole amount once; the envelope names source and destination networks (CAIP-2) and contracts, event, recipient, amount and epoch, and the replay key is the source event. From the Codex audit of t27 at 54f3c140. Status stays DESIGN: nothing here enables a bridge.
