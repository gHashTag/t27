//! Retain real compiler controls for the shared C diagnostic helpers.

mod common;

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "t27-cc-control-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("create own compiler control directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn compile(source: &str, extra: &[&str]) -> String {
    let dir = Scratch::new("source");
    let path = dir.0.join("control.c");
    std::fs::write(&path, source).expect("write real C control");
    common::cc_check(&path, extra)
}

#[test]
fn valid_c_has_zero_source_errors() {
    let diagnostics = compile("int value(void) { return 7; }\n", &[]);
    assert_eq!(common::error_count(&diagnostics), 0);
}

#[test]
fn invalid_c_has_real_source_errors() {
    let diagnostics = compile("int value(void) { return missing_identifier; }\n", &[]);
    assert!(common::error_count(&diagnostics) > 0);
}

#[test]
fn a_missing_include_counts_as_a_fatal_source_error() {
    let diagnostics = compile("#include \"t27_missing_control_header_47d4.h\"\n", &[]);
    assert!(common::error_count(&diagnostics) > 0);
}

#[test]
fn a_rejected_option_cannot_be_read_as_zero_errors() {
    assert!(std::panic::catch_unwind(|| {
        compile(
            "int value(void) { return 7; }\n",
            &["-ft27-nonexistent-option"],
        )
    })
    .is_err());
}

#[test]
fn syntax_probe_does_not_create_object_files() {
    if std::env::var_os("T27_CC_PROBE_CHILD").is_some() {
        assert!(std::fs::read_dir(".").unwrap().next().is_none());
        common::get_cc_args();
        assert!(
            std::fs::read_dir(".").unwrap().next().is_none(),
            "probe wrote a file"
        );
        return;
    }
    let dir = Scratch::new("probe");
    let out = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "syntax_probe_does_not_create_object_files",
            "--nocapture",
        ])
        .env("T27_CC_PROBE_CHILD", "1")
        .current_dir(&dir.0)
        .output()
        .expect("run isolated real compiler probe");
    assert!(
        out.status.success(),
        "isolated probe failed:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
