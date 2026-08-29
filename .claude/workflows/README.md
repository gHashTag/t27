# .claude/workflows/ — reusable multi-agent runs

Each file here is a Workflow script: a deterministic orchestration of subagents,
invoked by name rather than re-authored each time.

```
Workflow({ name: "loop-recon", args: { repo: "/path/to/checkout" } })
```

## What is here

| workflow | question it answers |
|---|---|
| `loop-recon.js` | What is weakest in this repository right now, what is the field already doing that we are not, and which `tri` command is missing — each finding adversarially refuted before it reaches a plan |
| `column-forensics.js` | An aggregate moved and the tool cannot say which rows. Names them from two per-item dumps, reads each cause out of the tool the gates run, and rules on each: regression, or a number that had been measuring a silent drop |

## The shape these follow, and why

**Pipeline, not barrier.** Findings from one dimension go to verification while
another dimension is still searching. A barrier between the phases would idle
the fast dimensions behind the slowest one, and there is no cross-dimension
dependency to justify it.

**Every finding is refuted before it is believed.** A separate agent is asked to
*kill* each claim, with `stands=false` as the default when it cannot be
confirmed from evidence the verifier gathered itself. This exists because a
plausible-but-wrong finding costs more than a missed one: it sends the next hour
somewhere real work is not.

**Structured output, not parsed prose.** `schema:` forces the subagent through a
validating tool call, so the script never regex-scrapes an answer out of English.

**The prompt states what is already known.** Each recon prompt lists the findings
already in hand and says "do NOT re-report these". Without it, every run
rediscovers the same top three and reads like progress.

**Effort is part of the finding.** `minutes | hours | days`, and the verifier is
asked to correct it. A true finding with a wrong cost estimate still plans the
next iteration badly.

## What they deliberately do not do

They do not edit anything. Recon returns findings; the decision about what to
act on stays with the main loop, where the context to weigh it lives.

They do not hard-code a checkout path. `REPO` comes from `args.repo` or the
working directory — baking one machine's absolute path into a shared file is
what `secret-scan` rejects, and a workflow is not exempt from a rule the
repository applies to everything else.

## `column-forensics` — the argument it takes

```
Workflow({ name: "column-forensics", args: {
  beforeTable: "/tmp/before.tsv",
  afterTable:  "/tmp/after.tsv",
  tool: "t27c corpus",
  columns: "zig(gen,build), rust, c, verilog, dropped"
}})
```

Produce the two tables with the SAME flag from two builds:

```
t27c corpus --per-spec /tmp/before.tsv     # built at the earlier commit
t27c corpus --per-spec /tmp/after.tsv
```

It refuses without both. There is no mode where it derives the tables itself:
recomputing them with a different invocation is how a second, disagreeing number
gets born, and this repository has paid for that more than once.

**The verdict phase defaults to `regression: false`.** A number that falls when a
silent drop is fixed was measuring the drop — on W699, recovering 1 049 discarded
tokens took Zig 217 to 215 because three specs had been accepted on less code
than they contain. Two of three lenses must affirmatively establish that working
behaviour was lost before a row is called a regression.
