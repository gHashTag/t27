# NOW — a verification that checked the adjacent thing (2026-08-23)

A gate written yesterday went red on real data today. What it found: eleven blog posts absent from a live site for two days.

- A deploy on 2026-08-21 replaced the site's application bundle with one built from a different repository. Eleven posts existed **only** in the old bundle — never migrated into the shared source — so the swap removed them. No static pages either, so nothing served them at all.
- That deploy **was** verified before pushing: *"Verified before pushing: deck/, .claude/, blog/ (29) and ru/ (11) untouched."* Every word true, and the wrong question. It checked the static trees; the loss was entirely inside the bundle.
- A verification aimed one step to the left of the thing being changed is harder to notice than no verification, because the commit reads as checked.

**The rule:** name what your change REPLACES, and verify that. The deploy replaced `assets/`; the check covered `blog/` and `ru/`.

**And a gate is not tested until it is red on data you did not plant.** Four planted cases all passed before it had seen a real deploy. Replaying it over 160 commits of history found three drops — two healed by the next deploy, one not. The replay is now a mode of the tool, not a script run once.

**Which needed its own two directions.** On its first real run the replay printed "still missing today: 0" for all three — correct, because the loss had just been repaired. A mode whose only observed output is green has not been shown to go red. Three planted cases now: an unhealed drop reds, a healed drop stays green and says why, a clean history does not invent one.

All eleven restored and live, both languages, verified by content on production rather than by exit code.
