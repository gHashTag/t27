# NOW -- LAW 8 is violated, and its validator printed a literal (2026-09-04)

## LAW 8 is violated, and its validator printed a literal (Closes #3104)

- architecture/graph_v2.json: 55 nodes, 91 edges -- 65 forward, 21 same-tier, 5 tier-backward, 1 cycle 17->19->18->17; LAW 8 does not hold today
- graph-depcheck.sh, advertised in skill.md as 'Validate graph dependencies', set local violations=0 and printed the tick from it; GRAPH_FILE was assigned and never read; the output is byte-identical from an empty directory
- tools/check_graph_law8.py measures both readings and holds them down-only at 1 and 5, because repairing the graph is an architectural decision and a checker red on arrival is a checker that gets muted
- Five mutations: a planted second cycle fails, repairing the recorded one fails until the ledger moves, an absent graph and an empty graph both refuse with 2, and crippling the cycle finder is caught by the self-check
