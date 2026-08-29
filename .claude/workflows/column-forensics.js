export const meta = {
  name: 'column-forensics',
  description: 'An aggregate moved. Name the rows behind it, and say for each whether the move is a regression or a repair.',
  whenToUse:
    'A headline number changed (acceptance count, pass count, coverage) and the tool that reports it cannot name which items moved. Run this instead of guessing, and instead of hand-rolling a harness that has not been checked against the tool the gates use.',
  phases: [
    { title: 'Rows', detail: 'diff the per-item tables and take each moved row' },
    { title: 'Cause', detail: 'one agent per row: the exact tool output that explains it' },
    { title: 'Verdict', detail: 'three lenses per row, refuting by default' },
    { title: 'Report', detail: 'one table, regressions first' },
  ],
}

// ---------------------------------------------------------------------------
// Why this exists
//
// `corpus` reported "Zig accepts it 215" where it had said 217. Nothing in the
// output could say WHICH two specs, and the first instinct was to write a local
// harness -- the same instinct that had already produced a harness reporting an
// implausible zero, which was correctly distrusted and thrown away.
//
// The rule this workflow encodes: an aggregate that can move needs a per-item
// dump, and every moved item needs a cause read from the SAME tool the gates
// run, not from a reimplementation of it.
//
// The verdict phase exists because "the number went down" is not the same as
// "something broke". Recovering silently-dropped input made two specs fail that
// had been passing on less code than they contained. Down was correct there.
// Three lenses, each told to refute, is what separates the two cases.
// ---------------------------------------------------------------------------

const A = args || {}
const before = A.beforeTable
const after = A.afterTable
const tool = A.tool || 'the tool that produced these tables'
const columns = A.columns || 'the columns in the table header'

if (!before || !after) {
  log('column-forensics needs { beforeTable, afterTable } -- two per-item dumps to diff.')
  log('Produce them with the SAME binary flag from two builds, e.g.')
  log('  t27c corpus --per-spec /tmp/before.tsv    (built at the earlier commit)')
  log('  t27c corpus --per-spec /tmp/after.tsv')
  return { error: 'missing beforeTable/afterTable' }
}

const ROWS = {
  type: 'object',
  required: ['rows'],
  properties: {
    rows: {
      type: 'array',
      items: {
        type: 'object',
        required: ['item', 'column', 'from', 'to'],
        properties: {
          item: { type: 'string' },
          column: { type: 'string' },
          from: { type: 'string' },
          to: { type: 'string' },
        },
      },
    },
    unchanged: { type: 'integer' },
  },
}

const CAUSE = {
  type: 'object',
  required: ['item', 'evidence', 'explanation', 'reproduction'],
  properties: {
    item: { type: 'string' },
    // The literal output. A paraphrase is where a wrong diagnosis hides.
    evidence: { type: 'string' },
    explanation: { type: 'string' },
    // Smaller than the item it came from, or the cause is not isolated yet.
    reproduction: { type: 'string' },
    reproduced: { type: 'boolean' },
  },
}

const VERDICT = {
  type: 'object',
  required: ['regression', 'why'],
  properties: {
    regression: { type: 'boolean' },
    why: { type: 'string' },
    lens: { type: 'string' },
  },
}

phase('Rows')
const moved = await agent(
  `Diff these two per-item tables and return every row whose columns differ.

  before: ${before}
  after:  ${after}

  Both are sorted, one line per item, tab-separated; the header names ${columns}.
  Use \`diff\`. Do NOT re-derive the tables and do not run ${tool} yourself --
  the tables are the reading, and recomputing them with a different invocation
  is how a second, disagreeing number gets born.

  Report the count of unchanged rows too: "3 of 650 moved" and "3 moved" are
  different claims, and only the first one says the run covered the corpus.`,
  { schema: ROWS, phase: 'Rows', label: 'diff' },
)

const rows = (moved && moved.rows) || []
log(`${rows.length} row(s) moved, ${(moved && moved.unchanged) ?? '?'} unchanged`)
if (!rows.length) {
  return { moved: [], note: 'No row differs. If an aggregate moved, the tables are not from the two builds you think they are.' }
}

const LENSES = [
  'correctness: is the new output actually wrong, or newly honest about input that used to vanish?',
  'history: did this item pass before because something was SKIPPED rather than because it worked?',
  'blast radius: does the same cause affect items that did not move, and are they simply not exercised?',
]

phase('Cause')
const findings = await pipeline(
  rows,
  (r) =>
    agent(
      `Item \`${r.item}\`, column \`${r.column}\`, went ${r.from} -> ${r.to}.

      Find the cause and QUOTE THE TOOL. Run the same commands ${tool} runs --
      read its source if you must to get the exact flags -- and paste the real
      diagnostic. A paraphrase is where a wrong diagnosis hides.

      Then shrink it: produce the smallest input that still shows the same
      diagnostic, and say whether it actually reproduced. Copy the real shape
      from the item; do not invent a shape the corpus does not contain, or you
      will draw a conclusion about a language nobody writes.

      If you cannot reproduce it, say so with reproduced:false. An unreproduced
      cause is a hypothesis, and it must not be reported as a finding.`,
      { schema: CAUSE, phase: 'Cause', label: `cause:${r.item}` },
    ),
  (cause, r) =>
    parallel(
      LENSES.map((lens) => () =>
        agent(
          `Row: \`${r.item}\` ${r.column} ${r.from} -> ${r.to}
          Claimed cause: ${cause ? cause.explanation : '(none produced)'}
          Evidence: ${cause ? cause.evidence : '(none)'}

          Judge it through ONE lens: ${lens}

          Try to REFUTE the reading that this is a regression. Default to
          regression:false if the evidence does not establish that working
          behaviour was lost -- a number that falls when a silent drop is fixed
          was measuring the drop, and reverting to restore it is the real
          regression.`,
          { schema: VERDICT, phase: 'Verdict', label: `judge:${r.item}` },
        ),
      ),
    ).then((vs) => {
      const votes = vs.filter(Boolean)
      const reg = votes.filter((v) => v.regression).length
      return {
        ...r,
        cause,
        regression: reg >= 2,
        votes: `${reg}/${votes.length} call it a regression`,
        reasons: votes.map((v) => v.why),
      }
    }),
)

const out = findings.filter(Boolean)
const regressions = out.filter((f) => f.regression)
const unreproduced = out.filter((f) => f.cause && f.cause.reproduced === false)

phase('Report')
log(`${regressions.length} regression(s), ${out.length - regressions.length} correct move(s)`)
if (unreproduced.length) {
  log(`${unreproduced.length} cause(s) NOT reproduced -- reported as hypotheses, not findings`)
}

return {
  moved: out.length,
  unchanged: (moved && moved.unchanged) ?? null,
  regressions,
  correct_moves: out.filter((f) => !f.regression),
  unreproduced: unreproduced.map((f) => f.item),
}
