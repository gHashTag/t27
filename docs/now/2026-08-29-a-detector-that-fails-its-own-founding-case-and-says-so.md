# NOW -- A detector that fails its own founding case, and says so (2026-08-29)

## A detector that fails its own founding case, and says so (Refs #2762)

- tri orphaned list: inputs named outright in production code that are not in the tree -- 21 findings, 110 fixtures filtered out of 241 literals
- historical control FAILED: at the commit before W702 it finds ZERO of one, because that path was assembled from a variable and a bare filename and never appears as a literal
- the window stays where it is; the miss is in the module docs, not tuned away
- it earned its place on a different case: the Railway server's static fallback and 404 page point at public/, which has never existed in any commit
