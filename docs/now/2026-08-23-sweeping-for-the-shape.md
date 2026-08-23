# NOW — sweeping for the shape (2026-08-23)

Yesterday's finding was a shape, not an incident: a verification aimed one step to the left of what the change replaces. So every deploy path in reach got the same two questions — **what does it REPLACE, and what does it verify?**

Four paths, one hit, and it was the worst one available.

- `publish-website.yml` runs on a **15-minute cron, unattended**, and replaces the site's `assets/` with a build from another repository. It called the blog regenerator and the drift checker and nothing that asks whether the incoming build still carries the posts the site is serving. The exact gap that cost eleven posts two days of downtime, sitting in the one path that fires ninety-six times a day with no human present.
- **The drift checker structurally cannot ask it.** It compares the shipped bundle against the static tree and fails on disagreement — the right check, and blind to a slug leaving *both sides at once*. A checker of agreement between two artefacts says nothing about preservation across time.
- The clean paths were clean for a copyable reason: the CNAME guard reads the CNAME file, the Pages guard reads the Pages API *and* curls production. Each verifies the artefact it names.

**Then the sweep turned on the instrument.** The new gate counted slugs by PRESENCE on disk. `rsync` is additive, so a slug dropped from the reachable chunk is still on disk in the unreachable one — reported live while the site 404s it, until the orphan prune deletes it for good. That is the 2026-08-21 mechanism exactly, and the gate built to catch it shared its blind spot. Today the apex has 16 chunks, one reachable, holding all 48 slugs: the old reading was right **by luck**.

Reachability from `index.html` now, not presence. Eighth control case, verified RED under the old reading.

Verified in CI rather than only locally: the cron fired on the commit that added the gate and printed `no post is lost — live serves 48, this build carries 48`.
