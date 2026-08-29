//! Checks whose input is not there.
//!
//! WHY THIS EXISTS
//! ---------------
//! The catalog gate's field-by-field comparison had never run. `gen/` is
//! gitignored and a cleanup commit untracked the artifacts it read, so the check
//! was left with nothing -- and it reported that into a variable the master gate
//! does not print. Zero occurrences of the word in the output of the command
//! that gates master (W702).
//!
//! **Removing a check's input does not make it fail. It makes it quiet.** That
//! is a shape, not an incident, and this looks for it.
//!
//! WHAT IT LOOKS FOR
//! -----------------
//! Path-shaped string literals in gate and tool sources whose path does not
//! exist in the tree, EXCLUDING the ones that are supposed to be absent:
//!
//!   * fixtures inside a self-check or a test module -- `specs/a.t27`,
//!     `docs/planted.md` and friends are planted into temp directories on
//!     purpose, and a first version that did not exclude them reported 126 hits
//!     of which almost none were real
//!   * outputs the code WRITES rather than reads
//!   * anything under `target/`, `/tmp`, or carrying a format placeholder
//!
//! THE CONTROL FAILED, AND THAT IS IN THE DOCUMENTATION RATHER THAN FIXED AWAY
//! ---------------------------------------------------------------------------
//! Run against the commit before W702's fix, this command does **not** find the
//! case it was written for. The pre-fix code built the path as
//!
//!     let json = emitted.join("formats_catalog.json");   // no slash
//!     ... .unwrap_or_else(|| Path::new("gen/numeric"))   // no extension
//!
//! Neither literal is a path by this command's test, and the assembled one never
//! appears as a literal at all. **Zero of one on its founding case.**
//!
//! The window is not being widened until it says one. Following `join` chains
//! through variables is a dataflow problem, and a heuristic stretched until it
//! hits its own motivating example has stopped being evidence (skill 177).
//!
//! What it does find is a different and real class: an input named outright in
//! production code that is not in the tree. That is how `public/index.html` was
//! found -- the Railway HTTP server's static fallback and its 404 page both point
//! at a directory that has never existed in any commit.
//!
//! WHAT IT CANNOT DO
//! -----------------
//! It cannot tell a check that is silent about a missing input from one that
//! reports it properly -- that needs the control flow, not the literal. It marks
//! a line `quiet?` when the nearest handling looks like a bare early return with
//! no report, and that mark is a HINT for a human, never a verdict.
//!
//! It also cannot see a path built by concatenation. Everything here is a lower
//! bound.
use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum OrphanedCmd {
    /// List gate and tool sources that name an input which is not in the tree.
    List {
        /// Also print the fixture paths that were filtered out, to check the filter.
        #[arg(long)]
        show_filtered: bool,
    },
}

/// Directories whose sources are gates, tools or the CLI itself.
const ROOTS: [&str; 3] = ["tools", "bootstrap/src", "cli/tri/src"];

/// A line is fixture context when it sits inside a self-check or a test module.
/// Tracked by a simple depth counter rather than a parser: these files put
/// `mod tests` and `def self_check` at column 0, and a counter that is wrong
/// would show up as fixtures leaking back into the report.
fn fixture_regions(text: &str) -> Vec<bool> {
    let mut out = Vec::with_capacity(text.lines().count());
    // Rust: a test module is a brace-delimited region. An indent rule cannot do
    // this -- a first version set the region's indent from `#[test]` and then let
    // the `fn` on the next line close it, so 41 fixture paths leaked back in.
    let mut rust_depth: Option<i32> = None;
    let mut depth: i32 = 0;
    // Python: indentation is the structure, and a `def` at or shallower than the
    // one that opened the region ends it.
    let mut py_indent: Option<usize> = None;

    for line in text.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        let opens_rust = trimmed.starts_with("#[cfg(test)]")
            || trimmed.starts_with("mod tests")
            || trimmed.starts_with("pub mod tests");
        let opens_py = trimmed.starts_with("def self_check")
            || trimmed.starts_with("def _self_check")
            || trimmed.starts_with("def test_");

        if opens_rust && rust_depth.is_none() {
            rust_depth = Some(depth);
        }
        if opens_py && py_indent.is_none() {
            py_indent = Some(indent);
        }
        if let Some(pi) = py_indent {
            if !trimmed.is_empty() && indent <= pi && !opens_py && trimmed.starts_with("def ") {
                py_indent = None;
            }
        }

        let inside = rust_depth.is_some() || py_indent.is_some();
        out.push(inside);

        // Count braces AFTER recording, so the line that opens the module is
        // itself inside it.
        for c in line.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        if let Some(entry) = rust_depth {
            // Back to the depth we entered at, and past the module body.
            if depth <= entry && !opens_rust && trimmed.contains('}') {
                rust_depth = None;
            }
        }
    }
    out
}

/// Extensions a repository file actually has. Without this, `v0.1` in a schema
/// identifier and `github.io` in a repository name both read as paths -- they
/// contain a slash and a dot, which is all the shape test asks for.
const FILE_EXT: [&str; 22] = [
    "py", "rs", "json", "md", "t27", "txt", "yml", "yaml", "toml", "sh", "html", "inc", "bin", "v",
    "sv", "lean", "tex", "csv", "tsv", "log", "zig", "c",
];

fn looks_like_path(s: &str) -> bool {
    let Some(file) = s.rsplit('/').next() else {
        return false;
    };
    let Some((_, ext)) = file.rsplit_once('.') else {
        return false;
    };
    s.contains('/')
        && FILE_EXT.contains(&ext)
        && !s.starts_with('/')
        && !s.starts_with("target/")
        && !s.starts_with("http")
        && !s.contains('{')
        && !s.contains('*')
        && !s.contains(' ')
        // Regex source is full of slashes and dots and is not a path.
        && !s.contains('^')
        && !s.contains('$')
        && !s.contains('|')
        && !s.contains('\\')
        && !s.contains('(')
        && !s.contains('?')
        && !s.contains('+')
        && s.len() > 4
}

/// Extract double-quoted literals from one line.
fn literals(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut inside = false;
    let mut prev = '\0';
    for c in line.chars() {
        if c == '"' && prev != '\\' {
            if inside {
                out.push(std::mem::take(&mut cur));
            }
            inside = !inside;
        } else if inside {
            cur.push(c);
        }
        prev = c;
    }
    out
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running `git rev-parse --show-toplevel`")?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8(out.stdout)?.trim()))
}

fn sources(root: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for r in ROOTS {
        let dir = root.join(r);
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            match p.extension().and_then(|x| x.to_str()) {
                Some("py") | Some("rs") => v.push(p),
                _ => {}
            }
        }
    }
    v.sort();
    v
}

/// Is this line WRITING the path rather than reading it?
///
/// A path a program creates is supposed to be absent. The first version did not
/// ask, and reported `save_active_skill` and `save_registry` -- two functions
/// whose entire job is to write the file it called missing.
fn is_write_site(lines: &[&str], at: usize) -> bool {
    let lo = at.saturating_sub(1);
    let hi = (at + 4).min(lines.len());
    let window = lines[lo..hi].join(" ");
    window.contains("fs::write")
        || window.contains("File::create")
        || window.contains("create_dir")
        || window.contains("OpenOptions")
        || window.contains("to_string_pretty")
        || window.contains("write_all")
        || window.contains(".write(")
}

/// A hint, not a verdict: does the handling nearby look like a bare exit?
///
/// MEASURED HIT RATE: 0 of 5. Every site this marked on its first run handled the
/// absence properly and said so with `println!` -- which the first version did not
/// count as reporting, because its vocabulary was a GATE's (`bail`, `FAIL`,
/// `exit(1`) while these are CLI commands that talk to a person.
///
/// `println!` is added and the mark is kept rather than deleted: a hint with a
/// stated hit rate is worth more than one without, and the next absent input
/// handled by a bare `return` is exactly what it exists to surface.
fn looks_quiet(lines: &[&str], at: usize) -> bool {
    let lo = at.saturating_sub(2);
    let hi = (at + 6).min(lines.len());
    let window = lines[lo..hi].join(" ");
    let exits = window.contains("return")
        || window.contains("continue")
        || window.contains("Ok(())")
        || window.contains("=> {}");
    let reports = window.contains("bail")
        || window.contains("findings.push")
        || window.contains("FAIL")
        || window.contains("eprintln")
        || window.contains("println")
        || window.contains("print(")
        || window.contains("panic")
        || window.contains("exit(1")
        || window.contains("exit(2");
    exits && !reports
}

pub fn run(cmd: &OrphanedCmd) -> Result<()> {
    let OrphanedCmd::List { show_filtered } = cmd;
    let root = repo_root()?;
    let mut hits = 0usize;
    let mut filtered = 0usize;
    let mut written = 0usize;
    let mut scanned = 0usize;

    for src in sources(&root) {
        let Ok(text) = std::fs::read_to_string(&src) else {
            continue;
        };
        let fixture = fixture_regions(&text);
        let lines: Vec<&str> = text.lines().collect();
        let rel = src
            .strip_prefix(&root)
            .unwrap_or(&src)
            .display()
            .to_string();
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with('#') || t.starts_with('*') {
                continue;
            }
            for lit in literals(line) {
                if !looks_like_path(&lit) {
                    continue;
                }
                scanned += 1;
                if root.join(&lit).exists() {
                    continue;
                }
                if fixture.get(i).copied().unwrap_or(false) {
                    filtered += 1;
                    if *show_filtered {
                        println!("  filtered (fixture)  {}:{}  {}", rel, i + 1, lit);
                    }
                    continue;
                }
                if is_write_site(&lines, i) {
                    written += 1;
                    if *show_filtered {
                        println!("  filtered (written)  {}:{}  {}", rel, i + 1, lit);
                    }
                    continue;
                }
                hits += 1;
                let quiet = if looks_quiet(&lines, i) {
                    "  quiet?"
                } else {
                    ""
                };
                println!("  {}:{}{}", rel, i + 1, quiet);
                println!("      names {} -- not in the tree", lit);
                println!("      {}", t.trim_end());
            }
        }
    }

    println!();
    println!(
        "  {hits} absent INPUT(s) named in production code; {filtered} fixture(s) and \
         {written} write-site(s) filtered out of {scanned} path literal(s)"
    );
    println!();
    println!("  `quiet?` marks a site that returns WITHOUT SAYING ANYTHING. That is a shape,");
    println!("  not a verdict: a loader returning an empty default for a missing state file is");
    println!("  correct, and both of today's marks are exactly that. Measured as a defect");
    println!("  predictor the mark is 0 for 5. It stays because the next absent input handled by");
    println!("  a bare `return` is what it exists to surface.");
    println!();
    println!("  This reads LITERALS. Paths built by concatenation are invisible here, so every");
    println!("  number above is a lower bound -- including on the case this command was written");
    println!("  for, which it does not find. See the module docs.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{fixture_regions, literals, looks_like_path};

    /// The leak that mattered: after `#[test]`, the `fn` on the next line used
    /// to close the region, and 41 fixture paths came back into the report.
    #[test]
    fn a_test_module_stays_a_fixture_region_past_its_first_fn() {
        let src = "fn real() {\n    read(\"specs/live.t27\");\n}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn one() { read(\"specs/a.t27\"); }\n    #[test]\n    fn two() { read(\"specs/b.t27\"); }\n}\n";
        let f = fixture_regions(src);
        assert!(!f[1], "production line");
        assert!(f[6], "first test fn");
        assert!(f[8], "SECOND test fn -- the one that used to leak");
    }

    #[test]
    fn a_fixture_inside_a_test_module_is_marked() {
        let src = "fn real() {\n    read(\"specs/live.t27\");\n}\n#[cfg(test)]\nmod tests {\n    fn t() { read(\"specs/a.t27\"); }\n}\n";
        let f = fixture_regions(src);
        assert!(!f[1], "the production line must not be a fixture");
        assert!(f[5], "the line inside mod tests must be");
    }

    /// The filter that mattered: without it a first run reported 126 hits, of
    /// which almost none were real.
    #[test]
    fn a_python_self_check_is_a_fixture_region() {
        let src = "def main():\n    open(\"gen/real.json\")\n\ndef self_check():\n    plant(\"docs/planted.md\")\n";
        let f = fixture_regions(src);
        assert!(!f[1]);
        assert!(f[4]);
    }

    #[test]
    fn path_shapes_are_recognised_and_noise_is_not() {
        assert!(looks_like_path("gen/numeric/formats_catalog.json"));
        assert!(looks_like_path("tools/gen_formats_catalog.py"));
        assert!(!looks_like_path("target/release/t27c"));
        assert!(!looks_like_path("https://example.com/x.json"));
        assert!(!looks_like_path("{}/out.json"));
        assert!(!looks_like_path("plain text here"));
        assert!(!looks_like_path("noslash.json"));
        // The two shapes that got through the first version.
        assert!(!looks_like_path("t27-conformance-index/v0.1"));
        assert!(!looks_like_path("gHashTag/ghashtag.github.io"));
        assert!(!looks_like_path(r"^(tbd|todo|wip|n/?a)$"));
    }

    /// The three false positives behind the first run's write-site noise.
    #[test]
    fn a_write_site_is_not_a_missing_input() {
        let src = vec![
            "fn save(root: &Path) -> Result<()> {",
            "    let p = trinity_path(root, \"state/active-skill.json\");",
            "    let data = serde_json::to_string_pretty(skill)?;",
            "    fs::write(&p, data)?;",
        ];
        assert!(
            super::is_write_site(&src, 1),
            "the save path must be filtered"
        );
    }

    #[test]
    fn a_reader_is_not_a_write_site() {
        let src = vec![
            "fn load(root: &Path) -> Result<X> {",
            "    let p = trinity_path(root, \"cells/registry.json\");",
            "    let data = fs::read_to_string(&p)?;",
        ];
        assert!(!super::is_write_site(&src, 1));
    }

    /// A CLI command reports to a person with `println!`, not with `bail`. The
    /// first version's vocabulary was a gate's, and it marked five sites that
    /// all reported properly.
    #[test]
    fn println_counts_as_reporting_the_absence() {
        let src = vec![
            "    let path = \"a/b.json\";",
            "    if !Path::new(&path).exists() {",
            "        println!(\"not found: {}\", path);",
            "        return Ok(());",
            "    }",
        ];
        assert!(!super::looks_quiet(&src, 0), "println is a report");
    }

    #[test]
    fn a_bare_return_with_no_report_is_still_marked() {
        let src = vec![
            "    let path = \"a/b.json\";",
            "    if !Path::new(&path).exists() {",
            "        return Ok(Default::default());",
            "    }",
        ];
        assert!(super::looks_quiet(&src, 0));
    }

    #[test]
    fn escaped_quotes_do_not_split_a_literal() {
        let v = literals(r#"let s = "a\"b/c.json";"#);
        assert_eq!(v.len(), 1, "{v:?}");
    }
}
