# AI code review and merge automation: mechanisms worth borrowing for the t27 reviewer bee

Research date: 2026-10-04. Read-only research. Nothing was posted, commented or submitted anywhere.
Every number below is vendor-reported unless marked otherwise. Treat vendor numbers as directional.
Each section ends with its source URLs. All sources are also collected in section 9.

## 0. Facts about gHashTag/t27 observed during this research (read-only GitHub API, 2026-10-04)

- Repo owner type is `User`, and the repo is `public`. GitHub's native merge queue is only
  available to organization-owned public repos, or to private repos on Enterprise Cloud. It is
  therefore NOT available to t27 as the repo stands. Several 2026 issues on other personal repos
  report the ruleset API rejecting the `merge_queue` rule with a 422 error. The way to get it is to
  move the repo into a free organization, which is $0 for public repos.
  https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue
  https://github.com/evgunter/cad/pull/1887
  https://github.com/ned2/dashpot/issues/250
- Ruleset `t27-master-protection` sets:
  - required checks: `validate`, `check-linked-issue`, `parse-ratchet`
  - `strict_required_status_checks_policy: false`, so branches need not be up to date
  - `required_approving_review_count: 0`
  - `dismiss_stale_reviews_on_push: true`
- Repo settings: `allow_auto_merge: true`, `allow_update_branch: false`.
- Open PRs: 49 in total.
  - 13 are CONFLICTING and 36 are MERGEABLE.
  - Median size is 175 lines changed (additions plus deletions), across a median of 2 files.
  - 10 of the 49 have at least one required check in a failed state.

These facts matter for the ranking:
- Conflicts and red required checks block about 23 of 49 PRs. That is mechanical work, not
  review work.
- The native merge queue needs an org move. Without one, a self-built queue in Actions is needed.

## 1. The reviewer bee's weaknesses, mapped to the mechanisms that address them

| Weakness | Mechanisms that address it (sections) |
|---|---|
| W1. Flash models state command output they never ran | Runner-executed evidence, evidence-id citation, fabricated-tool-use interceptor (3, 5.2, 7#1) |
| W2. Sloppy reasoning about acceptance criteria | Per-criterion structured verdict with citations, independent per-claim verification, fail closed (3.6, 3.9, 7#2, 7#7) |
| W3. Two flash models from one vendor "agreeing" | Correlated-error research; cross-vendor generator/grader; deterministic second vote (5.1, 3.13, 7#6) |
| W4. About 3 minutes per review | Risk tiers, skip rules, caching by SHA, parallel runs, a cheap lane for trivial PRs (3.11, 3.12, 7#3, 7#8) |
| W5. Stale or red required checks, merge conflicts | Auto-update, rebase and requeue; stuck-PR timeouts; merge queue with batching and bisection (4, 7#4, 7#5) |
| W6. Bees produce PRs faster than review consumes them | Per-source caps, WIP limits, track-record eligibility, PR size limits (3.14, 3.15, 7#9) |

## 2. Cross-cutting patterns: what almost every serious system converged on

1. **Generate, then verify.** Nearly every serious system now pairs a finder with a separate
   verifier or judge pass that tries to refute each finding before it is shown. Examples:
   - BugBot validator
   - CodeRabbit judge
   - Qodo judge plus self-reflection
   - Anthropic security review, with one false-positive filter sub-task per finding
   - Claude Code Review verification step
   - Ellipsis HallucinationFilter
   - Uber grader
   - Cloudflare coordinator
2. **Precision over recall.** OpenAI frames review as a utility trade-off. A false alarm costs
   human time and trust, so they tune for precision. Graphite, Greptile and Cloudflare say the
   same.
3. **Execution and tool grounding beat diff-only prompting.**
   - OpenAI found that repo-wide tools plus code execution cut incorrect comments compared with
     reviewing the diff alone.
   - BugBot's biggest gain came from switching to an agentic design.
   - Codex Security only marks a finding "validated" after reproducing it in a container.
4. **An LLM self-rated severity or confidence score alone is weak.**
   - Greptile found a 1-10 severity cutoff "nearly random".
   - Atlassian found an LLM factual-correctness gate had minimal effect in ablation.
   - A trained actionability classifier, embedding feedback, or majority voting worked better.
   - Confidence thresholds still help when the score comes from a separate verification call,
     as in the Anthropic security review and the Claude plugin.
5. **Risk-tier the work.** Who does this:
   - Meta RADAR, Cloudflare, Ona and Cursor Approval Agents
   - PR-Agent, with effort-based auto-approve
   - Copilot, whose approvals count only on admin-allowed paths

   Each one sends cheap, low-risk changes down a light lane and keeps humans on anything risky.
6. **Never approve on a PR's own policy.** Cursor reads approval policy from the base branch when
   a PR modifies it.
7. **Re-review only the delta, and auto-resolve fixed threads.** CodeRabbit, Cloudflare,
   Sourcery, Ellipsis and Copilot do this. Copilot also dismisses its own approval on new
   commits.
8. **Measure resolution, not volume.** The standard metric is whether the flagged issue was
   actually changed before merge. BugBot calls it resolution rate; Greptile and Uber call it
   address rate. Atlassian uses the same signal to train a classifier.
9. **Fail closed.** Two published bugs are fail-open defaults on judge failure:
   - PR-Agent #3833: a failed reflection silently became score 7
   - claude-code-security-review #139: a retired model silently disabled filtering

## 3. Products and open-source projects

### 3.1 Cursor BugBot

How it reduces false approvals and hallucinated claims:
- V1 ran 8 parallel passes, each with the diff in a randomized order.
- Similar findings were bucketed together, and a finding seen in only one pass was dropped by
  majority vote.
- A validator model then removed false positives.
- Findings were deduplicated against bugs already posted, and a category filter removed low-value
  classes such as compiler warnings and doc nits.

Execution grounding: in fall 2025 BugBot moved to a fully agentic loop. The model calls tools,
pulls context on demand, and verifies before posting. This produced the largest quality jump.

Learning: the quality metric is resolution rate. An LLM judge compares each flag against the
final merged code, and the judge was spot-checked as nearly always right. There is also an offline
eval set, BugBench. Reported results, V1 (July 2025) to V11 (January 2026):

| Metric | V1 | V11 |
|---|---|---|
| Resolution rate | 52% | over 70% |
| Bugs per run | 0.4 | 0.7 |
| Resolved bugs per PR | about 0.2 | about 0.5 |

Scale: over 2M PRs per month.

Source: https://cursor.com/blog/building-bugbot

### 3.2 Cursor PR Routing and Approval Agents

Risk and routing:
- Each PR gets a risk score, and a configurable maximum risk threshold caps what can be
  auto-approved.
- Any Bugbot or security-review finding that needs a human blocks auto-approval.
- Policy is hierarchical: `APPROVAL_POLICY.md` files plus `.cursor/approval-policies/ROUTING.md`.
- Reviewers are assigned by code ownership and commit history.

Anti-tamper: if a PR edits the approval policies, the agent evaluates it against the base-branch
version of those policies. This transfers directly to t27. The reviewer's prompts, criteria
parser and runner must be loaded from master, never from the PR head.

Source: https://cursor.com/docs/approval-agents

### 3.3 OpenAI Codex code review and Codex Security

How it reduces false approvals:
- The reviewer is trained separately, on the same base model as the generator.
- It is explicitly tuned for precision. Their utility is
  `P(correct)*C_saved - C_human_verification - P(incorrect)*C_false_alarm`.
- Precision and recall can be steered per repo through `AGENTS.md`.
- A verifier is cheap compared with generation, needing only a fraction of the generator's
  tokens.

Execution grounding:
- Repo-wide tools plus code execution cut incorrect comments compared with diff-only review.
- Codex is trained to cite terminal logs as evidence for claims.
- Codex Security ranks likely issues, then tries to reproduce each one in a clean container.
  Only reproduced findings are marked validated.

Numbers:
- Over 100k external PRs per day (October 2025)
- Over 80% positive reactions
- 52.7% of comments lead to a code change
- GPT-5-Codex makes about 70% fewer incorrect comments than GPT-5

Sources:
- https://alignment.openai.com/scaling-code-verification/
- https://openai.com/index/introducing-upgrades-to-codex/
- https://openai.com/index/codex-security-now-in-research-preview/

### 3.4 CodeRabbit

Architecture:
- A webhook feeds a Cloud Tasks queue, which runs each review in a microVM sandbox (8 vCPU,
  32 GB, 1-hour timeout).
- Over 20 linters and SAST tools run deterministically, and their output feeds the LLM.
- Cheap models (GPT-4.1 nano/mini class) compress context.
- The agent writes shell commands (cat, grep, ast-grep) rather than calling tool schemas, with a
  capped recursion depth.

How it reduces hallucinated claims: a judge or verification model scores each finding against the
gathered context and drops claims it cannot ground. About 7-8 models run per review, routed by
cost.

Incremental: commits already reviewed are not reviewed again. There is also a learnings layer
built from chat feedback.

Latency: about 1-5 minutes to the first comment.

Independent audit of 28 PRs: 35% of comments were genuine, 21% nitpicks, and 15% useless. The
"Code Review Is Not About Bugs" arXiv study is also relevant.

Sources:
- https://theaiengineer.substack.com/p/how-coderabbit-actually-works
- https://www.coderabbit.ai/blog/how-coderabbits-agentic-code-validation-helps-with-code-reviews
- https://arxiv.org/html/2607.03316v2

### 3.5 Graphite (Diamond / Graphite Agent, stacked PRs, merge queue)

Review:
- Precision first.
- Retrieval over past PRs.
- Up/downvotes on comments.
- Custom prompts and regex rules.

Claimed results:
- Under 3% false positives at launch
- Under 5% negative comment rate across 500k PRs
- 3.5% of comments downvoted

A competitor benchmark from Macroscope found Graphite had the lowest detection (18%), the lowest
false positives, and 0.62 comments per PR. That is the expected corner for a precision-first
tool. Graphite was reportedly acquired by Cursor in December 2025.

Merge queue: the stack-aware merge queue validates a whole stack as one unit.
- CI runs on the top PR, which contains everything below it.
- If CI passes, the queue fast-forwards trunk to the top of the stack in one step.
- If CI fails, only the failing PR and the PRs that depend on it are evicted.
- Batching groups stacks into one CI run, with bisection as a fallback.
- Parallel CI across stacks: about 1.5x faster merges reported (p75 down about 26%).
- Fast-track lets a single PR jump to the front of the queue.

PR size data (from Graphite's own platform, so possibly biased toward its pitch):
- The median PR is 47 lines.
- 50-line PRs are reviewed and merged about 40% faster than 250-line PRs, and are 15% less often
  reverted.
- PRs under 50 lines usually merge with no follow-up commits. PRs over 500 lines get about 5x
  more updates after publishing.
- Only 24% of PRs over 1000 lines receive any review comment.
- Sub-10-line PRs are reverted more often than 10-100-line PRs.

Sources:
- https://graphite.com/blog/graphite-reviewer-launch
- https://graphite.com/guides/ai-code-review-false-positives
- https://diamond.graphite.dev/
- https://macroscope.com/content/best-ai-code-review-tools-github-2026
- https://graphite.com/blog/the-first-stack-aware-merge-queue
- https://graphite.com/docs/graphite-merge-queue
- https://graphite.com/docs/get-started-merge-queue
- https://graphite.com/blog/the-ideal-pr-is-50-lines-long
- https://graphite.com/research/median-pr_size
- https://graphite.com/research/submits-by-pr-size

### 3.6 Greptile

Noise control, which is the most useful published negative result:
- Baseline: only 19% of comments were addressed, 2% were wrong, and 79% were nits.
- These failed:
  - prompting
  - few-shot examples
  - LLM-rated severity 1-10 with a cutoff of 7, which was "nearly random" and slow
- This worked: a per-team embedding store of past comments with their up/downvotes. A new
  comment is blocked if it is similar to 3 or more downvoted comments, and passed if similar to
  3 or more upvoted ones. Ambiguous cases pass.
- Result: address rate went from 19% to over 55% within two weeks.
- Metric: whether the author changed the flagged code in a later commit. Adapting to a new team
  takes about 2-3 weeks of reactions.

Merge gating: each PR gets a 0-5 confidence score based on:
- the severity and number of issues
- the complexity of the change
- how well the code fits codebase patterns

Several public agent repos gate merges on "5/5 on the head SHA, and no new findings".

v3 (September 2025) rebuilt review as an agentic loop with codebase-search tools, replacing a fixed
flowchart. It claims a 256% better upvote ratio and 75% lower cost than v2. It also has a
configurable maximum number of agentic turns per review, and auto-detects CLAUDE.md and
.cursor/rules.

Sources:
- https://www.greptile.com/blog/make-llms-shut-up
- https://www.greptile.com/docs/code-review/training-the-learning-system
- https://www.greptile.com/docs/code-review/first-pr-review
- https://www.greptile.com/blog/greptile-v3-agentic-code-review

### 3.7 Qodo Merge / PR-Agent (open source; self-hostable on GitHub Actions)

Self-reflection: a second LLM call scores each suggestion 0-10 and gives a reason. Score-0
suggestions are dropped, as is anything below `suggestions_score_threshold`; the rest are
re-ranked.

Bug lesson (PR #3833): when reflection failed, the fallback silently assigned score 7, which is a
fail-open. Judge failure must mean "not approved".

Auto-approve knobs:
- `enable_auto_approval`
- `auto_approve_for_low_review_effort`, using the review's 1-5 effort estimate
- `auto_approve_for_no_suggestions`
- `ensure_ticket_compliance`, which blocks approval unless the linked ticket is "Fully compliant"

Ticket compliance is exactly the t27 "linked issue with acceptance criteria" shape. Each
requirement is labelled:
- Fully compliant
- Partially compliant
- Not compliant
- PR Code Verified: met in the code, but needs a human or a run to confirm

`check_pr_additional_content` flags code unrelated to the ticket.

New Qodo: parallel specialist agents plus a judge that dedupes and drops low-confidence findings.
Its Rule Miner learns rules from PR history.

Sources:
- https://docs.pr-agent.ai/tools/review/
- https://docs.qodo.ai/code-review
- https://github.com/The-PR-Agent/pr-agent/pull/3833

### 3.8 GitHub Copilot code review

- **Architecture:** agentic tool calling plus deterministic tools (CodeQL, ESLint, PMD). GA in
  March 2026, running on GitHub Actions runners.
- **Fixes:** it can hand a fix to the coding agent, which opens it as a stacked PR.
- **Approvals:**
  - Copilot leaves a Comment review by default, so it does not count toward required approvals.
  - Since 2026-09-01, admins can let Copilot submit approvals that count.
  - Every review includes an "approval assessment".
  - Admins can limit which file paths a Copilot approval counts for.
  - The approval is dismissed when new commits are pushed.
- **Re-review on new pushes** is opt-in.

Sources:
- https://docs.github.com/copilot/using-github-copilot/code-review/using-copilot-code-review
- https://github.blog/changelog/2025-10-28-new-public-preview-features-in-copilot-code-review-ai-reviews-that-see-the-full-picture/
- https://github.blog/changelog/2025-11-20-linter-integration-with-copilot-code-review-now-in-public-preview/
- https://github.blog/changelog/2026-03-05-copilot-code-review-now-runs-on-an-agentic-architecture/
- https://dev.to/pwd9000/copilot-can-now-approve-pull-requests-should-it-count-toward-your-branch-protection-2b78

### 3.9 Anthropic: claude-code-security-review, code-review plugin, Claude Code Review

**claude-code-security-review** (GitHub Action, open source):
- Phases: context research, then comparative analysis, then vulnerability assessment.
- One sub-task finds candidates. Then one parallel sub-task per finding runs a false-positive
  filter and scores confidence 1-10; anything under 8 is dropped.
- It applies a hard exclusion list first (DOS, rate limiting, and others), then Claude filtering.
- Tools are read-only: git commands plus Read, Glob and Grep.
- It caches results so it can run on every commit.
- It is not hardened against prompt injection in the PR.
- Issue #139: a hard-coded retired model silently disabled the filter. Pin and health-check the
  model, and fail closed.

**code-review plugin:**
- It skips closed, draft, trivial and already-reviewed PRs.
- 4-5 parallel agents: two for CLAUDE.md compliance, one for bugs, one for git blame/history, and
  one for previous PR comments.
- Each issue is scored 0-100 and kept only at 80 or above, with each issue re-verified
  independently.
- Models are routed by task: a small model for prechecks, a mid model for compliance, and a large
  model for bugs.

**Claude Code Review** (managed product):
- A multi-agent run plus a verification step against actual code behaviour.
- Severities: Important, Nit, Pre-existing.
- It never approves or blocks. It posts a neutral check run with machine-readable severity counts,
  and you gate on those counts in your own CI.
- REVIEW.md patterns worth copying:
  - a verification bar: any behaviour claim needs a file:line citation
  - re-review convergence: suppress new nits after the first review
  - a nit cap
  - skip rules for machine-authored branches
- Numbers: about 20 minutes and $15-25 per review; under 1% of findings marked incorrect; 54% of
  PRs get substantive comments.

Sources:
- https://github.com/anthropics/claude-code-security-review
- https://github.com/anthropics/claude-code/blob/main/plugins/code-review/README.md
- https://code.claude.com/docs/en/code-review

### 3.10 Ellipsis

- Parallel comment generators run on mixed models, and each comment must carry Evidence (code
  links).
- A multistage filter follows:
  - a confidence threshold, tuned per customer
  - a DedupeFilter
  - a HallucinationFilter / logical-correctness filter that checks the claim against the attached
    evidence
- It learns from thumbs up/down.
- Newer versions review only the commits since the last pass.

Source: https://www.nsbradford.com/blog/how-we-built-ellipsis

### 3.11 Sourcery

- On a new push, it checks existing comments against the new commits and resolves the addressed
  ones.
- A thumbs-down suppresses similar comments. It takes a pattern of downvotes, not one vote.
- A comment that becomes outdated (the code changed) counts as a positive signal.
- Second-hand report: adding a validator raised usefulness from about 40% to about 60%. Cockpit
  maintainers found about 50% noise.

Sources:
- https://docs.sourcery.ai/reviews/anatomy-of-a-review/
- https://docs.sourcery.ai/Code-Review/Teaching-Sourcery/

### 3.12 Cloudflare internal AI code review (built on open-source OpenCode)

This is the best published example of risk tiering with cost numbers. A coordinator sends work to
up to 7 specialist reviewers, depending on tier:

| Tier | When | Agents | Cost per review |
|---|---|---|---|
| trivial | 10 lines or fewer | 2 | $0.20 |
| lite | 100 lines or fewer | 4 | $0.67 |
| full | over 100 lines, or security-sensitive paths | 7+ | $1.68 |

Overall numbers:
- Median cost $0.98; median latency 3m39s; p90 6m27s.
- 131k runs over 48k merge requests, about 2.7 reviews per MR.
- 85.7% prompt-cache hit rate.

Incremental re-review:
- Fixed findings are omitted and their threads auto-resolved.
- Unfixed findings are re-emitted.
- A human "I disagree" triggers re-evaluation.

Noise control:
- Every specialist prompt has a "What NOT to flag" section.
- Lockfiles and generated files are filtered out before review.
- The coordinator verifies with tools when it is unsure.

Verdict:
- Structured severities: critical, warning, suggestion.
- The approval rubric is biased toward approve: a critical finding blocks, and several warnings
  withdraw approval.
- Humans have a "break glass" override, used on 0.6% of MRs.

Reliability:
- circuit breakers, with failback to another model
- timeouts: 5 minutes per task, 25 minutes overall, and 60 seconds of inactivity
- heartbeat logs

Prompt-injection defence: boundary tags are stripped from untrusted PR text.

Source: https://blog.cloudflare.com/ai-code-review/

### 3.13 Uber uReview

The prompt chain runs in four steps:
1. generate
2. grade each comment's confidence, with thresholds per assistant, language and category
3. semantic dedupe
4. a category classifier that suppresses low-value categories

Config and generated files are skipped. Uber uses a golden-comments benchmark (precision, recall,
F1). The best configuration paired a Claude 4 Sonnet generator with an o4-mini grader, i.e. two
different vendors.

Numbers:
- Covers 90% of about 65k diffs per week.
- 75% of comments rated useful; 65% addressed (humans: 51%).
- 2026: multiple generators, about 25k comments per week, 67% addressed, 60% lower cost.

Source: https://www.uber.com/us/en/blog/ureview/

### 3.14 Atlassian RovoDev code reviewer (arXiv 2601.01129)

Two gates were tested:
- An LLM-judge factual-correctness gate (gpt-4o-mini, binary). Ablation showed it had minimal
  impact.
- A ModernBERT actionability classifier trained on 50k comments, labelled by whether the code was
  later changed. It was much more effective, adding 15-20 points.

Results:
- 38.7% of comments resolved, against 44.45% for human reviewers.
- PR cycle time down 30.8%.

Related work: HalluJudge, on detecting hallucinated review comments.

Sources:
- https://arxiv.org/html/2601.01129v2
- https://arxiv.org/pdf/2601.19072

### 3.15 Meta Diff Risk Score (DRS) and RADAR (auto-landing of bot-authored diffs)

This is the closest published analogue to "bees produce PRs and a bot lands them". RADAR gates
bot diffs from the ACE system in three steps:
1. static heuristics, such as a path denylist
2. a DRS percentile threshold: P50 for allowlisted runbooks, P20 otherwise
3. LLM review requiring confidence of 8/10 or more, where any risk signal means reject

Each source must also earn eligibility:
- a 60-day track record with zero incidents and few reverts
- a per-runbook daily cap (10 to 2000 per day)
- a denylist for runbooks that cause incidents
- a landing-delay window during which a human can veto
- deterministic codemods skip per-diff review entirely

Results:
- 535k diffs reviewed, 331k landed (60% approval rate).
- Reverts at 1/3.33 of baseline, incidents at 1/50.
- Time-to-close improved by more than 330%.

DRS-OSS, an open-source version: F1 0.64, AUC 0.89. Gating the riskiest 30% of changes blocks
86.4% of defect-inducing changes.

Sources:
- https://engineering.fb.com/2025/08/06/developer-tools/diff-risk-score-drs-ai-risk-aware-software-development-meta/
- https://arxiv.org/html/2605.30208v1
- https://arxiv.org/html/2410.06351
- https://arxiv.org/html/2511.21964v1

### 3.16 Ona: auto-approving low-risk PRs

A PR counts as low risk only if all six objective rules hold:
- under 1000 lines
- no protobuf changes
- no migrations
- no infra or CI changes
- no auth changes
- no audit-logging changes

The author cannot self-assign the tier. AI reviews every PR, but a human still clicks merge.
Swapping the model requires governance approval.

Results:
- Time to first approval: 2h49m to 3.8 minutes.
- Lead time down 74%.
- Merged PRs up 215%.

Source: https://ona.com/stories/auto-approving-low-risk-prs

## 4. Merge queues, batching, staleness and conflicts

### 4.1 GitHub native merge queue

- Required workflows must also trigger on `merge_group`.
- Checks run on temporary `gh-readonly-queue/*` commits.
- Settings:
  - build concurrency, 1-100
  - minimum and maximum group size, and wait time
  - "require all queue entries to pass"
  - status-check timeout (default about 60 minutes)
- Do not let `cancel-in-progress` kill `merge_group` runs.
- Not available to t27 today, because the owner type is User. Moving the repo to a free
  organization unlocks it.

Source: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue

### 4.2 Trunk Merge Queue

- **Batching:** 20 PRs at batch size 4 cost about 5 CI runs instead of 20.
- **Bisection:** a failed batch is split to find the culprit, and test results are cached during
  bisection.
- **Pending Failure Depth:** a failed PR waits for the PRs behind it before being evicted.
- **Optimistic Merging:** if batch ABCD fails but a later batch EFGH that contains it passes,
  ABCD is merged anyway.
- **Anti-flake protection.**

Sources:
- https://docs.trunk.io/merge/batching
- https://docs.trunk.io/merge-queue/concepts-and-optimizations/anti-flake-protection

### 4.3 Mergify

- `max_parallel_checks` (default 5) runs cumulative speculative checks.
- `batch_size` and `batch_max_wait_time` control batching. A failing batch is split to find the
  culprit.
- Two-step CI: `queue_conditions` gate entry and `merge_conditions` gate the merge, and expensive
  jobs run only on `mergify/merge-queue/` branches.
- Scopes split queues by file pattern.

Sources:
- https://docs.mergify.com/merge-queue/speculative-checks/
- https://docs.mergify.com/merge-queue/batches/
- https://docs.mergify.com/merge-queue/two-step/
- https://docs.mergify.com/merge-queue/scopes/

### 4.4 Aviator MergeQueue

- Parallel mode runs checks on draft bot PRs.
- Affected targets give independent queues to changes that do not overlap.
- `auto_update`: when the base advances, PRs are rebased or merged automatically, optionally only
  those with a label.
- Auto-requeue.
- A "Ready" hook (`ready.js`) updates a PR only when it is more than N commits behind.
- A stuck-PR timeout.
- A documented real deadlock came from stale label state. Labels used as gates need
  reconciliation.

Sources:
- https://docs.aviator.co/mergequeue/affected-targets
- https://docs.aviator.co/mergequeue/concepts/reducing-queue-failures-due-to-staleness
- https://docs.aviator.co/mergequeue/reference/complete-reference-guide

### 4.5 BulkPR-Bench (arXiv 2608.02685): a queue of interacting PRs is a separate problem

Governing a queue of interacting PRs (581 PRs across 18 repos) is harder than reviewing each PR
alone:
- The best relational delivery score was 66.6%, against a 53.1% sequential baseline.
- Only 8 of 324 runs got a whole queue right.

Lesson: the queue logic should be deterministic (rebase, rerun checks, bisect). Do not ask an LLM
to reason about how PRs interact.

Source: https://arxiv.org/abs/2608.02685

## 5. Research on LLM reviewer reliability

### 5.1 Correlated errors: two models agreeing is not two independent votes

- "Great Models Think Alike": LLM judges favour models similar to themselves, and model errors
  become more correlated as capability grows.
  https://arxiv.org/abs/2502.04313
- Kim et al.: models from the same provider have significantly higher error correlation. When two
  such models are both wrong, they often give the same wrong answer (about 60%).
  https://arxiv.org/abs/2506.07962
- Implication for t27: GLM-4.7-flash and GLM-4.5-flash are the same vendor and likely share
  training data. Their agreement is a weak independence signal. Uber's best configuration used
  generator and grader models from different vendors.

### 5.2 Fabricated tool use and claimed-but-unrun verification

- **Fabricated tool use** (arXiv 2604.16706): up to 37.5% of agent traces fabricate tool results
  (Gemini-2.0-Flash; 12% for 4o-mini). A runtime interceptor with three layers cut hallucination
  by up to 24 points:
  1. a tool-error counter
  2. uncertainty keywords in the reasoning
  3. numeric claims checked against what the tools actually returned

  LLM judges and substring heuristics failed at this (kappa 0.049).
  https://arxiv.org/html/2604.16706
- **AI contribution rules** (arXiv 2607.26819): agents claim "all tests pass" with no matching
  command in their log. GPT-5.3-Codex verified in only 4% of runs. The paper's rule is that
  machine-observed state outranks prose. One feedback message naming the omission raised
  verification to 100% (27/27).
  https://arxiv.org/html/2607.26819v1
- **Fabrication after tool failure and failure-transparent agents:** agents keep reporting
  success after a tool fails. A second-hand figure: 75.8% of failing runs still end with "Done",
  and LLM judges detect this at an AUROC of only 0.54-0.65.
  https://arxiv.org/pdf/2609.14758
  https://arxiv.org/html/2609.35732v1

### 5.3 LLM-as-judge for patches is lenient or noisy; execution-based checks anchor it

- Patch evaluation with a human in the loop: recall 0.93, precision 0.65. Judges approve too much.
  https://arxiv.org/html/2511.10865
- PATCHEVAL: LLM judges agree with dynamic validation 76-81% of the time.
  https://arxiv.org/pdf/2511.11019
- Overcorrection in LLM review.
  https://arxiv.org/html/2603.00539
- Sage.
  https://arxiv.org/abs/2512.16041
- Agentic Rubrics: a rubric agent explores the repo first and writes codebase-specific criteria,
  then grades against them. This maps to t27's per-issue acceptance criteria.
  https://arxiv.org/pdf/2601.04171
- R2E-Gym hybrid verifiers: run an execution-based filter first, then score without execution.
  Under 20% of generated tests actually discriminate between correct and incorrect patches, and
  up to 10% are "toxic".
  https://arxiv.org/html/2504.07164

### 5.4 Review benchmarks, for building a local eval set

- c-CRAB turns review comments into executable tests: https://arxiv.org/html/2603.23448v2
- CR-Bench: https://arxiv.org/pdf/2603.11078
- SWR-Bench: https://arxiv.org/pdf/2509.01494
- OpenCodeReview, "Determinism over Non-Determinism": https://arxiv.org/pdf/2608.09290
- CRJudgeBench: https://arxiv.org/html/2609.37216

## 6. Throughput arithmetic for the current setup (illustrative)

This section is illustrative and assumes a serial runner.
- With about 3 minutes per review and two models per approval, one approval costs about 6 model
  minutes.
- A 10-minute launchd tick can then finish at most 1-2 approvals per tick, or roughly 10-15 an
  hour at best.
- That is before re-reviews of PRs that have not changed.

The cheapest throughput gains therefore come from:
1. not reviewing the same (head, base) twice
2. running reviews in parallel, up to the free-tier rate limit
3. putting trivial or low-risk PRs into a cheap lane
4. fixing the about 23 of 49 PRs blocked on conflicts or red required checks, which is
   mechanical

Smarter models are not on this list.

## 7. RANKED TOP 10 mechanisms for t27

Constraints: free models only, no paid SaaS, self-hosted on one Mac plus GitHub Actions.
Effort: S = under a day, M = a few days, L = a week or more.

1. **Runner-executed evidence ledger.**
   - What: the harness, not the model, runs every command that matters: the required checks'
     commands, a test run, `git diff --stat`, and greps for the criteria. Each output is stored
     under an id such as `E3`. The verdict may only cite evidence ids. The runner rejects any
     verdict that states command output, test results or "passes" without a valid evidence id, or
     whose quoted numbers do not appear in that evidence.
   - Who:
     - OpenAI Codex, trained to cite terminal logs: https://alignment.openai.com/scaling-code-verification/
     - Codex Security, reproduce-to-validate: https://openai.com/index/codex-security-now-in-research-preview/
     - runtime interceptor: https://arxiv.org/html/2604.16706
     - "machine-observed state outranks prose": https://arxiv.org/html/2607.26819v1
   - Effect: removes W1, the fabricated-output class, structurally. It also shortens reviews,
     because the model no longer spends turns exploring with tools.
   - Effort: S-M.

2. **Per-criterion verdict with mandatory citations, checked mechanically, failing closed.**
   - What: each acceptance criterion gets a status of met, unmet, partial, or needs-human-run,
     plus a file:line reference or an evidence id. The runner checks that each cited line exists
     in the diff or the file and that each evidence id exists. A criterion with no citation counts
     as unmet. Parse errors, timeouts and model errors all mean "no approval".
   - Who:
     - Claude Code Review's "claims need file:line" bar: https://code.claude.com/docs/en/code-review
     - PR-Agent ticket compliance levels: https://docs.pr-agent.ai/tools/review/
     - Ellipsis evidence-checked hallucination filter: https://www.nsbradford.com/blog/how-we-built-ellipsis
     - fail-open bugs to avoid: https://github.com/The-PR-Agent/pr-agent/pull/3833 and https://github.com/anthropics/claude-code-security-review
   - Effect: fixes W2, sloppy reasoning about criteria, and cuts false approvals.
   - Effort: S.

3. **Deterministic risk tiers that decide review depth and auto-merge eligibility.**
   - What: code, not the model, puts each PR into a tier using these signals:
     - paths: `.github/`, `bootstrap/`, `gen/`, seals and frozen hashes, `tools/bees/` and the
       reviewer itself, CI and infra
     - size, in lines and in files
     - whether the PR touches the review policy

     A trivial or low tier gets one cheap pass and may auto-merge. Any high-risk path goes to a
     human. Prompts and criteria are always loaded from master, never from the PR head.
   - Who:
     - Ona's six objective rules: https://ona.com/stories/auto-approving-low-risk-prs
     - Meta RADAR path denylist plus DRS threshold: https://arxiv.org/html/2605.30208v1
     - Cloudflare trivial/lite/full tiers: https://blog.cloudflare.com/ai-code-review/
     - Cursor's base-branch policy rule: https://cursor.com/docs/approval-agents
   - Effect: most bee PRs (median 175 lines, 2 files) go into a cheap lane, and the riskiest go to
     a human. Ona reports merged PRs up 215%.
   - Effort: S.

4. **Freshness and conflict bot.**
   - What: on each tick, a deterministic job (no LLM) handles stale and conflicting PRs:
     - Update branches that are behind master, through the REST update-branch endpoint or a
       rebase. The `allow_update_branch` setting only controls the UI button; verify this.
     - Rerun required checks.
     - Classify each red check as "also red on master" (not the PR's fault) or "PR-caused".
     - For a conflicting PR, send the conflicting files back to the bee that opened it.
     - Close or recycle PRs stuck longer than N days.
   - Who:
     - Aviator auto_update, Ready hook and stuck timeout: https://docs.aviator.co/mergequeue/concepts/reducing-queue-failures-due-to-staleness
     - Mergify: https://docs.mergify.com/merge-queue/two-step/
   - Effect: directly unblocks the about 23 of 49 open PRs currently blocked (13 conflicting, 10
     with red required checks).
   - Effort: S.

5. **Batch-and-bisect merge train.**
   - What: take the approved PRs, build one merge commit of N of them on a temporary branch, and
     run the required checks once. If green, merge them all. If red, bisect. Add optimistic
     merging. The alternative is to move the repo to a free organization and use GitHub's native
     queue with `merge_group`.
   - Who:
     - Trunk: https://docs.trunk.io/merge/batching
     - Mergify: https://docs.mergify.com/merge-queue/batches/
     - Graphite: https://graphite.com/blog/the-first-stack-aware-merge-queue
     - GitHub native merge queue: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue
   - Effect: CI cost about N/batch, merges no longer serialize, and "stale against master" goes
     away at merge time.
   - Effort: M (self-built) or S (org move).

6. **Make the second vote actually independent.**
   - What: replace the GLM+GLM pair with models from different families. One option is GLM plus a
     local open-weights coder model on the Mac (via Ollama or MLX); another is a second free
     provider. Better still, make the second vote deterministic, such as the evidence-ledger check
     from #1. Randomize the diff and criteria order between the two passes.
   - Who:
     - same-vendor correlated errors: https://arxiv.org/abs/2506.07962 and https://arxiv.org/abs/2502.04313
     - Uber's cross-vendor generator and grader: https://www.uber.com/us/en/blog/ureview/
     - BugBot's randomized-order majority vote: https://cursor.com/blog/building-bugbot
   - Effect: fewer correlated false approvals (W3).
   - Effort: S-M.

7. **Independent per-claim verification pass.**
   - What: split the verdict into atomic claims. Re-check each claim in a separate small call that
     sees only the claim and the evidence it cites. Keep a claim only at confidence 8/10 or more;
     otherwise send it to a human.
   - Who:
     - Anthropic security review: https://github.com/anthropics/claude-code-security-review
     - Claude code-review plugin, threshold 80: https://github.com/anthropics/claude-code/blob/main/plugins/code-review/README.md
     - Qodo judge: https://docs.qodo.ai/code-review
     - CodeRabbit judge: https://theaiengineer.substack.com/p/how-coderabbit-actually-works
   - Effect: higher precision on approvals (W2, W3).
   - Effort: M.

8. **Incremental review, a verdict cache and skip rules.**
   - What: key each verdict on head SHA + base SHA + policy hash. Never re-review an unchanged key.
     After a push, review only the delta and re-check the open findings. Skip drafts, conflicting
     PRs, and PRs whose required checks are red. Run reviews in parallel up to the rate limit.
   - Who:
     - CodeRabbit: https://theaiengineer.substack.com/p/how-coderabbit-actually-works
     - Cloudflare: https://blog.cloudflare.com/ai-code-review/
     - Ellipsis: https://www.nsbradford.com/blog/how-we-built-ellipsis
     - Claude plugin: https://github.com/anthropics/claude-code/blob/main/plugins/code-review/README.md
   - Effect: frees most review minutes and fixes W4.
   - Effort: S.

9. **Producer-side throttling and a trust ledger per bee.**
   - What: per bee, cap open PRs (WIP) and daily output, and set a size cap (for example 200
     lines). Track each bee's record (reverts, master breakages, human overrides); good records
     earn light review, and repeat offenders are denylisted. Add a landing-delay window during
     which a human can veto.
   - Who:
     - Meta RADAR: https://arxiv.org/html/2605.30208v1
     - Graphite PR-size data: https://graphite.com/blog/the-ideal-pr-is-50-lines-long
   - Effect: matches PR inflow to review capacity (W6).
   - Effort: S-M.

10. **Measure outcomes and learn from them.**
   - What:
     - Log every verdict alongside its outcome: reverted, master red after merge, or a human
       override.
     - Build a golden set from past t27 PRs and gate every prompt or model change on it.
     - Later, add an embedding-based filter that learns from human disagreements.
   - Who:
     - BugBot resolution rate and BugBench: https://cursor.com/blog/building-bugbot
     - Uber golden comments: https://www.uber.com/us/en/blog/ureview/
     - Greptile embedding filter: https://www.greptile.com/blog/make-llms-shut-up
     - RovoDev classifier: https://arxiv.org/html/2601.01129v2
   - Effect: correctness becomes measurable, and model swaps become safe.
   - Effort: M.

Also worth adopting cheaply:
- Cloudflare-style timeouts, circuit breakers with model failback, and heartbeat logs
  (https://blog.cloudflare.com/ai-code-review/).
- Strip prompt-injection boundary tags from PR text, and treat the PR body and issue text as
  untrusted data.
- Copilot dismisses its approval when new commits land. `dismiss_stale_reviews_on_push: true` is
  already set for t27.

## 8. Things NOT to copy

- **An LLM rating its own severity or confidence as the only gate.** Greptile found it nearly
  random (https://www.greptile.com/blog/make-llms-shut-up). Atlassian found an LLM factual gate
  had minimal effect (https://arxiv.org/html/2601.01129v2). Use a separate verification call
  against evidence, or votes, or learned filters.
- **Asking an LLM to reason about merge order or how PRs interact** (BulkPR-Bench:
  https://arxiv.org/abs/2608.02685). Keep the queue deterministic.
- **Fail-open defaults on judge errors** (PR-Agent #3833; security-review #139).
- **Reading review policy from the PR head** (Cursor's base-branch rule).
- **Treating "the model says tests pass" as evidence**
  (https://arxiv.org/html/2607.26819v1).

## 9. Source list

Products:
- https://cursor.com/blog/building-bugbot
- https://cursor.com/docs/approval-agents
- https://alignment.openai.com/scaling-code-verification/
- https://openai.com/index/introducing-upgrades-to-codex/
- https://openai.com/index/codex-security-now-in-research-preview/
- https://theaiengineer.substack.com/p/how-coderabbit-actually-works
- https://www.coderabbit.ai/blog/how-coderabbits-agentic-code-validation-helps-with-code-reviews
- https://graphite.com/blog/graphite-reviewer-launch
- https://graphite.com/guides/ai-code-review-false-positives
- https://diamond.graphite.dev/
- https://graphite.com/blog/the-first-stack-aware-merge-queue
- https://graphite.com/docs/graphite-merge-queue
- https://graphite.com/docs/get-started-merge-queue
- https://graphite.com/blog/the-ideal-pr-is-50-lines-long
- https://graphite.com/research/median-pr_size
- https://graphite.com/research/submits-by-pr-size
- https://macroscope.com/content/best-ai-code-review-tools-github-2026
- https://www.greptile.com/blog/make-llms-shut-up
- https://www.greptile.com/docs/code-review/training-the-learning-system
- https://www.greptile.com/docs/code-review/first-pr-review
- https://www.greptile.com/blog/greptile-v3-agentic-code-review
- https://docs.pr-agent.ai/tools/review/
- https://docs.qodo.ai/code-review
- https://github.com/The-PR-Agent/pr-agent/pull/3833
- https://docs.github.com/copilot/using-github-copilot/code-review/using-copilot-code-review
- https://github.blog/changelog/2025-10-28-new-public-preview-features-in-copilot-code-review-ai-reviews-that-see-the-full-picture/
- https://github.blog/changelog/2025-11-20-linter-integration-with-copilot-code-review-now-in-public-preview/
- https://github.blog/changelog/2026-03-05-copilot-code-review-now-runs-on-an-agentic-architecture/
- https://dev.to/pwd9000/copilot-can-now-approve-pull-requests-should-it-count-toward-your-branch-protection-2b78
- https://github.com/anthropics/claude-code-security-review
- https://github.com/anthropics/claude-code/blob/main/plugins/code-review/README.md
- https://code.claude.com/docs/en/code-review
- https://www.nsbradford.com/blog/how-we-built-ellipsis
- https://docs.sourcery.ai/reviews/anatomy-of-a-review/
- https://docs.sourcery.ai/Code-Review/Teaching-Sourcery/
- https://blog.cloudflare.com/ai-code-review/
- https://www.uber.com/us/en/blog/ureview/
- https://engineering.fb.com/2025/08/06/developer-tools/diff-risk-score-drs-ai-risk-aware-software-development-meta/
- https://ona.com/stories/auto-approving-low-risk-prs

Merge queues:
- https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue
- https://docs.trunk.io/merge/batching
- https://docs.trunk.io/merge-queue/concepts-and-optimizations/anti-flake-protection
- https://docs.mergify.com/merge-queue/speculative-checks/
- https://docs.mergify.com/merge-queue/batches/
- https://docs.mergify.com/merge-queue/two-step/
- https://docs.mergify.com/merge-queue/scopes/
- https://docs.aviator.co/mergequeue/affected-targets
- https://docs.aviator.co/mergequeue/concepts/reducing-queue-failures-due-to-staleness
- https://docs.aviator.co/mergequeue/reference/complete-reference-guide
- https://github.com/evgunter/cad/pull/1887
- https://github.com/ned2/dashpot/issues/250

Research:
- https://arxiv.org/abs/2502.04313
- https://arxiv.org/abs/2506.07962
- https://arxiv.org/html/2604.16706
- https://arxiv.org/html/2607.26819v1
- https://arxiv.org/pdf/2609.14758
- https://arxiv.org/html/2609.35732v1
- https://arxiv.org/html/2511.10865
- https://arxiv.org/pdf/2511.11019
- https://arxiv.org/html/2603.00539
- https://arxiv.org/abs/2512.16041
- https://arxiv.org/pdf/2601.04171
- https://arxiv.org/html/2504.07164
- https://arxiv.org/html/2603.23448v2
- https://arxiv.org/pdf/2603.11078
- https://arxiv.org/pdf/2509.01494
- https://arxiv.org/pdf/2608.09290
- https://arxiv.org/html/2609.37216
- https://arxiv.org/html/2601.01129v2
- https://arxiv.org/pdf/2601.19072
- https://arxiv.org/html/2605.30208v1
- https://arxiv.org/html/2410.06351
- https://arxiv.org/html/2511.21964v1
- https://arxiv.org/abs/2608.02685
- https://arxiv.org/html/2607.03316v2
