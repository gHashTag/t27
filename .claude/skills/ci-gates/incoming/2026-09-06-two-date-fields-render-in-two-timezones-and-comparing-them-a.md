## Two date fields render in two timezones, and comparing them as strings invents a skew

`git log --format='%ad %cd' --date=format:'%m-%d %H:%M'` printed, for one commit,
`a=09-06 02:15  c=09-05 19:15`. I read that as a seven-hour committer-date skew and was one step
from publishing "the repository's clocks are wrong" as a finding.

There is no skew. `%ad` and `%cd` each render in their **own recorded offset** -- author `+00:00`
from GitHub, committer `-07:00` locally. The same instant prints as two wall-clock times.

The control is one command, comparing as epochs, where the offset does not participate:

```
git log --first-parent -20 --format='%ad %cd' --date=format:'%s' \
  | awk '{d=($1-$2)/3600; if(d>0.5||d<-0.5) n++; t++} END{print n, t}'
```

**0 of 20.** Same question, opposite answer.

- For COMPARING dates use `%at`/`%ct` or `--date=iso-local`. `--date=format:` is for display.
- The tell was already on screen: the order by date ran **against topology**. When a date ordering
  disagrees with `git merge-base --is-ancestor`, believe the topology and suspect the rendering.
- This mattered because the whole question was "how long was the gate red" -- a window whose size
  *is* the entire answer. An artefact of rendering was about to become the measurement.
