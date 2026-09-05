//! `tri now` -- write a docs/now/ entry without hand-writing the frame.
//!
//! Every pull request in this repository must add an entry (the
//! check-now-freshness gate), and the entry format is rigid enough that
//! writing it by hand invites drift: a forgotten date, a heading that does
//! not match the section, a missing issue reference. One forgotten entry
//! cost a full gate round trip. This stamps the frame; the caller supplies
//! only the content.
//!
//! Entries are one file per unit of work, `docs/now/<YYYY-MM-DD>-<slug>.md`.
//! This used to prepend to the single file docs/NOW.md, which meant every PR
//! rewrote the same first line and GitHub marked every concurrent PR
//! CONFLICTING. Writing a distinct path removes the shared line entirely, and
//! the writer gets simpler: a create, not a read-modify-write.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum NowCmd {
    /// Write a docs/now/ entry: title, bullets, optional issue ref.
    Add {
        /// Entry title, used for both the page heading and the section.
        title: String,
        /// Bullet lines, repeatable. At least one is required — an entry
        /// with no content is exactly the vacuous touch the gate invites.
        ///
        /// `allow_hyphen_values` because entry text legitimately starts with a
        /// flag name: writing this command's own NOW entry failed on the
        /// bullet describing `--expect`, which clap read as an argument.
        #[arg(long = "bullet", required = true, allow_hyphen_values = true)]
        bullets: Vec<String>,
        /// Issue number for the section's "(Closes #N)" suffix.
        ///
        /// This AUTOCLOSES the issue on merge, which LOOP-RULES R11 bans for
        /// tracking issues. Use `--refs` for those.
        #[arg(long)]
        closes: Option<u64>,
        /// Issue number for the section's "(Refs #N)" suffix -- cites the
        /// issue without closing it.
        ///
        /// Added because the command could express only the autoclosing form,
        /// so an entry that had to cite a long-lived tracking issue had three
        /// options: autoclose it (banned), hand-edit the generated file (the
        /// drift this command exists to prevent), or cite nothing. All three
        /// were taken at least once before the flag existed.
        #[arg(long, conflicts_with = "closes")]
        refs: Option<u64>,
    },
    /// Ask the REQUIRED `check` gate its own question, before pushing.
    Check {
        /// Judge exactly these paths. Overrides `--staged` and `--base`.
        paths: Vec<PathBuf>,
        /// Judge what the index adds -- what a pre-commit hook can see.
        #[arg(long)]
        staged: bool,
        /// Judge what this branch adds against a revision.
        #[arg(long, default_value = "origin/master")]
        base: String,
    },
}

pub fn run(cmd: &NowCmd) -> Result<()> {
    match cmd {
        NowCmd::Add {
            title,
            bullets,
            closes,
            refs,
        } => add(title, bullets, *closes, *refs),
        NowCmd::Check {
            paths,
            staged,
            base,
        } => check(paths, *staged, base),
    }
}

/// The section heading's issue suffix.
///
/// Separate from `add` so it can be tested without touching the filesystem:
/// the bug this guards against is a one-character difference between two
/// strings that both look right in a diff.
fn issue_suffix(closes: Option<u64>, refs: Option<u64>) -> String {
    match (closes, refs) {
        (Some(n), _) => format!(" (Closes #{n})"),
        (None, Some(n)) => format!(" (Refs #{n})"),
        (None, None) => String::new(),
    }
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("git is not installed or not on PATH")?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

/// Today's date from the local clock, YYYY-MM-DD, with no chrono dependency:
/// `git` is already a hard requirement of this command and its author date
/// formatting is stable.
fn today() -> Result<String> {
    let out = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .context("date is not available")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Filename-safe slug: lowercase, runs of non-alphanumerics collapsed to a
/// single `-`, trimmed, capped so paths stay readable. The gate's filename
/// pattern is `[A-Za-z0-9._-]+`, and this stays well inside it.
fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            // Non-ASCII and punctuation alike become a separator. Dropping the
            // character rather than transliterating keeps the filename ASCII,
            // which the repo's L3 PURITY gate requires of added lines anyway.
            pending_dash = true;
        }
    }
    // Cap at 60 chars, then trim a trailing dash the cut may have exposed.
    const MAX: usize = 60;
    if out.len() > MAX {
        out.truncate(MAX);
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn add(title: &str, bullets: &[String], closes: Option<u64>, refs: Option<u64>) -> Result<()> {
    let date = today()?;
    let slug = slugify(title);
    if slug.is_empty() {
        anyhow::bail!(
            "title {title:?} has no ASCII alphanumerics, so it yields an empty filename slug; \
             give the entry a title that can name a file"
        );
    }

    let dir = repo_root()?.join("docs").join("now");
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let path = dir.join(format!("{date}-{slug}.md"));

    // Refuse to clobber. Two entries on one day are fine -- they just need
    // distinct titles. Silently overwriting, or silently appending a numeric
    // suffix, would both hide a duplicate that is nearly always a mistake.
    if path.exists() {
        anyhow::bail!(
            "{} already exists; give this entry a distinct title",
            path.display()
        );
    }

    let suffix = issue_suffix(closes, refs);
    // No `Last updated:` line: the filename carries the date, and a second copy
    // inside the file is exactly the duplicated line the old layout fought over.
    let mut entry = format!("# NOW -- {title} ({date})\n\n## {title}{suffix}\n\n");
    for b in bullets {
        entry.push_str(&format!("- {b}\n"));
    }
    std::fs::write(&path, entry).with_context(|| format!("write {}", path.display()))?;
    println!("wrote NOW entry: {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{issue_suffix, slugify};

    #[test]
    fn refs_cites_without_closing() {
        // The whole point of the flag: the word must be "Refs", because
        // "Closes" is what GitHub acts on.
        assert_eq!(issue_suffix(None, Some(2161)), " (Refs #2161)");
        assert!(!issue_suffix(None, Some(2161)).contains("Closes"));
    }

    #[test]
    fn closes_still_autocloses_and_no_suffix_stays_empty() {
        assert_eq!(issue_suffix(Some(141), None), " (Closes #141)");
        assert_eq!(issue_suffix(None, None), "");
    }

    #[test]
    fn slug_lowercases_and_joins_words() {
        assert_eq!(
            slugify("Retire the NOW.md bottleneck"),
            "retire-the-now-md-bottleneck"
        );
    }

    #[test]
    fn slug_collapses_runs_of_punctuation() {
        assert_eq!(
            slugify("fix(ci):  now --  sync gate!!"),
            "fix-ci-now-sync-gate"
        );
    }

    #[test]
    fn slug_has_no_leading_or_trailing_dash() {
        let s = slugify("  ...leading and trailing...  ");
        assert!(!s.starts_with('-'), "{s:?}");
        assert!(!s.ends_with('-'), "{s:?}");
        assert_eq!(s, "leading-and-trailing");
    }

    #[test]
    fn slug_is_capped_and_still_clean() {
        let s = slugify(&"word ".repeat(40));
        assert!(s.len() <= 60, "len {} for {s:?}", s.len());
        assert!(!s.ends_with('-'), "{s:?}");
    }

    #[test]
    fn slug_drops_non_ascii() {
        assert_eq!(slugify("ternary \u{2014} node"), "ternary-node");
    }

    #[test]
    fn slug_empty_when_no_alphanumerics() {
        assert_eq!(slugify("--- ... ---"), "");
    }

    /// The filename this produces must satisfy the CI gate's own pattern.
    #[test]
    fn slug_matches_gate_filename_pattern() {
        let re = regex::Regex::new(r"^docs/now/[0-9]{4}-[0-9]{2}-[0-9]{2}-[A-Za-z0-9._-]+\.md$")
            .unwrap();
        for title in [
            "Retire the NOW.md bottleneck",
            "fix(ci): now -- sync gate!!",
            "wave 913: GF16 conformance",
        ] {
            let name = format!("docs/now/2026-08-20-{}.md", slugify(title));
            assert!(re.is_match(&name), "gate would reject {name:?}");
        }
    }
}

// ---------------------------------------------------------------------------
// `tri now check` -- the blocking gate's question, asked locally.
// ---------------------------------------------------------------------------

/// Judge the entries a change adds, using the gate's own implementation.
///
/// **Why this delegates instead of deciding.** Five local instruments already
/// read `docs/now/`: `.githooks/pre-commit` (via `scripts/tri check-now`),
/// `scripts/pre-commit`, `scripts/verify.sh`, `tri hooks now-gate` and
/// `tri hooks pre-commit`. Every one of them checks FRESHNESS -- an entry
/// exists, dated inside the window -- and the required `check` context checks
/// SHAPE. Same directory, same label, a different question, and measured on
/// one malformed entry dated today: the gate reported three complaints while
/// three of the five local instruments went green. `scripts/pre-commit` went
/// green **because of** that file: its freshness loop found the entry the gate
/// rejects and stopped looking.
///
/// A sixth reader written in Rust would answer the question and then drift
/// away from it. This one shells out to `tools/check_now_entry_shape.py
/// --check-files`, so the local answer IS the gate's answer and disagreement
/// is not something to test for.
/// What an empty added-file set MEANS, as a value rather than a side effect.
///
/// The test named for this behaviour used to assert only that `check` returned
/// `Ok`, so deleting the message left it green: a test whose NAME is the claim
/// and whose body does not check the claim. A verdict that is returned can be
/// asserted; one that is printed cannot.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Checked {
    /// No entry was added, so no shape was read. Not a pass.
    Nothing,
    /// This many entries were read.
    Files(usize),
}

pub(crate) const NOTHING_CHECKED: &str =
    "tri now check: this change adds no docs/now/ entry, so no SHAPE was checked.\n       Whether one is REQUIRED is a different question, and `tri hooks now-gate` does\n\
       NOT ask it -- it reads the docs/now DIRECTORY, which 165 in-window entries\n\
       make fresh whatever this change does. `tri hooks pre-push` asks it, by\n\
       running the required gate's own script over the range.";

/// Decide, and WRITE the sentence, through a handle the test can hold.
///
/// Asserting on the verdict alone still let the message be deleted: the branch
/// stayed, the value stayed, and the reader got silence. A verdict that is
/// returned can be asserted; a sentence that is printed can only be asserted
/// through the writer that printed it.
pub(crate) fn report_empty<W: std::io::Write>(w: &mut W, files: &[String]) -> Result<Checked> {
    let verdict = what_was_checked(files);
    if verdict == Checked::Nothing {
        writeln!(w, "{NOTHING_CHECKED}")?;
    }
    Ok(verdict)
}

pub(crate) fn what_was_checked(files: &[String]) -> Checked {
    if files.is_empty() {
        Checked::Nothing
    } else {
        Checked::Files(files.len())
    }
}

fn check(paths: &[PathBuf], staged: bool, base: &str) -> Result<()> {
    let root = repo_root()?;
    let script = gate_script(&root)?;
    // Ask the OS whether the interpreter exists, not an error message: a
    // message is the tool's to reword and PATH is not.
    if std::process::Command::new("python3")
        .arg("-c")
        .arg("")
        .output()
        .is_err()
    {
        anyhow::bail!(
            "python3 is not on PATH, so the gate could not be run and nothing was \
             checked. Reporting that rather than a pass this run did not earn."
        );
    }

    let files: Vec<String> = if !paths.is_empty() {
        paths.iter().map(|p| p.display().to_string()).collect()
    } else if staged {
        git_paths(
            &root,
            &["diff", "--cached", "--name-only", "--diff-filter=A"],
            None,
        )?
    } else {
        git_paths(
            &root,
            &["diff", "--name-only", "--diff-filter=A"],
            Some(&format!("{base}...HEAD")),
        )?
    };

    match report_empty(&mut std::io::stdout(), &files)? {
        Checked::Nothing => return Ok(()),
        Checked::Files(_) => {}
    }

    let status = std::process::Command::new("python3")
        .arg(&script)
        .arg("--check-files")
        .args(&files)
        .current_dir(&root)
        .status()
        .context("failed to run tools/check_now_entry_shape.py")?;
    if !status.success() {
        anyhow::bail!(
            "the required `check` context would refuse this change. \
             The complaints above are the gate's own words."
        );
    }
    Ok(())
}

/// The gate's own file, or a refusal naming what could not be checked.
///
/// Separate from `check` so the refusal can be exercised without changing the
/// process working directory: a test that does that races every other test in
/// the binary, which is a collision this repository has already paid for once.
fn gate_script(root: &Path) -> Result<PathBuf> {
    let script = root.join("tools/check_now_entry_shape.py");
    if !script.is_file() {
        anyhow::bail!(
            "{} is missing, so nothing was checked. This command is the gate's own \
             implementation reached from here; without the file it has no answer to \
             give, and printing a pass would be the failure it exists to prevent.",
            script.display()
        );
    }
    Ok(script)
}

/// Did the entries this branch adds satisfy the gate? For `tri gates preview`,
/// which needs the verdict rather than a process exit, and quietly.
///
/// **An empty set is a FAIL here and an OK in `tri now check`, and that is not
/// an inconsistency.** `tri now check` runs mid-work, where a commit that adds
/// no entry has no shape to judge. `gates preview` asks what the gate would
/// say about this branch AS A PULL REQUEST, and the gate's own words on an
/// empty set are *"FAIL: this change adds no docs/now/ entry"*. Mirroring the
/// gate includes mirroring what it does with nothing.
pub fn check_added(base: &str) -> Result<bool> {
    let root = repo_root()?;
    let script = gate_script(&root)?;
    let files = git_paths(
        &root,
        &["diff", "--name-only", "--diff-filter=A"],
        Some(&format!("{base}...HEAD")),
    )?;
    if files.is_empty() {
        return Ok(false);
    }
    let out = std::process::Command::new("python3")
        .arg(&script)
        .arg("--check-files")
        .args(&files)
        .current_dir(&root)
        .output()
        .context("failed to run tools/check_now_entry_shape.py")?;
    Ok(out.status.success())
}

/// The shape of what the index adds -- the pre-commit hook's entry point.
pub fn check_staged() -> Result<()> {
    check(&[], true, "origin/master")
}

/// Added `docs/now/*.md` paths from one git invocation, README excluded.
fn git_paths(root: &Path, args: &[&str], range: Option<&str>) -> Result<Vec<String>> {
    let mut cmd = std::process::Command::new("git");
    cmd.current_dir(root).args(args);
    if let Some(r) = range {
        cmd.arg(r);
    }
    cmd.args(["--", "docs/now/"]);
    let out = cmd.output().context("failed to invoke git diff")?;
    if !out.status.success() {
        anyhow::bail!(
            "git {:?} failed: {}. The range is wrong, not the tree -- and a wrong \
             range prints an empty list, which reads as \"nothing to check\".",
            args,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(entry_paths(&String::from_utf8_lossy(&out.stdout)))
}

/// The entry paths in one `git diff --name-only` listing.
///
/// Split from the git call so the filter can be mutated and seen to fail:
/// while it lived inside `git_paths` no test reached it, and a clause no test
/// reaches is indistinguishable from a dead one. The rule mirrors the gate's
/// own `added_now_entries` -- `README.md` is documentation about the
/// directory, not an entry in it, and a non-`.md` file is not one either.
fn entry_paths(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| l.ends_with(".md") && !l.ends_with("README.md"))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod check_tests {
    use super::*;

    /// The six shapes `tools/check_now_entry_shape.py --self-check` names,
    /// with the verdict that file itself states for each.
    ///
    /// They are transcribed here for a reason the pair makes sharp: this
    /// command DELEGATES, so any test comparing its answer to the gate's
    /// compares the gate to itself and cannot fail. What can fail is the
    /// wiring -- a wrong flag, a wrong working directory, a range that prints
    /// an empty list -- and that is what these exercise, end to end, through
    /// the real script.
    fn shapes() -> Vec<(&'static str, &'static str, &'static str, bool)> {
        let good = "# NOW -- A real entry about a real thing (2026-08-28)\n\n\
                    ## A real entry about a real thing (Refs #1)\n\n\
                    - something specific happened and here is what it was\n";
        vec![
            // (label, the date the FILENAME carries, body, gate accepts it)
            ("well formed", "2026-08-28", good, true),
            ("heading date disagrees", "2026-08-27", good, false),
            (
                "no bullets",
                "2026-08-28",
                "# NOW -- Title (2026-08-28)\n\n## Title\n",
                false,
            ),
            (
                "placeholder bullets",
                "2026-08-28",
                "# NOW -- Title (2026-08-28)\n\n## Title\n\n- TBD\n- ...\n",
                false,
            ),
            (
                "wrong first line",
                "2026-08-28",
                "# Some other heading\n\n## Title\n\n- a bullet that is long enough\n",
                false,
            ),
            ("empty file", "2026-08-28", "", false),
        ]
    }

    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "now_check_{}_{}_{}",
            tag,
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// The planted filename must be one the gate ACCEPTS on its own, or every
    /// case fails for the same unrelated reason and the table says nothing.
    ///
    /// The first version named the files `zz-check-test-<date>-...`, which the
    /// filename rule rejects, so `well formed` could never pass -- and the run
    /// was GREEN, because a sibling test changed the process working directory
    /// out of the repository and this one took its silent early return. Two
    /// defects, and the cwd one was mine: the guard against it is that no test
    /// here moves the process.
    #[test]
    fn every_shape_the_gate_names_reaches_the_same_verdict_from_here() {
        let root = repo_root().expect("tests run inside the repository");
        assert!(gate_script(&root).is_ok());
        let dir = root.join("docs/now");
        for (i, (label, date, text, want_ok)) in shapes().into_iter().enumerate() {
            let name = format!("{date}-zzcheck-{}-{i}.md", std::process::id());
            std::fs::write(dir.join(&name), text).unwrap();
            let got = check(
                &[PathBuf::from(format!("docs/now/{name}"))],
                false,
                "origin/master",
            )
            .is_ok();
            let _ = std::fs::remove_file(dir.join(&name));
            assert_eq!(
                got, want_ok,
                "{label}: the gate says {want_ok} and this command said {got}"
            );
        }
    }

    #[test]
    fn the_directorys_readme_is_not_an_entry_in_it() {
        let listing = "docs/now/README.md\n\
                       docs/now/2026-01-01-a-real-entry.md\n\
                       docs/now/notes.txt\n";
        assert_eq!(
            entry_paths(listing),
            vec!["docs/now/2026-01-01-a-real-entry.md".to_string()]
        );
    }

    #[test]
    fn an_empty_set_says_nothing_was_checked_rather_than_passing_quietly() {
        // The claim is in the name, so the assertion has to be about the
        // VERDICT and not about check() returning Ok. Asserting is_ok() left
        // this green with the message deleted -- the exact shape this file
        // exists to refuse, one level up.
        assert_eq!(what_was_checked(&[]), Checked::Nothing);
        assert!(
            NOTHING_CHECKED.contains("no SHAPE was checked"),
            "the sentence a reader gets must say nothing was checked"
        );

        // And the other half: a non-empty set is NOT Nothing, or the verdict
        // would be constant and could not fail.
        assert_eq!(
            what_was_checked(&["docs/now/2026-01-01-x.md".to_string()]),
            Checked::Files(1)
        );

        // And the sentence must actually REACH the reader. Asserting the
        // verdict alone left the message deletable: branch kept, value kept,
        // reader given silence.
        let mut out: Vec<u8> = Vec::new();
        let v = report_empty(&mut out, &[]).expect("writing to a Vec cannot fail");
        assert_eq!(v, Checked::Nothing);
        let said = String::from_utf8(out).expect("utf8");
        assert!(
            said.contains("no SHAPE was checked"),
            "the empty path must SAY nothing was checked, printed: {said:?}"
        );

        // The control: a non-empty set must not print that sentence.
        let mut out2: Vec<u8> = Vec::new();
        let v2 = report_empty(&mut out2, &["docs/now/2026-01-01-x.md".to_string()])
            .expect("writing to a Vec cannot fail");
        assert_eq!(v2, Checked::Files(1));
        assert!(
            out2.is_empty(),
            "a set with entries must not claim nothing was checked"
        );

        // Still reached through the real path, so the wiring is exercised too.
        let root = repo_root().expect("tests run inside the repository");
        assert!(gate_script(&root).is_ok());
        assert!(check(&[], false, "HEAD").is_ok());
    }

    #[test]
    fn a_missing_gate_script_refuses_instead_of_passing() {
        // This command's whole value is that it IS the gate, so being unable
        // to reach the gate must not print a pass.
        let dir = scratch("noscript");
        let r = gate_script(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            r.is_err(),
            "a tree with no gate script must refuse, not pass"
        );
    }

    #[test]
    fn the_script_is_found_where_it_lives() {
        // The other half: a refusal that fires everywhere is not a check.
        let root = repo_root().expect("tests run inside the repository");
        assert!(gate_script(&root).is_ok());
    }
}
