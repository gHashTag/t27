//! `tri payout`: the AUTHOR_ROYALTY table of one epoch from git history and the labs' receipts. Glue only (git, curl, `t27c corpus-receipt compare`, the maps, print): every rule is specs/tri/network/payout.t27, which calls specs/network/attribution.t27 and usage.t27.
#[path = "../../../gen/rust/tri/network/payout.rs"] #[allow(dead_code, unused_parens)]
pub mod pay; // t27c gen-rust of specs/tri/network/payout.t27
use {anyhow::Result, sha2::{Digest, Sha256}, std::collections::{BTreeMap, BTreeSet, HashMap}, std::io::Write, std::process::{Command, Stdio}};
/// Run `c` with `input` on its stdin; its stdout, leaked, since the generated rules read `&'static str`.
fn sh(c: &str, a: &[&str], input: &str) -> &'static str { let mut p = Command::new(c).args(a).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().expect(c); let _ = p.stdin.take().map(|mut i| i.write_all(input.as_bytes())); String::from_utf8_lossy(&p.wait_with_output().map(|o| o.stdout).unwrap_or_default()).into_owned().leak() }
pub fn cmd(epoch: u64, pool: u64, json: bool) -> Result<u8> {
    let auth = std::env::var("GH_TOKEN").map(|t| format!("Authorization: Bearer {t}\n")).unwrap_or_default();
    let commits: Vec<&'static str> = sh("git", &["log", "--first-parent", pay::LOG_EPOCH, pay::REV], "").lines().filter(|l| pay::in_epoch(l, epoch)).map(|l| &l[..40]).collect();
    let (head, tmp, cache) = (commits.first().copied().unwrap_or(""), std::env::temp_dir().join(format!("tri-payout-{}.json", std::process::id())), std::env::temp_dir().join(pay::GH_CACHE)); let _ = std::fs::create_dir_all(&cache);
    let (mut facts, mut tables, mut green, mut corpus, mut counting) = (format!("epoch {epoch} pool {pool} head {head}\n"), Vec::new(), None, BTreeSet::new(), 0u64);
    for c in &commits { let mut rows = Vec::new(); for k in 0..pay::LAB_COUNT {
        let t = sh("curl", &["-fsSL", "--max-time", "300", &format!("{}{}{c}{}", pay::lab_url(k), pay::RUNS, pay::RECEIPT_EXT)], ""); if t.is_empty() { continue; }
        std::fs::write(&tmp, t)?; let rc = Command::new(crate::mutate::resolve_t27c(None)).args(["corpus-receipt", "compare"]).arg(&tmp).arg(&tmp).output()?.status.code().unwrap_or(-1);
        let (a, n) = (pay::receipt_auth(rc, pay::rc_is(t, "commit", c)), pay::rc_nonce_bytes(t)); let counts = pay::usage_counts(pay::hex_u64(c, 0, 16), a, n); counting += counts as u64;
        facts += &format!("receipt {c} {k} {} {a} {n} {}\n", &t[pay::rc_from(t, "key_id")..pay::rc_to(t, "key_id")], &t[pay::rc_from(t, "input_root")..pay::rc_to(t, "input_root")]);
        let leaves = |l: u32| { let (mut at, mut s) = (pay::rc_list_at(t, l), BTreeSet::new()); while pay::rc_is_leaf(t, at) { if pay::leaf_kept(t, at, l) { s.insert(&t[pay::leaf_field_from(t, at, 0)..pay::leaf_field_to(t, at, 0)]); } at = pay::line_end(t, at) + 1; } s };
        if counts && *c == head && green.is_none() { green = Some(leaves(pay::LIST_VERDICT)); } let reads = if counts { leaves(pay::LIST_INPUT) } else { BTreeSet::new() }; corpus.extend(reads.iter().copied()); rows.push((pay::rc_key(t), a, n, reads));
    } tables.push((pay::hex_u64(c, 0, 16), rows)); }
    let usage: Vec<u64> = corpus.iter().map(|p| tables.iter().map(|(c, rows)| { let mut u = ([0u64; 8], [0u64; 8], [epoch; 8], [0u8; 8], [0u8; 8], [0u64; 8]); for (i, (key, a, n, r)) in rows.iter().enumerate() { (u.0[i], u.1[i], u.3[i], u.4[i], u.5[i]) = (*c, *key, *a, *n, r.contains(p) as u64); } pay::usage_of(u.0, u.1, u.2, u.3, u.4, u.5, 0, epoch) }).sum()).collect(); // per commit, summed; a spec's reads are bit 0
    let agent: HashMap<&str, bool> = sh("git", &["log", "--first-parent", pay::LOG_FACTS, head], "").split('\x01').filter(|r| r.len() > 40).map(|r| (&r[..40], pay::agent_record(r))).collect();
    let cv: Vec<&'static str> = corpus.iter().copied().collect(); let blames: Vec<&'static str> = std::thread::scope(|s| cv.chunks(cv.len() / pay::BLAME_JOBS + 1).map(|ch| s.spawn(move || ch.iter().map(|p| sh("git", &["blame", "--first-parent", "--porcelain", head, "--", p], "")).collect::<Vec<_>>())).collect::<Vec<_>>().into_iter().flat_map(|h| h.join().unwrap()).collect());
    let (mut gh, mut logins, mut claims, mut total, mut unread, mut full) = (HashMap::<&str, &'static str>::new(), BTreeMap::new(), BTreeMap::<u64, u64>::new(), 0u64, 0u64, 0u64);
    for ((p, b), u) in cv.iter().zip(&blames).zip(&usage) {
        let (mut per, mut at, mut sha) = (BTreeMap::<&str, (u64, u64)>::new(), 0, "");
        while at < b.len() { let e = pay::line_end(b, at); if pay::bl_header(b, at, e) { sha = &b[at..at + 40]; } if pay::bl_line(b, at, e) { let r = per.entry(sha).or_default(); *r = (r.0 + 1, r.1 + pay::bl_test(b, at, e) as u64); } at = e + 1; }
        let mut t = ([pay::NO_SPEC; 16], [0u64; 16], [0u64; 16], [0u64; 16], [0u8; 16]);
        for (c, (l, n)) in per { let ag = agent.get(c).copied().unwrap_or(false); let g = *gh.entry(c).or_insert_with(|| if pay::needs_github(ag) { std::fs::read_to_string(cache.join(c)).ok().map(|s| &*s.leak()).filter(|s| pay::gh_read(s, c)).unwrap_or_else(|| { let t = sh("curl", &["-fsSL", "--max-time", "60", "-H", "@-", &format!("{}{c}", pay::API_COMMITS)], &auth); if pay::gh_read(t, c) { let _ = std::fs::write(cache.join(c), t); } t }) } else { "" });
            unread += pay::gh_unread(g, c, ag) as u64; let (id, f) = (pay::gh_author(g), pay::row_flags(g, ag, green.as_ref().is_some_and(|s: &BTreeSet<&str>| s.contains(p))));
            let k = pay::table_slot(t.0, t.1, t.2, t.3, t.4, id, f, l, n) as usize; if k == pay::ATTR_ROWS as usize { full += 1; break; }
            (t.0[k], t.1[k], t.2[k], t.3[k], t.4[k]) = (0, id, t.2[k] + l, t.3[k] + n, f); facts += &format!("row {p} {c} {id} {l} {n} {f}\n");
            if pay::gh_read(g, c) { logins.insert(id, &g[pay::gh_login_from(g)..pay::gh_login_to(g)]); } }
        total = pay::total_add(total, *u);
        for id in (0..16).filter(|&k| t.0[k] != pay::NO_SPEC).map(|k| pay::credited_to(t.1[k], t.4[k])).chain([pay::POOL_ID]).collect::<BTreeSet<_>>() { let e = claims.entry(id).or_insert(0); *e = pay::claim_add(*e, *u, pay::weight_of(t.0, t.1, t.2, t.3, t.4, 0, id)); }
    }
    let (share, label) = (|id: &u64, c: &u64| pay::share_of(pool, *c, total, *id), |id: &u64| logins.get(id).copied().unwrap_or(pay::POOL_LABEL)); let paid: u64 = claims.iter().map(|(id, c)| share(id, c)).sum();
    let table = claims.iter().map(|(id, c)| format!("{id} {} claim {c} share {}\n", label(id), share(id, c))).collect::<String>() + &format!("{} left {}\n", pay::POOL_LABEL, pool - paid);
    let (di, dt, x) = (hex::encode(Sha256::digest(&facts)), hex::encode(Sha256::digest(&table)), pay::pay_exit(commits.len() as u64, counting, unread, full, total)); let _ = std::fs::remove_file(&tmp);
    if json { println!("{}", serde_json::json!({"epoch": epoch, "pool": pool, "head": head, "commits": commits.len(), "counting_receipts": counting, "specs": cv.len(), "total_claim": total, "rows": claims.iter().map(|(id, c)| serde_json::json!({"id": id, "login": label(id), "claim": c, "share": share(id, c)})).collect::<Vec<_>>(), "pool_left": pool - paid, "inputs_sha256": di, "output_sha256": dt, "exit": x})); } else { print!("epoch {epoch} pool {pool} head {head} commits {} counting receipts {counting} specs {} total claim {total}\n{table}inputs sha256 {di}\noutput sha256 {dt}\n{}\n", commits.len(), cv.len(), pay::pay_why(x)); } Ok(x)
}
