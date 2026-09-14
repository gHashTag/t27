# NOW -- Two sessions took the same three tasks, and the obvious lock does not lock (2026-09-06)

## Two sessions took the same three tasks, and the obvious lock does not lock (Refs #3331)

- A republish notice on the dashboard led to it: heading sets identical 738 = 738, byte counts equal, hashes different, the difference inside the platform's injected runtime - which meant my source file WAS a downloaded copy of the published page, and its stamp read 'pass 117 all three', entries I never wrote, above my own.
- A second session had already executed all three options I recommended; #3314 and #3317 were open as I read. I stopped rather than open a third parallel workstream.
- tri loop claim uses the one atomic operation git gives over a shared remote: creating a ref that does not exist. 0 taken, 1 refused and names the holder, 2 the attempt could not be made.
- The obvious version does not lock: pushing origin/master to the claim tag succeeds for the SECOND claimant too, because git treats re-pushing the same value as a no-op. Measured: first exit 0, second exit 0. Only a different value is rejected, so the claim is a commit no other claimant can produce.
- Two mutants killed. The second test needed fixing first: it sliced to end of file and asserted contains(exit(2)), which four unrelated sites satisfy, and the mutant survived it.
