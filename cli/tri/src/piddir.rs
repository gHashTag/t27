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
//! Liveness is asked of `ps -p`, the one answer available without `unsafe` or a
//! new dependency (this crate has neither `libc` nor `tempfile`). Anything short
//! of a clear "no such process" counts as running: removing a live run's
//! scratch is the one mistake this must not make, and keeping a dead one costs
//! only disk until the next sweep.

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

/// Whether `pid` names a running process.
///
/// `ps -p <pid>` exits 1 with nothing on stdout when there is no such process,
/// on macOS and on Linux procps alike. Every other outcome -- `ps` missing, a
/// signal, an unexpected code, any output -- answers true.
pub fn pid_is_running(pid: u32) -> bool {
    if pid == 0 || pid == std::process::id() {
        return true;
    }
    let out = match std::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "pid="])
        .output()
    {
        Ok(o) => o,
        Err(_) => return true,
    };
    if out.status.success() {
        return true;
    }
    let silent = out.stdout.iter().all(|b| b.is_ascii_whitespace());
    !(out.status.code() == Some(1) && silent)
}

/// The pid in a name shaped `<prefix><pid>` or `<prefix><pid>.<ext>`, or None
/// for any other name.
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
        if pid_is_running(pid) {
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

    /// A private parent for one test, removed by its own guard.
    fn parent(tag: &str) -> PidPath {
        let p = std::env::temp_dir().join(format!("tri_piddir_{tag}_{}", std::process::id()));
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
