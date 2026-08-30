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

const ROWS: [Row; 3] = [
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
        other => anyhow::bail!("no independent counter for {other}"),
    }
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
    let CensusCmd::Audit = cmd;
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
    Ok(())
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
