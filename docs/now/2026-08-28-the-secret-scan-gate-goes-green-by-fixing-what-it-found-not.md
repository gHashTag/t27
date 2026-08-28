# NOW -- The secret-scan gate goes green by fixing what it found, not by ignoring it (2026-08-28)

## The secret-scan gate goes green by fixing what it found, not by ignoring it (Refs #2754)

- 233 tracked files carried one developer home directory; 60 were load-bearing and were fixed in the previous change
- 208 committed Coq build artifacts are untracked here -- nothing reads them, coq-proofs.yml builds from source with coqc
- 98 markdown files now quote repository-relative paths
- four remain and are named individually in the gate, not swept under a count: they configure local tooling and must give an absolute path at run time
