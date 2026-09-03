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
        /// Print the titles excluded by the two-digit rule alone.
        #[arg(long)]
        single: bool,
        /// Print a systematic sample of this size. 0 prints only the population.
        #[arg(long, default_value_t = 0)]
        sample: usize,
        /// How many issues to read. The read is a LOWER BOUND when this many
        /// come back -- the output says so rather than presenting a page as a
        /// total.
        #[arg(long, default_value_t = 500)]
        limit: usize,
        /// Count the backlog as it stood at the END of this UTC day
        /// (`YYYY-MM-DD`), instead of now. Without it the population is a
        /// query whose answer changes on every open and close, and the number
        /// cannot be re-taken by a second reader.
        #[arg(long)]
        as_of: Option<String>,
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

/// Titles excluded from the population by the two-digit rule alone.
///
/// The digit rule requires a run of **two or more** digits. That threshold was
/// never documented and it does real work: 20 open titles carry a single-digit
/// figure and nothing else, and they are not one kind of thing. Roughly a dozen
/// state a count -- *"`implies` appears 9 times in live source and 0 times in
/// the compiler"*, *"MAX_SORRY counts 5 admitted proofs; 4 are in files nothing
/// compiles"*, *"4 of 7 passes have no precondition"*. The rest state a VALUE:
/// an exit code (`seal exits 0`), a literal (`the lexer turns 0o777 into 0`),
/// or arithmetic (`-3/2 is -1, -3>>1 is -2`).
///
/// So the threshold is a crude proxy for *not a value*, wrong in one direction,
/// and removing it takes the population from **288 to 308** while adding about
/// eight titles that state no count. It is kept -- and it is now **printed**.
/// A silent threshold makes 288 read as the whole population; a stated one
/// makes it read as 288 plus a named 20 that a reader can judge.
pub fn single_digit_only(title: &str) -> bool {
    let t = strip_addresses(title);
    let c: Vec<char> = t.chars().collect();
    let (mut i, mut one, mut two) = (0usize, false, false);
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
        if left && right {
            if i - start >= 2 {
                two = true;
            } else {
                one = true;
            }
        }
    }
    one && !two && !NUMERALS.iter().any(|w| has_word(&t, w))
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
            // A single digit with the boundary satisfied is NOT counted here.
            // The threshold is deliberate and measured; `single_digit_only`
            // carries the reason and the twenty titles it excludes.
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

/// Was this issue open at `instant`?
///
/// Both timestamps come from GitHub as ISO-8601 with a `Z` suffix, and such strings
/// compare lexicographically in chronological order -- so this needs no date library
/// and cannot drift from one. An issue with no `closedAt` is open now and was open
/// then, provided it existed.
///
/// The empty-`created` case returns false rather than defaulting to open: a row whose
/// creation time did not arrive is a row this cannot classify, and guessing would put
/// it in the population silently.
pub fn open_at(created: &str, closed: &str, instant: &str) -> bool {
    if created.is_empty() || created > instant {
        return false;
    }
    closed.is_empty() || closed > instant
}

/// Today's UTC date as `YYYY-MM-DD`.
///
/// Days since the epoch, then the civil date, by the standard algorithm. No
/// subprocess and no date crate: this file already compares ISO-8601 strings from
/// GitHub, and one more string of the same shape keeps the comparison lexicographic.
pub fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    format!("{y:04}-{m:02}-{d:02}")
}

/// Days since 1970-01-01 to a civil `(year, month, day)`.
///
/// Hinnant's algorithm, shifted to a March-based year so that the leap day lands at
/// the end and the month lengths become a single linear formula. It is transcribed
/// rather than invented, and it is tested against dates chosen for what they break:
/// an epoch, a leap day, a century that is not a leap year, and one that is.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `YYYY-MM-DD` to the last instant of that UTC day.
///
/// The END of the day, not the start, because that is what GitHub's own search means
/// by `created:<=2026-08-01` -- a bare date there covers the whole day. Two tools
/// answering the same question must mean the same thing by the same date, or the
/// second reader gets a different number and blames the first.
///
/// The shape check is `skillnum::is_iso_date`, the rule already mutation-proved for
/// the skill anchors, rather than a second copy of the same ten conjuncts here.
pub fn instant_of(date: &str, today: &str) -> Result<String> {
    let c: Vec<char> = date.chars().collect();
    if c.len() != 10 || !crate::skillnum::is_iso_date(&c) {
        anyhow::bail!(
            "--as-of wants YYYY-MM-DD and got `{date}`. A date this cannot read is \
             refused rather than silently treated as today, which would print a \
             number over the wrong population under an anchor that looks right."
        );
    }
    // A day that has not ENDED cannot be read as a completed day, and GitHub does not
    // say so: `created:<=2027-01-01` answers with today's set, and the first version
    // of this command printed today's 486 under the heading "AS OF 2027-01-01" -- a
    // number that looks anchored, reads as history, and is a clock reading wearing
    // next year's label. Today itself is refused for the same reason as tomorrow: its
    // end is still in the future, so the count will differ from itself by evening.
    if date >= today {
        anyhow::bail!(
            "--as-of {date} asks for a day that has not ended (today is {today} UTC). \
             The answer would be TODAY'S count under a heading that reads as history, \
             which is worse than no anchor at all -- GitHub answers such a query \
             without complaint. Ask for {today} or later tomorrow, or pick a day that \
             has closed."
        );
    }
    Ok(format!("{date}T23:59:59Z"))
}

/// Did the read reach the end, or did it fill the page?
///
/// `gh` returns at most `--limit` rows and says nothing about what it left behind, so
/// a FULL page is a lower bound and anything short of one is complete. The boundary is
/// the whole content of this function: at exactly `limit` rows there may or may not be
/// more, and the honest answer is that this cannot tell -- so it reports incomplete.
pub fn read_is_complete(returned: usize, limit: usize) -> bool {
    returned < limit
}

/// The population of re-measurable open issues, and a reproducible sample of it.
///
/// A rate is only worth taking if the same sample can be taken again, so the
/// sample is SYSTEMATIC -- every k-th issue by ascending number -- rather than
/// chosen. Nothing here is random: run it next month and the overlap is exact
/// wherever the backlog has not moved.
fn numbers(sample: usize, limit: usize, single: bool, as_of: Option<&str>) -> Result<()> {
    let lim = limit.to_string();
    let today = today_utc();
    let instant = as_of.map(|d| instant_of(d, &today)).transpose()?;
    // With --as-of the state filter has to come off: an issue open THEN may be
    // closed NOW, and `--state open` would drop exactly the ones that make the two
    // readings differ. The filtering is done here from the timestamps instead.
    let raw = if instant.is_some() {
        gh(&[
            "issue",
            "list",
            "--state",
            "all",
            "--limit",
            &lim,
            "--json",
            "number,title,createdAt,closedAt",
        ])?
    } else {
        gh(&[
            "issue",
            "list",
            "--state",
            "open",
            "--limit",
            &lim,
            "--json",
            "number,title",
        ])?
    };
    let v: serde_json::Value = serde_json::from_str(&raw).context("gh returned no JSON")?;
    let arr = v.as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        anyhow::bail!(
            "gh returned no open issues -- a repository with none and a query that \
             did not run print the same zero."
        );
    }
    // Whether the READ is complete is a different question from what it contains,
    // and it has to be asked before any total is printed. `gh` returns at most
    // --limit rows and says nothing about what it left behind, so a full page is a
    // LOWER BOUND. Measured 2026-09-03: 486 open against a default limit of 500 --
    // fourteen issues from printing a page as a census, in silence.
    let complete = read_is_complete(arr.len(), limit);
    let mut rows: Vec<(u64, String, Carries)> = arr
        .iter()
        .filter(|i| match instant.as_deref() {
            None => true,
            Some(t) => open_at(
                i["createdAt"].as_str().unwrap_or(""),
                i["closedAt"].as_str().unwrap_or(""),
                t,
            ),
        })
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

    match instant.as_deref() {
        Some(t) => println!(
            "OPEN ISSUES THAT STATE A COUNT IN THE TITLE, AS OF {t}\n\n  \
             This reading is ANCHORED: the population is the set of issues created\n  \
             at or before {t} and not closed by then, which does not move. Run it\n  \
             again next month and every number below is the same.\n"
        ),
        None => println!(
            "OPEN ISSUES THAT STATE A COUNT IN THE TITLE\n\n  \
             This reading is NOT anchored: `open issues` is a query, not a set, and\n  \
             its answer changes on every open and close. Pass --as-of YYYY-MM-DD to\n  \
             take a number a second reader can take again.\n"
        ),
    }
    if complete {
        println!("  issues read from gh           {}   (fewer than the --limit of {limit}, so the read is COMPLETE)", arr.len());
    } else {
        println!("  issues read from gh           {}   *** EQUALS the --limit of {limit}: this is a LOWER BOUND, not a total. Raise --limit and read again. ***", arr.len());
    }
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
    let singles: Vec<&(u64, String, Carries)> =
        rows.iter().filter(|r| single_digit_only(&r.1)).collect();
    println!(
        "  single-digit only, excluded   {}   (--single prints them)",
        singles.len()
    );

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

    println!(
        "\n  The digit rule requires TWO or more digits, and that threshold was\n  \
         never written down. Measured: it excludes {} titles, and they are not\n  \
         one kind of thing. Roughly a dozen state a count -- \"`implies` appears\n  \
         9 times in live source and 0 times in the compiler\" -- and the rest\n  \
         state a VALUE: an exit code, a literal, arithmetic. Dropping the\n  \
         threshold takes the population to {} and admits about eight titles that\n  \
         count nothing. It is kept, and now it is PRINTED: a silent threshold\n  \
         makes this population read as complete.",
        singles.len(),
        pop.len() + singles.len()
    );

    if single {
        println!("\n  EXCLUDED BY THE TWO-DIGIT RULE ALONE:\n");
        for (n, t, _) in singles.iter().copied() {
            println!("    #{n}  {}", &t[..t.len().min(88)]);
        }
    }

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
        IssuesCmd::Numbers {
            sample,
            limit,
            single,
            as_of,
        } => return numbers(*sample, *limit, *single, as_of.as_deref()),
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
    if !read_is_complete(issues.len(), *limit) {
        println!("  issues read from gh     {}   *** EQUALS the --limit of {limit}: a LOWER BOUND, not a total. Raise --limit and read again. ***", issues.len());
    }
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
    if !read_is_complete(issues.len(), limit) {
        println!("  issues read from gh           {}   *** EQUALS the --limit of {limit}: a LOWER BOUND, not a total. Raise --limit and read again. ***", issues.len());
    }
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

#[cfg(test)]
mod single_digit_tests {
    use super::*;

    /// The twenty titles the two-digit threshold removes, in miniature.
    #[test]
    fn a_lone_digit_is_excluded_and_said_so() {
        // Real ones from this backlog: counts the population does not carry.
        assert!(single_digit_only(
            "t27c seal exits 0 on a spec every backend rejects"
        ));
        assert!(single_digit_only("4 of 7 passes have no precondition"));
        assert!(single_digit_only("parser-fix blocker drops 6 -> 5"));
    }

    #[test]
    fn a_two_digit_run_anywhere_takes_it_out_of_the_excluded_set() {
        // It is IN the population, so it is not what this reports.
        assert!(!single_digit_only("5 of 36 gates pass an empty tree"));
        assert!(!single_digit_only("283 titles state a count"));
    }

    #[test]
    fn a_numeral_word_takes_it_out_too() {
        // Already counted as `Words`; reporting it as excluded would double it.
        assert!(!single_digit_only("Nine live sites and 0 in the compiler"));
    }

    #[test]
    fn an_address_is_not_a_lone_digit() {
        // `#2841` is stripped first; what remains states nothing.
        assert!(!single_digit_only(
            "Grep before you file -- #2964 duplicated #2822"
        ));
        assert!(!single_digit_only("Wave Loop 369 is an address"));
        // A SINGLE-digit address is the case that actually exercises the
        // stripping here: without it, `#7` reads as a lone figure. The
        // four-digit examples above pass either way, which is why they are
        // not a control on their own.
        assert!(!single_digit_only("The gate refuses an empty tree -- #7"));
        assert!(!single_digit_only("Prop. 5 is an address, not a count"));
    }

    /// The two sets must not overlap, or the printed totals double-count.
    #[test]
    fn the_population_and_the_excluded_set_are_disjoint() {
        for t in [
            "t27c seal exits 0 on a spec every backend rejects",
            "4 of 7 passes have no precondition",
            "5 of 36 gates pass an empty tree",
            "Nine live sites and 0 in the compiler",
            "#2964 duplicated #2822",
        ] {
            let in_pop = matches!(carries(t), Carries::Digits | Carries::Words | Carries::Both);
            assert!(
                !(in_pop && single_digit_only(t)),
                "{t:?} is in both the population and the excluded set"
            );
        }
    }
}

#[cfg(test)]
mod as_of_tests {
    use super::{instant_of, open_at, read_is_complete};

    const T: &str = "2026-08-01T23:59:59Z";

    /// The probe: the two shapes that ARE open at the instant.
    #[test]
    fn an_issue_is_open_then_if_it_existed_and_had_not_closed() {
        assert!(open_at("2026-07-01T10:00:00Z", "", T), "still open today");
        assert!(
            open_at("2026-07-01T10:00:00Z", "2026-09-01T10:00:00Z", T),
            "closed later, so it was open then"
        );
    }

    /// The counter-examples, and the second is the whole reason `--state open` cannot
    /// be left on the query: an issue closed before the instant is open NOW-negative
    /// and then-negative, but one closed AFTER it is open then and closed now.
    #[test]
    fn it_is_not_open_then_if_it_did_not_exist_or_had_closed() {
        assert!(!open_at("2026-09-01T10:00:00Z", "", T), "created after");
        assert!(
            !open_at("2026-07-01T10:00:00Z", "2026-07-15T10:00:00Z", T),
            "closed before"
        );
    }

    /// Both boundaries are inclusive-of-existing and exclusive-of-surviving, and they
    /// are opposite: created AT the instant counts as existing, closed AT the instant
    /// counts as closed. An issue opened and closed in the same second is not open.
    #[test]
    fn the_two_boundaries_point_opposite_ways() {
        assert!(open_at(T, "", T), "created exactly at the instant existed");
        assert!(!open_at(T, T, T), "closed exactly at the instant is closed");
    }

    /// A row this cannot classify does not get a default. Guessing "open" would put it
    /// in the population in silence, which is the failure this whole command is about.
    #[test]
    fn a_row_with_no_creation_time_is_not_counted() {
        assert!(!open_at("", "", T));
        assert!(!open_at("", "2026-09-01T10:00:00Z", T));
    }

    const TODAY: &str = "2026-09-04";

    #[test]
    fn a_date_becomes_the_last_instant_of_that_utc_day() {
        assert_eq!(
            instant_of("2026-08-01", TODAY).unwrap(),
            "2026-08-01T23:59:59Z"
        );
        assert_eq!(
            instant_of("2026-09-03", TODAY).unwrap(),
            "2026-09-03T23:59:59Z",
            "yesterday has ended, so it can be read"
        );
    }

    /// A day that has not ENDED cannot be read as a completed day. GitHub answers
    /// `created:<=2027-01-01` with today's set and no complaint, so the first version
    /// of this command printed today's 486 under the heading `AS OF 2027-01-01` -- a
    /// clock reading wearing next year's label. Today is refused for the same reason
    /// as tomorrow: its end is still in the future.
    #[test]
    fn a_day_that_has_not_ended_is_refused() {
        assert!(instant_of("2027-01-01", TODAY).is_err(), "next year");
        assert!(instant_of("2026-09-05", TODAY).is_err(), "tomorrow");
        assert!(
            instant_of(TODAY, TODAY).is_err(),
            "today has not ended either -- its count differs from itself by evening"
        );
    }

    /// The transcribed calendar, tested on the dates that break a wrong one: the
    /// epoch, a leap day, a century that is NOT a leap year, and one that is.
    #[test]
    fn the_civil_calendar_lands_on_the_dates_that_break_it() {
        use super::civil_from_days;
        assert_eq!(civil_from_days(0), (1970, 1, 1), "the epoch");
        assert_eq!(civil_from_days(-1), (1969, 12, 31), "before the epoch");
        assert_eq!(civil_from_days(59), (1970, 3, 1), "1970 is not a leap year");
        assert_eq!(
            civil_from_days(11_016),
            (2000, 2, 29),
            "2000 IS a leap year"
        );
        assert_eq!(civil_from_days(-25_567), (1900, 1, 1), "1900 is NOT");
        // The century divisor is load-bearing on exactly TWO days in a hundred
        // thousand -- 1900-03-01 and 2100-03-01, the day after a non-leap century's
        // February. Swap 36_524 for 36_525 and the calendar invents 1900-02-29, a
        // date that does not exist. Found by sweeping the mutant against an
        // independent calendar rather than left unproved.
        assert_eq!(
            civil_from_days(-25_508),
            (1900, 3, 1),
            "no 1900-02-29 exists"
        );
        assert_eq!(civil_from_days(47_541), (2100, 3, 1), "nor 2100-02-29");
        // Cross-checked against an independent calendar (python `datetime`), which
        // is how the sixth line of this test got fixed: the code was right and the
        // expectation was mine.
        assert_eq!(civil_from_days(20_699), (2026, 9, 3));
    }

    /// Refused, not defaulted. A date this cannot read must not become "today" under
    /// a heading that says the reading is anchored.
    #[test]
    fn a_date_it_cannot_read_is_refused() {
        for bad in [
            "2026-8-1",
            "01-08-2026",
            "2026-08-01T00:00:00Z",
            "",
            "yesterday",
        ] {
            assert!(instant_of(bad, TODAY).is_err(), "{bad} must be refused");
        }
    }

    /// The boundary IS the rule. Measured 2026-09-03: 486 open against a default limit
    /// of 500, so this is fourteen issues from mattering.
    #[test]
    fn a_full_page_is_a_lower_bound_and_a_short_one_is_a_total() {
        assert!(read_is_complete(486, 500), "short page: complete");
        assert!(
            !read_is_complete(500, 500),
            "full page: cannot tell, so not complete"
        );
        assert!(
            !read_is_complete(501, 500),
            "over the limit is not complete either"
        );
        assert!(
            read_is_complete(0, 1),
            "an empty read of a non-zero limit is complete"
        );
    }
}
