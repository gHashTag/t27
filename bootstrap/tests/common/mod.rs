use std::process::Command;
use std::sync::OnceLock;

static CC_ARGS: OnceLock<Vec<&'static str>> = OnceLock::new();

/// Returns the appropriate compiler arguments for the current `cc`.
pub fn get_cc_args() -> &'static [&'static str> {
    CC_ARGS.get_or_init(|| {
        // Probe the compiler for -ferror-limit=0 (clang) or -fmax-errors=0 (gcc)
        if Command::new("cc")
            .arg("-ferror-limit=0")
            .arg("-c")
            .arg("-x")
            .arg("c")
            .arg("-")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            vec!["-std=c11", "-ferror-limit=0", "-fsyntax-only", "-x", "c"]
        } else if Command::new("cc")
            .arg("-fmax-errors=0")
            .arg("-c")
            .arg("-x")
            .arg("c")
            .arg("-")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            vec!["-std=c11", "-fmax-errors=0", "-fsyntax-only", "-x", "c"]
        } else {
            vec!["-std=c11", "-fsyntax-only", "-x", "c"]
        }
    })
    .as_slice()
}