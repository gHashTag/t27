# NOW -- The gate printed ok over a line that exits 2 (2026-09-04)

## The gate printed ok over a line that exits 2 (Closes #3130)

- docs/TECHNOLOGY-TREE.md:342 invokes ./bootstrap/target/release/t27c validate-graph, which exits 2, and the gate said every t27c subcommand a live document names exists
- Three defects: the path prefix accepted only ./ and scripts/; the tri ceiling returned before the t27c half was reported; and the declaration window looked only forward, so it flagged this fix's own replacement text
- The whole Verification Commands block was dead -- four commands, all exit 2 -- and is rewritten to the one that exists, tools/check_graph_law8.py, with the other three named as not built
- The backward blank-line stop SURVIVED its mutation on this corpus, so it is priced by a constructed case in --self-check rather than left unproven
