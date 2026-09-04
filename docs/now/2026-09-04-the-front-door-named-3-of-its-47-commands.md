# NOW -- The front door named 3 of its 47 commands (2026-09-04)

## The front door named 3 of its 47 commands (Closes #3120)

- The Rust tri binary implements 47 subcommands and tri help named 3; 44 were reachable and invisible to anyone reading the front door's own help
- The count was frozen at seventeen in four places, one of them printed to the user on stderr, while the line twenty rows above argues that a baked-in list is a second source of truth that goes stale
- tri help now asks the binary -- 3 named becomes 38 -- and says NOT BUILT with the build line when it cannot ask, rather than printing nothing
- Two assertions, both mutation-checked: a frozen count put back fails, and printing nothing with no binary fails
