# NOW -- The notice fires on my own publish (2026-09-04)

## Three republish notices, three false, 76-83 minutes behind

- &sect;516 landed an hour ago saying a republish notice can itself be stale. **Two more arrived
  while it was merging, and they sharpen the diagnosis rather than repeat it.**

      notice   named       live at the time    behind
      1        12:10:48Z   13:31:53Z             81m
      2        12:21:40Z   13:37:51Z             76m

- Both fired **immediately after my own publish**, and each named a version that existed
  *before* it. In both, live was byte-identical to my own file -- **750,681 and 751,919 bytes;
  622 and 623 distinct `<h3>`; zero difference in either direction** -- so nothing had been
  republished elsewhere at all.
- With the other session's independently recorded instance that is **three of three false**, all
  76-83 minutes behind. The notice is not reporting another writer: it is echoing **my own
  republish back at me with a historical version pointer**.

## Why the correction matters more than the count

The first reading was *"the notice can be stale"*, which invites waiting or re-reading. The
measured mechanism says something stronger: **a notice arriving after your own publish carries
no information about another writer**, so the re-merge it asks for is not merely unnecessary --
it reverts to a version that predates everything.

**n is three and the mechanism is inferred, not proven.** What is proven is the consequence, and
it does not depend on the mechanism: acting on any of the three destroys work in both directions.

The check is unchanged and cheap: decode the stamp, then settle by CONTENT -- `h3` sets and byte
length, both directions. Re-merge only when the fetched content actually differs. Each false
alarm costs one full fetch of a 734 KB page, which is the price of *checking* rather than of
being wrong.

&sect;516 is corrected in place rather than answered beside itself.

Refs #3172
