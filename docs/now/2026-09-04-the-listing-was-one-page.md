# NOW -- The listing was one page, and it hid 45 of 50 (2026-09-04)

## The population fetch, not the value

- §517 fixed the streak inside `tri red`. **The listing that decides which workflows get examined
  at all was also one page.** Measured on `gHashTag/trinity-fpga`:

      active workflows 405   one page 100
      RED overall 50    RED tri red could see 5    RED INVISIBLE 45

- **Ninety percent of the red workflows in that repository were never reported**, by the command
  whose entire subject is *what is failing right now*. Not a wrong number: an unexamined
  population, which prints identically to a healthy one.
- **The identical fetch in `cibase.rs` has paginated all along** -- one fix that did not travel to
  its sibling, two `repos/{repo}/actions/workflows?per_page=100` lines in one crate, one carrying
  `--paginate` and one not.

## The census could not say so, and its abstention is where the defect lived

`tri gates fetches` takes the ENCLOSING FUNCTION as the subject of its guard question. `fn now`
held more than one fetch, so this site sat in `a guard, but two fetches` -- an honest *cannot
tell*. It resolved to `prints what it got` only when §517 changed the shape of that function, and
**that reclassification is what exposed the 405.**

## Self-critical: my own commit moved the census and said nothing

The §517 commit moved `prints what it got` from **1 to 3** and `fetch sites` from **23 to 25**,
and the pull request did not mention it. The A/B that finds this is two commands -- run the census
at `HEAD` and at `HEAD^` and diff -- and running it is how the 405 was found. **When a change
moves a census, re-read the census.**

Two of the three new rows were benign (`deep_runs_url` is the acknowledged url-builder shape,
`last_pass` is a deliberate `per_page=1`). The third was not, and I would have shipped past it.

## And the census had the same shape

Its walk is `cli/tri/src` and nothing else, while four loop helpers under `scripts/tri_loop/` bound
the same API with `--limit` and three carry no guard at all. It now names that surface and sizes it
-- **7** bounded reads, counted loosely and published as an exclusion notice rather than a
classification, because a count that quietly excludes part of its subject is this command's own
subject. Its closing prose also carried two stale literals beside numbers it computes (*"one of the
nine"*, *"a crate that has 24"*); both are computed now, and read **2 of 2** and **62 / 25**.

After the fix the crate has **zero** unguarded fetch sites: the two left in that bucket are both
the url-builder shape the census documents as a known false positive.

Refs #3176
