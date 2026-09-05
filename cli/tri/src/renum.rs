//! `tri skill renumber` -- give appended sections the numbers master left free.
//!
//! Six times in one week a branch and master appended sections to
//! `.claude/skills/ci-gates/SKILL.md` at the same time and took the same
//! numbers. Every one of those cost the same manual repair: rebuild the file
//! from `origin/master`, move my sections to the end, assert the master prefix
//! is byte-identical, re-run `tri skill check`. Six repetitions is the argument
//! for a command; the repair itself never varied.
//!
//! The invariant this leans on is the one the workflow already has: a section is
//! APPENDED. So the branch's file is the merge-base's file plus a tail, and the
//! tail is exactly what has to move. If that is not true -- someone edited an
//! existing section, or a previous conflict was resolved by hand -- this refuses
//! and says so rather than guessing which lines are yours.
//!
//! `--base` TAKES A BRANCH, AND POINTING IT AT A SIBLING IS A TRAP. Two of my own
//! open branches once claimed the same number, and `--base origin/<sibling>`
//! numbered correctly around it -- and rebuilt my file on the SIBLING, so my
//! branch then carried the sibling's sections too and would have merged them
//! under my PR. `--first N` is the answer that documentation was standing in
//! for: both branches number against the SHARED base, and the second one starts
//! higher. A number, not a different base. `--base` is for a different base
//! BRANCH, not for a peer.
//!
//! References are rewritten only for the numbers being moved and only INSIDE the
//! tail. A section that cites `&sect;447` keeps citing 447; a section that cites
//! its own sibling follows it. Both spellings are handled, and the word boundary
//! matters: renumbering 46 must not touch 460.

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// The tail of `mine` that is not in `base`, or `None` when `mine` is not
/// `base` plus an append.
pub fn appended_tail<'a>(base: &str, mine: &'a str) -> Option<&'a str> {
    // Trailing-newline differences are not an edit. Compare on the trimmed
    // prefix and hand back everything after it.
    let b = base.trim_end_matches('\n');
    if !mine.starts_with(b) {
        return None;
    }
    Some(&mine[b.len()..])
}

/// The highest section number in a file, or 0 when it has none.
pub fn max_section(text: &str) -> usize {
    crate::skillnum::sections(text)
        .iter()
        .map(|(n, _)| *n)
        .max()
        .unwrap_or(0)
}

/// Rewrite the tail's section headings to run from `first`, and follow every
/// reference to a moved number that lives inside the tail.
///
/// Returns the new text and the moves, old to new, in heading order.
pub fn renumber(tail: &str, first: usize) -> (String, Vec<(usize, usize)>) {
    let olds: Vec<usize> = crate::skillnum::sections(tail)
        .iter()
        .map(|(n, _)| *n)
        .collect();
    let moves: Vec<(usize, usize)> = olds
        .iter()
        .enumerate()
        .map(|(i, o)| (*o, first + i))
        .filter(|(o, n)| o != n)
        .collect();

    let mut out = String::with_capacity(tail.len());
    let mut heading = 0usize;
    for line in tail.split_inclusive('\n') {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some((num, title)) = rest.split_once(". ") {
                if num.parse::<usize>().is_ok() {
                    out.push_str(&format!("## {}. {title}", first + heading));
                    heading += 1;
                    continue;
                }
            }
        }
        out.push_str(line);
    }

    // References, after the headings, so a heading is never rewritten twice.
    let mut text = out;
    for (old, new) in &moves {
        for marker in ["&sect;", "\u{a7}"] {
            text = replace_ref(&text, marker, *old, *new);
        }
    }
    (text, moves)
}

/// `<marker><old>` -> `<marker><new>`, but only when `<old>` is the whole
/// number. `&sect;46` must not match inside `&sect;460`.
fn replace_ref(text: &str, marker: &str, old: usize, new: usize) -> String {
    let needle = format!("{marker}{old}");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(&needle) {
        let after = &rest[i + needle.len()..];
        let bounded = !after.chars().next().is_some_and(|c| c.is_ascii_digit());
        out.push_str(&rest[..i]);
        out.push_str(&if bounded {
            format!("{marker}{new}")
        } else {
            needle.clone()
        });
        rest = after;
    }
    out.push_str(rest);
    out
}

/// The tail of `mine`, found by TITLE when the byte prefix no longer matches.
///
/// The prefix test above is exact and fails for a reason that turned up on this
/// command's first real use: the base branch edited an EXISTING section while
/// the branch was open. The branch's file is then not any commit's file plus an
/// append -- it is an older base plus a tail, and the base has moved underneath
/// it in the middle.
///
/// Titles survive that. A section whose title appears in the base is the base's,
/// however its body was reworded; everything after the LAST such section is what
/// this branch added. Returns `None` when no section is shared, which would mean
/// the two files have nothing to do with each other.
/// Whether every section in `tail` is genuinely absent from `at_base`.
///
/// A byte-prefix tail is everything appended since the merge base, and that is
/// wrong the moment a SIBLING branch of yours lands part of it on the base: a
/// squash merge puts the section on `base` under a new commit, the same content
/// sits on both sides, and rebuilding as `at_base + tail` emits it twice.
///
/// Reproduced on this repository, 2026-09-05: branch tip 2ded340a against master
/// 747e4a1, merge base 013b829. `appended here 2`, moves 546 -> 547 and
/// 547 -> 548, and the output carried two sections with the same title, because
/// #3199 had squash-merged the first of them onto master while the branch was
/// open.
/// Titles present in `before` and absent from `after`, sorted.
///
/// By TITLE and not by count: a count guard passed while this command deleted a
/// section, because three heading lines quoted inside a fenced code block were
/// parsed as sections and made the arithmetic come out right.
pub fn titles_lost(before: &str, after: &str) -> Vec<String> {
    let have: std::collections::BTreeSet<String> = crate::skillnum::sections(after)
        .into_iter()
        .map(|(_, t)| t)
        .collect();
    let mut lost: Vec<String> = crate::skillnum::sections(before)
        .into_iter()
        .map(|(_, t)| t)
        .filter(|t| !have.contains(t))
        .collect();
    lost.sort();
    lost.dedup();
    lost
}

pub fn tail_is_new(tail: &str, at_base: &str) -> bool {
    let base_titles: std::collections::BTreeSet<String> = crate::skillnum::sections(at_base)
        .into_iter()
        .map(|(_, t)| t)
        .collect();
    !crate::skillnum::sections(tail)
        .into_iter()
        .any(|(_, t)| base_titles.contains(&t))
}

pub fn tail_by_title<'a>(at_base: &str, mine: &'a str) -> Option<&'a str> {
    let base_titles: std::collections::BTreeSet<String> = crate::skillnum::sections(at_base)
        .into_iter()
        .map(|(_, t)| t)
        .collect();
    let mut last_shared_end: Option<usize> = None;
    let mut offset = 0usize;
    let mut current_shared_start: Option<usize> = None;
    for line in mine.split_inclusive('\n') {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some((num, title)) = rest.split_once(". ") {
                if num.parse::<usize>().is_ok() {
                    if let Some(start) = current_shared_start.take() {
                        let _ = start;
                        last_shared_end = Some(offset);
                    }
                    if base_titles.contains(title.trim()) {
                        current_shared_start = Some(offset);
                    }
                }
            }
        }
        offset += line.len();
    }
    if current_shared_start.is_some() {
        // The last numbered section is the base's: nothing was appended.
        return Some(&mine[mine.len()..]);
    }
    last_shared_end.map(|i| &mine[i..])
}

/// The rewritten tail and the moves, given the base file and the appended tail.
///
/// The first number comes from `at_base` and NEVER from `tail`. The tail already
/// carries the numbers that collided; reading them back is exactly how the same
/// branch collided a second time, four hours after the first repair.
pub fn plan(at_base: &str, tail: &str) -> (String, Vec<(usize, usize)>) {
    renumber(tail, max_section(at_base) + 1)
}

/// The first number to use, given the base and an optional caller-chosen start.
///
/// `--first` exists because two of my own open branches once claimed the same
/// number and I reached for `--base origin/<sibling>`. That numbered correctly
/// around the sibling -- and rebuilt my file ON it, so my branch carried the
/// sibling's sections and would have merged them under my PR. The right move is
/// for both branches to number against the SHARED base and for the second one to
/// start higher, which needs a number and not a different base.
///
/// A start at or below the base's highest is refused. It would produce a
/// collision with the very file it is numbering against, which is the one thing
/// this command exists to prevent.
pub fn first_number(at_base: &str, requested: Option<usize>) -> Result<usize> {
    let floor = max_section(at_base);
    match requested {
        None => Ok(floor + 1),
        Some(n) if n > floor => Ok(n),
        Some(n) => bail!(
            "--first {n} is not above the base's highest section ({floor}). \
             Starting there collides with the file being numbered against."
        ),
    }
}

fn show(rev: &str, path: &str, root: &Path) -> Result<String> {
    let out = Command::new("git")
        .args(["show", &format!("{rev}:{path}")])
        .current_dir(root)
        .output()?;
    if !out.status.success() {
        bail!("git show {rev}:{path} failed -- is {rev} fetched?");
    }
    Ok(String::from_utf8(out.stdout)?)
}

/// Titles in the tail that the base once held, in the order they appear.
///
/// The predicate is injected so the decision can be tested without a git
/// history: what it does with a yes and a no is the part that has to be right,
/// and `git log -S` is the part that has to be measured.
pub fn withdrawn_titles<F>(carried: &[(usize, String)], once_held: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    carried
        .iter()
        .filter(|(_, t)| once_held(t))
        .map(|(_, t)| t.clone())
        .collect()
}

/// Whether `base`'s history has ever contained this section title.
///
/// A title the base once held and no longer does was taken out on purpose, and
/// carrying it forward resurrects a retraction. `git log -S` is the test that
/// survives squash-merging, which is what defeats both the merge base and
/// ancestry here.
fn base_once_held(base: &str, file: &str, title: &str, root: &Path) -> bool {
    let out = Command::new("git")
        .args(["log", base, "--format=%H", "-S", title, "--", file])
        .current_dir(root)
        .output();
    match out {
        Ok(o) if o.status.success() => !String::from_utf8_lossy(&o.stdout).trim().is_empty(),
        _ => false,
    }
}

fn merge_base(rev: &str, root: &Path) -> Result<String> {
    let out = Command::new("git")
        .args(["merge-base", "HEAD", rev])
        .current_dir(root)
        .output()?;
    if !out.status.success() {
        bail!("no merge base with {rev}");
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

/// `text` with every section whose title is in `drop` removed.
///
/// A section runs from its `## N. ` heading to the next one, and a heading
/// inside a fenced block is a quotation rather than a section -- the same rule
/// `skillnum::sections` uses, because this file quotes headings as evidence and
/// cutting at one of those would take the wrong span.
pub fn without_sections(text: &str, drop: &[String]) -> String {
    let drop: std::collections::BTreeSet<&str> = drop.iter().map(|s| s.as_str()).collect();
    let mut out: Vec<&str> = Vec::new();
    let mut skipping = false;
    let mut in_fence = false;
    for line in text.lines() {
        if let Some(info) = line.strip_prefix("```") {
            if !in_fence {
                in_fence = true;
            } else if info.trim().is_empty() {
                in_fence = false;
            }
            if !skipping {
                out.push(line);
            }
            continue;
        }
        let heading = if in_fence {
            None
        } else {
            line.strip_prefix("## ")
                .and_then(|r| r.split_once(". "))
                .filter(|(n, _)| n.parse::<usize>().is_ok())
                .map(|(_, t)| t.trim())
        };
        if let Some(t) = heading {
            skipping = drop.contains(t);
        }
        if !skipping {
            out.push(line);
        }
    }
    let mut s = out.join("\n");
    if text.ends_with('\n') && !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

pub fn run(
    base: &str,
    file: &str,
    check: bool,
    first_req: Option<usize>,
    drop_withdrawn: bool,
) -> Result<()> {
    let root = crate::find_trinity_root()?;
    let mb = merge_base(base, &root)?;
    let at_mb = show(&mb, file, &root)?;
    let at_base = show(base, file, &root)?;
    let mine = std::fs::read_to_string(root.join(file))?;

    // The byte-prefix tail is everything appended since the merge base -- which
    // is wrong the moment a SIBLING branch of yours lands part of that tail on
    // the base. A squash merge puts the section on `base` under a new commit, so
    // the same content sits on both sides, and rebuilding as `at_base + tail`
    // emits it twice.
    //
    // Reproduced on this repository, 2026-09-05: branch tip 2ded340a against
    // master 747e4a1, merge base 013b829. The tool reported `appended here 2`
    // and moved 546 -> 547, 547 -> 548, producing
    //
    //     ## 546. A mutation that also edits the test is not a mutation test
    //     ## 547. A mutation that also edits the test is not a mutation test
    //     ## 548. The tool that finds unchecked constants was counting its own tests
    //
    // because #3199 had squash-merged that first section onto master while this
    // branch was open. `tail_by_title` -- already here for the case where the
    // merge base is not a prefix -- answers exactly this, so the byte-prefix
    // tail is accepted only when it shares no title with the base.
    let prefix_tail = appended_tail(&at_mb, &mine).filter(|t| tail_is_new(t, &at_base));
    let (tail, how) = match prefix_tail {
        Some(t) => (Some(t), "byte prefix of the merge base"),
        None => (
            tail_by_title(&at_base, &mine),
            "TITLES shared with the base -- either the merge base is not a prefix\n  \
             (the base edited an existing section while this branch was open), or\n  \
             the appended tail carries a section the base ALREADY HAS, which is\n  \
             what a sibling branch of yours squash-merging does. Renumbering that\n  \
             tail by position would emit it twice.",
        ),
    };
    let Some(tail) = tail else {
        bail!(
            "{file} is not {} plus an append.\n  \
             This command moves the sections you APPENDED; it cannot tell which\n  \
             lines are yours once an existing section has been edited. Resolve by\n  \
             hand, or reset the file to {} and re-append.",
            &mb[..9.min(mb.len())],
            &mb[..9.min(mb.len())]
        );
    };

    // A section in the tail that the BASE ONCE HELD AND REMOVED is not mine to
    // carry. Absence from the base's head looks identical whether I wrote the
    // section or the base withdrew it, and on 2026-09-05 that cost a
    // resurrection: master rewrote 554 "Debt a fix cannot retire" in 6a49402c
    // because the claim was wrong, my branch had merged the version before the
    // correction, and a by-title rebuild put the withdrawn text back under a
    // fresh number.
    //
    // Two cheaper discriminators do not work here. The MERGE BASE is useless
    // once the branch has already merged the base -- it becomes the base's head
    // and everything looks equally new. ANCESTRY is useless because this
    // repository squash-merges, so the commit that introduced the withdrawn
    // section is not an ancestor of the base, and neither are mine.
    //
    // What works is the base's own history OF THE TEXT. Measured: the withdrawn
    // title returns 2 commits (one adding, one removing) and each of the two
    // sections I had actually written returns 0.
    let carried = crate::skillnum::sections(tail);
    let withdrawn = withdrawn_titles(&carried, |t| base_once_held(base, file, t, &root));
    // Of this command's refusals, this is the only one whose repair is
    // unambiguous. The others end in "resolve by hand" because the safe action
    // is unknown -- a section that would be DROPPED might be yours or the
    // base's, and guessing loses work. Here the answer is settled: the base
    // removed these on purpose, and carrying them forward resurrects a
    // retraction. So `--drop-withdrawn` does exactly that and names each one.
    //
    // It is opt-in, not the default. Deleting text nobody asked to delete is
    // how a tool earns distrust, and the refusal already prints the list.
    let owned_tail;
    let tail = if withdrawn.is_empty() {
        tail
    } else if drop_withdrawn {
        owned_tail = without_sections(tail, &withdrawn);
        println!();
        println!("  {} section(s) dropped: {base} removed them on purpose.", withdrawn.len());
        for t in &withdrawn {
            println!("    {t}");
        }
        println!("  --drop-withdrawn asked for this. Without it the run refuses and");
        println!("  prints the same list, which is the default.");
        owned_tail.as_str()
    } else {
        bail!(
            "{} section(s) in the tail were REMOVED from {base} on purpose:\n    {}\n  \
             Nothing was written. Absence from {base} today looks the same whether \
             you wrote a section or the base withdrew it; its history does not. \
             Re-run with --drop-withdrawn to remove exactly these, or keep them \
             deliberately and say why.",
            withdrawn.len(),
            withdrawn.join("\n    ")
        );
    };

    let first = first_number(&at_base, first_req)?;
    let (moved, moves) = renumber(tail, first);

    println!();
    println!("  {file}");
    println!("  merge base {}   {} section(s) there", &mb[..9.min(mb.len())], crate::skillnum::sections(&at_mb).len());
    println!("  {base} highest section  {}", max_section(&at_base));
    if first_req.is_some() {
        println!("  --first                 {first}   (asked for, not derived)");
    }
    println!("  appended here           {}", crate::skillnum::sections(tail).len());
    println!("  tail identified by      {how}");
    // The joiner is explicit, and it is the defect this command shipped with:
    // a tail from `tail_by_title` starts AT its first `## ` heading with no
    // leading newline, so concatenating it onto a trimmed base glued the
    // heading to the base's last line. The heading stopped being a heading --
    // 435 sections where 436 were expected -- and the count is what caught it,
    // not reading the file.
    let out = format!(
        "{}\n\n\n{}",
        at_base.trim_end_matches('\n'),
        moved.trim_start_matches('\n')
    );
    let rebuilt = out != mine;
    if moves.is_empty() {
        println!();
        println!("  Nothing to move -- your numbers already follow {base}.");
        if !rebuilt {
            return Ok(());
        }
        // The numbers are right and the FILE is still stale: this is the other
        // half of the same repair. Rebuilding on `at_base` is exactly the
        // conflict resolution, so say what it costs before doing it.
        println!("  The base region is still behind {base}, so the file is rebuilt");
        println!("  on it. Sections you APPENDED are kept; any edit you made to a");
        println!("  section the base already had is discarded -- that is the one");
        println!("  thing this command cannot tell from a base-side rewording.");
    }
    println!();
    for (o, n) in &moves {
        println!("      {o}  ->  {n}");
    }
    println!();

    if check {
        println!("  --check: nothing written.");
        return Ok(());
    }

    debug_assert!(out.starts_with(at_base.trim_end_matches('\n')));
    let secs = crate::skillnum::sections(&out);
    let expected = crate::skillnum::sections(&at_base).len() + crate::skillnum::sections(tail).len();
    if secs.len() != expected {
        bail!(
            "the rebuilt file has {} section(s) and must have {} ({} from {base} + {} appended). \
             Nothing was lost from disk -- the write is refused.",
            secs.len(),
            expected,
            crate::skillnum::sections(&at_base).len(),
            crate::skillnum::sections(tail).len()
        );
    }
    // The count guard above is a TOTAL, and a total cannot see a substitution.
    // It passed on 2026-09-05 while this command deleted a section: SKILL 548
    // quotes three `## N.` heading lines inside a fenced block as evidence, the
    // section parser counts every line that starts `## N. ` whether fenced or
    // not, and those three made the arithmetic come out right while the real
    // section they were quoting was dropped. Numbers matched; content did not.
    //
    // So the guard that matters is the SET of titles, which is the guard every
    // hand-written resolver in this loop had and this command did not.
    // Minus what --drop-withdrawn was asked to remove. Those ARE on disk and
    // ARE gone from the rebuild, so this guard sees them and refuses -- which
    // is correct in general and wrong here, because the caller named them.
    //
    // Two guards, one blocking the other's sanctioned repair. The first version
    // of --drop-withdrawn did exactly what it promised, printed what it dropped,
    // and then died on this line. **A guard has to know what the operator
    // authorised, or the authorisation is not real.**
    let sanctioned: std::collections::BTreeSet<&str> =
        withdrawn.iter().map(|s| s.as_str()).collect();
    let lost: Vec<String> = titles_lost(&mine, &out)
        .into_iter()
        .filter(|t| !sanctioned.contains(t.as_str()))
        .collect();
    if !lost.is_empty() {
        bail!(
            "the rebuild would DROP {} section(s) that are on disk now:\n    {}\n  \
             Nothing was written. The section count came out right, which is why \
             the count guard above did not stop it -- a total cannot see a \
             substitution. Resolve this file by hand.",
            lost.len(),
            lost.join("\n    ")
        );
    }
    let problems = crate::skillnum::problems(&secs);
    std::fs::write(root.join(file), &out)?;
    println!("  Written. {} section(s); {}", secs.len(), if problems.is_empty() {
        "no number is used twice.".to_string()
    } else {
        format!("STILL WRONG: {}", problems.join("; "))
    });
    if !problems.is_empty() {
        bail!("the rewrite did not settle the numbering");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `without_sections` cuts from a `## N. ` heading to the next one, and a
    /// heading inside a fenced block is a QUOTATION -- this file quotes headings
    /// as evidence, and cutting at one would take the wrong span.
    #[test]
    fn dropping_a_section_stops_at_the_next_real_heading() {
        let src = "## 1. Keep\nalpha\n\n## 2. Drop\nbeta\n\n```\n## 9. Quoted\n```\ngamma\n\n## 3. Keep too\ndelta\n";
        let out = without_sections(src, &["Drop".to_string()]);
        assert!(out.contains("alpha") && out.contains("delta"), "neighbours survive: {out}");
        assert!(!out.contains("beta"), "the body goes");
        assert!(!out.contains("gamma"), "and so does everything to the next REAL heading");
        assert!(!out.contains("## 2. Drop"), "heading included");
        assert_eq!(
            crate::skillnum::sections(&out).len(),
            2,
            "two sections left, and the quoted heading is not one of them"
        );
    }

    /// Dropping nothing must return the text unchanged, byte for byte: the
    /// ordinary run passes through this function and must not be reshaped by it.
    #[test]
    fn dropping_nothing_changes_nothing() {
        let src = "## 1. A\nalpha\n\n## 2. B\nbeta\n";
        assert_eq!(without_sections(src, &[]), src);
    }

    /// The guard that refuses a lost title must know what the operator
    /// AUTHORISED. The first version of `--drop-withdrawn` did exactly what it
    /// promised, printed what it dropped, and then died on `titles_lost` --
    /// which saw a section leave the file and refused.
    #[test]
    fn a_sanctioned_removal_is_not_a_lost_title() {
        let before = "## 1. A\n\n## 2. Withdrawn\n\n## 3. C\n";
        let after = without_sections(before, &["Withdrawn".to_string()]);
        assert!(
            titles_lost(before, &after).contains(&"Withdrawn".to_string()),
            "the guard DOES see it -- that is why it has to be told"
        );
        let sanctioned = ["Withdrawn".to_string()];
        let remaining: Vec<String> = titles_lost(before, &after)
            .into_iter()
            .filter(|t| !sanctioned.contains(t))
            .collect();
        assert!(
            remaining.is_empty(),
            "and with the authorisation applied, nothing is refused: {remaining:?}"
        );
    }

    /// A section the base once held and dropped is not mine to carry forward.
    ///
    /// Measured 2026-09-05: master rewrote 554 "Debt a fix cannot retire" in
    /// 6a49402c because the claim was wrong. My branch had merged the version
    /// before the correction, so the withdrawn title read as present-here-
    /// absent-there -- exactly like a section I had written -- and a by-title
    /// rebuild put it back under a fresh number. `git log origin/master -S` on
    /// that title returns 3 commits; on each section I had actually written, 0.
    #[test]
    fn a_title_the_base_once_held_is_not_carried() {
        let carried = vec![
            (554usize, "Debt a fix cannot retire".to_string()),
            (555, "The audit that found nothing".to_string()),
            (556, "Two bare headings in NOW.md".to_string()),
        ];
        let held = |t: &str| t == "Debt a fix cannot retire";
        assert_eq!(
            withdrawn_titles(&carried, held),
            vec!["Debt a fix cannot retire".to_string()],
            "only the one the base's history holds is refused"
        );
        assert!(
            withdrawn_titles(&carried, |_| false).is_empty(),
            "a base that never held any of them refuses none -- the ordinary case \
             must not start failing"
        );
        assert_eq!(
            withdrawn_titles(&carried, |_| true).len(),
            3,
            "and if the base held every one of them, every one is named"
        );
    }

    /// An empty tail cannot contain a withdrawal, and must not be reported as
    /// one: `renumber` runs on every merge and a false refusal blocks the work.
    #[test]
    fn an_empty_tail_refuses_nothing() {
        assert!(withdrawn_titles(&[], |_| true).is_empty());
    }

    /// The predicate is right and the call site can still never ask it.
    /// NINTH change in nine passes; mutating the call site first is now the rule.
    #[test]
    fn the_guard_is_consulted_before_the_renumber() {
        let src = include_str!("renum.rs");
        let boundary = src
            .lines()
            .position(|l| l == "#[cfg(test)]")
            .expect("the test module is a line of its own");
        let code: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        let call = concat!("let withdrawn = withdrawn_", "titles(&carried,");
        assert!(code.contains(call), "the guard has to be asked");
        // A non-empty answer either bails, or is dropped BECAUSE the operator
        // asked for it. Nothing else may happen to it.
        let opt_in = concat!("} else if drop_with", "drawn {");
        assert!(code.contains(opt_in), "removal must be opt-in, never the default");
        assert!(
            code.contains("REMOVED from {base} on purpose"),
            "and without the flag it still refuses, naming them"
        );
        assert!(
            code.find(call).unwrap() < code.find("let (moved, moves) = renumber(").unwrap(),
            "before the renumber, not after it"
        );
    }

    /// A sibling branch of yours squash-merging is the case the byte-prefix tail
    /// cannot see. Reproduced on this repository, 2026-09-05: branch tip
    /// 2ded340a against master 747e4a1, merge base 013b829. The tool reported
    /// `appended here 2` and produced two sections with the SAME title, because
    /// #3199 had landed the first of them on master while the branch was open.
    #[test]
    fn a_tail_the_base_already_carries_is_not_an_append() {
        let at_mb = "## 1. A\n\nbody a\n";
        // The base gained B -- which is also sitting in this branch's tail,
        // because it came from a sibling PR of the same author.
        let at_base = "## 1. A\n\nbody a\n\n## 2. B\n\nbody b\n";
        let mine = "## 1. A\n\nbody a\n\n## 2. B\n\nbody b\n\n## 3. C\n\nbody c\n";

        // The byte-prefix tail sees both B and C as appended here.
        let raw = appended_tail(at_mb, mine).expect("mb is a prefix of mine");
        assert_eq!(
            crate::skillnum::sections(raw).len(),
            2,
            "B and C both look appended, and rebuilding on the base would emit B twice"
        );

        // By title, only C is genuinely new.
        let by_title = tail_by_title(at_base, mine).expect("a shared title exists");
        let secs = crate::skillnum::sections(by_title);
        assert_eq!(secs.len(), 1, "only C is new: {secs:?}");
        assert_eq!(secs[0].1, "C");
    }

    /// `titles_lost` can be right while `run` never consults it, and the write
    /// goes ahead. SEVENTH change in seven passes whose surviving mutant was
    /// the wiring rather than the function -- and the second one predicted
    /// before it was run.
    #[test]
    fn run_refuses_the_write_when_a_title_would_be_lost() {
        let src = include_str!("renum.rs");
        let boundary = src
            .lines()
            .position(|l| l == "#[cfg(test)]")
            .expect("the test module is a line of its own");
        let code: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        let call = concat!("titles_", "lost(&mine, &out)");
        assert!(code.contains(call), "the guard has to be consulted before the write");
        // It is now filtered by what --drop-withdrawn authorised. Without that
        // filter the flag does its job, prints what it dropped, and dies here.
        let exempt = concat!("!sanctioned.contains(", "t.as_str())");
        assert!(
            code.contains(exempt),
            "and it has to know what the operator authorised, or the \
             authorisation is not real"
        );
        let refuses = concat!("if !lost.is_", "empty() {");
        assert!(
            code.contains(refuses),
            "and a non-empty answer has to stop the write, not merely be printed"
        );
        let write = code.find("std::fs::write(root.join(file)").expect("the write is here");
        assert!(
            code.find(call).unwrap() < write,
            "the guard must run BEFORE the write, or it reports a loss already on disk"
        );
    }

    /// A count is a total, and a total cannot see a substitution.
    ///
    /// On 2026-09-05 this command deleted a section while its count guard
    /// passed. SKILL 548 quotes three `## N.` heading lines inside a fenced
    /// block as evidence; the parser counts every line starting `## N. `
    /// whether fenced or not, so those three filled the seats of the real
    /// section that was dropped. The arithmetic was right and the content was
    /// gone.
    #[test]
    fn a_lost_title_is_named_even_when_the_count_matches() {
        let before = "## 1. A\n\n## 2. B\n\n## 3. C\n";
        // Same number of sections; B has been replaced by a second copy of A.
        let after = "## 1. A\n\n## 2. A\n\n## 3. C\n";
        assert_eq!(
            crate::skillnum::sections(before).len(),
            crate::skillnum::sections(after).len(),
            "the totals agree, which is exactly why a count guard let this through"
        );
        assert_eq!(
            titles_lost(before, after),
            vec!["B".to_string()],
            "and the set says what the count could not"
        );
    }

    #[test]
    fn nothing_lost_is_an_empty_list_not_a_zero() {
        let before = "## 1. A\n\n## 2. B\n";
        let after = "## 1. A\n\n## 2. B\n\n## 3. C\n";
        assert!(
            titles_lost(before, after).is_empty(),
            "adding C loses nothing, and a renumber that only appends must be allowed"
        );
        // Renumbering alone must never register as a loss: the guard is on
        // TITLES precisely so that moving 547 -> 548 is invisible to it.
        let moved = "## 41. A\n\n## 42. B\n";
        assert!(
            titles_lost(before, moved).is_empty(),
            "the numbers all changed and not one title did"
        );
    }

    /// The predicate that decides whether the byte-prefix tail may be trusted.
    #[test]
    fn a_tail_is_new_only_when_the_base_has_none_of_it() {
        let base = "## 1. A\n\nbody a\n\n## 2. B\n\nbody b\n";
        assert!(
            tail_is_new("## 3. C\n\nbody c\n", base),
            "C is nowhere on the base, so the byte-prefix tail is the precise answer"
        );
        assert!(
            !tail_is_new("## 2. B\n\nbody b\n\n## 3. C\n\nbody c\n", base),
            "B is ALREADY on the base -- renumbering this tail by position emits it twice"
        );
        assert!(
            !tail_is_new("## 9. B\n\nbody b\n", base),
            "the match is by TITLE, not by number: a renumbered duplicate is still a duplicate"
        );
        assert!(
            tail_is_new("", base),
            "an empty tail carries nothing the base could already have"
        );
    }

    /// `tail_is_new` can be right while `run` never consults it. Dropping the
    /// `.filter(...)` restores the defect with every test above still green.
    ///
    /// SIXTH change in six passes whose surviving mutant was the wiring rather
    /// than the function. The needle is split across two literals so this test's
    /// own body does not contain the string it searches for.
    #[test]
    fn run_actually_filters_the_byte_prefix_tail() {
        let src = include_str!("renum.rs");
        let boundary = src
            .lines()
            .position(|l| l == "#[cfg(test)]")
            .expect("the test module is a line of its own");
        let code: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        let guarded = concat!(".filter(|t| ", "tail_is_new(t, &at_base))");
        assert!(
            code.contains(guarded),
            "without this the byte-prefix tail is trusted even when the base \
             already carries part of it, and the rebuild emits a section twice"
        );
    }

    /// The ordinary case must not change: with no overlap the byte-prefix tail
    /// is still the precise answer, and it is the one that keeps a base-side
    /// rewording of an EXISTING section from being silently discarded.
    #[test]
    fn a_clean_append_still_uses_the_byte_prefix() {
        let at_mb = "## 1. A\n\nbody a\n";
        let at_base = "## 1. A\n\nbody a\n";
        let mine = "## 1. A\n\nbody a\n\n## 2. B\n\nbody b\n";
        let raw = appended_tail(at_mb, mine).expect("mb is a prefix");
        let base_titles: std::collections::BTreeSet<String> =
            crate::skillnum::sections(at_base)
                .into_iter()
                .map(|(_, t)| t)
                .collect();
        let overlaps = crate::skillnum::sections(raw)
            .into_iter()
            .any(|(_, t)| base_titles.contains(&t));
        assert!(!overlaps, "nothing in the tail is on the base, so the prefix stands");
    }

    const BASE: &str = "intro\n\n## 10. Ten\n\nbody ten\n";

    #[test]
    fn an_append_is_recognised_and_an_edit_is_not() {
        let mine = format!("{BASE}\n## 10. Ten\n\nmine\n");
        assert!(appended_tail(BASE, &mine).is_some());
        // An edited word inside the base is NOT an append, and the whole point
        // is that this returns None instead of guessing a split point.
        let edited = BASE.replace("body ten", "body TEN") + "\n## 11. X\n";
        assert!(appended_tail(BASE, &edited).is_none());
    }

    #[test]
    fn the_first_number_comes_from_the_base_not_from_the_tail() {
        let tail = "\n## 11. A\n\na\n\n## 12. B\n\nb\n";
        let (out, moves) = renumber(tail, 471);
        assert_eq!(moves, vec![(11, 471), (12, 472)]);
        assert!(out.contains("## 471. A"), "{out}");
        assert!(out.contains("## 472. B"), "{out}");
        assert!(!out.contains("## 11."), "{out}");
    }

    #[test]
    fn a_reference_to_a_moved_sibling_follows_it_and_a_foreign_one_does_not() {
        let tail = "\n## 11. A\n\nsee &sect;12 and &sect;447\n\n## 12. B\n\nb\n";
        let (out, _) = renumber(tail, 471);
        assert!(out.contains("&sect;472"), "the sibling did not follow:\n{out}");
        assert!(
            out.contains("&sect;447"),
            "a reference this command does not own was rewritten:\n{out}"
        );
    }

    #[test]
    fn a_prefix_of_a_longer_number_is_not_a_reference_to_it() {
        // The bug this exists for: renumbering 11 must not touch &sect;110.
        let tail = "\n## 11. A\n\nsee &sect;110\n";
        let (out, _) = renumber(tail, 471);
        assert!(
            out.contains("&sect;110"),
            "a longer number was corrupted by a prefix match:\n{out}"
        );
        assert!(!out.contains("&sect;4710"), "{out}");
    }

    /// The case that broke the prefix test on this command's first real use:
    /// the base REWORDED a section this branch also has. Byte-prefix says no;
    /// titles still separate the two files correctly.
    #[test]
    fn a_reworded_base_section_does_not_hide_the_tail() {
        let base = "## 10. Ten\n\nbody ten, REWRITTEN by master\n";
        let mine = "## 10. Ten\n\nbody ten\n\n## 11. Mine\n\nmine\n";
        assert!(
            appended_tail(base, mine).is_none(),
            "the exact test must still say no"
        );
        let t = tail_by_title(base, mine).expect("titles must find it");
        assert!(t.contains("## 11. Mine"), "{t:?}");
        assert!(!t.contains("## 10. Ten"), "the base's section leaked in: {t:?}");
    }

    #[test]
    fn a_branch_that_appended_nothing_yields_an_empty_tail() {
        let base = "## 10. Ten\n\nbody\n";
        let mine = "## 10. Ten\n\nbody reworded here\n";
        let t = tail_by_title(base, mine).expect("shares a section");
        assert!(
            crate::skillnum::sections(t).is_empty(),
            "nothing was appended, so nothing may move: {t:?}"
        );
    }

    /// The joiner, at the level where it can be tested without a repository.
    /// A tail found by title starts at `## `, and gluing that onto a trimmed
    /// base turns the heading into ordinary text.
    #[test]
    fn rebuilding_keeps_every_heading_at_the_start_of_a_line() {
        let base = "## 10. Ten\n\nbody ends here.";
        let mine = "## 10. Ten\n\nbody ends here.\n\n## 11. Mine\n\nmine\n";
        let tail = tail_by_title(base, mine).expect("tail");
        let (moved, _) = plan(base, tail);
        let out = format!(
            "{}\n\n\n{}",
            base.trim_end_matches('\n'),
            moved.trim_start_matches('\n')
        );
        assert_eq!(
            crate::skillnum::sections(&out).len(),
            2,
            "a heading was glued to the previous line:\n{out}"
        );
        assert!(out.contains("\n## 11. Mine"), "{out}");
    }

    #[test]
    fn two_files_with_no_shared_section_are_refused() {
        assert!(tail_by_title("## 10. Ten\n", "## 99. Other\n").is_none());
    }

    /// Two open branches, one shared base. Both number against the base and the
    /// second starts higher -- which needs a NUMBER, not a different base. The
    /// alternative I actually reached for, `--base origin/<sibling>`, rebuilds
    /// the file on the sibling and carries its sections into your PR.
    #[test]
    fn a_requested_start_above_the_base_is_used_verbatim() {
        let base = "## 470. last\n\nbody\n";
        assert_eq!(first_number(base, Some(475)).unwrap(), 475);
        assert_eq!(first_number(base, None).unwrap(), 471);
    }

    #[test]
    fn a_requested_start_that_would_collide_is_refused() {
        let base = "## 470. last\n\nbody\n";
        for n in [1, 470] {
            let e = first_number(base, Some(n));
            assert!(e.is_err(), "--first {n} must be refused against a base whose highest is 470");
            let msg = format!("{}", e.unwrap_err());
            assert!(msg.contains("470"), "the refusal must name the floor: {msg}");
        }
        // And the boundary is strict on the right side too.
        assert_eq!(first_number(base, Some(471)).unwrap(), 471);
    }

    #[test]
    fn a_tail_with_no_sections_moves_nothing() {
        let (out, moves) = renumber("\njust prose, no headings\n", 471);
        assert!(moves.is_empty());
        assert_eq!(out, "\njust prose, no headings\n");
    }

    /// The numbers a tail ALREADY carries must not reach the plan. Two tails
    /// with different existing numbers and the same shape must be renumbered
    /// identically -- otherwise the tail is deciding, and a branch that was
    /// already renumbered once will renumber itself off its own numbers instead
    /// of off the base. That is not hypothetical: it is the second of the six
    /// collisions.
    #[test]
    fn the_plan_ignores_the_numbers_the_tail_already_carries() {
        let base = "## 470. last\n\nbody\n";
        let fresh = "\n## 1. A\n\na\n\n## 2. B\n\nb\n";
        let already_moved = "\n## 468. A\n\na\n\n## 469. B\n\nb\n";
        let (o1, m1) = plan(base, fresh);
        let (o2, m2) = plan(base, already_moved);
        assert_eq!(
            m1.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
            m2.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
            "the tail's own numbers changed the destination"
        );
        assert_eq!(m1.iter().map(|(_, n)| *n).collect::<Vec<_>>(), vec![471, 472]);
        assert_eq!(o1, o2, "same base, same shape, different output");
    }

    #[test]
    fn the_highest_section_is_read_from_the_text_it_is_given() {
        assert_eq!(max_section("## 3. a\n## 470. b\n## 12. c\n"), 470);
        assert_eq!(max_section("no sections here"), 0);
    }
}
