# Conflicted type names, classified

`tri types dup` reports every type name in `specs/` that has more than one
definition, and `tri types ratchet` refuses to let the count rise. What neither
of them could say is **which kind of conflict each name is** -- and the two
kinds want opposite repairs:

  * **DRIFT** -- one concept that grew a second definition. Two spellings of a
    thing that is meant to be one thing. The repair is to converge them, and
    until that happens any lowering that enumerates types has to pick one and
    silently be wrong about the other.
  * **DISTINCT** -- two concepts that collided on a name. Nothing is broken
    except the namespace. The repair is to rename, or to accept the collision
    as a module-scoped fact and record that it was judged.

Every one of the 80 names was read. Not sampled: opened, both definitions
compared field by field, and decided with the reading written down.

    DRIFT     46
    DISTINCT  34

The per-name evidence is `docs/reports/type_conflicts_classified.json`. This
file is the summary; that file is the record.

## Why this is not a style complaint

A conflicted name is a **fork in the meaning of a program**, and the compiler
does not see it. `specs/` has no cross-module type identity: two modules may
each define `AgentState`, and both are correct in their own file. The moment
anything lowers across modules -- a codegen that emits one struct per type
name, a registry, a serialization boundary -- the two definitions become one
name with two layouts, and the reading depends on which file the lowering
happened to see first.

That is a green exit that is not a result: the build succeeds, the output is
wrong, and nothing red ever appears.

## Staleness

This classification is a reading of the tree taken on 2026-08-29. It goes out
of date the way every reading does: a name gets converged, a name gets added, a
definition moves. `tri types classified` cross-checks the file against a live
`tri types dup` and reports both directions --

    classified but no longer conflicting   -> the repair landed; drop the row
    conflicting but not classified         -> new conflict, unjudged

Non-empty drift in either direction exits non-zero. A classification nobody
re-reads becomes a claim about a tree that no longer exists.

## DRIFT -- 46 names

One concept, two definitions. These are the ones with a repair.

| Name | Defs | Where | Suggested repair |
|------|------|-------|------------------|
| `ActivationType` | 2 | 2 files | Hoist one ActivationType into specs/ml/activation/ covering the 10 shipped activations, and d... |
| `AdamWConfig` | 2 | ml/optimizer/adamw.t27 | Delete the line 28 block: PhiVariant::Damped already encodes `use_phi_betas: true`, so the ap... |
| `AttentionOutput` | 2 | 2 files | Pick the rank (`[][]f32` is the defensible one -- one row per head) and define AttentionOutpu... |
| `BenchmarkReport` | 2 | 2 files | Keep eval.t27's as the owner (benchmark.t27 already imports it), fold in pass_at_5/synth_rate... |
| `DataSample` | 2 | 2 files | Rename training.t27's record to TrainingSample (it carries strategy/weight/sacred_tags — corp... |
| `EvalResult` | 2 | 2 files | Rename the harness record LangEvalResult (it is keyed by language and already carries an aggr... |
| `FFNConfig` | 2 | 2 files | Delete feed_forward_network.t27's FFNConfig and have it `use` feed_forward.t27, or fold the t... |
| `FileInfo` | 3 | 3 files | Merge specs/tri/io/filesystem.t27 and specs/tri/io/fs.t27 — pick one timestamp type (u64 or I... |
| `Graph` | 2 | 2 files | Have graph_bfs.t27 `use` tri::graph::graph and drop its local Graph; separately, teach the re... |
| `HttpRequest` | 2 | 2 files | Extract one http-types module (HttpMethod, HttpHeader/HttpHeaders, HttpStatus) and have both ... |
| `HttpResponse` | 3 | 3 files | Delete router.t27's stub and import server/http.t27's HttpResponse; then reconcile adapters.t... |
| `HttpStatus` | 2 | 2 files | Pick `[]const u8` and have server/http.t27 import tri::net::http::HttpStatus. This is the che... |
| `HybridBigInt` | 2 | 2 files | Highest-value fix in this slice. Choose one representation (the Option-cache + dirty version ... |
| `Hypervector` | 2 | 2 files | Qualify the type in the contract doc (`hybrid_arithmetic::HybridBigInt`) or regenerate that s... |
| `JitCache` | 2 | 2 files | Decide which document is normative for the JIT (jit_semantics.t27 calls itself "JIT Compilati... |
| `JitCompiler` | 2 | 2 files | Reconcile with JitCache in the same pass: whether the code buffer is a fixed [65536]u8 or a h... |
| `LSTMWeights` | 2 | 2 files | Pick one parameterisation (split W_ii/W_hi is the interoperable one) and delete the other fil... |
| `LogEntry` | 2 | 2 files | Merge tri/utils/logger.t27 and tri/utils/logging.t27 into one module — the function sets are ... |
| `MHAConfig` | 2 | 2 files | Delete the stub specs/ml/transformer/multi_head_attention.t27 (module MultiHeadAttn) and keep... |
| `Match` | 2 | 2 files | Collapse Match/MatchResult/RegexMatch onto regex_advanced.t27's RegexMatch (it is already the... |
| `MemPort` | 2 | 2 files | Make fpga/hir.t27 import Memory's MemPort rather than restate it; reconcile MAX_MEM_PORTS (4 ... |
| `Message` | 3 | 3 files | Pick one Message and one MessageRole (with fixed discriminants) in provider/schema.t27; have ... |
| `OptimizerStepResult` | 3 | 2 files | Two fixes: (1) delete the second body of adamw.t27 (lines ~460-end duplicate lines ~14-459) o... |
| `PinAssignment` | 3 | 3 files | Hoist one PinAssignment (the 9-field version) into a shared boards module and have all three ... |
| `PinMapping` | 2 | 2 files | Either give each board its own name (ArtyA7PinMapping / QMTechA100TPinMapping) or use the spe... |
| `PolicyOutput` | 2 | 2 files | Rename to PPOPolicyOutput / SACPolicyOutput, matching the SACActorConfig convention already i... |
| `Port` | 3 | 3 files | Have igla/coder import fpga::hir::Port (or at least PortDir) instead of restating it with str... |
| `ProcessInfo` | 2 | 2 files | Unify on one ProcessInfo with an i32 exit code and one status enum that keeps both zombie and... |
| `ProviderConfig` | 2 | 2 files | One ProviderConfig in provider/schema.t27, imported by config/schema.t27. Fix the timeout uni... |
| `Rect` | 2 | 2 files | Pick one convention for specs/tri/trees/ (min/max is the usual choice for R-tree union/inters... |
| `Route` | 3 | 3 files | Reconcile RouteMethod and HttpMethod (fix the DELETE/PATCH discriminants) and keep one Route ... |
| `SacredConstants` | 2 | 2 files | Delete the 21-line stub in specs/sacred/sacred_constants.t27 or rename it (e.g. SacredConstan... |
| `SacredRule` | 2 | 2 files | Pick one governance spec as the owner of SacredRule (sacred_governance.t27 has the richer rul... |
| `SearchResult` | 4 | 4 files | Have specs/vsa/similarity_search.t27 use `vsa::core::SearchResult` and decide once whether th... |
| `Session` | 3 | 3 files | Extract the sandbox Session (plus Timestamp and SessionStatus) into one module both sandbox s... |
| `Signal` | 2 | 2 files | Decide whether RACE emits through the Trinity HIR. If yes, delete rtl.t27's Signal/Assignment... |
| `SystemConfig` | 2 | 2 files | Make one board-integration template with the full SystemConfig and let each board supply valu... |
| `Task` | 4 | 4 files | Give specs/tri/agent/ one Task + TaskStatus module that both lifecycle and swarm import; rena... |
| `TernaryWeight` | 2 | 2 files | Rename the training-side type QuantizedTernaryWeight (or move `scale` to a per-tensor descrip... |
| `TernaryWord` | 2 | 2 files | Resolve #2275 by naming the two shapes apart (TernaryWordCells for the memory view, PackedTer... |
| `ToolCall` | 3 | 3 files | Declare ToolCall once (tools/schema.t27's branded version is the most complete) and have prov... |
| `ToolResult` | 2 | 2 files | Have agent-runner.t27 import tools/schema.t27's ToolResult and drop its four-field copy; if t... |
| `TrainingConfig` | 2 | 2 files | Fold the pilot config into the staged one (model_size/seq_len/vocab_size become fields or a c... |
| `UnpackResult` | 2 | 2 files | Rename the scalar one UnpackTritResult (or make the buffer one UnpackBufferResult) and, separ... |
| `Url` | 2 | 2 files | Delete the Url declaration in specs/tri/net/http.t27 and import TriUrl; then decide once whet... |
| `Usage` | 2 | 2 files | Normalize on the provider abstraction's Usage and have server/api.t27 import it, or add the m... |

### The one that is fixable today with no cross-module decision

`AdamWConfig` has **both definitions in one file**,
`specs/ml/optimizer/adamw.t27` (lines 28 and 483). Six of seven fields are
identical; the seventh is `use_phi_betas: bool` widened to
`phi_variant: PhiVariant`. The file's own comment says the second was
"appended". There is no other module to negotiate with -- this is a single file
that defines the same config twice, and the later definition is the newer
design.

Every other DRIFT row needs a decision about which module owns the concept.
This one needs an edit.

## DISTINCT -- 34 names

Two concepts that met on a name. Nothing to converge; the question is only
whether to rename.

| Name | Defs | Where | Suggested repair |
|------|------|-------|------------------|
| `Agent` | 2 | 2 files |  |
| `AgentState` | 2 | 2 files | Rename to RLAgentState and AgentRunnerState; nothing outside each file depends on the bare name. |
| `AgentStatus` | 2 | 2 files |  |
| `AttentionConfig` | 2 | 2 files | Rename arch.t27's to CoderAttentionConfig (or GqaConfig); it is model-specific and has no lib... |
| `BenchmarkResult` | 2 | 2 files | Rename the training one to QuantizationBenchmarkResult -- it is the smaller blast radius (two... |
| `BusPort` | 2 | 2 files | Rename axi4.t27's to BusSignal (it is one wire) and reconcile the two MAX_BUS_PORTS values --... |
| `Color` | 3 | 3 files | Rename red_black_tree's to NodeColor and terminal's to AnsiColor, leaving utils/color.t27 the... |
| `CompileResult` | 2 | 2 files | Rename eval.t27's to SandboxCompileResult; it is used by exactly one function. |
| `Config` | 3 | 3 files | Rename the narrow two (MonitorConfig, ParsedConfig -- the third is really a parse result, not... |
| `Diagnostic` | 2 | 2 files | Leave both, but rename the protocol one LspDiagnostic (or require the qualified `lsp-schema::... |
| `EnvVar` | 2 | 2 files | Two fixes, unrelated: (a) leave the types alone, they are genuinely different; (b) fix the fi... |
| `HealthStatus` | 2 | 2 files | Rename railway_deploy's to HealthProbe -- it is a probe result, not a status -- or accept as ... |
| `Info` | 3 | 3 files | Two things: have account/repo.t27 `use account::schema` instead of re-declaring Info and the ... |
| `Instance` | 3 | 3 files | Leave the three types; the ambiguity is in the name. If cross-spec resolution matters, qualif... |
| `KnowledgeGraph` | 3 | 3 files | Delete specs/igla/coder/_tmp_pipeline_import.t27 — it is a leaked working copy, and removing ... |
| `Lexer` | 2 | 2 files | None on the types. If the census needs a single answer for `Lexer`, qualify at use sites — bu... |
| `LinkResult` | 2 | 2 files | Leave both; rename the binary one ImageLayout or LinkImage, which is what its fields actually... |
| `MigrationStep` | 2 | 2 files | Rename to ConfigFieldMigration and StorageMigration; the shared word buys nothing since neith... |
| `Node` | 2 | 2 files | Leave both; if the type namespace is ever flattened, rename the cache one to LruEntry. |
| `ParseError` | 2 | 2 files | Delete specs/tri/pipeline/codegen.t27 or give it a real body; nothing consumes its ParseError. |
| `ParseResult` | 2 | 2 files | Rename the CLI one to ArgsParseResult if the namespace is ever flattened; no defect today. |
| `Parser` | 2 | 2 files | None needed; if flattened, PinsParser is the natural rename for the pins one (it is already t... |
| `PipelineConfig` | 3 | 3 files | Delete specs/igla/coder/_tmp_pipeline_import.t27 — it is a temp import artifact that duplicat... |
| `PipelineResult` | 5 | 5 files | Rename per subsystem (FusionResult / CompilePipelineResult / GenerationResult / BatchEntryRes... |
| `Promise` | 2 | 2 files | None urgent. Note the report's field list for the async site is wrong — see the pattern note ... |
| `ProofStep` | 3 | 3 files | Rename the math one to DerivationStep (and share the single copy between phi_split_optimality... |
| `QueryResult` | 3 | 3 files | Rename per subsystem (DatalogAnswer / NotebookAnswer / SimilarityHit); no shared meaning to p... |
| `Response` | 3 | 3 files | Rename to JsonRpcResponse / CompletionResponse / MdnsResponse; three protocols in one binary ... |
| `Result` | 2 | 2 files | Rename git's to GitCommandOutput, and decide whether Result<T,E> is a builtin — if it is, com... |
| `Rule` | 2 | 2 files | Rename to DatalogRule and K3Implication — inside one package, an unqualified ar::Rule cannot ... |
| `SimResult` | 2 | 2 files | No defect. If cross-spec resolution is ever attempted, rename the PRM one to TestbenchPassRat... |
| `TaskResult` | 2 | 2 files | No defect in itself, but it inherits the Task ambiguity: rename to ProcessResult / AgentTaskR... |
| `ValidationResult` | 2 | 2 files | No defect to fix today, but the name is unresolvable across specs: rename to ConfigValidation... |
| `VerificationReport` | 2 | 2 files | No defect. If the name must resolve, GoldenFamilyAudit and FormulaConformanceTally describe w... |

## Four verdicts the tool did not earn

`tri types dup` decides CONFLICTED by comparing field lists. For four names --
`Agent`, `AgentStatus`, `Color`, `HealthStatus` -- one side's fields are a list
this reader cannot parse (`variants : ,`, the corpus's enum idiom), so it is
comparing an empty list against a full one and calling that a disagreement.

The verdicts in the tables above are not from that comparison; each of those
four was decided by opening the source. But the tool's own CONFLICTED for those
four is a coincidence of a reader limit, and a coincidence that happens to be
right is still not a measurement. Recorded here rather than left for someone to
rediscover as a bug.

## How this was produced

Seven agents, one per slice of the name list, each required to open both
definitions and quote the fields it compared -- with a separate pass that tried
to refute the verdicts. One refutation lens came back unsound
(coverage/double-counting), and its complaint is recorded rather than quietly
dropped: the count is of NAMES, so a name with three definitions is one row,
and any reading that counts definitions instead will disagree with
`tri types dup`.

The eightieth name, `HealthStatus`, was not in that run. It appeared when the
field reader was taught that `pub name: T` is a field (#2802) -- the same change
that moved the conflicted count 79 -> 80. `tri types classified` found it on its
first execution, which is the shape of thing that command exists for.

Related: `docs/CORPUS-RATCHET.md` (the ratchet that holds the count),
`.claude/skills/ci-gates/SKILL.md` (why a ratchet and not a refusal).
