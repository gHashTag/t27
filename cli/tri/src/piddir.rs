//! Per-pid scratch paths under the temp dir that do not outlive their run.
//!
//! Written for #5982. A test in `fpga.rs` built a lake package in
//! `temp_dir()/tri_m2l_standalone_pkg_<pid>`, which `require trinity` grows to
//! about 7.6 GB, and removed it only on the line after
//! `assert!(status.success())`. A failed `lake build` panicked past the cleanup,
//! a killed run skipped it entirely, and on 2026-10-04 the leftovers filled the
//! owner's disk twice -- free space fell to about 300 MB and other runs died
//! with ENOSPC. Every leftover carried a dead pid in its name.
//!
//! Two halves, because each covers what the other cannot:
//!
//! * [`PidPath`] removes its path when it drops, which happens on the success
//!   path AND while a panic unwinds. It has to exist BEFORE the step that can
//!   fail; a guard created after that step guards nothing.
//! * [`sweep_dead`] removes siblings named for a pid that is no longer running.
//!   `kill -9` runs no destructor, so the only cleanup a killed run can get is
//!   the next run doing it.
//!
//! Liveness is asked of `ps -p <pid> -o pid=`, the one answer available without
//! `unsafe` or a new dependency (this crate has neither `libc` nor `tempfile`).
//! Exactly one answer reads as dead: exit status 1 with NOTHING on stdout and
//! NOTHING on stderr, which is what macOS `ps` prints for a pid with no process
//! (checked 2026-10-04). Everything else reads as running: `ps` missing or
//! unable to start, killed by a signal, any other exit status, and exit 1 with
//! even one byte on either stream. That last case matters most: a `ps` that
//! rejects `-p` or `-o` also exits 1, and says why on stderr. Reading it as dead
//! makes every pid dead, and runs then delete each other's live 7.6 GB packages
//! -- the review of #5990 showed exactly that with a fake `ps` on PATH. Removing
//! a live run's scratch is the one mistake this must not make; keeping a dead
//! one costs only disk until a sweep that can tell. Linux procps was not
//! checked; if it answers a missing pid any other way, the sweep keeps
//! everything there.

use std::path::{Path, PathBuf};

/// A file or directory that is removed when this guard drops -- on return and
/// on unwind alike.
pub struct PidPath {
    path: PathBuf,
}

impl PidPath {
    pub fn new(path: PathBuf) -> PidPath {
        PidPath { path }
    }

    /// A `&PathBuf` rather than `&Path`: callers such as `measured_to_lean` take
    /// `Option<&PathBuf>`, and the guard should drop in where the path was.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for PidPath {
    fn drop(&mut self) {
        remove_any(&self.path);
    }
}

/// Remove `path` whatever it is. A symlink is removed, never followed.
fn remove_any(path: &Path) {
    match std::fs::symlink_metadata(path) {
        Ok(m) if m.is_dir() => {
            let _ = std::fs::remove_dir_all(path);
        }
        Ok(_) => {
            let _ = std::fs::remove_file(path);
        }
        Err(_) => {}
    }
}

/// The liveness probe: program first, then any leading arguments. The pid
/// arguments `-p <pid> -o pid=` are appended. Tests pass a fake in its place.
const PS: &[&str] = &["ps"];

/// Whether `pid` names a running process, asked of the real `ps`.
pub fn pid_is_running(pid: u32) -> bool {
    pid_is_running_by(PS, pid)
}

/// Whether `pid` names a running process, asked of `probe`.
///
/// Pid 0 and this process's own pid are running without asking: no probe
/// answer may turn either into a sweep target.
fn pid_is_running_by(probe: &[&str], pid: u32) -> bool {
    if pid == 0 || pid == std::process::id() {
        return true;
    }
    let Some((program, lead)) = probe.split_first() else {
        return true;
    };
    match std::process::Command::new(program)
        .args(lead)
        .args(["-p", &pid.to_string(), "-o", "pid="])
        .output()
    {
        Ok(out) => !says_no_such_process(&out),
        Err(_) => true,
    }
}

/// The one probe answer that reads as dead: exit status 1, empty stdout,
/// empty stderr. A signal has no exit status and so never matches.
fn says_no_such_process(out: &std::process::Output) -> bool {
    out.status.code() == Some(1) && out.stdout.is_empty() && out.stderr.is_empty()
}

/// The pid in a name shaped `<prefix><pid>` or `<prefix><pid>.<ext>`, or None
/// for any other name. The prefix must start the name.
fn pid_of(name: &str, prefix: &str) -> Option<u32> {
    let rest = name.strip_prefix(prefix)?;
    let digits = match rest.split_once('.') {
        Some((d, _)) => d,
        None => rest,
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Remove every entry of `parent` named `<prefix><pid>` or `<prefix><pid>.<ext>`
/// whose pid is not running, and return what was removed.
///
/// An entry of a running pid -- this process's included -- is never touched,
/// and neither is a name that does not parse to a pid.
pub fn sweep_dead(parent: &Path, prefix: &str) -> Vec<PathBuf> {
    sweep_dead_by(parent, prefix, pid_is_running)
}

/// [`sweep_dead`] with the liveness answer as a parameter.
fn sweep_dead_by(parent: &Path, prefix: &str, running: impl Fn(u32) -> bool) -> Vec<PathBuf> {
    let mut gone = Vec::new();
    let Ok(entries) = std::fs::read_dir(parent) else {
        return gone;
    };
    for e in entries.flatten() {
        let name = e.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(pid) = pid_of(name, prefix) else {
            continue;
        };
        if running(pid) {
            continue;
        }
        let p = e.path();
        remove_any(&p);
        if std::fs::symlink_metadata(&p).is_err() {
            gone.push(p);
        }
    }
    gone.sort();
    gone
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fake probes. Each is `sh -c <script> fake-ps`, so `fake-ps` is `$0` and
    /// the appended `-p <pid> -o pid=` land in `$1..$4`, as they would for `ps`.
    const SILENT_EXIT_1: &[&str] = &["sh", "-c", "exit 1", "fake-ps"];
    const STDERR_EXIT_1: &[&str] = &["sh", "-c", "echo err >&2; exit 1", "fake-ps"];
    const STDOUT_EXIT_1: &[&str] = &["sh", "-c", "echo 123; exit 1", "fake-ps"];
    const SILENT_EXIT_2: &[&str] = &["sh", "-c", "exit 2", "fake-ps"];
    const SIGNALLED: &[&str] = &["sh", "-c", "kill -9 $$", "fake-ps"];
    const MISSING: &[&str] = &["/nonexistent/tri-piddir-no-such-ps"];

    /// A private parent for one test, removed by its own guard. A parent a
    /// killed run of the same test left behind is swept first.
    fn parent(tag: &str) -> PidPath {
        let temp = std::env::temp_dir();
        sweep_dead(&temp, &format!("tri_piddir_{tag}_"));
        let p = temp.join(format!("tri_piddir_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        PidPath::new(p)
    }

    /// A pid that was running a moment ago and is not now: a child, waited for.
    fn reaped_pid() -> u32 {
        let mut child = std::process::Command::new("true").spawn().unwrap();
        let pid = child.id();
        child.wait().unwrap();
        pid
    }

    #[test]
    fn piddir_pid_of_reads_only_prefix_digits_and_an_extension() {
        assert_eq!(pid_of("x_pkg_123", "x_pkg_"), Some(123));
        assert_eq!(pid_of("x_pkg_123.json", "x_pkg_"), Some(123));
        assert_eq!(pid_of("x_pkg_123.tar.gz", "x_pkg_"), Some(123));
        assert_eq!(pid_of("x_pkg_", "x_pkg_"), None);
        assert_eq!(pid_of("x_pkg_12a", "x_pkg_"), None);
        assert_eq!(pid_of("x_pkg_-1", "x_pkg_"), None);
        assert_eq!(pid_of("x_pkg_.json", "x_pkg_"), None);
        assert_eq!(pid_of("y_pkg_123", "x_pkg_"), None);
        assert_eq!(pid_of("x_pkg_99999999999", "x_pkg_"), None);
        // A suffix after the digits is another name, not this pid's.
        assert_eq!(pid_of("x_pkg_123_x", "x_pkg_"), None);
        // The prefix must start the name, not merely occur in it.
        assert_eq!(pid_of("zx_pkg_123", "x_pkg_"), None);
    }

    /// The probe's answer, case by case. Only a silent exit 1 is dead.
    #[test]
    fn piddir_probe_dead_only_on_a_silent_exit_1() {
        let dead = reaped_pid();
        assert!(
            !pid_is_running_by(SILENT_EXIT_1, dead),
            "a silent exit 1 is the one dead answer"
        );
        assert!(
            pid_is_running_by(STDERR_EXIT_1, dead),
            "exit 1 with stderr read as dead"
        );
        assert!(
            pid_is_running_by(STDOUT_EXIT_1, dead),
            "exit 1 with stdout read as dead"
        );
        assert!(
            pid_is_running_by(SILENT_EXIT_2, dead),
            "exit 2 read as dead"
        );
        assert!(
            pid_is_running_by(SIGNALLED, dead),
            "a probe killed by a signal read as dead"
        );
        assert!(
            pid_is_running_by(MISSING, dead),
            "a probe that cannot start read as dead"
        );
        assert!(pid_is_running_by(&[], dead), "no probe at all read as dead");
    }

    /// Pid 0 and this process never ask the probe, so a probe that says dead
    /// cannot make them sweep targets.
    #[test]
    fn piddir_pid_zero_and_self_are_running_whatever_the_probe_says() {
        assert!(pid_is_running(0), "pid 0, real ps");
        assert!(
            pid_is_running_by(SILENT_EXIT_1, 0),
            "pid 0, a probe saying dead"
        );
        assert!(
            pid_is_running_by(SILENT_EXIT_1, std::process::id()),
            "this process, a probe saying dead"
        );
    }

    #[test]
    fn piddir_liveness_running_and_reaped() {
        assert!(pid_is_running(std::process::id()), "this process");
        assert!(pid_is_running(1), "pid 1 is always running");
        let dead = reaped_pid();
        assert!(
            !pid_is_running(dead),
            "reaped child {dead} still reads as running"
        );
    }

    #[test]
    fn piddir_sweep_removes_dead_pids_and_keeps_live_ones() {
        let parent = parent("sweep");
        let dir = parent.path();
        let dead = reaped_pid();
        let me = std::process::id();

        let dead_dir = dir.join(format!("probe_pkg_{dead}"));
        std::fs::create_dir_all(dead_dir.join(".lake/packages/mathlib")).unwrap();
        std::fs::write(dead_dir.join(".lake/packages/mathlib/blob"), [0u8; 4096]).unwrap();
        let dead_file = dir.join(format!("probe_pkg_{dead}.json"));
        std::fs::write(&dead_file, "{}").unwrap();

        let keep = [
            dir.join(format!("probe_pkg_{me}")),
            dir.join("probe_pkg_1"),
            dir.join("probe_pkg_latest"),
            dir.join(format!("other_pkg_{dead}")),
            dir.join(format!("probe_pkg_{dead}_x")),
            dir.join(format!("zprobe_pkg_{dead}")),
        ];
        for k in &keep {
            std::fs::create_dir_all(k).unwrap();
        }

        let gone = sweep_dead(dir, "probe_pkg_");

        let mut want = vec![dead_dir.clone(), dead_file.clone()];
        want.sort();
        assert_eq!(gone, want, "the sweep must report exactly what it removed");
        assert!(!dead_dir.exists(), "a dead pid's dir survived the sweep");
        assert!(!dead_file.exists(), "a dead pid's file survived the sweep");
        for k in &keep {
            assert!(
                k.exists(),
                "the sweep removed {}, which it must keep",
                k.display()
            );
        }
    }

    /// The review of #5990: a `ps` that exits 1 with an error on stderr made
    /// every pid dead, and the sweep removed `probe_pkg_1`. With any probe that
    /// is not a silent exit 1, the sweep removes nothing.
    #[test]
    fn piddir_sweep_removes_nothing_when_the_probe_is_unclear() {
        let parent = parent("unclear");
        let dir = parent.path();
        let dead = reaped_pid();
        let planted = [
            dir.join("probe_pkg_1"),
            dir.join(format!("probe_pkg_{dead}")),
            dir.join(format!("probe_pkg_{dead}.json")),
        ];
        for p in &planted {
            std::fs::create_dir_all(p).unwrap();
        }
        for probe in [
            STDERR_EXIT_1,
            STDOUT_EXIT_1,
            SILENT_EXIT_2,
            SIGNALLED,
            MISSING,
        ] {
            let gone = sweep_dead_by(dir, "probe_pkg_", |pid| pid_is_running_by(probe, pid));
            assert!(gone.is_empty(), "probe {probe:?} swept {gone:?}");
            for p in &planted {
                assert!(p.exists(), "probe {probe:?} removed {}", p.display());
            }
        }
    }

    /// A killed test run leaves its own parent behind; the next run of the
    /// same test sweeps it before making its own.
    #[test]
    fn piddir_test_parent_sweeps_a_killed_runs_parent() {
        let dead = reaped_pid();
        let left = std::env::temp_dir().join(format!("tri_piddir_killed_{dead}"));
        std::fs::create_dir_all(left.join("probe_pkg_1")).unwrap();
        let _mine = parent("killed");
        assert!(!left.exists(), "a killed run's test parent survived");
    }

    #[test]
    fn piddir_guard_removes_its_path_on_unwind() {
        let parent = parent("unwind");
        let dir = parent.path().join("scratch");
        let file = parent.path().join("side.json");
        let (d, f) = (dir.clone(), file.clone());
        let r = std::panic::catch_unwind(move || {
            let _dg = PidPath::new(d.clone());
            let _fg = PidPath::new(f.clone());
            std::fs::create_dir_all(d.join("nested")).unwrap();
            std::fs::write(&f, "x").unwrap();
            panic!("the inner step failed");
        });
        assert!(r.is_err());
        assert!(!dir.exists(), "the guarded dir outlived a panic");
        assert!(!file.exists(), "the guarded file outlived a panic");
    }
}
