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
    known: Mutex<HashMap<u64, Reference>>,
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

/// One reference-cache row, newline included: key, spec path, verdict, tab
/// separated. Tabs and line breaks in the path, and line breaks in the
/// verdict's reason, become spaces so a row is always exactly one line.
pub fn cache_row(key: u64, file: &Path, r: &Reference) -> String {
    let path = file.display().to_string().replace(['\t', '\n', '\r'], " ");
    let why = r.encode().replace(['\n', '\r'], " ");
    format!("{:016x}\t{}\t{}\n", key, path, why)
}

/// Every well-formed row of the cache file at `path` (none if it is missing).
/// A damaged line is skipped.
pub fn read_cache(path: &Path) -> HashMap<u64, Reference> {
    let mut known = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(path) {
        for l in text.lines() {
            let mut it = l.splitn(3, '\t');
            let (Some(k), Some(_path), Some(r)) = (it.next(), it.next(), it.next()) else { continue };
            if k.len() != 16 {
                continue;
            }
            if let (Ok(k), Some(r)) = (u64::from_str_radix(k, 16), Reference::decode(r)) {
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
        use std::io::Write;
        let row = cache_row(key, file, r);
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
    /// by spec path, spec source, and the content of the t27c binary
    /// (`binary_stamp`); a changed spec or a rebuilt t27c misses, a copied
    /// t27c hits.
    pub fn new(
        t27c: PathBuf,
        specs_dir: PathBuf,
        timeout: Duration,
        scratch: PathBuf,
        cap_bytes: u64,
        cache: Option<PathBuf>,
    ) -> Result<RefRunner, String> {
        let stamp = binary_stamp(&t27c)?;
        let known = cache.as_deref().map(read_cache).unwrap_or_default();
        let cache = cache.map(CacheWriter::new);
        Ok(RefRunner { t27c, specs_dir, timeout, scratch, cap_bytes, cache, stamp, known: Mutex::new(known) })
    }

    fn key(&self, file: &Path, src: &[u8]) -> u64 {
        let mut b = Vec::with_capacity(src.len() + 64);
        b.extend_from_slice(file.to_string_lossy().as_bytes());
        b.push(0);
        b.extend_from_slice(src);
        b.extend_from_slice(&self.stamp.to_le_bytes());
        fnv64(&b)
    }

    /// The reference verdict on `file`, run in `worker`'s scratch directory.
    /// The bool is true when it came from the cache.
    pub fn run(&self, worker: usize, file: &Path) -> (Reference, bool) {
        let src = std::fs::read(file).unwrap_or_default();
        let key = self.key(file, &src);
        if let Some(r) = self.known.lock().unwrap().get(&key) {
            return (r.clone(), true);
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
            .arg("--specs-dir")
            .arg(&self.specs_dir)
            .env("ZIG_GLOBAL_CACHE_DIR", &global)
            .env("ZIG_LOCAL_CACHE_DIR", &local)
            .env("TMPDIR", &tmp);
        let r = match run_capture(&mut cmd, self.timeout) {
            Ok(Some(c)) if c.code == Some(0) => parse_test_report(&c.stdout),
            Ok(Some(c)) => Reference::Blocked(format!(
                "t27c test-report exited {:?}: {}",
                c.code.or(c.signal),
                c.stderr.lines().next().unwrap_or("").trim()
            )),
            Ok(None) => {
                // A killed run leaves its working directory behind.
                let _ = std::fs::remove_dir_all(&tmp);
                Reference::Timeout
            }
            Err(e) => Reference::Blocked(e),
        };
        self.known.lock().unwrap().insert(key, r.clone());
        if let Some(c) = &self.cache {
            c.append(key, file, &r);
        }
        (r, false)
    }
}
