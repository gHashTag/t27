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

pub fn run(base: &str, file: &str, check: bool, first_req: Option<usize>) -> Result<()> {
    let root = crate::find_trinity_root()?;
    let mb = merge_base(base, &root)?;
    let at_mb = show(&mb, file, &root)?;
    let at_base = show(base, file, &root)?;
    let mine = std::fs::read_to_string(root.join(file))?;

    let (tail, how) = match appended_tail(&at_mb, &mine) {
        Some(t) => (Some(t), "byte prefix of the merge base"),
        None => (
            tail_by_title(&at_base, &mine),
            "TITLES shared with the base -- the merge base is not a prefix, which\n  \
             happens when the base edited an existing section while this branch\n  \
             was open",
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
