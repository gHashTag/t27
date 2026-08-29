# NOW -- Four workflows are red on master, and the window that hid it (2026-08-29)

## Four workflows are red on master, and the window that hid it (Refs #2783)

- gh run list --limit 60 on master returned all-green; that window spanned 2 hours 28 minutes, and the five-day picture is 940/60 across 40 workflows
- nine workflows had a failure as their most recent master run, none re-run after the fix PR landed; I retook the reading for the five with no outward effect and four are red today
- two of the four are green by not running (paths: filter), two never run on master at all
- the four with outward effects -- release, deploy, git push, docker push -- are the owner's to retake
