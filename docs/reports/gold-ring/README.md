# [GOLD-RING] proposal: compound assignment (`-=`, `*=`, `/=`, `%=`)

**Not applied.** `bootstrap/src/compiler.rs` is FROZEN_HASH-enforced
(`build.rs:206`); FROZEN.md requires a [GOLD-RING] PR with Architect approval,
and W780's precedent is verify, document, do not apply. This directory is the
verification and the documentation. The patch includes the matching
`stage0/FROZEN_HASH` update, as the freeze process requires.

## What the grammar lacks, measured

`+=` has been the only compound assignment for the grammar's whole life —
`git log -S'"*="' -- bootstrap/src/compiler.rs` is empty. Minimal repros
(five-line specs, one construct each):

    t += x   PASS          t -= x   FAIL
                           t *= x   FAIL
                           t /= x   FAIL
                           t %= x   FAIL

## The patch, validated on a scratchpad build

Five sites, each mirroring the existing `+=` handling:

| site | change |
|---|---|
| `TokenKind` enum | `MinusEquals`, `StarEquals`, `SlashEquals`, `PercentEquals` |
| lexer (`two ==` chain) | four two-byte cases beside `[b'+', b'=']` |
| parser (`parse_stmt` assign arm) | the `if PlusEquals` becomes a five-way `match` filling `extra_op` |
| Zig + C codegen | `match extra_op` writes the operator through |
| Verilog codegen | every compound desugars to `lhs <= lhs <op> rhs`, exactly as `+=` already did |

Built cleanly; all four repros parse; existing repro suite unaffected.

## Honest blast radius — smaller than the wave that found it assumed

Of the 201 unique stale-seal specs, **165 already parse** with the unpatched
compiler — and (corrected in W882) every one of those 165 also COMPILES and
resealed cleanly: the earlier claim that "most refusals are backend coverage" was
wrong; a probe over the whole stale set found **zero** parseable-but-none specs.
Refusals coincide exactly with parse failures. Against the 36 that fail parse:

    fixed outright by this patch     1    specs/numeric/tf3.t27
    advanced but not through         1    specs/numeric/gf16.t27 (L6 SSOT):
                                          824 -> 3507, where the next gap is a
                                          NESTED fn (fn phi_dist inside a fn)
                                          with an if-expression body
    untouched                       ~34   need nested fns / if-exprs / others

So this patch is necessary for the L6 SSOT and sufficient for one spec.

**The remaining 34 are not gaps — they are DIALECTS** (measured W882, one probe
per failure line): eleven `tri/collections` files open with generic structs
(`pub const Map(K, V) = struct {`); three open with an `algorithm X {` DSL;
several use Rust forms (`let mut x: [T] = []`, `impl X {`, `for i in 1..=10`);
two use namespaced module headers; one uses Zig's `while (..) : (i += 1)`. The
meta compiler sealed them all. Which dialect is canonical is an Architect
decision that no patch series should pre-empt — the nested-fn gap (gf16's next
blocker, one file) is the only further [GOLD-RING]-sized item.

## Reproduction

    cp -R bootstrap <scratch>/ && cd <scratch>/bootstrap
    printf '\n[workspace]\n' >> Cargo.toml        # detach from the parent workspace
    git apply 0001-compound-assignment.patch
    cargo build --release
    ./target/release/t27c parse specs/numeric/tf3.t27   # rc=0, was rc=1

---

# [GOLD-RING] proposal 0002: nested `fn`, hoisted (prototype)

`0002-nested-fn.CUMULATIVE.patch` carries 0001 + 0002 together (they were
validated in one scratchpad copy; the 0002 delta is the three `hoisted_fns`
hunks: the parser field, the `KwFn` interception in `parse_fn_body`, and the
drain beside `module.children.push(decl)`).

A nested `fn` is parsed by the ordinary fn parser and HOISTED to module level;
no statement node is produced, so no backend needs a no-op.

**The capture check is IN the patch** (W884), at the only scope that matters:
hoisting moves the nested fn out of the enclosing body, so the one thing it must
not reach is the enclosing fn's own bindings — parameters and locals declared
before that point. A free name that is NOT such a binding resolves identically
before and after hoisting, so imported constants are none of the check's
business. The first version asked "is it module-level?" and rejected gf16 for an
imported `PHI_INV` — the wrong question, kept in the record. Negative test: a
nested fn using an enclosing `const scale` fails with
`captures enclosing locals ["scale"]`; the module-const control passes.

## Measured result

    gf16.t27 (the L6 SSOT)   parses COMPLETELY and produces REAL gen hashes
                             on all four backends -- sealable for the first
                             time under the repository's own compiler
    tf3.t27                  fixed by 0001 alone
    blast radius             0001+0002 fix exactly the SSOT-family pair;
                             the remaining 34 failures are the three DIALECTS
                             (see DIALECTS.md) and stay untouched by design

---

# [GOLD-RING] proposal 0003: tuple-when lowering + per-clause fallback

`0003-bdd-tuple-when.CUMULATIVE.patch` (carries 0001+0002+0003; the 0003 delta
is two hunks in `parse_bdd_clauses`).

**Change A — tuple patterns in bindings.** `when (a_out, psum) = pe(...)`
lowers exactly as `parse_local_decl`'s tuple path does (StmtLocal, empty name,
comma-joined `extra_field`). This one shape was ~60 % of every dropped when-line.

**Change B — per-clause fallback.** The whole-block fallback made one
unsupported clause lose its siblings; most of the corpus's dropped test lines
were collateral. Now a failed clause restores to its own checkpoint and skips to
the next clause keyword, counting every skipped token into the truncation
ledger. The safety contract holds: only statements are ever ADDED, the skip is
bounded by `is_block_boundary`, and the over-consumption case (a greedy
`parse_expr` swallowing the next clause) keeps the whole-block fallback.

**Measured:**

    systolic_ternary.t27      5,358 -> 1,469 discarded tokens  (-73 %)
    corpus (624 non-scratch)  67,760 -> 58,187                 (-14 %)
    regressions               none: repros, SSOT pair, capture-check all hold

**Next fallback cause, identified:** expressions inside clauses the expression
grammar rejects — e.g. `given results = []EvalResult{}` (an empty typed-array
literal). That is an expression-grammar item, not a lowering item, and is left
for a measured 0004 rather than bundled.

---

# 0004a — braceless `bench` joins the shared clause parser

One-line lowering change: `bench` blocks without braces route through
`parse_bdd_clauses` exactly as `test` and `invariant` do. Corpus:
58,187 → 57,680 discarded tokens. Zero regressions.

# 0005 — the `and` clause never worked, and now it does

ddmin reduced an 80-line "contextual parser-state" repro to FOUR lines, and the
context was one `and` clause. Two mechanisms, both fixed:

1. `and` lexes as the logical-operator token (KwAnd), never Ident — the clause
   keyword the lowering claimed to accept was unreachable for the function's
   whole life; every block OPENING with `and` fell back wholesale.
2. After any successful clause, `parse_expr`'s greedy and-loop consumed the next
   `and` clause as a conjunction, stopped on its `=`, and the whole block fell
   back — every MID-BLOCK `and` clause died this way. In clause-value mode an
   `and` followed by `ident =` now terminates the expression (bounded three-token
   lookahead via the parser's own save/restore); genuine logical `and` in clause
   values still parses as the operator — probed in both directions.

**Recorded revision:** 0003's per-clause skip is withdrawn by its own
regressions — its boundary set stopped at `}` and `fn` inside clause junk (a
lambda in a `then`, struct literals in `given`s) and handed fragments to module
level, which errors HARD where the old whole-block fallback skipped safely.
Four files that parsed before regressed; all four recover with the fallback
restored. The collateral win survives because the and-fix makes most blocks
lower completely.

**Measured (624 non-scratch specs):**

    base    67,760 discarded, 137 files, 173 parse-fails
    0005    42,926 (−37 %),   126 files, 171 parse-fails (−2: the SSOT pair)
    consume-all 314 → 327; zero regressions at every rung

# Instrument — every discard channel records its spans

`parse-complete --show` printed "nothing discarded" for a file the corpus mode
charged 2,438 tokens — same binary, same file. The counter increments in three
places; the span recorder lived in one. Skipped brace bodies and statement-level
recovery counted without recording — the mechanism behind the first inventory
missing 311+ BDD lines in one file (W892). All channels record now; per-file
span totals equal the corpus counter exactly: 42,926 = 42,926 over 126 files.

With the honest instrument, the lost-tests inventory was remeasured line-by-line
under 0005: **21,444 of 23,033 BDD lines are READ (93.1 %); 1,589 remain
dropped; 0 files unparse.** The migrate-vs-teach decision now weighs 1,589
lines, not 4,665.

# 0006 — the array literal ate the next clause's keyword as a "type"

The residual map blamed struct literals; exact-line probes ACQUITTED them.
ddmin found the killer three clauses later: `parse_array_literal` consumed a
following Ident unconditionally as the Zig element type (`[3]u8`), across
newlines — `and params = [1.0]` + `when result = ...` ate `when`. `and`
survived only because KwAnd is not an Ident, which made 0005 hide this bug
behind the struct literal one clause earlier. **Presence is not causality.**

Rule: Ident-after-`]` is a type only same-line (legacy corpus shapes keep
parsing) or when an initialiser brace follows — and in clause-value mode a
clause keyword is never a type. A 72-attempt adversarial panel broke v1 twice
(one-line pairs still eaten; `then {1} == xs` forging the brace test into a
SILENT false-green with the assertion vanishing) — both closed, the forged
probe kept as a negative test.

**Measured:** 42,926 → 37,786 (−44 % from base); parse-fails 171, zero new;
inventory 96.9 % READ (22,329 / 23,033 BDD lines; was 4,665 dropped at W892).

# The residual map correction (W900)

The map's "0006 candidate — struct literal in clause value" dissolved on
probing: both literal syntaxes ALREADY parse. Reader classifications named the
first discarded clause's construct; the CAUSE sat in a later clause whose
array value ate the following keyword. The verifiers confirmed the constructs
exist at the cited lines — presence, not causality; only intervention (ddmin,
variant probes) assigns cause. Remaining true queue: bare-call given clauses
(`given uart_tx_send(0x55)`), `measure`/`target` bench pairs, one-line
`invariant name : EXPR;` — then forall (the ring question, see
FORALL-DECISION.md) and dialect bodies.
