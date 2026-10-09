//! tri_test runs a spec's tests and reads the verdict through report.t27 (#8140): a green spec, a red copy
//! with one planted failure, and a stand-in t27c that only lists the tests, as `t27c test` did.
use serde_json::{json, Value}; use sha2::{Digest, Sha256};
use std::{io::Write, path::Path, process::{Command, Stdio}};

fn sha(p: &Path) -> Vec<u8> { Sha256::digest(std::fs::read(p).unwrap()).to_vec() }
/// One tools/call of tri_test on a server started in `root`: the result's text as JSON, and isError.
fn tri_test(root: &Path, t27c: &Path) -> (Value, bool) {
    let mut s = Command::new(env!("CARGO_BIN_EXE_tri-mcp")).current_dir(root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let args = json!({"spec_path": "gft_relu.t27", "t27c": t27c});
    writeln!(s.stdin.take().unwrap(), "{}", json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "tri_test", "arguments": args}})).unwrap();
    let r: Value = serde_json::from_slice(&s.wait_with_output().unwrap().stdout).unwrap();
    (serde_json::from_str(r["result"]["content"][0]["text"].as_str().unwrap()).unwrap(), r["result"]["isError"].as_bool().unwrap())
}

#[test]
fn tri_test_runs_the_tests_and_a_listing_is_not_a_pass() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let t27c = std::env::var_os("T27C").map(Into::into).or_else(|| ["release", "debug"].iter().map(|d| repo.join("target").join(d).join("t27c")).find(|p| p.exists()));
    let t27c: std::path::PathBuf = t27c.expect("no t27c: run `cargo build -p t27c` or set T27C");
    let (spec, root) = (repo.join("specs/ternary/gft_relu.t27"), std::env::temp_dir().join(format!("tri-mcp-test-{}", std::process::id())));
    let (before, copy) = (sha(&spec), root.join("gft_relu.t27")); // a temp root, so the tracked akashic log stays as it is
    std::fs::create_dir_all(root.join(".trinity")).unwrap();
    std::fs::copy(&spec, &copy).unwrap();
    let (green, err) = tri_test(&root, &t27c);
    assert!(!err && green["passes"] == true && green["tests"] == 4 && green["pass"] == 4 && green["fail"] == 0, "{green}");
    let src = std::fs::read_to_string(&copy).unwrap();
    std::fs::write(&copy, src.replacen("{ return 0; }   // negative", "{ return x; }   // negative", 1)).unwrap(); // #7400's plant
    let (red, err) = tri_test(&root, &t27c);
    assert!(err && red["passes"] == false && red["tests"] == 4 && red["pass"] == 3 && red["fail"] == 1 && red["failed"] == json!(["negz"]), "{red}");
    std::fs::write(&copy, &src).unwrap();
    assert_eq!(sha(&copy), before, "the copy is restored byte for byte");
    let stub = |name: &str, body: &str| { let p = root.join(name); assert!(Command::new("sh").args(["-c", "printf '%s' \"$2\" > \"$1\" && chmod +x \"$1\"", "sh"]).arg(&p).arg(body).status().unwrap().success()); p };
    let (listed, err) = tri_test(&root, &stub("lists-only", "#!/bin/sh\necho listed 4 tests\n")); // negative control: what `t27c test` gives
    assert!(err && listed["passes"] == false && listed["verdict"] == 4 && listed["tests"] == 0, "{listed}");
    let (torn, err) = tri_test(&root, &stub("green-exit-1", "#!/bin/sh\nprintf '\\n  tests       4\\n  pass        4\\n  FAIL        0\\n'\nexit 1\n"));
    assert!(err && torn["passes"] == false && torn["verdict"] == 7 && torn["tests"] == 4, "{torn}"); // a green report from a failed run
    std::fs::remove_dir_all(&root).unwrap();
    assert_eq!(sha(&spec), before, "the spec in the tree is untouched");
}
