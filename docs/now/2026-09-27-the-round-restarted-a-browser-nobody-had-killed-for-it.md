# NOW -- The trigger fired: a killed browser, and a round that restarted it unasked (2026-09-27)

## What version 3 refused to claim, and how it was made claimable (Refs #4838)

- Version 3 split the restart into a MECHANISM (proven: the platform accepts the corrected query and the pod goes down and comes up) and a TRIGGER (unproven: nobody had watched a round find a dead browser and reach for the cure). The owner asked for the second half directly -- kill the browser and see whether it comes back on its own.
- The browser was killed the way that reproduces the original outage rather than a convenient one: `supervisorctl stop chromium` inside the pod, leaving `cdp-bridge` RUNNING. Supervisord does not respawn a program stopped by hand, so the pod went on answering on its CDP port with no browser behind it -- the exact lie this spec is named after. The probe, which opens a tab and runs code in it, failed with `socket hang up` while the port was still there to connect to.
- Nothing was told to restart anything. The round walked, found every connected site blank, and decided: `[browser:restart] 144022504 201 answers`, then `browser was blank; restarted; [...]`, 76 s end to end. It reported the restart only after a tab opened on the new pod had run `(() => 1 + 1)()` and answered 2 -- the same rule the spec asserts, now exercised by the code path that needs it most.
- PID 1 is again the only witness: aged `01:32:30` before and `01:22` after. A restart reuses the deployment, so the platform's own record was unchanged, exactly as in yesterday's entry.
- The restart cured the round, not just the pod. The second walk read live pages on seven sites -- hh.ru came back with 16 unread -- and the five it named without walking are the ones with no login. 197 cookies in the profile and the same six doors open: google, youtube, x, linkedin, tiktok, reddit.
- The CDP target count went 42 -> 37, which is not a leak: a restarted pod restores the person's own tabs from the profile volume, and the pages left are the owner's (reddit mod page, t27.ai/blog, an x.com chat, chrome://newtab).

## A fast `false` from a restart is a missing wire, not a failed restart

- The first harness got `browser was blank; restart failed` in 803 ms. That was the harness, not the product: `restartBrowserAsAgent` opens with `const deps = agentStartDeps?.()` and returns false if there is no driver, and that slot is filled only by `configureAgentStart(browserBrokerDeps)` at render-server.ts:137. Outside the server process the restart refuses before asking the platform anything, and the refusal is indistinguishable from a real failure except by how fast it comes.
- 803 ms is too fast to have reached the platform, and version 4 asserts that as the tell. Reproducing that single wiring line with the same product pieces is what let the real trigger run.

## What is still not claimed

- The CLOCK. This round was asked for; no round has been observed starting itself on the 180-minute interval in render-server.ts. `CLOCK_STARTED_ROUND_OBSERVED` stays false and says so by name, and `ROUND_RAN_OUTSIDE_THE_SERVER_PROCESS` records that the observed round was driven from a sibling process with the server's own wiring rather than by the server's own timer.
- Arithmetic checked the way the 2026-09-26 entry said it must be, by evaluating each `assert` against the constants `gen-js` folded: 95 hold, 0 fail. `t27c test` still only counts declarations.
