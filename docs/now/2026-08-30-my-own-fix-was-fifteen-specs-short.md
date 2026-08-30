# NOW -- My own fix was fifteen specs short (2026-08-30)

## My own fix was fifteen specs short (Closes #2939)

- #2931 recovered the scaffold binding's type and left the initialiser as `0`; for a struct that is not an initialiser at all
- `EmbeddingWeights input = 0;` -> `error: initializing 'EmbeddingWeights' with an expression of incompatible type 'int'`
- **15 specs carried it as their ONLY remaining family** -- the incomplete fix cost exactly the specs it was written to unblock
- `(T){0}` is valid for every complete object type, so one spelling serves scalars and structs and no type test is needed at the emission site
- `cc accepts` **242 -> 257**, zero regressions
- a fix that unblocks a construct PROMOTES the next defect in the same construct to visible; re-run the census after every landing rather than carrying it forward
