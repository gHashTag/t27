# NOW -- Re-bless the quiet census my own fix moved three passes ago (2026-09-05)

## Re-bless the quiet census my own fix moved three passes ago (Refs #3265)

- cli-tri.yml has been red on master, and after the numbering repair the failing step turned out to be the census ratchet, not the numbering one: steps in a quiet shape moved 32 to 31 and no commit said so.
- The mover was my pass-107 fix. phi-loop-ci.yml's L8 lint ended '... 2>/dev/null ... || echo PASSED', which reports L8 PASSED when ffi/src is missing; making it exit 2 removed it from the failure-branch-passes class, 16 to 15. coq-kernel.yml's Admitted gate gained its own exit-2 path in the same PR.
- Blessed: quiet 32 to 31, failure branch passes 16 to 15, a tracked path present 11 to 10, named a path but not quiet 123 to 128.
- The gate is right and I ignored it for three passes because it is not a required context. Its own message cites the measurement that 8 of 39 commits moved a census and only 4 mentioned it - that measurement was mine.
