# NOW -- Zero of one is a result (2026-08-29)

## Zero of one is a result (Refs #2804)

- the new detector does not find the case it was written for: that path was assembled from a variable and a bare filename and never appears as a literal
- write the founding case down as a test input BEFORE writing the detector, or you build something else and do not notice
- it reported its own test assertions until the fixture region was counted by braces instead of indent -- 41 fixtures had leaked back in
- not-in-the-tree has three innocent meanings; git log --all -- <path> and .gitignore separate them in two commands
