//! Open issues whose headline calls a workflow red, against what that workflow
//! does on master today.
//!
//! An open issue is a claim with a date on it: *this is broken*. The repository
//! holds 477 of them, and 268 carry a number in the title -- a stated
//! measurement of the tree, taken once. Nothing re-reads them.
//!
//! One subset can be re-measured exactly, because its truth is defined outside
//! the repository: an issue whose title says a named workflow is red. GitHub
//! records what that workflow last concluded on master, so the claim can be
//! checked without judgement.
//!
//! **What a green reading here does NOT mean.** It does not say the issue is
//! resolved, and this command never suggests closing one. An issue titled
//! "cli-tri has been red on master for three days" also argues that a `paths:`
//! filter kept it from running, and that argument can outlive the redness
//! entirely. What a green reading says is narrower and still worth having: the
//! sentence at the top of that issue is no longer true, so a reader who stops
//! at the title is misled about the state of the tree.

use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, clap::Subcommand)]
pub enum IssuesCmd {
    /// Open issues whose title calls a workflow red, and what it does today.
    Stale {
        /// How many open issues to read. The default is the repository's
        /// current open count rounded up; a smaller number is a SAMPLE and the
        /// output says so rather than presenting it as a census.
        #[arg(long, default_value_t = 500)]
        limit: usize,
    },
    /// Open issues that state a COUNT in the title, and a reproducible sample.
    Numbers {
        /// Print a systematic sample of this size. 0 prints only the population.
        #[arg(long, default_value_t = 0)]
        sample: usize,
        /// How many open issues to read.
        #[arg(long, default_value_t = 500)]
        limit: usize,
    },
    /// Open issues whose figure is ANCHORED, so re-measuring it proves nothing.
    Dated {
        /// How many open issues to read.
        #[arg(long, default_value_t = 500)]
        limit: usize,
        /// Also print the anchored issues, one line each.
        #[arg(long)]
        list: bool,
    },
}

/// The phrases a title uses to call something red.
///
/// Matched against the TITLE only. A body may mention redness in passing --
/// recounting history, or describing what a fix prevented -- while the issue is
/// about something else entirely; the title is where the claim is made.
pub fn claims_red(title: &str) -> bool {
    let t = title.to_lowercase();
    const PHRASES: [&str; 7] = [
        "red on master",
        "cannot succeed",
        "never once succeeded",
        "never succeeded",
        "landed red",
        "suite is red",
        "is red and",
    ];
    if PHRASES.iter().any(|p| t.contains(p)) {
        return true;
    }
    // "failed 62 of 62 runs", "failed 8 of 8"
    if let Some(at) = t.find("failed ") {
        let rest = &t[at + 7..];
        if rest.starts_with(|c: char| c.is_ascii_digit()) && rest.contains(" of ") {
            return true;
        }
    }
    // "has been red for three days", "red for 3 days"
    t.contains("red for ") && t.contains("day")
}

/// Every name a workflow answers to, mapped back to its file.
///
/// Three keys per workflow, and the third is the one that matters: issues name
/// a workflow the way GitHub DISPLAYS it. `seal-coverage.yml` displays as
/// "Seal Coverage", and #2851 -- "Seal Coverage has been red on master" -- is
/// invisible to a reader keyed on the file stem alone. Measured on this tree:
/// 22 of 49 workflows have a `name:` that differs from their stem, and adding
/// the display name took this command's population from 8 to 9.
pub fn workflow_keys(dir: &Path) -> Result<BTreeMap<String, String>> {
    let mut keys = BTreeMap::new();
    let rd = std::fs::read_dir(dir)
        .with_context(|| format!("{}: cannot read the workflow directory", dir.display()))?;
    let mut files = 0usize;
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("yml") {
            continue;
        }
        let file = match p.file_name().and_then(|x| x.to_str()) {
            Some(f) => f.to_string(),
            None => continue,
        };
        files += 1;
        let stem = file.trim_end_matches(".yml").to_string();
        keys.insert(file.clone(), file.clone());
        keys.insert(stem, file.clone());
        if let Ok(text) = std::fs::read_to_string(&p) {
            for line in text.lines() {
                if let Some(rest) = line.strip_prefix("name:") {
                    let disp = rest.trim().trim_matches(['"', '\'']).to_string();
                    if !disp.is_empty() {
                        keys.insert(disp, file.clone());
                    }
                    break;
                }
            }
        }
    }
    if files == 0 {
        anyhow::bail!(
            "{}: no *.yml found. A zero here would read as \"no issue names a \
             workflow\", which is a different statement.",
            dir.display()
        );
    }
    Ok(keys)
}

/// Is `key` present in `hay` as a whole token?
///
/// Workflow names contain hyphens and spaces, so the boundary cannot be the
/// usual word boundary: `cli-tri` must not match inside `cli-tri-mcp`, and
/// `release` must not match inside `pre-release`.
fn names(hay: &str, key: &str) -> bool {
    let bound = |c: char| !(c.is_alphanumeric() || c == '-' || c == '_');
    let mut from = 0usize;
    while let Some(rel) = hay[from..].find(key) {
        let i = from + rel;
        let j = i + key.len();
        let before = hay[..i].chars().next_back().is_none_or(bound);
        let after = hay[j..].chars().next().is_none_or(bound);
        if before && after {
            return true;
        }
        from = i + 1;
    }
    false
}

/// The workflows an issue names, by file.
pub fn named_workflows(text: &str, keys: &BTreeMap<String, String>) -> Vec<String> {
    let mut out: Vec<String> = keys
        .iter()
        .filter(|(k, _)| names(text, k))
        .map(|(_, v)| v.clone())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// What kind of number a title carries.
///
/// The distinction is the whole point. `Wave Loop 369` and `#2841` and `Prop. 65`
/// are ADDRESSES -- they identify a thing, they measure nothing, and a reader who
/// counts them is counting the tracker's own numbering. `Twelve quantified
/// clauses call a function with the wrong number of arguments` is a COUNT, and it
/// is written in words, which a digit matcher cannot see at all.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Carries {
    /// A count in digits, after every address has been removed.
    Digits,
    /// A count written as a numeral word.
    Words,
    /// Both.
    Both,
    /// Only a quantifier -- `every`, `all`, `none`, `half`. Measurable in
    /// principle and not a number, so counting it would inflate the population
    /// with claims that have no figure to re-read.
    QuantifierOnly,
    /// Nothing to re-measure.
    None,
}

/// Strip the spellings this repository uses to ADDRESS things.
///
/// Measured on 478 open issues: a matcher that reads any two-digit run in a
/// title reports 329, of which **145 are addresses** -- 44% of the population is
/// the tracker's own numbering. Every one of those has nothing to re-measure.
pub fn strip_addresses(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let b: Vec<char> = title.chars().collect();
    let mut i = 0usize;
    let low: String = title.to_lowercase();
    let lb: Vec<char> = low.chars().collect();
    // the prefixes that make a number an address, longest first
    const PRE: [&str; 7] = [
        "wave loop ",
        "wave ",
        "prop. ",
        "prop ",
        "adr-",
        "rfc-",
        "ci-",
    ];
    while i < b.len() {
        // #1234
        if b[i] == '#' && b.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            out.push(' ');
            continue;
        }
        // w699 / W12 -- a bare w followed by digits, on a word boundary
        if (b[i] == 'w' || b[i] == 'W')
            && b.get(i + 1).is_some_and(|c| c.is_ascii_digit())
            && (i == 0 || !b[i - 1].is_alphanumeric())
        {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j - i >= 3 && (j >= b.len() || !b[j].is_alphanumeric()) {
                out.push(' ');
                i = j;
                continue;
            }
        }
        // wave loop 369 / prop. 65 / ci-01
        let mut matched = false;
        for p in PRE {
            let pc: Vec<char> = p.chars().collect();
            if i + pc.len() <= lb.len()
                && lb[i..i + pc.len()] == pc[..]
                && (i == 0 || !b[i - 1].is_alphanumeric())
            {
                let mut j = i + pc.len();
                let start = j;
                while j < b.len() && b[j].is_ascii_digit() {
                    j += 1;
                }
                if j > start {
                    out.push(' ');
                    i = j;
                    matched = true;
                    break;
                }
            }
        }
        if matched {
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

/// Numeral words. `every`, `all`, `none` and `half` are deliberately NOT here:
/// they quantify without giving a figure, so a reader has nothing to re-measure.
const NUMERALS: [&str; 26] = [
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
    "thirty",
    "forty",
    "fifty",
    "hundred",
    "thousand",
];
const QUANTIFIERS: [&str; 4] = ["every", "all", "none", "half"];

fn has_word(hay: &str, w: &str) -> bool {
    let h = hay.to_lowercase();
    let mut from = 0usize;
    while let Some(rel) = h[from..].find(w) {
        let i = from + rel;
        let j = i + w.len();
        let l = h[..i]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric());
        let r = h[j..].chars().next().is_none_or(|c| !c.is_alphanumeric());
        if l && r {
            return true;
        }
        from = i + 1;
    }
    false
}

/// What a title carries, once addresses are gone.
pub fn carries(title: &str) -> Carries {
    let t = strip_addresses(title);
    // A digit run counts only when nothing alphanumeric touches it. Without
    // that boundary the rule fires inside identifiers -- `t27`, `GF16`,
    // `dlc10`, `SRL16E`, `0o777` -- and reports twelve issues here whose
    // titles state no count at all. Found by running an independent reader
    // over the same backlog and subtracting: 295 against 283, and the Rust was
    // a strict superset, which is what an over-loose matcher looks like.
    let digits = {
        let c: Vec<char> = t.chars().collect();
        let mut i = 0usize;
        let mut found = false;
        while i < c.len() {
            if !c[i].is_ascii_digit() {
                i += 1;
                continue;
            }
            let start = i;
            while i < c.len() && c[i].is_ascii_digit() {
                i += 1;
            }
            let left = start == 0 || !c[start - 1].is_alphanumeric();
            let right = i >= c.len() || !c[i].is_alphanumeric();
            if i - start >= 2 && left && right {
                found = true;
            }
        }
        found
    };
    let words = NUMERALS.iter().any(|w| has_word(&t, w));
    match (digits, words) {
        (true, true) => Carries::Both,
        (true, false) => Carries::Digits,
        (false, true) => Carries::Words,
        (false, false) => {
            if QUANTIFIERS.iter().any(|w| has_word(&t, w)) {
                Carries::QuantifierOnly
            } else {
                Carries::None
            }
        }
    }
}

/// The population of re-measurable open issues, and a reproducible sample of it.
///
/// A rate is only worth taking if the same sample can be taken again, so the
/// sample is SYSTEMATIC -- every k-th issue by ascending number -- rather than
/// chosen. Nothing here is random: run it next month and the overlap is exact
/// wherever the backlog has not moved.
fn numbers(sample: usize, limit: usize) -> Result<()> {
    let lim = limit.to_string();
    let raw = gh(&[
        "issue",
        "list",
        "--state",
        "open",
        "--limit",
        &lim,
        "--json",
        "number,title",
    ])?;
    let v: serde_json::Value = serde_json::from_str(&raw).context("gh returned no JSON")?;
    let arr = v.as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        anyhow::bail!(
            "gh returned no open issues -- a repository with none and a query that \
             did not run print the same zero."
        );
    }
    let mut rows: Vec<(u64, String, Carries)> = arr
        .iter()
        .map(|i| {
            let n = i["number"].as_u64().unwrap_or(0);
            let t = i["title"].as_str().unwrap_or("").to_string();
            let c = carries(&t);
            (n, t, c)
        })
        .collect();
    rows.sort_by_key(|r| r.0);

    let c = |k: Carries| rows.iter().filter(|r| r.2 == k).count();
    let pop: Vec<&(u64, String, Carries)> = rows
        .iter()
        .filter(|r| matches!(r.2, Carries::Digits | Carries::Words | Carries::Both))
        .collect();

    println!("OPEN ISSUES THAT STATE A COUNT IN THE TITLE\n");
    println!("  open issues read              {}", rows.len());
    println!("  count in digits only          {}", c(Carries::Digits));
    println!("  count in words only           {}", c(Carries::Words));
    println!("  both                          {}", c(Carries::Both));
    println!("  POPULATION                    {}", pop.len());
    println!(
        "  quantifier only, excluded     {}",
        c(Carries::QuantifierOnly)
    );
    println!("  no figure                     {}", c(Carries::None));

    println!(
        "\n  An ADDRESS is not a count. `#2841`, `Wave Loop 369`, `Prop. 65`, `w699`\n  \
         and `CI-01` identify a thing and measure nothing. A matcher reading any\n  \
         two-digit run reports 329 on this backlog, of which 145 -- 44% -- are\n  \
         addresses, and every one of them has nothing to re-read.\n\n  \
         A count written in WORDS is still a count. \"Twelve quantified clauses\n  \
         call a function with the wrong number of arguments\" is re-measurable and\n  \
         invisible to a digit matcher: {} issues here state their figure only in\n  \
         words. Reading digits alone gets the population wrong in BOTH directions.\n\n  \
         `every`, `all`, `none` and `half` quantify without giving a figure, so\n  \
         they are excluded and counted separately rather than dropped in silence.",
        c(Carries::Words)
    );

    if sample > 0 {
        let k = pop.len().checked_div(sample).unwrap_or(1).max(1);
        let picked: Vec<&&(u64, String, Carries)> = pop.iter().step_by(k).take(sample).collect();
        println!(
            "\n  SYSTEMATIC SAMPLE -- every {k}th of {} by ascending number, {} taken.\n  \
             Not random and not chosen: re-run it and the overlap is exact wherever\n  \
             the backlog has not moved, which is what makes a rate comparable.\n",
            pop.len(),
            picked.len()
        );
        for (n, t, _) in picked.iter().copied() {
            println!("    #{n}  {}", &t[..t.len().min(92)]);
        }
    }
    Ok(())
}

fn gh(args: &[&str]) -> Result<String> {
    let out = Command::new("gh")
        .args(args)
        .output()
        .context("gh is not installed or not on PATH")?;
    if !out.status.success() {
        anyhow::bail!(
            "gh {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn repo_root() -> Result<std::path::PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(std::path::PathBuf::from(
        String::from_utf8(out.stdout)?.trim(),
    ))
}

/// One issue and the verdict of every workflow it names.
struct Row {
    number: u64,
    title: String,
    /// `(workflow file, conclusion on master)`.
    verdicts: Vec<(String, String)>,
    /// True when the workflows above are the ones the TITLE names.
    ///
    /// The claim is made in the title, so the title's workflows are the ones
    /// that carry it. Falling back to the body finds more, and attributes
    /// worse: #2292's title says "cargo test -p t27c landed red on master" and
    /// names no workflow at all, while its body mentions `release` -- which is
    /// red for its own reasons and has nothing to do with the sentence. Reading
    /// those together marked a stale headline as still red.
    from_title: bool,
}

pub fn run(cmd: &IssuesCmd) -> Result<()> {
    let limit = match cmd {
        IssuesCmd::Numbers { sample, limit } => return numbers(*sample, *limit),
        IssuesCmd::Dated { limit, list } => return dated(*limit, *list),
        IssuesCmd::Stale { limit } => limit,
    };
    let root = repo_root()?;
    let keys = workflow_keys(&root.join(".github/workflows"))?;
    let files: std::collections::BTreeSet<&String> = keys.values().collect();

    let lim = limit.to_string();
    let raw = gh(&[
        "issue",
        "list",
        "--state",
        "open",
        "--limit",
        &lim,
        "--json",
        "number,title,body",
    ])?;
    let issues: serde_json::Value = serde_json::from_str(&raw).context("gh returned no JSON")?;
    let issues = issues.as_array().cloned().unwrap_or_default();
    if issues.is_empty() {
        anyhow::bail!(
            "gh returned no open issues. That is either a repository with none \
             or a query that did not run, and the two print the same zero."
        );
    }

    let mut rows = Vec::new();
    for it in &issues {
        let title = it["title"].as_str().unwrap_or("");
        if !claims_red(title) {
            continue;
        }
        let body = it["body"].as_str().unwrap_or("");
        let from_title = named_workflows(title, &keys);
        let (named, in_title) = if from_title.is_empty() {
            (named_workflows(&format!("{title}\n{body}"), &keys), false)
        } else {
            (from_title, true)
        };
        if named.is_empty() {
            continue;
        }
        let mut verdicts = Vec::new();
        for wf in named {
            let out = gh(&[
                "run",
                "list",
                "--workflow",
                &wf,
                "--branch",
                "master",
                "-L",
                "1",
                "--json",
                "conclusion",
                "--jq",
                ".[0].conclusion // \"no master run\"",
            ])
            .unwrap_or_else(|_| "unreadable".into());
            verdicts.push((wf, out.trim().to_string()));
        }
        rows.push(Row {
            number: it["number"].as_u64().unwrap_or(0),
            title: title.to_string(),
            verdicts,
            from_title: in_title,
        });
    }

    println!("OPEN ISSUES THAT CALL A WORKFLOW RED, AND WHAT IT DOES ON MASTER TODAY\n");
    println!("  open issues read        {}", issues.len());
    println!("  workflow files          {}", files.len());
    println!("  titles claiming red     {}", rows.len());

    let all_green =
        |r: &Row| !r.verdicts.is_empty() && r.verdicts.iter().all(|(_, c)| c == "success");
    let stale: Vec<&Row> = rows.iter().filter(|r| all_green(r)).collect();
    let by_body = rows.iter().filter(|r| !r.from_title).count();
    println!(
        "  of those, every named workflow is green today   {}",
        stale.len()
    );
    println!(
        "  attributed from the body, not the title         {}\n",
        by_body
    );

    for r in &rows {
        let mark = if all_green(r) {
            "HEADLINE STALE"
        } else {
            "still red"
        };
        let src = if r.from_title {
            ""
        } else {
            "   (workflow named in the body, not the title)"
        };
        println!("  #{}  {mark}{src}", r.number);
        println!("      {}", &r.title[..r.title.len().min(96)]);
        for (wf, c) in &r.verdicts {
            println!("        {:<28} master: {}", wf.trim_end_matches(".yml"), c);
        }
        println!();
    }

    println!(
        "  A green reading does NOT mean the issue is resolved, and this command\n  \
         never suggests closing one. \"cli-tri has been red on master for three\n  \
         days\" also argues that a `paths:` filter kept it from running, and that\n  \
         argument outlives the redness. What it says is narrower: the sentence at\n  \
         the top of that issue is no longer true, so a reader who stops at the\n  \
         title is misled about the tree.\n\n  \
         A workflow is matched by its FILE NAME, its stem, and the `name:` it\n  \
         displays under -- because issues use the display name. On this tree 22 of\n  \
         49 workflows display under a different name than their file, and adding\n  \
         that key moved this population from 8 to 9. The one it added is #2851,\n  \
         \"Seal Coverage has been red on master\"."
    );
    if issues.len() >= *limit {
        println!(
            "\n  NOTE: gh returned {} issues at a limit of {}. This is a SAMPLE, not\n  \
             a census -- raise --limit past the open count before quoting a total.",
            issues.len(),
            limit
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The nine real titles this command was built from, and what it must do
    /// with each. The last three are the counter-examples: a title that reports
    /// a REPAIR, one that mentions a workflow with no redness claim, and one
    /// whose redness is in the body rather than the claim.
    #[test]
    fn the_matcher_reads_the_claim_and_not_the_topic() {
        for t in [
            "cli-tri has been red on master for three days; the path filter meant it never ran",
            "Seal Coverage has been red on master since #2841: four gen-c fixes changed output",
            "coq-proofs.yml has failed 62 of 62 runs at opam init",
            "brain-seal-refresh.yml cannot succeed: its last step is a push the ruleset forbids",
            "emit-bitexact is red on master and nothing was watching",
            "master's tri test suite is red, and two gates cannot see it",
            "cargo test -p t27c landed red on master",
        ] {
            assert!(claims_red(t), "should claim red: {t}");
        }
        for t in [
            "seal-coverage is green again after 29 red runs",
            "cli-tri builds one crate behind a filter covering another",
            "The gate against gates-green-by-not-running covers 15 of 47 workflows",
            "Four workflows were red and have been repaired",
        ] {
            assert!(!claims_red(t), "should NOT claim red: {t}");
        }
    }

    /// The recall bug this command exists around. A reader keyed on the file
    /// stem alone cannot see an issue that names the workflow the way GitHub
    /// displays it.
    #[test]
    fn a_workflow_is_found_by_the_name_it_displays_under() {
        let dir = tempdir("keys");
        std::fs::write(
            dir.join("seal-coverage.yml"),
            "name: Seal Coverage\non:\n  push:\n",
        )
        .unwrap();
        let keys = workflow_keys(&dir).unwrap();
        let hay = "Seal Coverage has been red on master since #2841";
        assert_eq!(
            named_workflows(hay, &keys),
            vec!["seal-coverage.yml".to_string()],
            "the display name must resolve to the file"
        );
        // and the stem still works
        assert_eq!(
            named_workflows("seal-coverage.yml has failed", &keys),
            vec!["seal-coverage.yml".to_string()]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A hyphenated name must not match inside a longer one. `cli-tri` and
    /// `cli-tri-mcp` are different things in this repository, and #2903 is
    /// about exactly that distinction.
    #[test]
    fn a_name_does_not_match_inside_a_longer_one() {
        let dir = tempdir("bound");
        std::fs::write(dir.join("cli-tri.yml"), "name: cli-tri\n").unwrap();
        let keys = workflow_keys(&dir).unwrap();
        assert!(named_workflows("cli-tri is red on master", &keys).len() == 1);
        assert!(
            named_workflows("cli-tri-mcp is in neither list", &keys).is_empty(),
            "cli-tri must not match inside cli-tri-mcp"
        );
        assert!(
            named_workflows("the acli-tri thing", &keys).is_empty(),
            "and not inside a longer word on the left either"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An empty workflow directory is a refusal, not a zero: "no issue names a
    /// workflow" and "this command could not read any workflow" are different
    /// statements and must not print the same.
    #[test]
    fn an_empty_workflow_directory_is_refused() {
        let dir = tempdir("empty");
        let e = workflow_keys(&dir).unwrap_err().to_string();
        assert!(e.contains("no *.yml"), "{e}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The claim is in the title, so the title's workflows carry it. #2754
    /// says "secret-scan and cli-tri are red on master" -- both green today --
    /// while its body also mentions `release`, which is red for its own
    /// reasons. Reading them together marks a stale headline as still red.
    #[test]
    fn the_title_names_the_workflows_the_claim_is_about() {
        let dir = tempdir("title");
        for (f, n) in [
            ("cli-tri.yml", "cli-tri"),
            ("secret-scan.yml", "secret-scan"),
            ("release.yml", "Release Pipeline"),
        ] {
            std::fs::write(dir.join(f), format!("name: {n}\n")).unwrap();
        }
        let keys = workflow_keys(&dir).unwrap();
        let title = "secret-scan and cli-tri are red on master, and a paths: filter hid it";
        let body = "See also the Release Pipeline, which is red for its own reasons.";
        let from_title = named_workflows(title, &keys);
        assert_eq!(
            from_title,
            vec!["cli-tri.yml".to_string(), "secret-scan.yml".to_string()],
            "the title names two, and release is not one of them"
        );
        let both = named_workflows(&format!("{title}\n{body}"), &keys);
        assert!(
            both.contains(&"release.yml".to_string()) && both.len() == 3,
            "reading the body too pulls in the third: {both:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An address identifies a thing and measures nothing. This repository
    /// writes five spellings of them, and a matcher that reads any two-digit run
    /// reports 329 titles here of which 145 -- 44% -- are only these.
    #[test]
    fn an_address_is_not_a_count() {
        for t in [
            "Seal Coverage has been red on master since #2841",
            "Wave Loop 369 -- IGLA CODER+RACE + retry board flash",
            "Wave 679: a _CoqProject nobody runs builds nothing",
            "formal: the memory axiom, over a symbolic address (Prop. 78)",
            "cli/tri: six names unresolvable on w699",
            "Catalog Count Invariant CI-01 fires on every run",
        ] {
            assert_ne!(
                carries(t),
                Carries::Digits,
                "the only digits here are an address: {t}"
            );
        }
    }

    /// The bug an independent reader found by subtraction. Without a word
    /// boundary the digit rule fires INSIDE identifiers, and this repository is
    /// full of them: 295 against 283, the loose reader a strict superset.
    #[test]
    fn a_digit_inside_an_identifier_is_not_a_count() {
        for t in [
            "[IGLA-Coder] P8 Integration into t27 and publication",
            "feat(fpga): tri CLI integration for openXC7 GF16 flow",
            "fix(igla): add Digilent FTDI cable support to cli/dlc10",
            "openXC7 emits a wrong bitstream for SRL16E (same class as DSP48E1)",
            "The lexer turns 0o777 into 0",
            "Wave Loop 564 -- layer-boundary requantizer; 2'b11 proved unreachable",
            "parser: Expected LParen in specs/tri/collections/bitset.t27",
        ] {
            assert_eq!(
                carries(t),
                Carries::None,
                "digits are inside an identifier, and there is no numeral word: {t}"
            );
        }
    }

    /// A count in words is still a count, and a digit matcher cannot see it.
    /// 98 titles on this backlog state their figure only this way.
    #[test]
    fn a_numeral_word_is_a_count_and_a_quantifier_is_not() {
        for t in [
            "Twelve quantified clauses call a function with the wrong number of arguments",
            "Five open issues say a workflow is red",
            "Eight invariants spell determinism as f(x) == f(x)",
            "specs/ml/optimizer/adamw.t27 contains two complete copies of the AdamW module",
        ] {
            assert_eq!(
                carries(t),
                Carries::Words,
                "a numeral word is a figure: {t}"
            );
        }
        for t in [
            "Wave Loop 601 -- every assumption audited for what it removes",
            "formal: a verdict for every property, and none of them is dead (Prop. 64)",
        ] {
            assert_eq!(
                carries(t),
                Carries::QuantifierOnly,
                "quantifies without giving a figure: {t}"
            );
        }
    }

    /// A title can carry both, and the address in it must not change that.
    #[test]
    fn digits_and_words_together_are_both() {
        assert_eq!(
            carries("Three pull requests have run almost no CI, and 40 checks were skipped"),
            Carries::Both
        );
        assert_eq!(
            carries("Wave 690: the corpus reaches 12 on every parse metric"),
            Carries::Digits,
            "the wave number is stripped; 12 is the count"
        );
    }

    fn tempdir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "tri-issues-{tag}-{}-{}",
            std::process::id(),
            NTH.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }
    static NTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
}

// ---------------------------------------------------------------------------
// `tri issues dated` -- the figures that re-measurement cannot judge.
// ---------------------------------------------------------------------------

/// The revisions a body pins, by their spelling in the text.
///
/// A hex run of 7..=40 characters carrying at least one letter and one digit is
/// the shape of an abbreviated commit id -- and it is also the shape of a chunk
/// of a float. Two counter-examples from this backlog, both real:
///
/// * #2824 prints `s[0] = -1.7594823e-05`, whose `7594823e` is a perfect match.
/// * #2658 lists `` `5.391247e-44` `` INSIDE BACKTICKS, so "quoted like code"
///   does not separate them either.
///
/// Two rules reject them: a revision is never preceded by a decimal point, and
/// a hex run ending in `e` immediately before a sign is a mantissa, not an id.
/// Shape alone matched 45 of the 486 open BODIES; the rules bring it to 43,
/// and both dropped were floats. (The command's own population is smaller --
/// only issues whose title carries a figure -- so its `pins a revision` line
/// is not this 43.)
///
/// Neither rule is redundant, and measuring that took a mutation: across all
/// 486 open bodies BOTH rejections are caught by BOTH rules, so deleting either
/// one leaves every test green. `each_float_rule_decides_a_case_the_other_misses`
/// supplies the two inputs that separate them -- `5391247e-44` has no dot, and
/// `1.2345678e12` has no sign for the `e` to sit before.
pub fn revision_pins(body: &str) -> Vec<String> {
    let b: Vec<char> = body.chars().collect();
    let hex = |c: char| c.is_ascii_digit() || ('a'..='f').contains(&c);
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        if !hex(b[i]) || (i > 0 && b[i - 1].is_alphanumeric()) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < b.len() && hex(b[j]) {
            j += 1;
        }
        let s: String = b[i..j].iter().collect();
        let long = (7..=40).contains(&s.len());
        let mixed =
            s.chars().any(|c| c.is_ascii_digit()) && s.chars().any(|c| c.is_ascii_alphabetic());
        let after_point = i > 0 && b[i - 1] == '.';
        let exponent = s.ends_with('e') && b.get(j).is_some_and(|c| *c == '-' || *c == '+');
        if long && mixed && !after_point && !exponent && (j >= b.len() || !b[j].is_alphanumeric()) {
            out.push(s);
        }
        i = j.max(i + 1);
    }
    out.sort();
    out.dedup();
    out
}

/// Does the body say, in words, that its figure was taken at a fixed point?
///
/// Matched on token boundaries so `unfrozen` and `snapshots/` do not decide it.
pub fn says_as_of(body: &str) -> bool {
    const WORDS: [&str; 8] = [
        "as of",
        "as-of",
        "snapshot",
        "frozen",
        "freeze",
        "frozen_hash",
        "at commit",
        "pinned",
    ];
    let low = body.to_lowercase();
    WORDS.iter().any(|w| names(&low, w))
}

/// Why re-measuring this issue's figure would not judge it.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Anchor {
    /// The body pins a revision, so the figure is a reading OF that revision.
    Revision,
    /// The body says the figure was taken at a fixed point.
    AsOf,
    /// Someone has already answered in the thread.
    Answered,
    /// Nothing anchors it: this figure is a claim about the tree as it is, and
    /// re-measuring it is a real test.
    Free,
}

/// Classify one issue. `Revision` outranks `AsOf` outranks `Answered`: the
/// order is from most mechanical to least, so the reported reason is the one a
/// second reader can check with the least judgement.
pub fn anchor_of(body: &str, comments: usize) -> Anchor {
    if !revision_pins(body).is_empty() {
        Anchor::Revision
    } else if says_as_of(body) {
        Anchor::AsOf
    } else if comments > 0 {
        Anchor::Answered
    } else {
        Anchor::Free
    }
}

fn dated(limit: usize, list: bool) -> Result<()> {
    let lim = limit.to_string();
    let raw = gh(&[
        "issue",
        "list",
        "--state",
        "open",
        "--limit",
        &lim,
        "--json",
        "number,title,body,comments",
    ])?;
    let issues: serde_json::Value = serde_json::from_str(&raw).context("gh returned no JSON")?;
    let issues = issues.as_array().cloned().unwrap_or_default();
    if issues.is_empty() {
        anyhow::bail!(
            "gh returned no open issues. That is either a repository with none \
             or a query that did not run, and the two print the same zero."
        );
    }

    let mut pop: Vec<(u64, String, Anchor)> = Vec::new();
    let mut no_figure = 0usize;
    for it in &issues {
        let title = it["title"].as_str().unwrap_or("");
        match carries(title) {
            Carries::Digits | Carries::Words | Carries::Both => {}
            _ => {
                no_figure += 1;
                continue;
            }
        }
        let body = it["body"].as_str().unwrap_or("");
        let comments = it["comments"].as_array().map(|a| a.len()).unwrap_or(0);
        pop.push((
            it["number"].as_u64().unwrap_or(0),
            title.to_string(),
            anchor_of(body, comments),
        ));
    }
    pop.sort_by_key(|(n, _, _)| *n);

    let c = |a: Anchor| pop.iter().filter(|(_, _, x)| *x == a).count();
    let anchored = pop.len() - c(Anchor::Free);

    println!("OPEN ISSUES WHOSE FIGURE RE-MEASUREMENT CANNOT JUDGE\n");
    println!("  open issues read              {}", issues.len());
    println!("  no figure in the title        {no_figure}");
    println!("  POPULATION (carries a figure) {}", pop.len());
    println!("  pins a revision               {}", c(Anchor::Revision));
    println!("  says as-of / snapshot         {}", c(Anchor::AsOf));
    println!("  already answered in thread    {}", c(Anchor::Answered));
    println!("  ANCHORED                      {anchored}");
    println!("  free to re-measure            {}", c(Anchor::Free));

    println!(
        "\n  Re-measuring a number is not testing the claim that carries it. An\n  \
         issue that pins a revision states a reading OF that revision; today's\n  \
         tree disagreeing with it is what a snapshot IS, not a defect. An issue\n  \
         someone has already answered has been judged by a person, and a fresh\n  \
         reading adds nothing a reader of the thread does not have.\n\n  \
         This command names no issue stale and never suggests closing one. It\n  \
         says only which figures a second reading can decide: {} of {}. The\n  \
         other {anchored} need the claim read, not the number re-run.\n\n  \
         Cost of skipping it, measured: one verdict of mine called a figure\n  \
         stale by re-measuring it to 0. The issue pinned a snapshot hash, its\n  \
         own script refuses to run once the corpus moves, the owner had already\n  \
         commented the new figures, and the 18 lines I read as missing were\n  \
         settled by a decision the issue PREDICTED. Every one of those four is\n  \
         visible from the fields above -- and that issue, #2160, is in the\n  \
         anchored list this command prints.",
        c(Anchor::Free),
        pop.len()
    );

    if list {
        println!("\n  ANCHORED, by number:\n");
        for (n, t, a) in pop.iter().filter(|(_, _, x)| *x != Anchor::Free) {
            let tag = match a {
                Anchor::Revision => "revision",
                Anchor::AsOf => "as-of   ",
                Anchor::Answered => "answered",
                Anchor::Free => unreachable!(),
            };
            println!("    {tag}  #{n}  {}", &t[..t.len().min(78)]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod dated_tests {
    use super::*;

    #[test]
    fn a_float_is_not_a_revision() {
        // #2824, verbatim: the exponent form puts a hex-shaped run after a dot.
        assert!(revision_pins("s[0] = -1.7594823e-05  required > 0.02").is_empty());
        // #2658, verbatim, and inside backticks -- quoting does not separate them.
        assert!(revision_pins("literals in 23 specs: `1.0e38`, `5.391247e-44`").is_empty());
    }

    /// Two rules reject those floats, and on this backlog neither ever fires
    /// alone: 486 bodies, 2 rejections, both caught twice over. Removing either
    /// rule left the tests green, which is a control that cannot fail. These
    /// two inputs are constructed so that exactly one rule decides each.
    #[test]
    fn each_float_rule_decides_a_case_the_other_misses() {
        // No decimal point before it -- only the exponent rule sees this.
        assert!(revision_pins("delta 5391247e-44 rad").is_empty());
        // Exponent without a sign, so the run swallows it -- only the dot sees this.
        assert!(revision_pins("scale 1.2345678e12 units").is_empty());
    }

    #[test]
    fn a_revision_is_a_revision() {
        assert_eq!(revision_pins("merged as `9adbb6910`;"), vec!["9adbb6910"]);
        assert_eq!(
            revision_pins("built from `t27` @ `40003ed1`,"),
            vec!["40003ed1"]
        );
        assert_eq!(
            revision_pins("spec_hash=sha256:9597edff..."),
            vec!["9597edff"]
        );
    }

    #[test]
    fn shape_alone_is_not_enough() {
        assert!(revision_pins("deadbee").is_empty(), "no digit");
        assert!(revision_pins("1234567").is_empty(), "no letter");
        assert!(revision_pins("32cb50").is_empty(), "six is too short");
        assert!(
            revision_pins("c0ffee1c0ffee1c0ffee1c0ffee1c0ffee1c0ffee1").is_empty(),
            "41 is too long"
        );
    }

    #[test]
    fn as_of_needs_a_whole_token() {
        assert!(says_as_of("measured as of 2026-08-20"));
        assert!(says_as_of("the FROZEN_HASH in stage0"));
        assert!(!says_as_of("the unfrozen corpus"));
        assert!(!says_as_of("counted the snapshots directory"));
    }

    #[test]
    fn a_plain_claim_about_the_tree_is_free() {
        assert_eq!(
            anchor_of("125 lines in 65 files are corrupt.", 0),
            Anchor::Free
        );
    }

    #[test]
    fn every_anchor_is_reachable_and_ordered() {
        assert_eq!(anchor_of("at `40003ed1`, as of then", 3), Anchor::Revision);
        assert_eq!(anchor_of("as of 2026-08-20", 3), Anchor::AsOf);
        assert_eq!(anchor_of("plain prose", 1), Anchor::Answered);
    }
}
