# NOW -- A cache key that mapped two different inputs onto one key (2026-08-23)

## A cache key that mapped two different inputs onto one key (Closes #2559)

- sha_of hashed an unreadable file as the empty string, so a missing control and an empty one shared a key, as did any two unreadable paths.
- Contents were concatenated with no separator, so the boundary could move: [ab, c] and [a, bc] hashed identically, reachable for any gate declaring more than one control. Both close with a length-prefixed record per file.
- The interrupt-marker write was 'let _ = fs::write(...)'; on failure the run mutated anyway with nothing on disk to tell the next run. It now refuses, verified with target/ at mode 555.
