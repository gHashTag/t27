//! `t27c test-report` stops a test that never returns (#7255).
//!
//! Each test runs as its own process. `test-report` waited on it with no
//! limit, so one looping test hung the command, and `seal --save` with it,
//! for good; an outer kill left the test binary spinning with PPID 1.

use std::fs;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Past this the per-test limit was not applied: the test kills `t27c` and
/// the probe with it, and fails, rather than hang the suite as the defect
/// did. Under a minute, so a host that reaps long test binaries itself does
/// not end the probe first.
const OUTER: Duration = Duration::from_secs(50);

/// Its first test never returns: `i * 1` never reaches `n`.
const LOOP_PROBE: &str = "module cli_loop_probe;\n\n\
fn spin(n: u64) -> u64 {\n    var i: u64 = 0;\n    while (i < n) {\n        i = i * 1;\n    }\n    return i;\n}\n\n\
test \"spins_forever\" {\n    assert spin(1) == 1;\n}\n\n\
test \"returns\" {\n    assert spin(0) == 0;\n}\n";

fn zig_on_path() -> bool {
    Command::new("zig")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Work dirs `test-report` would leave for this probe in the temp dir.
fn leftover_work_dirs() -> Vec<String> {
    fs::read_dir(std::env::temp_dir())
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.starts_with("t27c-test-report-cli_loop_probe-"))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(unix)]
#[test]
fn a_looping_test_fails_at_the_limit_the_environment_sets() {
    use std::os::unix::process::CommandExt;
    if !zig_on_path() {
        eprintln!("skipped: zig not on PATH");
        return;
    }
    let dir = fs::canonicalize(std::env::temp_dir())
        .unwrap()
        .join(format!("t27-test-timeout-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("specs/probe")).expect("specs dir");
    fs::write(dir.join("specs/probe/cli_loop_probe.t27"), LOOP_PROBE).expect("write spec");

    let start = Instant::now();
    // Its own process group, so the kill below reaches the probe too.
    let child = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .current_dir(&dir)
        .env("T27C_TEST_TIMEOUT", "2")
        .args(["test-report", "specs/probe/cli_loop_probe.t27"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .expect("run t27c test-report");
    let group = format!("-{}", child.id());
    let (done, watch) = mpsc::channel::<()>();
    let dog = std::thread::spawn(move || {
        let fired = watch.recv_timeout(OUTER).is_err();
        if fired {
            // `--`: procps-ng 4.0.2 reads `kill -9 -<pgid>` as no target,
            // exits 0 and kills nothing.
            let _ = Command::new("kill").args(["-9", "--", &group]).status();
        }
        fired
    });
    let out = child.wait_with_output().expect("wait on t27c test-report");
    let _ = done.send(());
    let fired = dog.join().unwrap();
    let took = start.elapsed();
    let _ = fs::remove_dir_all(&dir);
    if fired {
        // The killed t27c could not remove its work dir.
        for n in leftover_work_dirs() {
            let _ = fs::remove_dir_all(std::env::temp_dir().join(n));
        }
    }
    assert!(!fired, "test-report still ran after {OUTER:?}: the per-test limit was not applied");

    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}\n{}", String::from_utf8_lossy(&out.stderr));
    assert!(text.contains("  FAIL  spins_forever\n"), "{text}");
    assert!(text.contains("spins_forever   (timed out after 2 s, not counted)"), "{text}");
    assert!(text.contains("  pass        1\n"), "{text}");
    assert!(took < Duration::from_secs(30), "took {took:?}");
    assert_eq!(leftover_work_dirs(), Vec::<String>::new());
}
