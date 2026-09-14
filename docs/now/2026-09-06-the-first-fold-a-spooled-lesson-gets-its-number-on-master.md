# NOW -- the first fold: a spooled lesson gets its number on master (2026-09-06)

## the first fold: a spooled lesson gets its number on master (Refs #3236)

- tri skill fold run on master for the first time. One spooled lesson became section 596, numbered against the file as it stands, and the spool file was deleted in the same commit.
- The diff is the point: 46 insertions and ZERO deletions in SKILL.md, plus one deleted spool file. That is the structural signature -- one section added per spool file removed -- which is what makes a direct append distinguishable from a fold in a branch diff.
- This is the step the whole spool exists for, and it had never been run from master. Numbering happens here, once, on one branch, with the file in front of it -- so two branches cannot choose the same number.
