# NOW — a ruler that misses its own case (2026-08-23)

After fixing `--invert`, I swept all 55 boolean flags in the `tri` CLI for the same shape: declared, and barely referenced.

- Seven surfaced at the "two or fewer references" threshold. Every one was a correct forward — the declaration plus the call it feeds. Two references is what a working flag *looks like*.
- `--invert`, with the bug in place, had **three**: signature, `if invert`, destructure. The sweep would have cleared it.
- The flag was used. It was used in a `println!`. Reference counting cannot express the difference between a mode and a decoration.

Reported as a null result with its own limit stated, rather than as "seven candidates, all clean" — which would have read as coverage. The check that works is behavioural: assert the new mode produces something the other modes do not, on a fixture where all of them have a site.
