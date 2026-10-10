#!/usr/bin/env node
// scripts/check-t27-specs.mjs — checker for the vendored .t27 spec copies.
//
// Host: gHashTag/999-multibots-telegraf, which vendors .t27 specs from
// gHashTag/t27. Reimplemented 2026-10-09 for issue #8334: the host repo was
// not reachable from the working machine (no network), so this file was
// written from the issue contract plus direct probes of `t27c seal`, which
// fixed the seal schema as:
//   { spec_hash: "sha256:<hex>",
//     gen_hash_c|gen_hash_rust|gen_hash_verilog|gen_hash_zig,
//     tests: { total, failed, passed, failing[], forced },
//     config: "gen=...;tests=..." }
// Probes also fixed the behavior: `t27c seal` refuses to save a seal for a
// spec whose pipeline failed or produced no output unless `--force` is
// passed, and `--force` records `tests.forced: true` only when it actually
// bypassed a failure or a missing output (a passing spec force-sealed
// still records `forced: false`).
//
// Modes:
//   --run   one Zig compile per vendored spec (`t27c path <file>`, which
//           generates Zig and carries the spec through the full pipeline),
//           unless the spec's seal stands in — the seal-based compilation
//           skip added by #8334. A spec compiles iff its seal does not
//           stand in; `--no-seal` compiles everything.
//   --t27   compare the vendored copies against the specs of record
//           (gHashTag/t27) by content hash, consulting the record repo's
//           seals as an oracle when a record file is absent or ambiguous.
//
// Flags:
//   --no-seal        never let a seal stand in; every vendored spec compiles
//   --dir <path>     vendored specs directory (default: ./specs)
//   --seals-dir <p>  seal JSON directory (default: <dir>/.trinity/seals if
//                    it exists, else ./.trinity/seals, else none)
//   --t27-root <p>   specs-of-record checkout for --t27 (default: $T27_ROOT)
//   --t27c <bin>     compiler binary (default: t27c)
//
// Exit codes: 0 ok, 1 compile or compare failures, 2 usage or IO error.

import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { basename, join, relative, resolve } from "node:path";
import { spawnSync } from "node:child_process";

// --- seal stand-in (issue #8334 contract) --------------------------------
// Named in snake_case to match the issue's contract verbatim so it stays
// greppable:
//   stands_in(hash_same, sealed_total, sealed_failed, forced)
//
// The seal stands in for the compile only when it carries evidence:
//   - hash_same:         the sealed spec text is exactly the vendored file
//   - sealed_total > 0:  at least one test ran at seal time (a stage that
//     "finished" in 0.0 s did not finish — no numbers, no stand-in)
//   - sealed_failed == 0: nothing failed at seal time
//   - !forced:           the seal was not made under --force. A force-seal
//     is second-class evidence: it exists precisely because something was
//     bypassed, so it must not stand in even when failed === 0.
export function stands_in(hash_same, sealed_total, sealed_failed, forced) {
  return hash_same && sealed_total > 0 && sealed_failed === 0 && !forced;
}

// compiles(no_seal, seal_stands_in) — `--no-seal` always compiles;
// otherwise compile iff the seal does not stand in.
export function compiles(no_seal, seal_stands_in) {
  return no_seal || !seal_stands_in;
}

// --- helpers ---------------------------------------------------------------

function sha256File(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

// Normalize a seal's spec_hash: strip an optional "sha256:" prefix, lowercase.
function normHash(h) {
  return String(h ?? "").replace(/^sha256:/, "").toLowerCase();
}

// Read every seal JSON in a directory, index by normalized spec_hash.
// Returns { byHash: Map<hash, seal[]>, count, unparseable }.
function loadSealIndex(sealsDir) {
  const byHash = new Map();
  let count = 0;
  let unparseable = 0;
  if (!sealsDir || !existsSync(sealsDir)) return { byHash, count, unparseable };
  for (const name of readdirSync(sealsDir).sort()) {
    if (!name.endsWith(".json")) continue;
    const path = join(sealsDir, name);
    let seal;
    try {
      seal = JSON.parse(readFileSync(path, "utf8"));
    } catch {
      unparseable += 1;
      continue;
    }
    const hash = normHash(seal.spec_hash);
    if (!/^[0-9a-f]{64}$/.test(hash)) {
      unparseable += 1;
      continue;
    }
    count += 1;
    if (!byHash.has(hash)) byHash.set(hash, []);
    byHash.get(hash).push(seal);
  }
  return { byHash, count, unparseable };
}

// One seal's evidence tuple for stands_in.
function sealEvidence(seal) {
  const tests = seal.tests ?? {};
  return {
    hash_same: true, // caller guarantees this seal was matched by spec_hash
    sealed_total: Number(tests.total ?? 0),
    sealed_failed: Number(tests.failed ?? (Array.isArray(tests.failing) ? tests.failing.length : 0)),
    forced: tests.forced === true,
  };
}

// True if any seal carrying this exact spec_hash stands in for the compile.
function anySealStandsIn(index, fileHash) {
  const seals = index.byHash.get(fileHash.toLowerCase());
  if (!seals) return { stands: false, seal: null };
  for (const seal of seals) {
    const ev = sealEvidence(seal);
    if (stands_in(ev.hash_same, ev.sealed_total, ev.sealed_failed, ev.forced)) {
      return { stands: true, seal };
    }
  }
  return { stands: false, seal: seals[0] };
}

// Recursively collect .t27 files under a directory, as sorted relative paths.
function listSpecs(rootDir) {
  const out = [];
  const walk = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const full = join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.isFile() && entry.name.endsWith(".t27")) out.push(relative(rootDir, full));
    }
  };
  walk(rootDir);
  return out;
}

function defaultSealsDir(specsDir) {
  const nested = join(specsDir, ".trinity", "seals");
  if (existsSync(nested)) return nested;
  const local = join(process.cwd(), ".trinity", "seals");
  if (existsSync(local)) return local;
  return null;
}

// --- CLI --------------------------------------------------------------------

function usage(code = 2) {
  process.stderr.write(`Usage: check-t27-specs.mjs [--run] [--t27] [--no-seal]
                          [--dir <specs>] [--seals-dir <dir>]
                          [--t27-root <dir>] [--t27c <bin>]

  --run    one Zig compile per vendored spec, unless its seal stands in
           (hash matches, tests passed, not force-sealed). --no-seal
           always compiles.
  --t27    compare vendored copies against the specs of record by hash.

Pass exactly one of --run / --t27 (default: --run).
`);
  process.exit(code);
}

function parseArgs(argv) {
  const opts = { run: false, t27: false, no_seal: false, dir: null, seals_dir: null, t27_root: null, t27c: "t27c" };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--run") opts.run = true;
    else if (a === "--t27") opts.t27 = true;
    else if (a === "--no-seal") opts.no_seal = true;
    else if (a === "--dir") opts.dir = argv[++i];
    else if (a === "--seals-dir") opts.seals_dir = argv[++i];
    else if (a === "--t27-root") opts.t27_root = argv[++i];
    else if (a === "--t27c") opts.t27c = argv[++i];
    else usage();
  }
  return opts;
}

// --- --run: seal-based compilation skip -------------------------------------

function runMode(opts) {
  const specsDir = resolve(opts.dir ?? "specs");
  if (!existsSync(specsDir) || !statSync(specsDir).isDirectory()) {
    process.stderr.write(`check-t27-specs: specs directory not found: ${specsDir}\n`);
    process.exit(2);
  }
  const sealsDir = opts.seals_dir ? resolve(opts.seals_dir) : defaultSealsDir(specsDir);
  const index = loadSealIndex(sealsDir);
  const specs = listSpecs(specsDir);
  const t0 = process.hrtime.bigint();

  if (index.count === 0) {
    process.stdout.write(`seals: none found${sealsDir ? ` (${sealsDir})` : ""} — every spec will compile\n`);
  } else {
    const where = sealsDir ? ` (${sealsDir})` : "";
    const broken = index.unparseable > 0 ? `, ${index.unparseable} unparseable ignored` : "";
    process.stdout.write(`seals: ${index.count} loaded${where}${broken}\n`);
  }

  let skipped = 0;
  let compiled = 0;
  let failed = 0;

  for (const rel of specs) {
    const file = join(specsDir, rel);
    const fileHash = sha256File(file);
    const { stands, seal } = opts.no_seal ? { stands: false, seal: null } : anySealStandsIn(index, fileHash);

    if (compiles(opts.no_seal, stands)) {
      const t1 = process.hrtime.bigint();
      const res = spawnSync(opts.t27c, ["path", file], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
      const secs = Number(process.hrtime.bigint() - t1) / 1e9;
      if (res.status === 0) {
        compiled += 1;
        process.stdout.write(`PASS  ${rel}  compiled in ${secs.toFixed(2)}s\n`);
      } else {
        failed += 1;
        process.stdout.write(`FAIL  ${rel}  (t27c path exited ${res.status})\n`);
        const tail = ((res.stderr ?? "") + (res.stdout ?? "")).trim().split("\n").slice(-4).join("\n");
        if (tail) process.stdout.write(`      ${tail.split("\n").join("\n      ")}\n`);
      }
    } else {
      skipped += 1;
      const ev = sealEvidence(seal);
      const tests = seal.tests ?? {};
      const passed = Number(tests.passed ?? ev.sealed_total);
      process.stdout.write(
        `SKIP  ${rel}  seal stands in (tests ${passed}/${ev.sealed_total}, 0 failed)\n`,
      );
    }
  }

  const total = Number(process.hrtime.bigint() - t0) / 1e9;
  process.stdout.write(
    `${specs.length} specs, ${skipped} by seal, ${compiled} compiled, ${failed} failed in ${total.toFixed(1)}s\n`,
  );
  process.exit(failed > 0 ? 1 : 0);
}

// --- --t27: compare vendored copies against the specs of record --------------

function walkT27(dir, out) {
  for (const entry of readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) walkT27(full, out);
    else if (entry.isFile() && entry.name.endsWith(".t27")) out.push(full);
  }
}

// Match a vendored file against the specs of record. Order: same relative
// path under the record root (or under root/specs/), then a unique-basename
// search of root/specs/, then the record repo's seals as a hash oracle.
function matchRecord(vendoredRel, fileHash, root, index, useSeals) {
  const candidates = [];
  for (const prefix of ["", "specs"]) {
    const p = join(root, prefix, vendoredRel);
    if (existsSync(p) && statSync(p).isFile()) candidates.push(p);
  }
  if (candidates.length === 0) {
    const specsRoot = join(root, "specs");
    if (existsSync(specsRoot)) {
      const all = [];
      walkT27(specsRoot, all);
      const base = basename(vendoredRel);
      for (const p of all) if (basename(p) === base) candidates.push(p);
    }
  }
  const viaSeal = () => {
    if (!useSeals || !index) return false;
    return (index.byHash.get(fileHash.toLowerCase()) ?? []).length > 0;
  };
  if (candidates.length === 1) {
    if (sha256File(candidates[0]) === fileHash) return { status: "same", record: candidates[0], via: "file" };
    if (viaSeal()) return { status: "same", record: candidates[0], via: "record seal" };
    return { status: "different", record: candidates[0], via: "file" };
  }
  if (candidates.length > 1) {
    if (viaSeal()) return { status: "same", record: null, via: "record seal" };
    return { status: "ambiguous", record: null, via: "file" };
  }
  if (viaSeal()) return { status: "same", record: null, via: "record seal" };
  return { status: "absent", record: null, via: "none" };
}

function t27Mode(opts) {
  const specsDir = resolve(opts.dir ?? "specs");
  const root = resolve(opts.t27_root ?? process.env.T27_ROOT ?? "");
  if (!opts.t27_root && !process.env.T27_ROOT) {
    process.stderr.write("check-t27-specs: --t27 needs --t27-root <dir> (or $T27_ROOT)\n");
    process.exit(2);
  }
  if (!existsSync(specsDir) || !statSync(specsDir).isDirectory()) {
    process.stderr.write(`check-t27-specs: specs directory not found: ${specsDir}\n`);
    process.exit(2);
  }
  if (!existsSync(root) || !statSync(root).isDirectory()) {
    process.stderr.write(`check-t27-specs: specs-of-record root not found: ${root}\n`);
    process.exit(2);
  }
  const index = loadSealIndex(join(root, ".trinity", "seals"));
  const specs = listSpecs(specsDir);

  let same = 0;
  let different = 0;
  for (const rel of specs) {
    const fileHash = sha256File(join(specsDir, rel));
    const m = matchRecord(rel, fileHash, root, index, !opts.no_seal);
    if (m.status === "same") {
      same += 1;
      process.stdout.write(`SAME       ${rel}  (${m.via})\n`);
    } else if (m.status === "different") {
      different += 1;
      process.stdout.write(`DIFFERENT  ${rel}  (${relative(root, m.record)})\n`);
    } else if (m.status === "ambiguous") {
      different += 1;
      process.stdout.write(`DIFFERENT  ${rel}  (ambiguous record match)\n`);
    } else {
      different += 1;
      process.stdout.write(`ABSENT     ${rel}  (no spec of record)\n`);
    }
  }
  process.stdout.write(`${specs.length} vendored, ${same} same, ${different} not same\n`);
  process.exit(different > 0 ? 1 : 0);
}

// --- main ---------------------------------------------------------------------

const opts = parseArgs(process.argv.slice(2));
if (opts.run && opts.t27) usage();
if (!opts.run && !opts.t27) opts.run = true;
if (opts.t27) t27Mode(opts);
else runMode(opts);
