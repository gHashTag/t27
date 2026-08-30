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
    let IssuesCmd::Stale { limit } = cmd;
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
