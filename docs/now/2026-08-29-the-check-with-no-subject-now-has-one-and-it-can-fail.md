# NOW -- The check with no subject now has one, and it can fail (2026-08-29)

## The check with no subject now has one, and it can fail (Refs #2762)

- check_emitted compared 109 SSOT records against an emitted JSON that gen/ .gitignore keeps out of the tree; it reported absent into r.emitted, which is not a finding, and suite prints findings only -- zero occurrences of the word in the master gate's output
- the generator is pure-stdlib and deterministic, so the gate now runs it into a temp dir; a generator it cannot run is a FINDING, not a shrug
- not a tautology: the Rust gate and the Python generator parse the same file with different code, so the comparison is two accounts of one source
- 436 numeric fields across 109 records now compared, and a planted control proves it fails: [emitted-agrees] gf10 SSOT bits=10 but emitted bits=999
