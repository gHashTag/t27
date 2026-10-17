# NOW -- gen-ts lowers the bodies of pure functions (2026-10-08)

## gen-ts lowers the bodies of pure functions (Closes #7871)

- 2437 functions in 534 specs, all pass tsc --strict
- anything outside the subset is announced, a caller of an unlowered fn too
