# NOW -- Two specs for one outage: the browser sign-in playbook, and a restart that never happened (2026-09-26)

## Two specs for one outage: the browser sign-in playbook, and a restart that never happened (Refs #4838)

- 2026-09-24..26 no social-network sign-in was possible through the agent. One symptom, two defects, so two specs under specs/automation/.
- browser-pod-restart.t27: HTTP liveness is not liveness. /json/version answered while Runtime.evaluate timed out at 21 s; the round detected the blankness and the cure it called was a documented no-op that handed the same broken Chromium back and reported success. Healthy now means a page it just opened RUNS something and answers.
- browser-sign-in.t27: LOGINS_NOTE pointed at a sign-in playbook that existed nowhere. The order is now pinned (door, read, press by label, the PERSON types the secret, prove by cookie), and what a site wants stays learned in browser_recipes rather than declared.
- Both compile; 10 of 10 test blocks pass under t27c test-report. A completed live sign-in and an unattended pod restart are explicitly NOT claimed -- the restart needs two service variables, one of them a token only the owner can issue.
