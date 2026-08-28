export const meta = {
  name: 'loop-recon',
  description: 'Weak points of t27 and who else is in this space, adversarially verified',
  whenToUse: 'Each /loop iteration: find what is weakest and what the field already solved. Pass {repo: "<path>"} to point it at a checkout.',
  phases: [
    { title: 'Recon', detail: 'weak points, competitors, unmeasured claims, quick wins' },
    { title: 'Verify', detail: 'refute each finding before it reaches the plan' },
  ],
}

// The repository this is run from. Hard-coding one worktree's absolute path
// is the same defect `secret-scan` rejects, so it comes from args or the cwd.
const REPO = (args && args.repo) || process.env.PWD || '.'

const FINDINGS = {
  type: 'object',
  required: ['findings'],
  properties: {
    findings: {
      type: 'array',
      items: {
        type: 'object',
        required: ['title', 'evidence', 'why_it_matters', 'effort'],
        properties: {
          title: { type: 'string' },
          evidence: { type: 'string', description: 'file:line, command output, or URL' },
          why_it_matters: { type: 'string' },
          effort: { type: 'string', enum: ['minutes', 'hours', 'days'] },
          remedy: { type: 'string' },
        },
      },
    },
  },
}

const VERDICT = {
  type: 'object',
  required: ['stands', 'reasoning'],
  properties: {
    stands: { type: 'boolean' },
    reasoning: { type: 'string' },
  },
}

const DIMENSIONS = [
  {
    key: 'weakest',
    prompt: `In ${REPO} (the t27 spec-first ternary language: .t27 specs -> Verilog/C/Rust/Zig via t27c).

Current state, already known — do NOT re-report these:
  - suite 2424 passed / 0 failed
  - seal gate, specs-generate gate, gate-preconditions all exit 0
  - t27c 0.2.0 published to crates.io today
  - #2747 nothing builds the Lean proofs (250 theorems, 45 workflows, no lake build)
  - #2754 five gates have no master baseline; five tracked .py files do not parse

Find what is WEAKEST that is NOT on that list. Concretely:
  - run \`./target/release/t27c corpus\` and \`./target/release/t27c suite\` and read what they say is worst
  - the gap between "generates" and "accepts" per backend -- which backend is furthest behind and what is the single largest cause
  - any tool in tools/ that reports a number nobody acts on
  - anything where a count is going the WRONG way

Rank by (impact / effort). Quote real command output. Effort must be honest: "minutes" means a single edit.`,
  },
  {
    key: 'rustc',
    prompt: `In ${REPO}, the corpus report says rustc accepts 0 of 559 generated Rust files -- the only backend at zero, while Zig accepts 217 and cc accepts 157.

Measure WHY, precisely:
  - generate Rust for a sample of specs (\`./target/release/t27c gen-rust <spec>\`), compile each with rustc, and classify the errors
  - report the top error classes by COUNT and by NUMBER OF SPECS BLOCKED (these rank differently and the second is what matters)
  - for the largest class, say whether it is one emitter defect or many
  - state how many specs would compile if the single largest cause were fixed -- measure this by actually removing/working around it on a sample, not by assuming

A previous measurement said 499 distinct error classes with the largest being 688 occurrences of a missing \`serde\`. Verify or refute that, and say which measurement is right.`,
  },
  {
    key: 'competitors',
    prompt: `Research who else works on this and what they have that t27 does not. Use WebSearch and WebFetch (load them with ToolSearch first).

t27 is: a spec-first language where a .t27 spec is the single source of truth and t27c emits Verilog, C, Rust and Zig from it, with seals (hashes pinning each spec to its four outputs), a corpus ratchet, and a formal Lean model of the Verilog-lowerable subset.

Search for the actual field:
  - spec-first / single-source-of-truth HDL generation (Chisel, SpinalHDL, Amaranth/nMigen, Veryl, PyMTL3, Bluespec, Clash, Filament, Calyx)
  - multi-target codegen from one spec (Kaitai Struct, Protobuf/Cap'n Proto as prior art for the "one spec, N backends" pattern)
  - formally verified compiler backends (CompCert, Vellvm, Lean/Coq-modelled lowering)
  - ternary / BitNet / low-precision hardware toolchains

For each of the 5-8 most relevant: what it does, what it has that t27 lacks, what t27 has that it lacks, and one concrete idea worth stealing. Give URLs. Be blunt about where t27 is behind — the point is to find work, not to reassure.`,
  },
  {
    key: 'tri-cli',
    prompt: `In ${REPO}, read cli/tri/src/ and list every subcommand \`tri\` has today (\`./target/release/tri --help\`, and each subcommand's --help).

Then propose NEW commands that would speed up the recurring work in this repository. The work that actually recurs, from the last day:
  - "did my change move the corpus?" -- currently requires building two binaries and diffing per spec by hand
  - "which gates have no baseline on master?" -- currently requires reading 45 workflow files and gh run list per file
  - "is this number still true?" -- claims in docs drifting from what the tools print
  - "what did this loop iteration change?" -- assembling the report by hand each time

For each proposal: the exact command line, what it prints, which manual sequence it replaces, and roughly how it would be implemented (which existing code it can reuse). Prefer 2-3 commands that are genuinely load-bearing over a long list. Say explicitly if an existing command already covers it.`,
  },
]

phase('Recon')
const recon = await pipeline(
  DIMENSIONS,
  d => agent(d.prompt, { label: `recon:${d.key}`, phase: 'Recon', schema: FINDINGS }),
  (res, d) => {
    if (!res || !res.findings || res.findings.length === 0) return []
    return parallel(
      res.findings.slice(0, 6).map(f => () =>
        agent(
          `Refute this, using the repository at ${REPO} and the network where the claim is about the outside world.

CLAIM: ${f.title}
EVIDENCE: ${f.evidence}
WHY IT MATTERS: ${f.why_it_matters}
EFFORT CLAIMED: ${f.effort}

Check it yourself -- run the command, read the file, fetch the URL. Default to stands=false if you cannot confirm it with evidence you gathered. If the claim is true but the EFFORT is wrong, say stands=true and correct the effort in your reasoning.`,
          { label: `verify:${(f.title || '').slice(0, 34)}`, phase: 'Verify', schema: VERDICT }
        ).then(v => ({ ...f, dimension: d.key, verdict: v }))
      )
    )
  }
)

const all = recon.flat().filter(Boolean)
const confirmed = all.filter(f => f.verdict && f.verdict.stands)
log(`${all.length} findings, ${confirmed.length} survived`)

return {
  confirmed: confirmed.map(f => ({
    dimension: f.dimension,
    title: f.title,
    effort: f.effort,
    evidence: f.evidence,
    remedy: f.remedy,
    note: f.verdict.reasoning.slice(0, 400),
  })),
  refuted: all.filter(f => f.verdict && !f.verdict.stands).map(f => ({ title: f.title, why: f.verdict.reasoning.slice(0, 200) })),
}
