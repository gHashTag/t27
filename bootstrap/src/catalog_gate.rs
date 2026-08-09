//! W602: verify `specs/numeric/formats_catalog.t27`, whose payload is invisible
//! to the compiler.
//!
//! ## Why a command and not invariants
//!
//! W601 gave the nine GF constant specs invariants, because their data lives in
//! `const` declarations the compiler can see. The catalog cannot be done that
//! way. It declares itself *"Single source of truth for every numeric format"*
//! and feeds six codegen targets, but **all 83 of its functions are
//! `fn binary16() -> str { return "binary16"; }`** — the entire payload is in
//! structured comments:
//!
//! ```text
//! // CATALOG: id=gf10 name="GF10 (rule-derived)" bits=10 s=1 e=3 m=6 bias=3
//! //          phi_distance=0.118 storage=u16 cluster=GoldenFloat status=Open
//! //          source="specs/numeric/gf10.t27"
//! ```
//!
//! The file's own header explains why: struct literals were not parseable when
//! it was written, so "per-format records live as fn getters that the codegen
//! reads from the AST". The consequence is that **nothing the compiler does can
//! check any of it**, and 83 records have never been checked by anything.
//!
//! ## The classification this gate exists to encode
//!
//! The obvious check — `s + e + m == bits` — reports **13 violations**, and
//! twelve of them are not violations at all:
//!
//! | Class | n | Why the rule does not apply |
//! |---|---|---|
//! | **Tapered** (`posit*`, `takum*`) | 8 | variable-length regime; `e` is the *es* parameter and mantissa width varies per value |
//! | **Parametric** (`bits=0`) | 4 | `q_format`, `minifloat`, `unum_i`, `tapered_fp` are families, not formats |
//! | **Alphabet** (`bits < 4`) | 1 | `gfternary` is the 3-value set {−φ, 0, +φ}; it has no s/e/m decomposition |
//!
//! A gate that did not know this would emit thirteen false alarms and be turned
//! off within a wave. Encoding the exceptions IS the work.

use std::collections::BTreeMap;
use std::path::Path;

const PHI: f64 = 1.618_033_988_749_895;
/// φ² = φ + 1 (L5).
const PHI2: f64 = PHI + 1.0;
/// Tolerance for `phi_distance`, which the catalog records to 3-4 decimals.
const PHI_DIST_TOL: f64 = 0.0015;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Shape {
    /// s + e + m == bits is meaningful.
    FixedLayout,
    /// posit / takum: the regime field is variable-length.
    Tapered,
    /// `bits=0` -- a family, not a concrete format.
    Parametric,
    /// Fewer than 4 bits: an alphabet, not a field layout.
    Alphabet,
}

pub struct Record {
    pub id: String,
    pub fields: BTreeMap<String, String>,
    pub shape: Shape,
}

impl Record {
    fn num(&self, k: &str) -> Option<i64> {
        self.fields.get(k)?.parse().ok()
    }
    fn f64(&self, k: &str) -> Option<f64> {
        self.fields.get(k)?.parse().ok()
    }
}

pub struct Finding {
    pub id: String,
    pub check: &'static str,
    pub detail: String,
}

pub struct Report {
    pub records: usize,
    pub getters: usize,
    pub by_shape: BTreeMap<&'static str, usize>,
    pub by_cluster: BTreeMap<String, usize>,
    pub checked: BTreeMap<&'static str, usize>,
    pub findings: Vec<Finding>,
}

fn parse_records(src: &str) -> Vec<Record> {
    let mut out = Vec::new();
    for line in src.lines() {
        let body = match line.find("CATALOG:") {
            Some(i) => &line[i + "CATALOG:".len()..],
            None => continue,
        };
        let mut fields = BTreeMap::new();
        // k=v, where v is either a quoted string or a bare token.
        let bytes: Vec<char> = body.chars().collect();
        let mut i = 0;
        while i < bytes.len() {
            // key
            if !bytes[i].is_alphanumeric() && bytes[i] != '_' {
                i += 1;
                continue;
            }
            let ks = i;
            while i < bytes.len() && (bytes[i].is_alphanumeric() || bytes[i] == '_') {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != '=' {
                continue;
            }
            let key: String = bytes[ks..i].iter().collect();
            i += 1; // '='
            let val: String = if i < bytes.len() && bytes[i] == '"' {
                i += 1;
                let vs = i;
                while i < bytes.len() && bytes[i] != '"' {
                    i += 1;
                }
                let v: String = bytes[vs..i].iter().collect();
                i += 1;
                v
            } else {
                let vs = i;
                while i < bytes.len() && !bytes[i].is_whitespace() {
                    i += 1;
                }
                bytes[vs..i].iter().collect()
            };
            fields.insert(key, val);
        }
        let id = fields.get("id").cloned().unwrap_or_default();
        if id.is_empty() {
            continue;
        }
        let bits: i64 = fields.get("bits").and_then(|b| b.parse().ok()).unwrap_or(-1);
        let cluster = fields.get("cluster").cloned().unwrap_or_default();
        let shape = if bits == 0 {
            Shape::Parametric
        } else if cluster == "PositUnumIII" {
            Shape::Tapered
        } else if bits > 0 && bits < 4 {
            Shape::Alphabet
        } else {
            Shape::FixedLayout
        };
        out.push(Record { id, fields, shape });
    }
    out
}

fn count_getters(src: &str) -> usize {
    src.lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("fn ") && t.contains("-> str") && t.contains("return")
        })
        .count()
}

pub fn run(catalog: &Path, specs_root: &Path) -> std::io::Result<Report> {
    let src = std::fs::read_to_string(catalog)?;
    let records = parse_records(&src);
    let getters = count_getters(&src);

    let mut r = Report {
        records: records.len(),
        getters,
        by_shape: BTreeMap::new(),
        by_cluster: BTreeMap::new(),
        checked: BTreeMap::new(),
        findings: Vec::new(),
    };

    if records.len() != getters {
        r.findings.push(Finding {
            id: "(file)".into(),
            check: "getter-parity",
            detail: format!(
                "{} CATALOG records but {} getters -- codegen reads the AST, so a \
                 record without a getter is invisible to it",
                records.len(),
                getters
            ),
        });
    }

    for rec in &records {
        let name = match rec.shape {
            Shape::FixedLayout => "FixedLayout",
            Shape::Tapered => "Tapered",
            Shape::Parametric => "Parametric",
            Shape::Alphabet => "Alphabet",
        };
        *r.by_shape.entry(name).or_insert(0) += 1;
        *r.by_cluster
            .entry(rec.fields.get("cluster").cloned().unwrap_or_default())
            .or_insert(0) += 1;

        // -- mandatory fields, every shape ---------------------------------
        for k in ["id", "name", "bits", "cluster", "status"] {
            if !rec.fields.contains_key(k) {
                r.findings.push(Finding {
                    id: rec.id.clone(),
                    check: "mandatory-field",
                    detail: format!("no `{}=`", k),
                });
            }
        }
        *r.checked.entry("mandatory-field").or_insert(0) += 1;

        // -- shapes without an s/e/m layout must not CLAIM one ---------------
        //
        // The first version of this gate simply skipped them, which turned a
        // false alarm into a silent exemption -- strictly worse, because the
        // data is still there and still wrong. If the decomposition does not
        // apply, the fields should be absent or zero; a record that exempts
        // itself from the rule and then states values under it is asserting
        // something no reader can act on.
        if matches!(rec.shape, Shape::Alphabet | Shape::Parametric) {
            *r.checked.entry("no-spurious-layout").or_insert(0) += 1;
            let s = rec.num("s").unwrap_or(0);
            let e = rec.num("e").unwrap_or(0);
            let m = rec.num("m").unwrap_or(0);
            let b = rec.num("bits").unwrap_or(0);
            if s + e + m != 0 && s + e + m != b {
                r.findings.push(Finding {
                    id: rec.id.clone(),
                    check: "no-spurious-layout",
                    detail: format!(
                        "shape is {:?} -- s/e/m do not apply -- yet the record states \
                         s={} e={} m={} (sum {}) against bits={}",
                        rec.shape,
                        s,
                        e,
                        m,
                        s + e + m,
                        b
                    ),
                });
            }
        }

        if rec.shape != Shape::FixedLayout {
            continue;
        }

        // -- s + e + m == bits ---------------------------------------------
        if let (Some(b), Some(s), Some(e), Some(m)) =
            (rec.num("bits"), rec.num("s"), rec.num("e"), rec.num("m"))
        {
            *r.checked.entry("widths-partition").or_insert(0) += 1;
            if s + e + m != b {
                r.findings.push(Finding {
                    id: rec.id.clone(),
                    check: "widths-partition",
                    detail: format!("s+e+m = {}+{}+{} = {}, but bits = {}", s, e, m, s + e + m, b),
                });
            }

            // -- the GoldenFloat generating rule ---------------------------
            if rec.fields.get("cluster").map(|c| c == "GoldenFloat") == Some(true) {
                *r.checked.entry("gf-closed-form").or_insert(0) += 1;
                let e_rule = (((b - 1) as f64) / PHI2).round() as i64;
                if e != e_rule || m != b - 1 - e_rule {
                    r.findings.push(Finding {
                        id: rec.id.clone(),
                        check: "gf-closed-form",
                        detail: format!(
                            "e = round(({}-1)/phi^2) = {}, m = {}; record says e={} m={}",
                            b,
                            e_rule,
                            b - 1 - e_rule,
                            e,
                            m
                        ),
                    });
                }
                // -- is e the MINIMISER, not merely the rule's output? -----
                //
                // T7: the rule solves e/m = 1/phi exactly and then rounds, but
                // e/(N-1-e) is nonlinear, so the nearest integer to the root is
                // not always the integer that minimises the ratio error. Over
                // N in [4, 4000] it fails at N = 5, 73 and 1293. No published
                // rung is one of those -- but the rule is a HEURISTIC, and a
                // future rung at such a width would be suboptimal by the
                // ladder's own criterion. So check the property the ladder
                // actually wants, not the procedure that usually produces it.
                *r.checked.entry("gf-ratio-optimal").or_insert(0) += 1;
                let target = 1.0 / PHI;
                let err_of = |e: i64| ((e as f64) / ((b - 1 - e) as f64) - target).abs();
                let mut best = e;
                for cand in 1..(b - 1) {
                    if err_of(cand) < err_of(best) {
                        best = cand;
                    }
                }
                if best != e {
                    r.findings.push(Finding {
                        id: rec.id.clone(),
                        check: "gf-ratio-optimal",
                        detail: format!(
                            "e={} gives |e/m - 1/phi| = {:.8}, but e={} gives {:.8} -- \
                             the rule rounded to the wrong side (T7)",
                            e,
                            err_of(e),
                            best,
                            err_of(best)
                        ),
                    });
                }

                // -- phi_distance = |e/m - 1/phi| --------------------------
                if let (Some(pd), true) = (rec.f64("phi_distance"), m != 0) {
                    *r.checked.entry("gf-phi-distance").or_insert(0) += 1;
                    let computed = ((e as f64) / (m as f64) - 1.0 / PHI).abs();
                    if (pd - computed).abs() > PHI_DIST_TOL {
                        r.findings.push(Finding {
                            id: rec.id.clone(),
                            check: "gf-phi-distance",
                            detail: format!(
                                "recorded {:.4}, |e/m - 1/phi| = {:.4}",
                                pd, computed
                            ),
                        });
                    }
                }
            }
        }

        // -- source= names a spec: do its constants agree? -------------------
        if let Some(srcpath) = rec.fields.get("source") {
            if srcpath.ends_with(".t27") {
                let p = specs_root.join(srcpath.trim_start_matches("specs/"));
                let p = if p.is_file() {
                    p
                } else {
                    Path::new(srcpath).to_path_buf()
                };
                if let Ok(spec) = std::fs::read_to_string(&p) {
                    *r.checked.entry("source-agrees").or_insert(0) += 1;
                    for (field, konst) in [
                        ("bits", "TOTAL_BITS"),
                        ("s", "SIGN_BITS"),
                        ("e", "EXP_BITS"),
                        ("m", "MANT_BITS"),
                    ] {
                        let want = match rec.num(field) {
                            Some(v) => v,
                            None => continue,
                        };
                        let pat = format!("const {} ", konst);
                        let got = spec.lines().find(|l| l.contains(&pat)).and_then(|l| {
                            l.rsplit('=')
                                .next()?
                                .trim()
                                .trim_end_matches(';')
                                .trim()
                                .parse::<i64>()
                                .ok()
                        });
                        if let Some(g) = got {
                            if g != want {
                                r.findings.push(Finding {
                                    id: rec.id.clone(),
                                    check: "source-agrees",
                                    detail: format!(
                                        "catalog {}={} but {} says {}={}",
                                        field, want, srcpath, konst, g
                                    ),
                                });
                            }
                        }
                    }
                } else {
                    r.findings.push(Finding {
                        id: rec.id.clone(),
                        check: "source-exists",
                        detail: format!("source={} does not exist", srcpath),
                    });
                }
            }
        }
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_squared_is_phi_plus_one() {
        // L5, and the constant this gate's central rule depends on.
        assert!((PHI2 - (PHI + 1.0)).abs() < 1e-12);
        assert!((PHI2 + 1.0 / PHI2 - 3.0).abs() < 1e-12);
    }

    #[test]
    fn a_quoted_value_with_spaces_parses_whole() {
        let recs = parse_records(r#"// CATALOG: id=x name="A B C" bits=8 s=1 e=3 m=4"#);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].fields.get("name").map(String::as_str), Some("A B C"));
        assert_eq!(recs[0].fields.get("bits").map(String::as_str), Some("8"));
    }

    #[test]
    fn shape_classification_matches_the_three_exceptions() {
        let recs = parse_records(
            "// CATALOG: id=a bits=8 s=1 e=3 m=4 cluster=Ieee754Binary status=Verified\n\
             // CATALOG: id=posit8 bits=8 s=1 e=2 m=0 cluster=PositUnumIII status=Verified\n\
             // CATALOG: id=q_format bits=0 s=1 e=0 m=0 cluster=IntegerFixed status=Verified\n\
             // CATALOG: id=gfternary bits=2 s=1 e=0 m=2 cluster=GoldenFloat status=Verified\n",
        );
        assert_eq!(recs[0].shape, Shape::FixedLayout);
        assert_eq!(recs[1].shape, Shape::Tapered);
        assert_eq!(recs[2].shape, Shape::Parametric);
        assert_eq!(recs[3].shape, Shape::Alphabet);
    }

    /// T7: the rule is not a proven minimiser. These three widths are where it
    /// rounds to the wrong side, and the gate must catch them if a rung is ever
    /// added at one -- this test is the record that they exist.
    #[test]
    fn the_rule_is_not_the_minimiser_at_5_73_and_1293() {
        let target = 1.0 / PHI;
        for n in [5i64, 73, 1293] {
            let e_rule = (((n - 1) as f64) / PHI2).round() as i64;
            let err = |e: i64| ((e as f64) / ((n - 1 - e) as f64) - target).abs();
            let best = (1..(n - 1)).min_by(|a, b| err(*a).partial_cmp(&err(*b)).unwrap()).unwrap();
            assert_ne!(best, e_rule, "N={} was expected to be an exception", n);
        }
    }

    /// ... and every published rung is outside that exceptional set.
    #[test]
    fn every_published_rung_is_ratio_optimal() {
        let target = 1.0 / PHI;
        for n in [4i64, 6, 8, 10, 12, 14, 16, 20, 24, 32, 48, 64, 96, 128, 256, 512, 1024] {
            let e_rule = (((n - 1) as f64) / PHI2).round() as i64;
            let err = |e: i64| ((e as f64) / ((n - 1 - e) as f64) - target).abs();
            let best = (1..(n - 1)).min_by(|a, b| err(*a).partial_cmp(&err(*b)).unwrap()).unwrap();
            assert_eq!(best, e_rule, "gf{} is not ratio-optimal", n);
        }
    }

    #[test]
    fn the_gf_rule_reproduces_the_published_ladder() {
        // Every rung, from the catalog, checked against e = round((N-1)/phi^2).
        for (n, e) in [
            (4, 1),
            (6, 2),
            (8, 3),
            (10, 3),
            (12, 4),
            (14, 5),
            (16, 6),
            (20, 7),
            (24, 9),
            (32, 12),
            (48, 18),
            (64, 24),
            (96, 36),
            (128, 49),
            (256, 97),
            (512, 195),
            (1024, 391),
        ] {
            assert_eq!(
                (((n - 1) as f64) / PHI2).round() as i64,
                e,
                "gf{} exponent width",
                n
            );
        }
    }
}
