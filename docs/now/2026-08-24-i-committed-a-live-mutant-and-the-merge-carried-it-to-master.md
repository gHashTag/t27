# NOW -- I committed a live mutant and the merge carried it to master (2026-08-24)

## I committed a live mutant and the merge carried it to master (Refs #2161)

- Refs #2161. I launched a full boundary sweep in the background, kept working, and later ran `git add -A && git commit --amend`. At that instant the sweep was holding tools/gft_backprop_microcode.py mutated. `if d >= 26` went in as `if d > 26` -- into the commit, the pull request, 30 green checks, and onto master
- Nothing broke: that is the mutant I had PROVED equivalent two ticks earlier, 0 differences over 525918 points, and the line carries a `# mutant-equivalent:` comment saying exactly that. So the code stopped matching the comment directly above it and every test stayed green -- the precise condition under which a wrong line survives indefinitely
- The dirty-tree guard answers "is the tree clean before I start". Nobody was asking "is a sweep running while I stage", and `git add -A` cannot tell a mutant from an edit. A BACKGROUND PROCESS THAT MUTATES THE WORKING TREE TURNS EVERY `git add -A` INTO A LOTTERY
- Three defences, cheapest first: stage paths rather than -A while a sweep runs; check `target/.tri-mutating` before staging (it exists for exactly this and was sitting right there); and read the summary git prints -- the commit said "20 insertions, 1 deletion" and I had written only insertions. A deletion I did not intend was in the line git handed me and I read past it
- Third self-inflicted git loss this session and the only one that reached master. Same shape each time: a command whose blast radius is the whole tree, run while my attention was on one file. The detector was not a test -- it was mutate refusing to start on a dirty tree the next time I ran it, one tick late
