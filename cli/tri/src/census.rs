//! Does each census's printed population match one counted a different way?
//!
//! Three defects in three passes had one shape: a command narrowed the set it
//! spoke about, and the narrowing was invisible because the only number on
//! screen was the command's own.
//!
//!   * `unparsed locate` checked the message FORMAT before the error's STAGE,
//!     so its buckets summed to 80 against a population of 76 (#2935).
//!   * `quantifiers report` matched `find("forall ")` -- the spelling with a
//!     trailing space -- and read 922 where the corpus holds 923 (#2938).
//!   * `mods orphan` walked a hardcoded two crates of five and printed their
//!     sum, 132, where the workspace holds 136 (#2941).
//!
//! Each was found the same way and none by reading the code: count the
//! population by a DIFFERENT route and subtract. This is that subtraction, run
//! for every census at once, so the fourth instance does not need a fourth
//! accident to be noticed.
//!
//! A fourth row was built for `seals hollow` and REMOVED. Its counter tested
//! json files for the text `"spec_path"` while the census parses the same
//! field, so the obvious mutation -- planting one more seal file -- moved BOTH
//! numbers to 1314 and the row stayed green. There is no realistic input that
//! makes the two disagree, and a control that cannot fail is not a control.
//! Three rows that bite are worth more than four where one is decoration.
//!
//! The counters below are written here, from scratch. A counter that called
//! the census's own helper would agree with it by construction and measure
//! nothing -- the precondition of differential testing is independence, and it
//! is the precondition this file exists to keep.

use anyhow::Result;
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum CensusCmd {
    /// Every census's printed population, against one counted another way.
    Audit,
    /// Pin each pure census's output, so a change that moves one has to say so.
    Pin {
        /// Compare against the ledger and exit 1 on any difference.
        #[arg(long)]
        gate: bool,
        /// Rewrite the ledger from the current tree.
        #[arg(long)]
        bless: bool,
    },
    /// For each pinned census that moved: every line that moved, and which
    /// changed file moves it -- each put back to the blessed commit alone, in a
    /// scratch copy, and the census run again.
    Explain {
        /// At most this many changed files are tried per run; each costs one
        /// run of every moved census. The rest are counted, not tried.
        #[arg(long, default_value_t = 600)]
        max: usize,
    },
}

/// The censuses this pins, and why only these.
///
/// Each one's POPULATION IS A DIRECTORY, so it cannot move unless a file in that
/// directory changes -- which is what makes a pin cheap rather than a daily tax.
/// Measured over the 39 most recent transitions on master with one fixed
/// instrument: **8 moved a census, and every one of the 8 had edited that
/// census's own subject** (fetches 4/4 touched `cli/tri/src`, shell 4/4 and
/// quiet 1/1 touched `.github/workflows`). Not one moved as a side effect of an
/// unrelated change.
///
/// The other censuses are excluded and the reason is measured, not assumed:
/// `dead` and `unmeasured` read the GitHub API, so their answer moves when the
/// world moves and pinning them would redden on somebody else's push. `empty`
/// runs 15s. `preview` exits non-zero by design outside a pull request.
const PINNED: &[(&str, &[&str])] = &[
    ("fetches", &["gates", "fetches"]),
    ("quiet", &["gates", "quiet"]),
    ("shell", &["gates", "shell"]),
];

/// Where the blessed output lives, one file per census.
fn ledger_dir() -> Result<std::path::PathBuf> {
    Ok(repo_root()?.join("tools/census"))
}

/// Run one census against the current tree, as the ledger records it.
///
/// This pins the OUTPUT, not numbers parsed out of it. Parsing a tool's own
/// human report to check the tool is the re-implementation trap one layer up:
/// the parser would disagree with the printer and the disagreement would be the
/// parser's. A byte comparison cannot have that bug, and the failure prints the
/// actual diff, which is what a reader needs.
fn run_census(args: &[&str]) -> Result<String> {
    run_census_in(args, None)
}

/// The same run with the census reading another tree: every pinned census
/// finds its root with `git rev-parse --show-toplevel` from where it starts.
fn run_census_in(args: &[&str], dir: Option<&Path>) -> Result<String> {
    let exe =
        std::env::current_exe().map_err(|e| anyhow::anyhow!("cannot locate this binary: {e}"))?;
    let mut cmd = std::process::Command::new(exe);
    cmd.args(args);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    let out = cmd
        .output()
        .map_err(|e| anyhow::anyhow!("running tri {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        anyhow::bail!(
            "tri {} exited {} -- refusing to pin a census that could not run",
            args.join(" "),
            out.status.code().unwrap_or(-1)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// The gate, callable from the commit hook.
///
/// `tri census pin --gate` was a CI-only reading, and the cost of that showed up twice in
/// one day: the `fetches` census went red on master at 10:46Z and stayed red for an hour,
/// and then the very session that repaired it moved the `shell` census two steps and
/// pushed without blessing -- having written the "re-bless in the SAME commit" rule that
/// morning. The reading takes 133 ms. There is no reason for it to be a CI-only question.
pub fn gate() -> Result<()> {
    pin(true, false)
}

fn pin(gate: bool, bless: bool) -> Result<()> {
    if gate && bless {
        anyhow::bail!("--gate and --bless ask opposite questions; pick one");
    }
    let dir = ledger_dir()?;
    if bless {
        std::fs::create_dir_all(&dir)?;
    }
    let mut moved: Vec<String> = Vec::new();
    for (name, args) in PINNED {
        let now = run_census(args)?;
        let path = dir.join(format!("{name}.txt"));
        if bless {
            std::fs::write(&path, &now)?;
            println!("  blessed  {name}  ({} bytes)", now.len());
            continue;
        }
        let was = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => {
                // Absence is not amnesty: a missing ledger is a census nothing
                // is watching, which is the state this exists to end.
                moved.push(format!("{name}: NO LEDGER at {}", path.display()));
                continue;
            }
        };
        if was != now {
            let first = was
                .lines()
                .zip(now.lines())
                .find(|(a, b)| a != b)
                .map(|(a, b)| format!("\n      was: {}\n      now: {}", a.trim(), b.trim()))
                .unwrap_or_else(|| " (length differs)".into());
            moved.push(format!("{name} moved:{first}"));
        }
    }
    if bless {
        println!(
            "\n  {} census(es) re-recorded. Say in the commit message WHICH number\n  \
             moved and why -- the ledger records that it moved, not why.",
            PINNED.len()
        );
        return Ok(());
    }
    if !gate {
        for (name, _) in PINNED {
            println!("  {name}");
        }
        println!(
            "\n  {} census(es) pinned under tools/census/.",
            PINNED.len()
        );
        println!("  `--gate` compares, `--bless` re-records.");
        return Ok(());
    }
    // A ledger with no census is the mirror of a census with no ledger, and it
    // is the case an A/B of the OUTPUT cannot see: drop a name from `PINNED` and
    // every remaining reading still matches, so the gate goes green having
    // stopped watching something. `insta` calls this `--unreferenced=reject`.
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) != Some("txt") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if !PINNED.iter().any(|(n, _)| *n == stem) {
                moved.push(format!(
                    "{stem}: LEDGER WITH NO CENSUS -- `{stem}` is not in PINNED, so \
                     nothing regenerates {}. Delete the file or restore the entry.",
                    path.display()
                ));
            }
        }
    }

    if moved.is_empty() {
        println!("PASS: no pinned census moved.\n");
        println!(
            "  This says the {} pinned readings are unchanged. It says nothing\n  \
             about whether they are RIGHT -- that is what `tri census audit` asks.",
            PINNED.len()
        );
        return Ok(());
    }
    let missing = moved.iter().filter(|m| m.contains("NO LEDGER")).count();
    for m in &moved {
        println!("FAIL: {m}");
    }
    if missing == moved.len() {
        println!(
            "\n  Absence is not amnesty: a census with no ledger is one nothing is\n  \
             watching, which is the state this exists to end. Run\n  \
             `tri census pin --bless` and commit the ledger."
        );
        std::process::exit(1);
    }
    println!(
        "\n  A census moved. That is not a defect by itself -- it is a defect when\n  \
         nobody says so. Measured over 39 commits: 8 moved a census and only 4\n  \
         mentioned it, and one of the silent four had made 45 of 50 red workflows\n  \
         invisible.\n\n  \
         Re-bless in the SAME commit (`tri census pin --bless`) and say in the\n  \
         message which number moved and why."
    );
    std::process::exit(1);
}

/// One census, the line it prints its population on, and how to count that
/// population without asking it.
struct Row {
    census: &'static str,
    args: &'static [&'static str],
    /// Text that identifies the line carrying the population.
    marker: &'static str,
    /// Which number on that line is the population, counting from zero.
    nth: usize,
    what: &'static str,
    /// A difference here is not necessarily a defect, so it is reported and
    /// does not set the exit code.
    ///
    /// `unparsed report` says "specs TRACKED"; the counter walks the disk. An
    /// untracked `.t27` makes the two differ and neither is wrong -- and a
    /// gate that goes red because somebody left a scratch spec in the tree is
    /// a gate that gets muted. `tri unparsed agree` is where that difference is
    /// examined; here it is named and passed over.
    soft: bool,
    /// Why a difference is or is not a defect, printed beside it.
    reading: &'static str,
}

const ROWS: [Row; 4] = [
    Row {
        census: "unparsed report",
        args: &["unparsed", "report"],
        marker: "specs tracked",
        nth: 0,
        what: ".t27 files",
        soft: true,
        reading: "a spec on disk that git does not track -- see `tri unparsed agree`",
    },
    Row {
        census: "quantifiers report",
        args: &["quantifiers", "report"],
        marker: "quantified clauses found",
        nth: 0,
        what: "lines carrying a quantifier keyword",
        soft: false,
        reading: "the census reads the corpus with a matcher; this reads the bare letters",
    },
    Row {
        census: "mods orphan",
        args: &["mods", "orphan"],
        marker: "files, carrying",
        nth: 1,
        what: ".rs under every workspace member",
        soft: false,
        reading: "the census walked a list; this walks the cargo workspace",
    },
    Row {
        census: "lean vacuous",
        args: &["lean", "vacuous"],
        marker: "models in the file",
        nth: 0,
        what: "`theorem` lines in Completeness.lean",
        soft: false,
        reading: "the census counts `def NAME : Module := {`; this counts the theorems, \
                  one per model, and a hand-transcribed file can gain either without the other",
    },
];

/// A census this audit does NOT check, and the reason.
///
/// Without this list the audit is narrow in exactly the way it exists to
/// catch: a page of green rows looks like coverage until somebody asks what is
/// not on it. Each entry is a measurement, not a shrug.
struct Uncovered {
    census: &'static str,
    why: &'static str,
}

const UNCOVERED: [Uncovered; 3] = [
    Uncovered {
        census: "seals hollow",
        why: "built and removed. Its counter tested json text for `\"spec_path\"` while the \
              census parses that same field, so planting one more seal moved BOTH numbers \
              to 1314 and the row stayed green. No input makes them disagree.",
    },
    Uncovered {
        census: "types dup",
        why: "measured: a counter loose enough to be independent reads 1182 where the census \
              reads 1180, and the two extra are `struct = 21,` -- enum members named `struct`, \
              which the census correctly rejects. Any counter accurate enough to agree is a \
              copy of its matcher.",
    },
    Uncovered {
        census: "discard classify",
        why: "its population is parser events produced at run time, not artefacts on disk. \
              Counting them a second way means running the same parser, which is not a \
              second opinion.",
    },
];

/// Every number on a line, in order.
fn numbers(line: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in line.chars() {
        if c.is_ascii_digit() {
            cur.push(c);
        } else if !cur.is_empty() {
            if let Ok(n) = cur.parse() {
                out.push(n);
            }
            cur.clear();
        }
    }
    if let Ok(n) = cur.parse() {
        out.push(n);
    }
    out
}

/// Run the census and read the number it publishes.
///
/// A census whose output changed shape must FAIL here rather than quietly
/// leaving the audit -- that is the same silence this file is about.
fn claimed(exe: &Path, repo: &Path, r: &Row) -> Result<Option<usize>> {
    let out = std::process::Command::new(exe)
        .args(r.args)
        .current_dir(repo)
        .output()?;
    let text =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    // A census that could not RUN is a different fact from one that disagrees,
    // and reporting the first as the second sends the reader to the wrong file.
    // `unparsed report` needs a built `t27c`; the other two need nothing.
    if !out.status.success() {
        if r.soft {
            return Ok(None);
        }
        anyhow::bail!(
            "`tri {}` exited {} -- the census could not run, so there is no claim to check. \
             First line: {:?}",
            r.census,
            out.status.code().unwrap_or(-1),
            text.lines().find(|l| !l.trim().is_empty()).unwrap_or("")
        );
    }
    let line = text.lines().find(|l| l.contains(r.marker)).ok_or_else(|| {
        anyhow::anyhow!(
            "`tri {}` prints no line containing {:?} -- either the census changed its \
                 wording or it stopped printing its population, and this audit cannot tell \
                 those apart from here",
            r.census,
            r.marker
        )
    })?;
    let ns = numbers(line);
    ns.get(r.nth).copied().map(Some).ok_or_else(|| {
        anyhow::anyhow!(
            "`tri {}`: line {:?} carries {} number(s), wanted the one at index {}",
            r.census,
            line.trim(),
            ns.len(),
            r.nth
        )
    })
}

fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if p.is_dir() {
            if name == ".git" || name == "target" || name == "node_modules" {
                continue;
            }
            walk(&p, ext, out);
        } else if name.ends_with(ext) {
            out.push(p);
        }
    }
}

/// `//` comments and string bodies blanked, columns kept. Written here rather
/// than shared with `quant.rs`: a masker shared with the census under test is
/// not a second opinion.
fn bare(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let b: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i];
        if in_str {
            out.push(' ');
            if c == '"' && (i == 0 || b[i - 1] != '\\') {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
            out.push(' ');
        } else if c == '/' && i + 1 < b.len() && b[i + 1] == '/' {
            while i < b.len() {
                out.push(' ');
                i += 1;
            }
            break;
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

fn ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Members of the cargo workspace, read from the manifest.
fn members(repo: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(repo.join("Cargo.toml"))?;
    let i = text
        .find("members")
        .ok_or_else(|| anyhow::anyhow!("Cargo.toml names no `members`"))?;
    let rest = &text[i..];
    let a = rest
        .find('[')
        .ok_or_else(|| anyhow::anyhow!("`members` is not a list"))?;
    let b = rest
        .find(']')
        .ok_or_else(|| anyhow::anyhow!("`members` is not closed"))?;
    Ok(rest[a + 1..b]
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// The population, counted without asking the census.
fn independent(census: &str, repo: &Path) -> Result<usize> {
    match census {
        "unparsed report" => {
            let mut v = Vec::new();
            walk(repo, ".t27", &mut v);
            Ok(v.len())
        }
        "quantifiers report" => {
            let mut v = Vec::new();
            walk(&repo.join("specs"), ".t27", &mut v);
            let mut extra = Vec::new();
            walk(&repo.join("compiler"), ".t27", &mut extra);
            v.extend(extra);
            const KW: [&str; 4] = ["forall", "for all", "for any", "for positive"];
            let mut n = 0usize;
            for p in &v {
                let Ok(src) = std::fs::read_to_string(p) else {
                    continue;
                };
                for raw in src.lines() {
                    let t = bare(raw);
                    let t = t.trim();
                    let bs = t.as_bytes();
                    if KW.iter().any(|k| {
                        t.match_indices(k).any(|(i, _)| {
                            let j = i + k.len();
                            (i == 0 || !ident(bs[i - 1])) && (j >= bs.len() || !ident(bs[j]))
                        })
                    }) {
                        n += 1;
                    }
                }
            }
            Ok(n)
        }
        "mods orphan" => {
            let mut n = 0usize;
            for m in members(repo)? {
                let src = repo.join(&m).join("src");
                if !src.is_dir() {
                    continue;
                }
                let mut v = Vec::new();
                walk(&src, ".rs", &mut v);
                n += v.len();
            }
            Ok(n)
        }
        "lean vacuous" => {
            // A different marker for the same population: the census counts
            // `def NAME : Module := {`, this counts the theorem each model is
            // supposed to carry. The file is found by name rather than by the
            // path the census uses.
            let mut v = Vec::new();
            walk(&repo.join("proofs"), "Completeness.lean", &mut v);
            let p = v.first().ok_or_else(|| {
                anyhow::anyhow!("no Completeness.lean under proofs/ -- nothing to count")
            })?;
            let src = std::fs::read_to_string(p)?;
            Ok(src
                .lines()
                .filter(|l| l.trim_start().starts_with("theorem "))
                .count())
        }
        other => anyhow::bail!("no independent counter for {other}"),
    }
}

/// Break a sentence into lines of at most `w` characters, on word boundaries.
fn wrap(s: &str, w: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        if !cur.is_empty() && cur.len() + 1 + word.len() > w {
            out.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

pub fn run(cmd: &CensusCmd) -> Result<()> {
    match cmd {
        CensusCmd::Pin { gate, bless } => return pin(*gate, *bless),
        CensusCmd::Explain { max } => return explain(*max),
        CensusCmd::Audit => {}
    }
    audit()
}

fn audit() -> Result<()> {
    let repo = repo_root()?;
    let exe = std::env::current_exe()?;

    println!();
    println!("  Each census prints a population. Each row below counts the same");
    println!("  population a different way and subtracts. A counter sharing the");
    println!("  census's own reader would agree by construction.");
    println!();
    println!(
        "      {:<22}{:>10}{:>14}   population",
        "census", "printed", "counted here"
    );

    let mut bad: Vec<String> = Vec::new();
    let mut noted: Vec<String> = Vec::new();
    for r in &ROWS {
        let Some(said) = claimed(&exe, &repo, r)? else {
            println!(
                "      {:<22}{:>10}{:>14}   {}   <- did not run",
                r.census, '-', '-', r.what
            );
            noted.push(format!(
                "{}: the census exited non-zero, so it made no claim to check",
                r.census
            ));
            continue;
        };
        let mine = independent(r.census, &repo)?;
        let mark = match (said == mine, r.soft) {
            (true, _) => "",
            (false, true) => "   <- differs",
            (false, false) => "   <- DISAGREE",
        };
        println!(
            "      {:<22}{said:>10}{mine:>14}   {}{mark}",
            r.census, r.what
        );
        if said != mine {
            let line = format!(
                "{}: prints {said}, {} counts {mine} -- {}",
                r.census, r.what, r.reading
            );
            if r.soft {
                noted.push(line);
            } else {
                bad.push(line);
            }
        }
    }
    println!();
    for n in &noted {
        println!("      note  {n}");
    }
    if !noted.is_empty() {
        println!();
    }

    if !bad.is_empty() {
        for b in &bad {
            println!("      {b}");
        }
        println!();
        anyhow::bail!(
            "{} census(es) speak about a population they did not count. One of the two \
             routes is filtering something it does not admit to.",
            bad.len()
        );
    }
    println!("  AGREED. Every census speaks about the population it prints.");
    println!();
    println!("  Not checked here, and why -- because a page of green rows looks");
    println!("  like coverage until somebody asks what is missing from it:");
    for u in &UNCOVERED {
        println!();
        println!("      {}", u.census);
        for chunk in wrap(u.why, 68) {
            println!("          {chunk}");
        }
    }
    Ok(())
}

/// A printed line with each digit run folded to `#` and its spacing collapsed,
/// so `run: steps   285` and `run: steps  1000` are one row read twice.
fn row_key(line: &str) -> String {
    let mut out = String::new();
    let mut in_num = false;
    for c in line.chars() {
        if c.is_ascii_digit() {
            if !in_num {
                out.push('#');
            }
            in_num = true;
        } else {
            in_num = false;
            out.push(c);
        }
    }
    squash(&out)
}

fn squash(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One printed line that differs between two readings of a census.
#[derive(Debug, PartialEq, Eq)]
enum Moved {
    /// The same row, another reading.
    Reads { was: String, now: String },
    /// A row only the earlier reading printed.
    Gone(String),
    /// A row only the later reading printed.
    New(String),
}

impl Moved {
    /// `run: steps 285->286` when only numbers changed, which is the usual case.
    fn show(&self) -> String {
        match self {
            Moved::Reads { was, now } => {
                let (a, b) = (digit_runs(was), digit_runs(now));
                if a.len() != b.len() || a.iter().zip(&b).any(|(x, y)| x.0 != y.0) {
                    return format!("{was}\n          -> {now}");
                }
                a.iter()
                    .zip(&b)
                    .map(|(x, y)| {
                        if x.1 == y.1 {
                            y.1.clone()
                        } else {
                            format!("{}->{}", x.1, y.1)
                        }
                    })
                    .collect()
            }
            Moved::Gone(l) => format!("gone: {l}"),
            Moved::New(l) => format!("new:  {l}"),
        }
    }
}

/// A line cut into alternating runs of digits and of everything else.
fn digit_runs(s: &str) -> Vec<(bool, String)> {
    let mut out: Vec<(bool, String)> = Vec::new();
    for c in s.chars() {
        let d = c.is_ascii_digit();
        match out.last_mut() {
            Some((kind, run)) if *kind == d => run.push(c),
            _ => out.push((d, c.to_string())),
        }
    }
    out
}

/// Every line that differs between two readings, rows matched by `row_key`.
///
/// `pin --gate` prints the FIRST differing line and stops. That says a census
/// moved and not what moved it: on t27#5787 the fetches census moved 4 -> 5 and
/// the line that named the cause ("0 do not" -> "1 do not") was the second.
fn moved_lines(was: &str, now: &str) -> Vec<Moved> {
    use std::collections::BTreeMap;
    let rows = |s: &str| -> Vec<String> {
        s.lines()
            .filter(|l| !l.trim().is_empty())
            .map(String::from)
            .collect()
    };
    let (was, now) = (rows(was), rows(now));
    let mut old: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    for l in &was {
        old.entry(row_key(l)).or_default().push(l);
    }
    let mut out = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for l in &now {
        let k = row_key(l);
        let i = seen.entry(k.clone()).or_insert(0);
        match old.get(&k).and_then(|v| v.get(*i)) {
            Some(w) if squash(w) == squash(l) => {}
            Some(w) => out.push(Moved::Reads {
                was: squash(w),
                now: squash(l),
            }),
            None => out.push(Moved::New(squash(l))),
        }
        *i += 1;
    }
    let mut taken: BTreeMap<String, usize> = BTreeMap::new();
    for l in &was {
        let k = row_key(l);
        let i = taken.entry(k.clone()).or_insert(0);
        if *i >= seen.get(&k).copied().unwrap_or(0) {
            out.push(Moved::Gone(squash(l)));
        }
        *i += 1;
    }
    out
}

/// `git` with optional locks off: `git diff` against the working tree would
/// otherwise refresh the index, and this command says it writes nothing.
fn git_bytes(repo: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}

fn git_paths(repo: &Path, args: &[&str]) -> Result<Vec<String>> {
    let raw = git_bytes(repo, args)?;
    let mut v: Vec<String> = raw
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8_lossy(p).to_string())
        .collect();
    v.sort();
    v.dedup();
    Ok(v)
}

/// The working tree -- tracked and untracked files, not ignored ones -- copied
/// into a fresh repository of its own, and deleted when this is dropped.
///
/// A whole copy rather than the census's own directory: `gates quiet` asks
/// whether each path a step names exists, so a copy of `.github/workflows`
/// alone would read every subject as absent and explain a difference it made.
struct Scratch {
    dir: PathBuf,
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

impl Scratch {
    fn new(repo: &Path) -> Result<Scratch> {
        // A whole working-tree copy is a few hundred MB, and `kill -9` runs no
        // Drop: a killed run's copy is removed here, by the next one (#5982).
        for gone in crate::piddir::sweep_dead(&std::env::temp_dir(), "tri-census-explain-") {
            eprintln!(
                "tri census: removed {}, left by a run that is no longer alive",
                gone.display()
            );
        }
        let dir = std::env::temp_dir().join(format!("tri-census-explain-{}", std::process::id()));
        if dir.exists() {
            anyhow::bail!("{} already exists; refusing to reuse it", dir.display());
        }
        std::fs::create_dir_all(&dir)?;
        let s = Scratch { dir };
        for p in git_paths(repo, &["ls-files", "-z", "-co", "--exclude-standard"])? {
            s.copy_in(repo, &p)?;
        }
        let init = std::process::Command::new("git")
            .arg("-C")
            .arg(&s.dir)
            .args(["init", "-q"])
            .output()?;
        if !init.status.success() {
            anyhow::bail!("git init in {} failed", s.dir.display());
        }
        Ok(s)
    }

    /// Make `rel` in the copy what it is in the tree: copied, or absent.
    fn copy_in(&self, repo: &Path, rel: &str) -> Result<()> {
        let (src, dst) = (repo.join(rel), self.dir.join(rel));
        let _ = std::fs::remove_file(&dst);
        let Ok(meta) = std::fs::symlink_metadata(&src) else {
            return Ok(());
        };
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if meta.file_type().is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(std::fs::read_link(&src)?, &dst)?;
        } else if meta.is_file() {
            std::fs::copy(&src, &dst)?;
        }
        Ok(())
    }

    /// Make `rel` in the copy what it was at `commit`: its blob, or absent.
    fn put_back(&self, repo: &Path, commit: &str, rel: &str) -> Result<()> {
        let dst = self.dir.join(rel);
        let _ = std::fs::remove_file(&dst);
        let spec = format!("{commit}:{rel}");
        let there = std::process::Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["cat-file", "-e", &spec])
            // "absent at that commit" is an answer here, not an error to print.
            .stderr(std::process::Stdio::null())
            .status()?
            .success();
        if there {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dst, git_bytes(repo, &["show", &spec])?)?;
        }
        Ok(())
    }
}

/// What a moved census's change is made of, one file at a time.
fn explain(max: usize) -> Result<()> {
    let repo = repo_root()?;
    let dir = ledger_dir()?;
    let mut scratch: Option<Scratch> = None;
    let mut any_moved = false;

    println!("tri census explain -- what moved each pinned census since it was blessed");
    println!();
    for (name, args) in PINNED {
        let ledger = dir.join(format!("{name}.txt"));
        let Ok(was) = std::fs::read_to_string(&ledger) else {
            any_moved = true;
            println!(
                "  {name:<8} NO LEDGER at {} -- nothing to explain against",
                ledger.display()
            );
            continue;
        };
        let now = run_census(args)?;
        if was == now {
            println!("  {name:<8} unchanged");
            continue;
        }
        any_moved = true;
        println!();
        let rel = format!("tools/census/{name}.txt");
        let bless = String::from_utf8_lossy(&git_bytes(
            &repo,
            &["log", "-1", "--format=%H", "--", &rel],
        )?)
        .trim()
        .to_string();
        let short = &bless[..bless.len().min(9)];
        println!(
            "  {name:<8} MOVED since blessed{} ({rel})",
            if bless.is_empty() {
                String::new()
            } else {
                format!(" at {short}")
            }
        );
        let lines = moved_lines(&was, &now);
        if lines.is_empty() {
            println!("      the difference is spacing or blank lines only");
        }
        for m in &lines {
            println!("      {}", m.show());
        }
        if bless.is_empty() {
            println!(
                "    The ledger was never committed, so there is no commit to put files back to."
            );
            continue;
        }

        if scratch.is_none() {
            scratch = Some(Scratch::new(&repo)?);
        }
        let s = scratch.as_ref().expect("made above");
        let base = run_census_in(args, Some(&s.dir))?;
        if base != now {
            let first = moved_lines(&now, &base);
            println!(
                "    The scratch copy reads this census differently from the tree ({}),",
                first
                    .first()
                    .map(|m| m.show())
                    .unwrap_or_else(|| "spacing".into())
            );
            println!("    so a per-file answer would describe the copy. Not attempted.");
            continue;
        }

        let mut files = git_paths(
            &repo,
            &["diff", "--name-only", "--no-renames", "-z", &bless],
        )?;
        files.extend(git_paths(
            &repo,
            &["ls-files", "-z", "-o", "--exclude-standard"],
        )?);
        files.retain(|f| !f.starts_with("tools/census/"));
        files.sort();
        files.dedup();
        let total = files.len();
        files.truncate(max);

        println!(
            "    which change moves it -- each changed file put back to {short} alone, the census run again:"
        );
        let mut movers = 0usize;
        for f in &files {
            s.put_back(&repo, &bless, f)?;
            let alone = run_census_in(args, Some(&s.dir));
            s.copy_in(&repo, f)?;
            let alone = match alone {
                Ok(text) => text,
                Err(e) => {
                    movers += 1;
                    println!("      {f}   the census could not run with it put back: {e}");
                    continue;
                }
            };
            if alone == now {
                continue;
            }
            movers += 1;
            let mine = moved_lines(&alone, &now);
            println!("      {f}   moves {} line(s)", mine.len().max(1));
            for m in &mine {
                println!("          {}", m.show());
            }
        }
        println!(
            "    {total} file(s) changed since {short}: {} tried, {movers} move this census, {} do not alone.",
            files.len(),
            files.len() - movers
        );
        if files.len() < total {
            println!(
                "    {} NOT tried (`--max {max}`): the answer below covers the tried ones only.",
                total - files.len()
            );
        }
        for f in &files {
            s.put_back(&repo, &bless, f)?;
        }
        let together = run_census_in(args, Some(&s.dir));
        for f in &files {
            s.copy_in(&repo, f)?;
        }
        let together = match together {
            Ok(text) => text,
            Err(e) => {
                println!("    All tried files put back together: the census could not run ({e}).");
                continue;
            }
        };
        if together == was {
            println!("    All tried files put back together: reproduces the ledger -- yes.");
        } else {
            let rest = moved_lines(&was, &together);
            println!(
                "    All tried files put back together: reproduces the ledger -- NO, {} line(s) still differ:",
                rest.len()
            );
            for m in rest.iter().take(5) {
                println!("          {}", m.show());
            }
            println!("    That remainder is not in the files: the census code itself (this build), or untried files.");
        }
    }

    println!();
    println!("NOT ESTABLISHED: a change that moves a number only together with another");
    println!("reads as moving nothing alone -- the together line is where that shows.");
    println!("The census code is this build: putting back a census's own source moves");
    println!("what it reads, not how. Nothing in the repository was written; the scratch");
    println!("copy is deleted on exit.");
    // `process::exit` runs no destructors: without this the copy of the whole
    // tree outlives every run that has something to explain.
    drop(scratch);
    if any_moved {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod explain_tests {
    use super::{moved_lines, row_key, Moved};

    #[test]
    fn a_row_is_its_words_not_its_numbers_or_spacing() {
        assert_eq!(row_key("  run: steps   285"), "run: steps #");
        assert_eq!(row_key("run: steps 1000"), "run: steps #");
        assert_eq!(row_key("bash 264 (0 fatal)"), "bash # (# fatal)");
        assert_ne!(row_key("jobs 3"), row_key("run: steps 3"));
    }

    #[test]
    fn every_moved_line_is_listed_not_only_the_first() {
        let was = "files read 4\nrun: steps 285\nbash 264\nunchanged 7\n";
        let now = "files read 4\nrun: steps 286\nbash 265\nunchanged 7\n";
        let m = moved_lines(was, now);
        assert_eq!(m.len(), 2, "{m:?}");
        assert_eq!(m[0].show(), "run: steps 285->286");
        assert_eq!(m[1].show(), "bash 264->265");
    }

    #[test]
    fn a_line_with_new_words_is_new_and_its_old_self_is_gone() {
        let m = moved_lines("a 1\nold row 2\n", "a 1\nnew row 2\n");
        assert_eq!(
            m,
            vec![
                Moved::New("new row 2".into()),
                Moved::Gone("old row 2".into())
            ]
        );
    }

    #[test]
    fn repeated_rows_pair_in_order_and_spacing_alone_is_no_move() {
        let was = "x 1\nx 2\n\n  y   3\n";
        let now = "x 1\nx 5\ny 3\n";
        assert_eq!(
            moved_lines(was, now),
            vec![Moved::Reads {
                was: "x 2".into(),
                now: "x 5".into()
            }]
        );
        assert!(moved_lines("a  1\n\n", "a 1\n").is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The population is the number the census PUBLISHES, taken off its own
    /// line -- so the reader has to agree with each census's wording.
    #[test]
    fn the_population_is_read_off_the_line_the_census_prints() {
        assert_eq!(numbers("  specs tracked          745"), vec![745]);
        // `mods orphan` writes "7 of 136 files": the population is the second.
        assert_eq!(
            numbers("  7 of 136 files, carrying 0 test(s) that do not exist"),
            vec![7, 136, 0]
        );
        assert_eq!(numbers("  quantified clauses found      923"), vec![923]);
        assert_eq!(numbers("no digits here"), Vec::<usize>::new());
    }

    /// A comment or a string is not source. Written here rather than shared
    /// with the census under test, which is the whole point of the file.
    #[test]
    fn a_keyword_in_a_comment_or_a_string_is_not_source() {
        assert_eq!(bare("forall x : u8").trim(), "forall x : u8");
        assert_eq!(bare("// forall x : u8").trim(), "");
        assert_eq!(bare("let s = \"forall\";").trim_end(), "let s =         ;");
        // The comment is blanked and the columns before it are kept.
        assert_eq!(bare("forall a // forall b").trim_end(), "forall a");
        assert_eq!(bare("forall a // forall b").len(), 20, "columns preserved");
    }

    /// Every row has a counter.
    ///
    /// `independent` bails on a name it does not know, so a row added without
    /// one fails at run time, on somebody else's commit. This says it at build
    /// time instead.
    #[test]
    fn every_row_has_an_independent_counter() {
        let nowhere = std::path::Path::new("/nonexistent-census-audit-probe");
        for r in &ROWS {
            let e = independent(r.census, nowhere);
            if let Err(e) = e {
                assert!(
                    !e.to_string().contains("no independent counter"),
                    "{} has no counter in `independent`",
                    r.census
                );
            }
        }
    }

    /// A census is either checked or explicitly not, never neither.
    ///
    /// The audit's own coverage is the same class it exists to catch: a page
    /// of green rows looks like the whole story. A name in both lists, or an
    /// exclusion with no measurement behind it, puts it back there.
    #[test]
    fn nothing_is_both_checked_and_excused() {
        for u in &UNCOVERED {
            assert!(
                !ROWS.iter().any(|r| r.census == u.census),
                "{} is listed as unchecked and also has a row",
                u.census
            );
            assert!(
                u.why.len() > 60,
                "{}: an exclusion is a measurement, not a shrug -- {:?}",
                u.census,
                u.why
            );
        }
    }

    /// The reasons are printed, so they have to fit the page.
    #[test]
    fn a_reason_wraps_on_word_boundaries() {
        let w = wrap("one two three four five", 9);
        assert_eq!(w, vec!["one two", "three", "four five"]);
        assert!(wrap("", 10).is_empty());
        // A word longer than the width is not cut in half.
        assert_eq!(
            wrap("supercalifragilistic ok", 8),
            vec!["supercalifragilistic", "ok"]
        );
    }

    /// A soft row states why its difference is not a defect.
    ///
    /// Without that sentence a reader cannot tell "measured and forgiven" from
    /// "nobody looked", which is the distinction this whole file is about.
    #[test]
    fn a_soft_row_says_why_it_is_soft() {
        for r in &ROWS {
            assert!(
                !r.reading.is_empty(),
                "{} prints a difference with no reading",
                r.census
            );
        }
    }
}

#[cfg(test)]
mod pin_tests {
    use super::PINNED;

    /// The case an A/B of the output cannot see. Drop a name from `PINNED` and
    /// every remaining reading still matches, so the gate goes green having
    /// quietly stopped watching something -- the silent-drop that a comparison
    /// of numbers is structurally blind to. `insta` ships this as
    /// `--unreferenced=reject`; the survey named it and this repository had the
    /// hole.
    ///
    /// Pinned as a name check rather than a filesystem walk so it holds in any
    /// checkout: every ledger this command will ever write is named for an entry
    /// in `PINNED`, so a `tools/census/*.txt` outside that set is unreferenced
    /// by construction.
    #[test]
    fn a_ledger_with_no_census_is_a_thing_nobody_watches() {
        let names: Vec<&str> = PINNED.iter().map(|(n, _)| *n).collect();
        assert!(
            names.contains(&"fetches") && names.contains(&"quiet") && names.contains(&"shell"),
            "the three committed ledgers must each still have a census that \
             regenerates them, or the gate is green over a file nothing writes"
        );
        assert_eq!(
            names.len(),
            names
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            "a duplicated entry would write one ledger twice and hide the other"
        );
    }

    /// Pinning a census that reads the GitHub API would redden this gate when
    /// SOMEBODY ELSE pushes, which is how a gate gets muted. The exclusion is a
    /// measurement, not a preference: `dead` takes over four minutes and
    /// `unmeasured` about fifty seconds because both walk the API, and their
    /// answers move with the world rather than with the tree.
    #[test]
    fn no_pinned_census_reads_the_network() {
        for networked in ["dead", "unmeasured", "required", "prs"] {
            assert!(
                !PINNED.iter().any(|(n, _)| *n == networked),
                "`{networked}` reads the API: pinning it makes this gate fail on \
                 changes that are not in this repository at all"
            );
        }
    }

    /// Every pinned census must walk a DIRECTORY, because that is what makes the
    /// pin cheap: it cannot move unless a file in that directory changes.
    /// Measured over 39 transitions on master with one fixed instrument -- 8
    /// moved a census and all 8 had edited that census's own subject.
    #[test]
    fn every_pinned_census_is_a_gates_subcommand_over_the_tree() {
        assert!(!PINNED.is_empty(), "an empty pin list watches nothing");
        for (name, args) in PINNED {
            assert_eq!(
                args.first().copied(),
                Some("gates"),
                "`{name}` is pinned but is not a `gates` census; the cheapness \
                 argument rests on its population being a directory"
            );
            assert_eq!(args.len(), 2, "`{name}` should be `gates <name>`");
            assert_eq!(args[1], *name, "the ledger file and the command must agree");
        }
    }
}
