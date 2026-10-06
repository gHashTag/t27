//! `t27b corpus --blockers`: which constructs stand between t27b and whole
//! files, and in which order supporting them unlocks the most files.
//!
//! A file is unlocked only when every construct it uses is supported, so the
//! count of first rejections says little about what to build next: a construct
//! that is the first rejection in 300 files may be the last one in none. This
//! module works from the full set of blockers per file (`lower::blockers`)
//! and orders constructs greedily by the number of whole files each one
//! unlocks, given the ones already chosen.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// One greedy step: supporting `construct`, after every construct of the
/// earlier steps, unlocks `unlocked` more files, `cumulative` in all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub construct: String,
    pub unlocked: usize,
    pub cumulative: usize,
}

/// Greedy set-cover order over `files` (each the set of constructs a file
/// needs; an empty set is a file already unlocked and is not counted).
///
/// Each step picks the construct that unlocks the most files outright. When
/// no single construct unlocks any file, it picks the one that moves the most
/// files closest to done: the score of a construct is the sum, over the
/// files that need it, of 1 / (constructs the file still needs). Ties go to
/// the construct needed by more files, then to the name, so the order is
/// deterministic. Stops when every file is unlocked.
pub fn greedy(files: &[BTreeSet<String>]) -> Vec<Step> {
    let mut remaining: Vec<BTreeSet<String>> = files.iter().filter(|f| !f.is_empty()).cloned().collect();
    let mut steps = Vec::new();
    let mut cumulative = 0usize;
    while !remaining.is_empty() {
        // construct -> (files it unlocks alone, progress score, files needing it)
        let mut score: BTreeMap<&str, (usize, f64, usize)> = BTreeMap::new();
        for f in &remaining {
            let k = f.len();
            for c in f {
                let e = score.entry(c.as_str()).or_insert((0, 0.0, 0));
                if k == 1 {
                    e.0 += 1;
                }
                e.1 += 1.0 / k as f64;
                e.2 += 1;
            }
        }
        let best = score
            .iter()
            .max_by(|a, b| {
                let (x, y) = (a.1, b.1);
                x.0.cmp(&y.0)
                    .then(x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal))
                    .then(x.2.cmp(&y.2))
                    // Reverse on the name: the alphabetically first wins a tie.
                    .then(b.0.cmp(a.0))
            })
            .map(|(c, _)| c.to_string());
        let Some(c) = best else { break };
        for f in remaining.iter_mut() {
            f.remove(&c);
        }
        let before = remaining.len();
        remaining.retain(|f| !f.is_empty());
        let unlocked = before - remaining.len();
        cumulative += unlocked;
        steps.push(Step { construct: c, unlocked, cumulative });
    }
    steps
}

/// The cumulative number of `files` unlocked after each step of `order`
/// (constructs a file needs that the order never names keep it locked).
pub fn replay(order: &[Step], files: &[BTreeSet<String>]) -> Vec<usize> {
    let mut chosen: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::with_capacity(order.len());
    for s in order {
        chosen.insert(s.construct.as_str());
        let n = files
            .iter()
            .filter(|f| !f.is_empty() && f.iter().all(|c| chosen.contains(c.as_str())))
            .count();
        out.push(n);
    }
    out
}

/// FNV-1a, 64 bits: the key of a file's source in the reference cache.
pub fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// `t27b corpus`: give every file that timed out in the parallel pass exactly
/// one more run, one file at a time and in corpus order, so the retry is not
/// under the contention that may have caused the timeout (#6063: on the
/// Railway lab at `--jobs 24`, files that pass natively in milliseconds timed
/// out at 60 s). `rerun` replaces the item in place with the retry's verdict,
/// which is final, timeout or not: a genuine infinite loop stays a timeout,
/// and nothing is retried twice. Returns, per item, whether it was retried.
pub fn retry_timeouts_once<T>(
    items: &mut [T],
    is_timeout: impl Fn(&T) -> bool,
    mut rerun: impl FnMut(&mut T),
) -> Vec<bool> {
    let retried: Vec<bool> = items.iter().map(|x| is_timeout(x)).collect();
    for (item, r) in items.iter_mut().zip(&retried) {
        if *r {
            rerun(item);
        }
    }
    retried
}

/// What the reference path (t27c's Zig backend, `t27c test-report`) did with
/// one spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reference {
    /// It compiled, and every test it ran passed (possibly none).
    Pass,
    /// It did not produce a test binary: the generated Zig does not compile.
    Blocked(String),
    /// It ran, and at least one test failed.
    Fail(String),
    /// It did not finish in time.
    Timeout,
}

impl Reference {
    pub fn passes(&self) -> bool {
        *self == Reference::Pass
    }

    /// One-word tag plus reason, for the cache file and listings.
    pub fn encode(&self) -> String {
        match self {
            Reference::Pass => "pass".into(),
            Reference::Blocked(w) => format!("blocked\t{}", w),
            Reference::Fail(w) => format!("fail\t{}", w),
            Reference::Timeout => "timeout".into(),
        }
    }

    pub fn decode(s: &str) -> Option<Reference> {
        decode_row(s).map(|x| x.0)
    }

    fn decode_verdict(s: &str) -> Option<Reference> {
        let (tag, why) = s.split_once('\t').unwrap_or((s, ""));
        Some(match tag {
            "pass" => Reference::Pass,
            "blocked" => Reference::Blocked(why.to_string()),
            "fail" => Reference::Fail(why.to_string()),
            "timeout" => Reference::Timeout,
            _ => return None,
        })
    }
}

/// One test's verdict: its name and whether it passed. A file's list is in
/// the order the tests ran.
pub type Verdicts = Vec<(String, bool)>;

const TESTS_FIELD: &str = "tests=";

fn escape_name(n: &str) -> String {
    let mut o = String::with_capacity(n.len());
    for c in n.chars() {
        match c {
            '%' => o.push_str("%25"),
            ';' => o.push_str("%3B"),
            '\t' => o.push_str("%09"),
            '\n' => o.push_str("%0A"),
            '\r' => o.push_str("%0D"),
            c => o.push(c),
        }
    }
    o
}

fn unescape_name(n: &str) -> String {
    n.replace("%3B", ";").replace("%09", "\t").replace("%0A", "\n").replace("%0D", "\r").replace("%25", "%")
}

/// The per-test field of a cache row: `tests=` then `+name` (pass) or
/// `-name` (fail), joined by `;`.
pub fn encode_verdicts(v: &[(String, bool)]) -> String {
    let items: Vec<String> =
        v.iter().map(|(n, ok)| format!("{}{}", if *ok { '+' } else { '-' }, escape_name(n))).collect();
    format!("{}{}", TESTS_FIELD, items.join(";"))
}

fn decode_verdicts(field: &str) -> Option<Verdicts> {
    let body = field.strip_prefix(TESTS_FIELD)?;
    if body.is_empty() {
        return Some(Vec::new());
    }
    body.split(';')
        .map(|it| {
            let (sign, name) = it.split_at(it.char_indices().nth(1).map_or(it.len(), |x| x.0));
            match sign {
                "+" => Some((unescape_name(name), true)),
                "-" => Some((unescape_name(name), false)),
                _ => None,
            }
        })
        .collect()
}

/// A verdict plus, when the row carries it, the per-test list (#6441). Rows
/// written before #6441 have no list and decode to `None`.
pub fn decode_row(s: &str) -> Option<(Reference, Option<Verdicts>)> {
    if let Some((head, last)) = s.rsplit_once('\t') {
        if last.starts_with(TESTS_FIELD) {
            let v = decode_verdicts(last)?;
            return Reference::decode_verdict(head).map(|r| (r, Some(v)));
        }
    }
    Reference::decode_verdict(s).map(|r| (r, None))
}

/// The per-test verdicts of `t27c test-report <spec> --verbose`: every line
/// between the header and the first blank line is `pass  <name>` or
/// `FAIL  <name>`. `None` when the output is not that (blocked, no
/// `--verbose`, or a count that does not match the `tests` total): a list
/// that may be partial is never returned.
pub fn parse_test_verdicts(stdout: &str) -> Option<Verdicts> {
    let mut lines = stdout.lines().skip_while(|l| !l.starts_with("--- test report:"));
    lines.next()?;
    let mut out = Vec::new();
    for l in lines.by_ref() {
        let t = l.trim_start();
        if t.is_empty() {
            break;
        }
        if let Some(n) = t.strip_prefix("pass  ") {
            out.push((n.to_string(), true));
        } else if let Some(n) = t.strip_prefix("FAIL  ") {
            out.push((n.to_string(), false));
        } else {
            return None;
        }
    }
    let total: usize = stdout.lines().find_map(|l| l.trim().strip_prefix("tests")?.trim().parse().ok())?;
    (total == out.len()).then_some(out)
}

/// t27b's own per-test verdicts from `t27b test` output without `--quiet`:
/// `PASS <name>` and `FAIL <name>: ...` lines, invariants left out (t27c's
/// Zig backend checks an invariant at compile time, so it has no test
/// verdict to compare with). Repeated names get t27c's `__dupN` suffix
/// (compiler.rs `gen_test_block`), so the two lists name the same test alike.
pub fn t27b_verdicts(stdout: &str) -> Verdicts {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::new();
    for l in stdout.lines() {
        let (name, ok) = if let Some(n) = l.strip_prefix("PASS ") {
            (n, true)
        } else if let Some(r) = l.strip_prefix("FAIL ") {
            (r.split_once(": ").map_or(r, |x| x.0), false)
        } else {
            continue;
        };
        let k = seen.entry(name.to_string()).or_insert(0);
        *k += 1;
        let name = if *k == 1 { name.to_string() } else { format!("{}__dup{}", name, k) };
        out.push((name, ok));
    }
    out
}

/// Per-test differential against the reference (#6441): one line per test
/// whose verdict differs, or that only one side ran, in name order. Empty
/// means the two agree test by test. This, not the JIT/interpreter
/// cross-check (both of which run lower.rs's IR), is what says t27b computes
/// what t27c + zig compute.
pub fn disagreements(t27b: &[(String, bool)], reference: &[(String, bool)]) -> Vec<String> {
    let a: BTreeMap<&str, bool> = t27b.iter().map(|(n, ok)| (n.as_str(), *ok)).collect();
    let b: BTreeMap<&str, bool> = reference.iter().map(|(n, ok)| (n.as_str(), *ok)).collect();
    let word = |ok: bool| if ok { "pass" } else { "FAIL" };
    let names: BTreeSet<&str> = a.keys().chain(b.keys()).copied().collect();
    let mut out = Vec::new();
    for n in names {
        match (a.get(n), b.get(n)) {
            (Some(x), Some(y)) if x != y => out.push(format!("{}: t27b {}, reference {}", n, word(*x), word(*y))),
            (Some(x), None) => out.push(format!("{}: t27b {}, reference has no such test", n, word(*x))),
            (None, Some(y)) => out.push(format!("{}: t27b has no such test, reference {}", n, word(*y))),
            _ => {}
        }
    }
    out
}

/// Read `t27c test-report <spec>` output.
pub fn parse_test_report(stdout: &str) -> Reference {
    for l in stdout.lines() {
        if let Some(why) = l.trim_start().strip_prefix("BLOCKED") {
            // Drop the per-run temporary directory from the message.
            let why = why.trim();
            let why = match why.find("spec.zig:") {
                Some(i) => format!("does not compile: spec.zig:{}", &why[i + "spec.zig:".len()..]),
                None => why.to_string(),
            };
            return Reference::Blocked(why);
        }
    }
    let field = |name: &str| -> Option<usize> {
        stdout.lines().find_map(|l| {
            let t = l.trim();
            let rest = t.strip_prefix(name)?;
            rest.trim().parse().ok()
        })
    };
    match (field("tests"), field("FAIL")) {
        (Some(_), Some(0)) => Reference::Pass,
        (Some(total), Some(failed)) => Reference::Fail(format!("{} of {} tests fail", failed, total)),
        _ => Reference::Blocked("unreadable test-report output".into()),
    }
}

// ------------------------------------------------------------ processes

/// What a child process left behind.
pub struct Captured {
    /// Exit code, or None when a signal ended it.
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// SIGKILL to the process group led by `pgid`, through libc's `kill(2)`
/// with a negative pid. std links libc already; this is the one call t27b
/// needs from it, so it is declared here rather than pulled in as a crate.
fn kill_group(pgid: u32) {
    extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }
    const SIGKILL: i32 = 9;
    // A pid that does not fit i32, or 0/1, would turn a negative pid into
    // "every process": refuse those outright.
    match i32::try_from(pgid) {
        Ok(p) if p > 1 => {
            // SAFETY: kill(2) takes two integers and touches no memory.
            unsafe {
                kill(-p, SIGKILL);
            }
        }
        _ => {}
    }
}

/// Run `cmd` to completion or until `timeout`, capturing both pipes. The child
/// leads its own process group, and a timeout kills the whole group, so the
/// `zig` and test binaries a reference run spawns die with it. `Ok(None)` is
/// a timeout.
pub fn run_capture(cmd: &mut Command, timeout: Duration) -> Result<Option<Captured>, String> {
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());
    cmd.process_group(0);
    let mut child = cmd.spawn().map_err(|e| format!("spawn: {}", e))?;
    // Drain the pipes on helper threads so a chatty child cannot block.
    let mut so = child.stdout.take().expect("piped stdout");
    let mut se = child.stderr.take().expect("piped stderr");
    let ho = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::Read::read_to_string(&mut so, &mut s);
        s
    });
    let he = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::Read::read_to_string(&mut se, &mut s);
        s
    });
    let deadline = Instant::now() + timeout;
    let mut nap = Duration::from_millis(1);
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Some(st),
            Ok(None) => {
                if Instant::now() >= deadline {
                    // Negative pid: the group, signalled directly. Spawning
                    // `kill -KILL -<pgid>` instead killed the corpus driver
                    // itself on the Railway lab (procps-ng 4.0.2, Debian
                    // bookworm). Then the child itself, in case the group
                    // signal failed.
                    kill_group(child.id());
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(nap);
                nap = (nap * 2).min(Duration::from_millis(20));
            }
            Err(_) => break None,
        }
    };
    let stdout = ho.join().unwrap_or_default();
    let stderr = he.join().unwrap_or_default();
    Ok(status.map(|st| Captured { code: st.code(), signal: st.signal(), stdout, stderr }))
}

// ------------------------------------------------------------ reference path

/// Runs the reference path -- `t27c test-report <spec>`: t27c's Zig backend,
/// compiled once, one test per process -- on one spec at a time.
///
/// Every worker gets its own Zig caches and temporary directory under
/// `scratch`, and wipes them once they pass `cap_bytes`: a corpus sweep
/// otherwise grows the shared Zig cache by gigabytes.
pub struct RefRunner {
    pub t27c: PathBuf,
    pub specs_dir: PathBuf,
    pub timeout: Duration,
    pub scratch: PathBuf,
    pub cap_bytes: u64,
    cache: Option<CacheWriter>,
    stamp: u64,
    known: Mutex<HashMap<u64, (Reference, Option<Verdicts>)>>,
}

fn dir_bytes(p: &Path) -> u64 {
    let Ok(rd) = std::fs::read_dir(p) else { return 0 };
    let mut n = 0;
    for e in rd.flatten() {
        match e.file_type() {
            Ok(t) if t.is_dir() => n += dir_bytes(&e.path()),
            Ok(_) => n += e.metadata().map_or(0, |m| m.len()),
            Err(_) => {}
        }
    }
    n
}

/// The t27c binary's part of every reference-cache key: FNV-1a over its
/// bytes, read once per run. Content, not mtime, so a byte-identical copy of
/// the binary (the lanes keep fixed copies so they can rebuild while
/// measuring) hits the same cache rows, and a rebuilt binary misses them
/// (#6332). Caches written under the old size-and-mtime key are still read;
/// their rows simply never match.
pub fn binary_stamp(t27c: &Path) -> Result<u64, String> {
    let bytes = std::fs::read(t27c).map_err(|e| format!("cannot read {}: {}", t27c.display(), e))?;
    let mut b = Vec::with_capacity(bytes.len() + 16);
    b.extend_from_slice(b"t27c-content\0");
    b.extend_from_slice(&bytes);
    Ok(fnv64(&b))
}

/// The toolchain half of every reference-cache key (#6443): the t27c
/// binary's content and the zig that compiles what it emits. `zig version`
/// that cannot run is recorded as such, so it still keys consistently.
pub fn toolchain_stamp(t27c: &Path) -> Result<u64, String> {
    let bin = binary_stamp(t27c)?;
    let zig = Command::new("zig")
        .arg("version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|e| format!("unavailable: {}", e));
    let mut b = Vec::with_capacity(64);
    b.extend_from_slice(b"t27c+zig\0");
    b.extend_from_slice(&bin.to_le_bytes());
    b.extend_from_slice(zig.as_bytes());
    Ok(fnv64(&b))
}

/// bootstrap/src/use_resolve.rs `find_specs_root`, for the cache key.
fn specs_root(file: &Path) -> Option<PathBuf> {
    let mut dir = file.parent()?.to_path_buf();
    loop {
        if dir.join("specs").is_dir() {
            return Some(dir.join("specs"));
        }
        if dir.file_name().map(|n| n == "specs").unwrap_or(false) && dir.is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// The spec and every spec it imports with `use`, transitively: t27c splices
/// the imported declarations in, so all their bytes decide the verdict
/// (#6443). Mirrors bootstrap/src/use_resolve.rs `use_targets`. The spec
/// itself comes first, the rest sorted.
pub fn use_closure(file: &Path) -> Vec<PathBuf> {
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut todo = vec![file.to_path_buf()];
    while let Some(p) = todo.pop() {
        if seen.contains(&p) {
            continue;
        }
        seen.push(p.clone());
        let (root, src) = match (specs_root(&p), std::fs::read_to_string(&p)) {
            (Some(r), Ok(s)) => (r, s),
            _ => continue,
        };
        for line in src.lines() {
            let rest = match line.trim().strip_prefix("use ") {
                Some(r) => r,
                None => continue,
            };
            let rest = rest.split("//").next().unwrap_or("");
            let expr = rest.trim().trim_end_matches(';').trim();
            let segs: Vec<&str> = expr.split("::").flat_map(|s| s.split('.')).collect();
            if segs.is_empty() || segs.iter().any(|s| s.is_empty()) {
                continue;
            }
            let mut t = root.clone();
            for s in segs {
                t.push(s);
            }
            t.set_extension("t27");
            if t.is_file() {
                todo.push(t);
            }
        }
    }
    let first = seen.remove(0);
    seen.sort();
    seen.insert(0, first);
    seen
}

/// Whether a cached verdict may be served (#6443): a timeout says how busy
/// the machine was, not what the spec does, so it is never cached.
pub fn cacheable(r: &Reference) -> bool {
    !matches!(r, Reference::Timeout)
}

/// One reference-cache row, newline included: key, spec path, verdict, tab
/// separated. Tabs and line breaks in the path, and line breaks in the
/// verdict's reason, become spaces so a row is always exactly one line.
pub fn cache_row(key: u64, file: &Path, r: &Reference) -> String {
    cache_row_tests(key, file, r, None)
}

/// `cache_row` with the per-test verdicts as a last `tests=` field (#6441).
pub fn cache_row_tests(key: u64, file: &Path, r: &Reference, tests: Option<&[(String, bool)]>) -> String {
    let path = file.display().to_string().replace(['\t', '\n', '\r'], " ");
    let why = r.encode().replace(['\n', '\r'], " ");
    match tests {
        Some(t) => format!("{:016x}\t{}\t{}\t{}\n", key, path, why, encode_verdicts(t)),
        None => format!("{:016x}\t{}\t{}\n", key, path, why),
    }
}

/// Every well-formed row of the cache file at `path` (none if it is missing).
/// A damaged line is skipped.
pub fn read_cache(path: &Path) -> HashMap<u64, Reference> {
    read_cache_tests(path).into_iter().map(|(k, v)| (k, v.0)).collect()
}

/// `read_cache` with each row's per-test verdicts (`None` on a row written
/// before #6441).
pub fn read_cache_tests(path: &Path) -> HashMap<u64, (Reference, Option<Verdicts>)> {
    let mut known = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(path) {
        for l in text.lines() {
            let mut it = l.splitn(3, '\t');
            let (Some(k), Some(_path), Some(r)) = (it.next(), it.next(), it.next()) else { continue };
            if k.len() != 16 {
                continue;
            }
            if let (Ok(k), Some(r)) = (u64::from_str_radix(k, 16), decode_row(r)) {
                known.insert(k, r);
            }
        }
    }
    known
}

/// The append side of the reference cache, shared by every worker. Each row
/// is formatted into one buffer and written with a single `write_all` to a
/// file opened once in append mode, under a lock, so rows from concurrent
/// workers cannot interleave (#6332: `writeln!` could split one row into
/// several writes, and one damaged line held two rows).
pub struct CacheWriter {
    path: PathBuf,
    file: Mutex<Option<std::fs::File>>,
}

impl CacheWriter {
    pub fn new(path: PathBuf) -> CacheWriter {
        CacheWriter { path, file: Mutex::new(None) }
    }

    /// Append one row. A cache that cannot be opened or written is not an
    /// error: the run goes on without it.
    pub fn append(&self, key: u64, file: &Path, r: &Reference) {
        self.append_tests(key, file, r, None)
    }

    /// `append` with the per-test verdicts (#6441).
    pub fn append_tests(&self, key: u64, file: &Path, r: &Reference, tests: Option<&[(String, bool)]>) {
        use std::io::Write;
        let row = cache_row_tests(key, file, r, tests);
        let mut g = self.file.lock().unwrap_or_else(|e| e.into_inner());
        if g.is_none() {
            *g = std::fs::OpenOptions::new().create(true).append(true).open(&self.path).ok();
        }
        if let Some(f) = g.as_mut() {
            let _ = f.write_all(row.as_bytes());
        }
    }
}

impl RefRunner {
    /// `cache`, when given, is a tab-separated file of earlier results keyed
    /// by spec path, the source of the spec and of its `use` closure, the
    /// content of the t27c binary and the zig version (`toolchain_stamp`); a
    /// changed spec or import, a rebuilt t27c or another zig misses, a copied
    /// t27c hits. Timeouts are never cached (#6443).
    pub fn new(
        t27c: PathBuf,
        specs_dir: PathBuf,
        timeout: Duration,
        scratch: PathBuf,
        cap_bytes: u64,
        cache: Option<PathBuf>,
    ) -> Result<RefRunner, String> {
        let stamp = toolchain_stamp(&t27c)?;
        let mut known = cache.as_deref().map(read_cache_tests).unwrap_or_default();
        // Caches written before #6443 kept timeouts; they never hit now (the
        // key changed), but drop them so nothing can serve one.
        known.retain(|_, r| cacheable(&r.0));
        let cache = cache.map(CacheWriter::new);
        Ok(RefRunner { t27c, specs_dir, timeout, scratch, cap_bytes, cache, stamp, known: Mutex::new(known) })
    }

    fn key(&self, file: &Path) -> u64 {
        let mut b = Vec::with_capacity(4096);
        b.extend_from_slice(file.to_string_lossy().as_bytes());
        b.push(0);
        for p in use_closure(file) {
            let src = std::fs::read(&p).unwrap_or_default();
            b.extend_from_slice(p.to_string_lossy().as_bytes());
            b.push(0);
            b.extend_from_slice(&(src.len() as u64).to_le_bytes());
            b.extend_from_slice(&src);
        }
        b.extend_from_slice(&self.stamp.to_le_bytes());
        fnv64(&b)
    }

    /// The reference verdict on `file`, run in `worker`'s scratch directory,
    /// with its per-test verdicts when it ran tests. The bool is true when it
    /// came from the cache. A cached pass or fail without per-test verdicts
    /// (a row from before #6441) is a miss: it cannot be compared test by
    /// test, and a file-level answer is what #6441 replaces.
    pub fn run(&self, worker: usize, file: &Path) -> (Reference, Option<Verdicts>, bool) {
        let key = self.key(file);
        if let Some((r, t)) = self.known.lock().unwrap().get(&key) {
            let ran = matches!(r, Reference::Pass | Reference::Fail(_));
            if t.is_some() || !ran {
                return (r.clone(), t.clone(), true);
            }
        }
        let dir = self.scratch.join(format!("w{}", worker));
        if dir_bytes(&dir) > self.cap_bytes {
            let _ = std::fs::remove_dir_all(&dir);
        }
        let (global, local, tmp) = (dir.join("zig-global"), dir.join("zig-local"), dir.join("tmp"));
        for d in [&global, &local, &tmp] {
            let _ = std::fs::create_dir_all(d);
        }
        let mut cmd = Command::new(&self.t27c);
        cmd.arg("test-report")
            .arg(file)
            .arg("--verbose")
            .arg("--specs-dir")
            .arg(&self.specs_dir)
            .env("ZIG_GLOBAL_CACHE_DIR", &global)
            .env("ZIG_LOCAL_CACHE_DIR", &local)
            .env("TMPDIR", &tmp);
        let mut tests = None;
        // `keep` is false for what the machine did rather than the spec
        // (#6443): a timeout, a t27c killed by a signal, one that never ran.
        let (r, keep) = match run_capture(&mut cmd, self.timeout) {
            Ok(Some(c)) if c.code == Some(0) => {
                let r = parse_test_report(&c.stdout);
                if matches!(r, Reference::Pass | Reference::Fail(_)) {
                    tests = parse_test_verdicts(&c.stdout);
                }
                (r, true)
            }
            Ok(Some(c)) => (
                Reference::Blocked(format!(
                    "t27c test-report exited {:?}: {}",
                    c.code.or(c.signal),
                    c.stderr.lines().next().unwrap_or("").trim()
                )),
                c.code.is_some(),
            ),
            Ok(None) => {
                // A killed run leaves its working directory behind.
                let _ = std::fs::remove_dir_all(&tmp);
                (Reference::Timeout, false)
            }
            Err(e) => (Reference::Blocked(e), false),
        };
        if keep && cacheable(&r) {
            self.known.lock().unwrap().insert(key, (r.clone(), tests.clone()));
            if let Some(c) = &self.cache {
                c.append_tests(key, file, &r, tests.as_deref());
            }
        }
        (r, tests, false)
    }
}
