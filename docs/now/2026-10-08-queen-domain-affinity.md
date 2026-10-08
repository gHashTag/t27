# NOW -- a domain is an affinity, not a cap (control.t27 section 5)

## specs/queen/control.t27 section 5 and specs/queen/domains.t27 (Closes #7674, part of #6657)

- `placement(idle_in_domain, running, cap)` takes the swarm's capacity. The removed `DOMAIN_CAP = 4`
  would have held the 70-lane swarm to 16 runtimes. Owner decision 2026-10-08: domains are an
  affinity, not a hard limit.
- `specs/queen/domains.t27` (new) classifies a task by the first path of its boundary, using
  per-domain prefix lists for t27-c, t27-b and queen-ops. An unnamed path is spec work. It lives
  apart from control.t27 because the supervisor reads control.t27 through a vendored compiler wasm.
  Measured: that typechecker reports 3 errors on the classifier's byte walks, and a refused card
  stops every dispatch. control.t27 as changed here typechecks there with 0 errors.
- Tests: control.t27 17 of 17 and domains.t27 2 of 2 (`t27c gen` + `zig test`).
- Not established here: the runtime placement. gHashTag/BrowserOS slice 5 wires it.
