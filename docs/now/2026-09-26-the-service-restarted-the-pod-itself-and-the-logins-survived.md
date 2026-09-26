# NOW -- The service restarted the pod with its own variables, and the logins survived (2026-09-26)

## What was asked and what was true (Refs #4838)

- Version 2 of browser-pod-restart.t27 left `UNATTENDED_RESTART_OBSERVED` false, and the owner asked the obvious question of that line: has the pod restarted itself?
- It had not, and the deployment list cannot answer it either way -- a restart reuses the deployment, so the pod's record still read `4c99ba3f ... SUCCESS 2026-09-23T11:21:26Z` exactly as before. The only witness is the container's own PID 1, and it read `02:19:34` against `date -u` of 15:27:58 UTC: started 13:08:24, which is the manual restart of that afternoon.
- Nothing had asked it to restart, either. The path fires when a round finds a dead browser, and the browser answered a probe in 173 ms. So a healthy browser is the reason no restart happened -- not a broken restart.

## So the path was run on purpose, from inside the service

- `restartBrowserAsAgent` is reachable only through the agent round (`rounds-wiring.ts:112`); no HTTP route exposes it. The restart it delegates to was therefore run inside the vibee-render container, with the service's OWN variables and the corrected query, which is what was actually unproven.
- Three live unknowns fell at once: the token reaches the process (43 characters, present), the API accepts the corrected `status: { in: ['SUCCESS'] }` lookup and names the pod's deployment, and `deploymentRestart` answered `true`.
- The pod then went down and came up: PID 1 aged `02:19:34` before and `01:33` after, a minute and a half against two hours and nineteen. No HTTP sensor could have told the difference -- which is the whole point of the spec this belongs to.
- The browser came back alive, not merely listening: a tab opened after the restart ran `(() => 1 + 1)()` and answered 2.
- The restart kept every login. 194 cookies in the profile, and the same six doors open: google, youtube, x, linkedin, tiktok, reddit. Nothing had to be signed in again, which is the claim that makes a restart a cheap cure rather than an expensive one.

## What is still not claimed

- Version 3 separates two things version 2 ran together. The MECHANISM is now observed end to end. The TRIGGER is not: nobody has watched a round find a dead browser and restart it with no hand on it. `UNATTENDED_RESTART_OBSERVED` stays false, and `WHAT_IS_UNOBSERVED_IS_THE_TRIGGER_NOT_THE_MECHANISM` says which half is which, so the next reader is not left guessing what the false line covers.
- The spec's arithmetic was checked the way yesterday's entry said it must be -- by evaluating each `assert` against the constants `gen-js` folded, since `t27c test` only counts declarations. 74 hold, 0 fail. `t27c test` prints `Total: 11 declarations` and would print it just the same if a number were wrong.
