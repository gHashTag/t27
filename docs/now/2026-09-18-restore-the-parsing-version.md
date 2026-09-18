# NOW -- Restore the parsing version of 14 specs (2026-09-18)

## Restore the parsing version of 14 specs (Closes #4272)

- Fourteen specs on master did not parse (NOPARSE), each had a working version in the branch that implemented it whose PR could not merge because the file was overwritten by a non-parsing version on master
- The broken version of utf8.t27 carried `parse error in fn 'encode' near line 110: unexpected token after expression statement: KwReturn` from e71afdfc3
- Required checks (`validate`, `check-linked-issue`) do not run the compiler over changed specs, so nothing gated the regression
- Restored the parsing versions from the implementation branches: 14 files, all now print IMPLEMENTED with zero "not yet implemented" stubs
- NOPARSE 79 -> 65, IMPLEMENTED 499 -> 513 (measured with t27c built from master)
- t27c spec-status over all 948 specs before and after; per file `t27c gen <f> > /tmp/g.zig && grep -c 'not yet implemented' /tmp/g.zig` = 0
- Four files (trie, quick_sort, csv, hex) left alone because their broken versions declare helper functions the working ones do not; restoring them over a parsing file is follow-up work