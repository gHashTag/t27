# NOW -- The review queue has a budget, and the watchman now names it (Closes #4469)

## Sixteen to twenty unreviewed all afternoon, under a threshold set at forty

- `dispatches.unreviewed` sat between 16 and 20 for hours while every bee that finished waited. The `review-backlog` rule could not see it: its threshold was **twice** the lane count, which at twenty lanes means forty. The number that matters is whether the backlog is bigger than what the swarm produces in one round, because past that point it can only grow. The threshold is now one lane's worth, and a backlog that is **draining** still does not fire.
- **The cause is a budget, and the rule says so.** The review sweep buys `TRIOS_QUEEN_REVIEWS_PER_ROUND` reviews a round -- **default 3** -- and `TRIOS_QUEEN_MEASUREMENTS_PER_ROUND` criteria measurements, also 3, both capped at 32. Neither was set on the deployment. Faster bees do not make a faster swarm if the sweep still buys three: the swarm went from 8.5 to 80 dispatches an hour this afternoon and the review budget did not move.
- Raised on the deployment to 8 reviews and 6 measurements a round, which is a step rather than the ceiling: the sweep runs BEFORE the dispatch half of the round and holds it, so a budget large enough to fill four minutes is a swarm that stops handing out work.
- Twenty-eight rule shapes now, eleven of them ones a moving system must NOT fire -- including three unreviewed against ten lanes, which is not a backlog.
