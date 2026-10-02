# NOW -- Active is not working (2026-09-21)

## `tri swarm` reads lanes, throughput and the model ranking in one screen (Closes #4512)

- For eighteen minutes the swarm read `active 20` and finished six turns. Every bee was waiting out a 30-second backoff: NVIDIA answered 1,369 requests in twenty minutes with 429 or 503. `active` counts bees started and not finished; it cannot tell working from waiting.
- `tri swarm` prints `/queen/status` as one screen: lanes, dispatch counts, and the server's model ranking (BrowserOS #496) with each candidate's cost per 500-token step, success rate, speed and tool-call verdict. `tri swarm --since 600` samples twice and reports finished per hour, the number a lane or model change should be judged by.
- Exits 2 when the status endpoint does not answer: a swarm that is down is the finding, not an error in the tool.
- What this does NOT establish: that a finished turn did useful work, or whether a low success rate is 503 (model overloaded - switching models helps) or 429 (one key's rate limit - fewer lanes per key helps). The server log separates those; this page does not.
