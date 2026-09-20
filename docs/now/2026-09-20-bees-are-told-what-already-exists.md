# NOW -- Every issue the feeder writes now says how to find what already exists (2026-09-20)

## The other half of the duplicate ratchet: a bee could not ask, so it wrote the function again (Closes #4292)

- #4290 measured it: 576 of 4021 function bodies in `specs/` are byte-identical copies, `magadd` written 30 times, `sadd` 29. It also added `Duplicate Body Ratchet`, which stops the number growing.
- A gate alone turns a structural problem into a red check. The bee that wrote the thirtieth `magadd` was not careless: its brief named the file, the signatures and the criteria, and said nothing about the twenty-nine that existed. There was no command it could run to find out.
- The feeder's preamble now carries that command, in every issue it writes: `python3 tools/dupe_scan.py --name <function>` for where a function already lives, and `--like <this spec>` for what in this file is written elsewhere. Both answer with a file and a line, which is the only useful form of the answer.
- It also says what to do with the answer - reuse it with `use module::name;`, which 494 parsing specs already do - and that the ratchet will fail a pull request that adds a new copy.
- Not changed: what the feeder measures or how it verifies its claims. It still executes every command it quotes before opening an issue.
