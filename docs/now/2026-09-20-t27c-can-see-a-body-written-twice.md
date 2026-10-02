# NOW -- t27c can see a body written twice, not only a name written twice (2026-09-20)

## `t27c dupes --bodies` and `--name`: the compiler answers the question a bee has to ask before it writes (Closes #4294)

- `t27c dupes` compared declaration NAMES. Over master that is 9246 lines, most of them legitimate - `gf4.t27` through `gf64.t27` each carry a `validate_format` by design - and it cannot see the duplication that costs: the same body written again, under any name, in another file.
- Comparing bodies structurally instead: **686 functions in 189 groups**, the largest being `magadd` at 30 copies of the same 935 characters. Verified by hand against `specs/ternary/gft_axpy.t27:11` and `specs/ternary/gft_bitnet_neuron.t27:12`.
- The digest is the parsed children of the `fn`, with `line` left out and every other field kept. The first version hashed `format!("{:?}", node)`, which carries `line`, so identical bodies at different line numbers hashed differently and the command reported 276 functions where the text-based walk found 555 over the same files. An instrument that answers half is worse than one that refuses.
- Leaving other fields out would be the opposite mistake: bodies differing only in a cast kind, an array size or a pragma would be called copies, which is a false accusation of plagiarism against a bee that wrote something different.
- The output is deterministic. A `HashMap` is not ordered, and two runs of the first version listed the same groups in different order, so a reader comparing them saw shuffling; groups are now sorted by size then by their first member, and members are sorted within a group.
- Two instruments, stated rather than reconciled: `tools/dupe_scan.py` (#4290) walks text and therefore covers all 948 specs including the 61 that do not parse; `t27c dupes --bodies` walks the AST and therefore sees only the 887 that do, but sees through formatting. The gate uses the first; a bee asking "does this already exist" is better served by either.
- `t27c dupes` with no flag behaves exactly as before.
