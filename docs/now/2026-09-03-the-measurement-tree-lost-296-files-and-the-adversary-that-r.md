# NOW -- The measurement tree lost 296 files, and the adversary that routed around it was right (2026-09-03)

## The measurement tree lost 296 files, and the adversary that routed around it was right (Refs #2983)

- git status in the loop worktree reported 296 tracked files deleted; 293 of them exist on origin/master, so the worktree was silently truncated rather than the repository. 19 of 49 files were missing from .github/workflows, which two commands shipped this week read off disk to build their population.
- Found by accident: git stopped working because all three worktrees under /private/tmp had lost their .git pointer file, while the checkout outside /private/tmp still had one. git worktree repair restored all three; git checkout -f origin/master restored the files. The cause is NOT established -- a tmp reaper fits both symptoms and was not tested.
- What saved the published numbers was luck, not design. tri issues stale had printed workflow files 49, matching origin/master, which is what proves that reading was taken on an intact tree. Print the size of what you walked, every time.
- And the delayed adversarial phase corrected me: I hand-verified three STALE verdicts because the skeptic had not run, and one was wrong. The issue's number is an explicitly frozen hash-pinned reading whose own tool refuses to run when the corpus moves, the owner had already commented the new figures, and the load-bearing sentence was about a substitution RULE's reach rather than the tree -- so its prediction was vindicated and I reported a confirmed forecast as a wrong number.
- Final sample: 24 of 24, STALE 8 claimed, 6 judged, 5 confirmed, 1 refuted; HOLDS 3, UNMEASURABLE 13. My published 21/7/3/11 was a partial reading and is corrected here.
