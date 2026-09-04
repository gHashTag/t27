# NOW -- The gate went red on the section describing it (2026-09-04)

## The gate went red on the section describing it (Closes #3100)

- documented-commands.yml and the ci-gates sections about it merged from separate branches; on master the gate reported 6 live mentions, all six in the section about dead commands, quoting dead commands
- The declaration window was one line, and English corrects AFTER it names the thing -- 'There is no such subcommand' sat two lines below the name it retracts; the window is now the paragraph, stopping at the blank line
- Four of the six were never invocations: they were the wide matcher's own false positives written as if they were commands, and are now written as what they are
- Mutations: collapse the window to one line and two return; remove the blank-line stop and the self-check exits 2; a dead command back in the README still exits 1
