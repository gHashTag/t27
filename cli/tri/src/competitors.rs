//! The competitor table, read against its own contract.
//!
//! `specs/igla/coder/benchmark.t27` holds one `CompetitorScore` record per
//! external system, and the struct's own doc line says what the fields are:
//! "published Pass@K scores from external research". A record is therefore a
//! citation -- it states what somebody else measured and published.
//!
//! Three ways that contract breaks, and this command counts each of them:
//!
//! 1. **A published score of zero that nobody published.** The struct has no
//!    way to say "this competitor cites no Pass@K", so a record with no score
//!    to cite states `0.0`. That value is legal, in range, and indistinguishable
//!    from a measured zero. `compare_with_competitor` subtracts it, and returns
//!    our own score as the margin -- a lead computed against an absent number.
//!
//! 2. **One paper entered as two competitors.** A later pass adds a system that
//!    is already in the table under a different function name. Any count of
//!    "systems surveyed" then double-counts it.
//!
//! 3. **One name defined twice.** `tri types redef` already reports this class
//!    across the whole tree; it is repeated here only as the intersection with
//!    this table, so the three readings come off one walk of one file.
//!
//! The attribution rule for (2) is stated in the output on purpose. The first
//! version of this measurement took "the last arXiv id within 400 characters
//! before the function" and reported 16 double-entered papers. Six were real;
//! ten were the *neighbouring* record's citation, read across the boundary. A
//! rule that a reader cannot see is a rule a reader cannot check, so the rule
//! is printed beside the number, and `a_neighbours_citation_is_not_this_ones`
//! holds it.

use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The table this command reads. One file, named here rather than globbed: a
/// glob would silently start reporting on a second table someone adds, and the
/// ceilings below are calibrated against this one.
const TABLE: &str = "specs/igla/coder/benchmark.t27";

/// Where the four counts are pinned, so they can only go down.
const RATCHET: &str = "docs/reports/competitor_table.json";

#[derive(Debug, clap::Subcommand)]
pub enum CompetitorsCmd {
    /// Count the records, the double-entered papers and the unpublished zeros.
    Audit {
        /// Fail when any count rises above the pinned ceiling.
        #[arg(long)]
        gate: bool,
        /// Write today's counts as the new ceilings. Only ever lowers them.
        #[arg(long)]
        bless: bool,
    },
    /// When the survey behind this table last read a paper, by the month its
    /// citations encode.
    Freshness {
        /// Read the gap against this month instead of today's, as `YYYY-MM`.
        /// The system clock makes an output that cannot be pinned in a test;
        /// every reading in this command's tests supplies its own month.
        #[arg(long)]
        as_of: Option<String>,
    },
}

/// One `CompetitorScore` record as it stands in the file.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// The `pub fn` this record is the body of.
    pub func: String,
    /// 1-based line of the `pub fn`.
    pub line: usize,
    /// The `name:` field -- what a table built from this file would print.
    pub name: String,
    /// `pass_at_1`, `pass_at_5`, `pass_at_10`, in that order. `None` when the
    /// field is absent, which is a different thing from a stated zero.
    pub scores: [Option<f32>; 3],
    /// The arXiv id in the doc comment immediately above, if there is one.
    pub arxiv: Option<String>,
}

impl Record {
    /// True when every score this record states is zero.
    ///
    /// Not "some score is zero": a system can genuinely score 0 at pass@10
    /// while scoring at pass@1. It is the all-zero row that carries no citation
    /// at all, and that is the row `compare_with_competitor` turns into a
    /// full-width lead.
    pub fn cites_nothing(&self) -> bool {
        self.scores.iter().flatten().count() > 0 && self.scores.iter().flatten().all(|v| *v == 0.0)
    }
}

/// Read every `CompetitorScore` record out of the table's text.
///
/// The doc block is the *contiguous* run of `///` lines directly above the
/// `pub fn`. A blank line, a `}`, or anything else ends it. That is the whole
/// attribution rule, and it is the fix for the 400-character window that read
/// the previous record's citation as this one's.
pub fn records(src: &str) -> Vec<Record> {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for (i, ln) in lines.iter().enumerate() {
        let func = match ln
            .strip_prefix("pub fn ")
            .and_then(|r| r.split_once("()"))
            .filter(|(_, rest)| rest.contains("-> CompetitorScore"))
        {
            Some((f, _)) => f.to_string(),
            None => continue,
        };
        let mut doc = String::new();
        let mut j = i;
        while j > 0 && lines[j - 1].trim_start().starts_with("///") {
            j -= 1;
            doc.insert_str(0, lines[j]);
            doc.insert(0, '\n');
        }
        let body: String = lines[i..lines.len().min(i + 12)].join("\n");
        let body = match body.find("\n}") {
            Some(e) => body[..e].to_string(),
            None => body,
        };
        out.push(Record {
            func,
            line: i + 1,
            name: field(&body, "name").unwrap_or_default(),
            scores: [
                num(&body, "pass_at_1"),
                num(&body, "pass_at_5"),
                num(&body, "pass_at_10"),
            ],
            arxiv: arxiv_of(&doc),
        });
    }
    out
}

/// The first `arXiv:NNNN.NNNNN` in a doc block, without the prefix.
pub fn arxiv_of(doc: &str) -> Option<String> {
    let at = doc.find("arXiv:")? + 6;
    let rest = &doc[at..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    let id = &rest[..end];
    let (a, b) = id.split_once('.')?;
    if a.len() == 4 && (4..=5).contains(&b.len()) && b.chars().all(|c| c.is_ascii_digit()) {
        Some(id.to_string())
    } else {
        None
    }
}

fn field(body: &str, key: &str) -> Option<String> {
    let at = body.find(&format!("{key}:"))? + key.len() + 1;
    let rest = body[at..].trim_start();
    let q = rest.strip_prefix('"')?;
    Some(q[..q.find('"')?].to_string())
}

fn num(body: &str, key: &str) -> Option<f32> {
    let at = body.find(&format!("{key}:"))? + key.len() + 1;
    let rest = body[at..].trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '.')?;
    rest[..end].parse().ok()
}

/// The four counts, each read off the same walk.
#[derive(Debug, Default, PartialEq)]
pub struct Counts {
    /// Records in the file.
    pub records: usize,
    /// Distinct `pub fn` names among them.
    pub names: usize,
    /// arXiv ids carried by more than one function.
    pub papers_twice: usize,
    /// Records whose every stated score is zero.
    pub cites_nothing: usize,
    /// Records stating zero at pass@1 -- the metric `compare_with_competitor`
    /// subtracts when it is not told which one. Larger than `cites_nothing` by
    /// the records that cite pass@10 only, and it is this number, not that one,
    /// that governs the default comparison.
    pub zero_at_1: usize,
}

pub fn counts(recs: &[Record]) -> Counts {
    let mut by_paper: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for r in recs {
        if let Some(a) = &r.arxiv {
            by_paper.entry(a).or_default().insert(&r.func);
        }
    }
    Counts {
        records: recs.len(),
        names: recs.iter().map(|r| &r.func).collect::<BTreeSet<_>>().len(),
        papers_twice: by_paper.values().filter(|s| s.len() > 1).count(),
        cites_nothing: recs.iter().filter(|r| r.cites_nothing()).count(),
        zero_at_1: recs.iter().filter(|r| r.scores[0] == Some(0.0)).count(),
    }
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8(out.stdout)?.trim()).to_path_buf())
}

/// Read the pinned ceilings. A missing file is not a pass: the gate says so
/// and fails, because "no baseline" and "no drift" print the same green
/// otherwise.
fn ceilings(root: &Path) -> Result<Counts> {
    let p = root.join(RATCHET);
    let txt = std::fs::read_to_string(&p).map_err(|e| {
        anyhow::anyhow!(
            "{}: {e} -- run `tri competitors audit --bless`",
            p.display()
        )
    })?;
    let g = |k: &str| -> Result<usize> {
        let at = txt
            .find(&format!("\"{k}\""))
            .ok_or_else(|| anyhow::anyhow!("{}: no \"{k}\"", p.display()))?;
        let rest = &txt[at + k.len() + 2..];
        let s = rest
            .find(|c: char| c.is_ascii_digit())
            .ok_or_else(|| anyhow::anyhow!("{}: \"{k}\" has no number", p.display()))?;
        let e = rest[s..]
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len() - s);
        Ok(rest[s..s + e].parse()?)
    };
    Ok(Counts {
        records: g("records")?,
        names: g("names")?,
        papers_twice: g("papers_twice")?,
        cites_nothing: g("cites_nothing")?,
        zero_at_1: g("zero_at_1")?,
    })
}

/// An arXiv identifier dates its paper: since 2007 the form is `YYMM.NNNNN`,
/// and `YYMM` is the month of FIRST submission. A later version does not change
/// it, so the id dates the paper and not the reading -- which is exactly the
/// property wanted here.
///
/// Returns `(year, month)`.
pub fn month_of(id: &str) -> Option<(u32, u32)> {
    let (a, b) = id.split_once('.')?;
    if a.len() != 4 || !(4..=5).contains(&b.len()) {
        return None;
    }
    let yy: u32 = a[..2].parse().ok()?;
    let mm: u32 = a[2..].parse().ok()?;
    if !(1..=12).contains(&mm) {
        return None;
    }
    Some((2000 + yy, mm))
}

/// Months from `(y0, m0)` to `(y1, m1)`, negative if the second is earlier.
pub fn months_between(from: (u32, u32), to: (u32, u32)) -> i64 {
    (to.0 as i64 - from.0 as i64) * 12 + (to.1 as i64 - from.1 as i64)
}

/// Parse a `YYYY-MM`.
fn as_of_month(s: &str) -> Result<(u32, u32)> {
    let (y, m) = s
        .split_once('-')
        .ok_or_else(|| anyhow::anyhow!("--as-of wants YYYY-MM, got {s:?}"))?;
    let y: u32 = y
        .parse()
        .map_err(|_| anyhow::anyhow!("--as-of year is not a number: {s:?}"))?;
    let m: u32 = m
        .parse()
        .map_err(|_| anyhow::anyhow!("--as-of month is not a number: {s:?}"))?;
    if !(1..=12).contains(&m) {
        anyhow::bail!("--as-of month {m} is not a month");
    }
    Ok((y, m))
}

/// Every arXiv month cited by the table, counted twice over two populations.
///
/// `cited` is the months of papers attached to a competitor RECORD -- the
/// survey proper. `anywhere` is every arXiv id in the file, including the ones
/// written into a `benchmark:` string or a prose comment. They answer different
/// questions and a single number would hide which one it is: the first says
/// when the survey last added a competitor, the second when the file last
/// mentioned a paper at all.
/// A year and a month, as an arXiv identifier encodes them.
pub type YearMonth = (u32, u32);

/// `(cited, anywhere)` -- the two populations `months` returns, named so the
/// signature says which is which. They must not be summed: the first is the
/// survey, the second is every mention in the file.
pub type Months = (Vec<YearMonth>, Vec<YearMonth>);

pub fn months(src: &str, recs: &[Record]) -> Months {
    let cited: Vec<(u32, u32)> = recs
        .iter()
        .filter_map(|r| r.arxiv.as_deref())
        .filter_map(month_of)
        .collect();
    let mut anywhere = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = src[from..].find("arXiv:") {
        let at = from + rel + 6;
        let rest = &src[at..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(rest.len());
        if let Some(m) = month_of(&rest[..end]) {
            anywhere.push(m);
        }
        from = at;
    }
    (cited, anywhere)
}

/// How old the competitive survey is, and what would settle the question.
///
/// A gap is NOT a defect on its own: a table with nothing from the last two
/// months either stopped looking, or the field published nothing. Those look
/// identical from inside the repository, so this command reports the gap and
/// says what distinguishes them, rather than failing. It has no `--gate` on
/// purpose -- a gate that reddens by the calendar, with nobody having changed
/// anything, is a gate that gets muted.
fn freshness(root: &Path, as_of: Option<&str>) -> Result<()> {
    let path = root.join(TABLE);
    let src =
        std::fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
    let recs = records(&src);
    let (cited, anywhere) = months(&src, &recs);
    if cited.is_empty() {
        anyhow::bail!(
            "{TABLE}: no record carries an arXiv id in the doc block above it. \
             Either the citation style changed or the reader stopped matching; \
             a zero here would otherwise read as a table with no sources."
        );
    }

    let now = match as_of {
        Some(s) => as_of_month(s)?,
        None => {
            let d = chrono::Local::now().date_naive();
            (
                chrono::Datelike::year(&d) as u32,
                chrono::Datelike::month(&d),
            )
        }
    };

    let newest_cited = *cited.iter().max().unwrap();
    let newest_any = *anywhere.iter().max().unwrap();

    println!("COMPETITOR SURVEY, BY THE MONTH ITS CITATIONS ENCODE -- {TABLE}\n");
    println!(
        "  reading taken as of         {:04}-{:02}{}",
        now.0,
        now.1,
        if as_of.is_some() { "   (--as-of)" } else { "" }
    );
    println!(
        "  newest paper CITED BY A RECORD   {:04}-{:02}   {} month(s) ago",
        newest_cited.0,
        newest_cited.1,
        months_between(newest_cited, now)
    );
    println!(
        "  newest paper mentioned anywhere  {:04}-{:02}   {} month(s) ago",
        newest_any.0,
        newest_any.1,
        months_between(newest_any, now)
    );

    // the last twelve months, so a cliff is visible rather than inferred
    println!("\n  records added per month of publication, last 12:");
    let mut any = false;
    for k in (0..12).rev() {
        let mm = ((now.1 as i64 - 1 - k) % 12 + 12) % 12 + 1;
        let yy = now.0 as i64 + (now.1 as i64 - 1 - k).div_euclid(12);
        let key = (yy as u32, mm as u32);
        let n = cited.iter().filter(|m| **m == key).count();
        if n > 0 {
            any = true;
        }
        if any || n > 0 {
            let bar: String = "#".repeat(n.min(40));
            println!("    {:04}-{:02}  {:4}  {}", key.0, key.1, n, bar);
        }
    }

    println!(
        "\n  A gap is not a defect by itself. A table with nothing recent either\n  \
         stopped looking or the field published nothing, and those are\n  \
         indistinguishable from inside this repository. What settles it is ONE\n  \
         counterexample: a paper in the gap that belongs in this table. Find one\n  \
         and the survey stopped; fail to find one after looking and the field did.\n\n  \
         There is no `--gate` here on purpose. A gate that reddens because a month\n  \
         passed, with nobody having changed anything, is a gate that gets muted."
    );
    Ok(())
}

pub fn run(cmd: &CompetitorsCmd) -> Result<()> {
    let root = repo_root()?;
    let (gate, bless) = match cmd {
        CompetitorsCmd::Freshness { as_of } => return freshness(&root, as_of.as_deref()),
        CompetitorsCmd::Audit { gate, bless } => (gate, bless),
    };
    let path = root.join(TABLE);
    let src = std::fs::read_to_string(&path).map_err(|e| {
        anyhow::anyhow!(
            "{}: {e} -- the table this command reads is gone",
            path.display()
        )
    })?;
    let recs = records(&src);
    if recs.is_empty() {
        anyhow::bail!(
            "{}: no `CompetitorScore` record found. Either the table moved or the \
             record shape changed; a zero here would otherwise read as a clean table.",
            TABLE
        );
    }
    let c = counts(&recs);

    println!("COMPETITOR TABLE -- {TABLE}\n");
    println!("  records                     {}", c.records);
    println!(
        "  distinct function names     {}   ({} redefinition(s))",
        c.names,
        c.records - c.names
    );
    println!("  papers entered twice        {}", c.papers_twice);
    println!(
        "  records citing no score     {}   of {}",
        c.cites_nothing, c.records
    );
    println!(
        "  stating zero at pass@1      {}   ({} of them cite pass@10 only)",
        c.zero_at_1,
        c.zero_at_1 - c.cites_nothing
    );

    let mut by_paper: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for r in &recs {
        if let Some(a) = &r.arxiv {
            by_paper.entry(a).or_default().insert(&r.func);
        }
    }
    let twice: Vec<_> = by_paper.iter().filter(|(_, s)| s.len() > 1).collect();
    if !twice.is_empty() {
        println!("\n  one paper, two competitors:");
        for (a, fns) in twice {
            let names: BTreeSet<&str> = recs
                .iter()
                .filter(|r| r.arxiv.as_deref() == Some(*a))
                .map(|r| r.name.as_str())
                .collect();
            println!(
                "    arXiv:{a}  {}\n{:>18}prints as {}",
                fns.iter().copied().collect::<Vec<_>>().join(" + "),
                "",
                names.iter().copied().collect::<Vec<_>>().join(" / ")
            );
        }
    }

    println!(
        "\n  A paper is attributed to a record only from the contiguous run of `///`\n  \
         lines directly above its `pub fn`. A wider window reads the previous\n  \
         record's citation as this one's: that window reported 16 papers here,\n  \
         and ten of them were the neighbour's."
    );
    println!(
        "\n  `pass_at_1: 0.0` is the value a record takes when there is no published\n  \
         score to cite -- the struct has no other way to say it. It is in range and\n  \
         it is what `compare_with_competitor` subtracts, so every one of those {}\n  \
         records yields our own score as the margin over it at pass@1. Read the\n  \
         two counts apart: {} records cite nothing at any metric, and {} more cite\n  \
         pass@10 alone -- true citations that still read as zero to the default\n  \
         comparison.",
        c.zero_at_1,
        c.cites_nothing,
        c.zero_at_1 - c.cites_nothing
    );

    if *bless {
        let txt = format!(
            "{{\n  \"_what\": \"Ceilings for `tri competitors audit`. Down only: a rise means \
             a record was added without a citation, or a paper was entered twice.\",\n  \
             \"_table\": \"{TABLE}\",\n  \"records\": {},\n  \"names\": {},\n  \
             \"papers_twice\": {},\n  \"cites_nothing\": {},\n  \"zero_at_1\": {}\n}}\n",
            c.records, c.names, c.papers_twice, c.cites_nothing, c.zero_at_1
        );
        std::fs::write(root.join(RATCHET), txt)?;
        println!("\n  blessed -> {RATCHET}");
        return Ok(());
    }

    if *gate {
        let ceil = ceilings(&root)?;
        let mut over = Vec::new();
        if c.papers_twice > ceil.papers_twice {
            over.push(format!(
                "papers entered twice {} > {}",
                c.papers_twice, ceil.papers_twice
            ));
        }
        if c.cites_nothing > ceil.cites_nothing {
            over.push(format!(
                "records citing no score {} > {}",
                c.cites_nothing, ceil.cites_nothing
            ));
        }
        if c.zero_at_1 > ceil.zero_at_1 {
            over.push(format!(
                "stating zero at pass@1 {} > {}",
                c.zero_at_1, ceil.zero_at_1
            ));
        }
        if c.records - c.names > ceil.records - ceil.names {
            over.push(format!(
                "redefinitions {} > {}",
                c.records - c.names,
                ceil.records - ceil.names
            ));
        }
        if !over.is_empty() {
            println!("\n  OVER CEILING:");
            for o in &over {
                println!("    {o}");
            }
            anyhow::bail!("{} count(s) rose above the pinned ceiling", over.len());
        }
        println!("\n  at or under every ceiling");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO: &str = r#"
/// alpha_competitor() -> CompetitorScore
/// Alpha (arXiv:2606.00001): a system with a published score.
pub fn alpha_competitor() -> CompetitorScore {
    return CompetitorScore {
        name: "Alpha",
        pass_at_1: 0.5,
        pass_at_5: 0.6,
        pass_at_10: 0.0,
        benchmark: "VerilogEval",
    };
}

/// beta_competitor() -> CompetitorScore
/// Beta: a hardware paper with no Pass@K to cite.
pub fn beta_competitor() -> CompetitorScore {
    return CompetitorScore {
        name: "Beta",
        pass_at_1: 0.0,
        pass_at_5: 0.0,
        pass_at_10: 0.0,
        benchmark: "an accelerator, not a generator",
    };
}
"#;

    #[test]
    fn a_record_is_read_off_its_own_body() {
        let r = records(TWO);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].func, "alpha_competitor");
        assert_eq!(r[0].name, "Alpha");
        assert_eq!(r[0].scores, [Some(0.5), Some(0.6), Some(0.0)]);
        assert_eq!(r[0].arxiv.as_deref(), Some("2606.00001"));
    }

    /// The bug this whole command was rewritten around. Beta's doc block names
    /// no paper; Alpha's, two lines earlier, does. A window-based reader hands
    /// Alpha's citation to Beta and reports one paper as two competitors.
    #[test]
    fn a_neighbours_citation_is_not_this_ones() {
        let r = records(TWO);
        assert_eq!(r[1].func, "beta_competitor");
        assert_eq!(
            r[1].arxiv, None,
            "Beta cites no paper; the id above it belongs to Alpha"
        );
        assert_eq!(counts(&r).papers_twice, 0);
    }

    /// A row that scores zero at pass@10 and nonzero at pass@1 has a citation.
    /// Only the all-zero row has none.
    #[test]
    fn a_zero_at_one_metric_is_not_an_absent_citation() {
        let r = records(TWO);
        assert!(!r[0].cites_nothing(), "Alpha scores 0.5 at pass@1");
        assert!(r[1].cites_nothing(), "Beta states zero at every metric");
        assert_eq!(counts(&r).cites_nothing, 1);
        assert_eq!(counts(&r).zero_at_1, 1);
    }

    /// The two counts are different questions, and the live table answers them
    /// differently: 141 records cite nothing at all, 144 state zero at pass@1.
    /// The three between them cite pass@10 only. Reading either number as the
    /// other is how the same population gets counted twice.
    #[test]
    fn citing_only_pass_at_ten_is_not_citing_nothing() {
        let src = TWO.replace(
            "        pass_at_1: 0.0,\n        pass_at_5: 0.0,\n        pass_at_10: 0.0,\n        benchmark: \"an accelerator",
            "        pass_at_1: 0.0,\n        pass_at_5: 0.0,\n        pass_at_10: 0.9,\n        benchmark: \"an accelerator",
        );
        let c = counts(&records(&src));
        assert_eq!(c.cites_nothing, 0, "Beta cites pass@10");
        assert_eq!(
            c.zero_at_1, 1,
            "and still reads as zero to the default metric"
        );
    }

    /// Two functions under one id is the double entry; the same function seen
    /// once is not.
    #[test]
    fn one_paper_under_two_functions_counts_once_as_a_double() {
        let src = TWO.replace(
            "/// Beta: a hardware paper with no Pass@K to cite.",
            "/// Beta (arXiv:2606.00001): the same paper, entered again.",
        );
        let c = counts(&records(&src));
        assert_eq!(c.papers_twice, 1);
        assert_eq!(c.records, 2);
        assert_eq!(c.names, 2, "two distinct functions, one paper");
    }

    /// The rule is contiguity, so a blank line between the citation and the
    /// function severs it. Without this, "contiguous" is a word in a comment
    /// rather than a property of the reader.
    #[test]
    fn a_blank_line_ends_the_doc_block() {
        let src = TWO.replace(
            "/// Alpha (arXiv:2606.00001): a system with a published score.\npub fn",
            "/// Alpha (arXiv:2606.00001): a system with a published score.\n\npub fn",
        );
        let r = records(&src);
        assert_eq!(r[0].func, "alpha_competitor");
        assert_eq!(r[0].arxiv, None, "a blank line ends the block");
    }

    /// An arXiv id dates its paper, and the two halves have to be read as a
    /// date rather than as a number: `2601` is January 2026, not the 2601st of
    /// anything, and `2613` is not a month at all.
    #[test]
    fn an_arxiv_id_carries_the_month_of_first_submission() {
        assert_eq!(month_of("2606.15500"), Some((2026, 6)));
        assert_eq!(month_of("2607.13079"), Some((2026, 7)));
        assert_eq!(month_of("2501.00001"), Some((2025, 1)));
        assert_eq!(month_of("0704.0001"), Some((2007, 4)));
        assert_eq!(month_of("2613.00001"), None, "13 is not a month");
        assert_eq!(month_of("2600.00001"), None, "0 is not a month");
        assert_eq!(month_of("260.15500"), None, "three digits is not YYMM");
        assert_eq!(month_of("2606"), None, "no point, no id");
    }

    /// A gap in months has to cross a year boundary correctly, because the
    /// interesting readings all sit near December.
    #[test]
    fn the_gap_counts_months_across_a_year() {
        assert_eq!(months_between((2026, 6), (2026, 8)), 2);
        assert_eq!(months_between((2025, 11), (2026, 2)), 3);
        assert_eq!(months_between((2026, 8), (2026, 8)), 0);
        assert_eq!(
            months_between((2026, 8), (2026, 6)),
            -2,
            "a future citation"
        );
    }

    /// The two populations answer different questions and must not be summed.
    /// A paper named only inside a `benchmark:` string is mentioned by the file
    /// and cited by no record.
    #[test]
    fn a_paper_named_in_a_string_is_mentioned_and_not_cited() {
        let src = TWO.replace(
            r#"        benchmark: "an accelerator, not a generator","#,
            r#"        benchmark: "an accelerator (arXiv:2607.00001), not a generator","#,
        );
        let recs = records(&src);
        let (cited, anywhere) = months(&src, &recs);
        assert_eq!(
            cited,
            vec![(2026, 6)],
            "only Alpha's doc block cites a paper"
        );
        assert!(
            anywhere.contains(&(2026, 7)),
            "the id inside the string is mentioned: {anywhere:?}"
        );
        assert!(
            !cited.contains(&(2026, 7)),
            "and it is NOT a citation: no record's doc block names it"
        );
        assert_eq!(
            anywhere.len(),
            cited.len() + 1,
            "mentioned {anywhere:?} is cited {cited:?} plus the string"
        );
    }

    /// The reading is pinned by `--as-of`, which is why the command takes one:
    /// a gap measured against the system clock is a number that changes while
    /// nobody edits anything, and cannot be asserted in a test at all.
    #[test]
    fn the_gap_is_reproducible_when_the_month_is_supplied() {
        let recs = records(TWO);
        let (cited, _) = months(TWO, &recs);
        let newest = *cited.iter().max().expect("Alpha cites 2026-06");
        assert_eq!(newest, (2026, 6));
        assert_eq!(months_between(newest, (2026, 8)), 2);
        assert_eq!(months_between(newest, (2027, 6)), 12);
    }

    /// The live table, against the same numbers the ratchet pins. If this
    /// drifts, one of the two is wrong and the gate should be the one to say so.
    #[test]
    fn the_live_table_still_parses_into_records() {
        let root = match repo_root() {
            Ok(r) => r,
            Err(_) => return,
        };
        let src = match std::fs::read_to_string(root.join(TABLE)) {
            Ok(s) => s,
            Err(_) => return,
        };
        let c = counts(&records(&src));
        assert!(
            c.records > 100,
            "the table holds {} records -- the reader stopped seeing them",
            c.records
        );
        assert!(c.names <= c.records);
        assert!(c.cites_nothing <= c.records);
    }
}
