# NOW -- a domain is an affinity, not a cap (control.t27 section 5)

## specs/queen/control.t27 section 5 (Closes #7674, part of #6657)

- `placement(idle_in_domain, running, cap)` takes the swarm's capacity. The removed `DOMAIN_CAP = 4`
  would have held the 70-lane swarm to 16 runtimes. Owner decision 2026-10-08: domains are an
  affinity, not a hard limit.
- `domain_of_path` classifies a task by the first path of its boundary, using per-domain prefix lists
  (t27-c, t27-b, queen-ops). An unnamed path is spec work.
- Tests: 18 of 18 pass (`t27c gen` + `zig test`). New: `a_domain_is_an_affinity_not_a_cap` and
  `a_task_takes_the_domain_of_its_first_path`.
- Not established here: the runtime placement. gHashTag/BrowserOS slice 5 wires it.
