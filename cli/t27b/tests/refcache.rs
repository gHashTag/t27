//! `t27b corpus --reference-cache` (#6332): rows written by concurrent
//! workers never interleave, and the key follows the t27c binary's content,
//! not its mtime.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use t27b::blockers::{binary_stamp, cache_row, cacheable, read_cache, use_closure, CacheWriter, Reference};

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("t27b-refcache-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn concurrent_appends_write_only_whole_rows() {
    let d = scratch("append");
    let cache = d.join("cache.tsv");
    let w = Arc::new(CacheWriter::new(cache.clone()));
    let (threads, per) = (16u64, 200u64);
    // A long reason makes each row big enough that a split write would show.
    let long = "x".repeat(3000);
    let mut hs = Vec::new();
    for t in 0..threads {
        let (w, long) = (w.clone(), long.clone());
        hs.push(std::thread::spawn(move || {
            for i in 0..per {
                let key = t * 1_000_000 + i;
                let r = match i % 4 {
                    0 => Reference::Pass,
                    1 => Reference::Blocked(format!("t{} i{} {}", t, i, long)),
                    2 => Reference::Fail(format!("t{} i{}\nsecond line", t, i)),
                    _ => Reference::Timeout,
                };
                w.append(key, Path::new(&format!("specs/t{}/f{}.t27", t, i)), &r);
            }
        }));
    }
    for h in hs {
        h.join().unwrap();
    }
    let text = std::fs::read_to_string(&cache).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len() as u64, threads * per, "one line per row");
    for l in &lines {
        let mut it = l.splitn(3, '\t');
        let (k, p, r) = (it.next().unwrap(), it.next().unwrap(), it.next().unwrap());
        let key = u64::from_str_radix(k, 16).expect("hex key");
        let (t, i) = (key / 1_000_000, key % 1_000_000);
        assert_eq!(p, format!("specs/t{}/f{}.t27", t, i), "path belongs to its key: {:.120}", l);
        let want = match i % 4 {
            0 => Reference::Pass,
            1 => Reference::Blocked(format!("t{} i{} {}", t, i, long)),
            2 => Reference::Fail(format!("t{} i{} second line", t, i)),
            _ => Reference::Timeout,
        };
        assert_eq!(Reference::decode(r), Some(want), "row: {:.120}", l);
    }
    assert_eq!(read_cache(&cache).len() as u64, threads * per);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn cache_row_is_one_line() {
    let row = cache_row(0xab, Path::new("a\tb\nc.t27"), &Reference::Fail("x\r\ny".into()));
    assert_eq!(row, "00000000000000ab\ta b c.t27\tfail\tx  y\n");
}

#[test]
fn old_and_damaged_rows_are_read_harmlessly() {
    let d = scratch("read");
    let cache = d.join("cache.tsv");
    std::fs::write(
        &cache,
        "00000000000000aa\tspecs/a.t27\tpass\n\
         garbage line\n\
         00000000000000bb\tspecs/b.t27\tfail\tboom\n\
         zz\tspecs/c.t27\tpass\n\
         00000000000000cc\tspecs/c.t27\tnonsense\n",
    )
    .unwrap();
    let known = read_cache(&cache);
    assert_eq!(known.len(), 2);
    assert_eq!(known.get(&0xaa), Some(&Reference::Pass));
    assert_eq!(known.get(&0xbb), Some(&Reference::Fail("boom".into())));
    assert!(read_cache(&d.join("missing.tsv")).is_empty());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn stamp_follows_content_not_mtime() {
    let d = scratch("stamp");
    let a = d.join("t27c");
    std::fs::write(&a, b"\x7fELF pretend t27c binary 1").unwrap();
    // A copy made later has a different mtime and the same bytes.
    std::thread::sleep(std::time::Duration::from_millis(20));
    let b = d.join("t27c-copy");
    std::fs::write(&b, std::fs::read(&a).unwrap()).unwrap();
    let (ma, mb) = (std::fs::metadata(&a).unwrap().modified().unwrap(), std::fs::metadata(&b).unwrap().modified().unwrap());
    assert_ne!(ma, mb, "the copy must have its own mtime for this test to mean anything");
    assert_eq!(binary_stamp(&a).unwrap(), binary_stamp(&b).unwrap(), "byte-identical copy, same key");
    // A rebuilt binary (same size, different bytes) misses.
    let c = d.join("t27c-rebuilt");
    std::fs::write(&c, b"\x7fELF pretend t27c binary 2").unwrap();
    assert_ne!(binary_stamp(&a).unwrap(), binary_stamp(&c).unwrap());
    assert!(binary_stamp(&d.join("absent")).is_err());
    let _ = std::fs::remove_dir_all(&d);
}

/// #6441: a row carries the per-test verdicts; a row from before has none.
#[test]
fn rows_carry_per_test_verdicts() {
    use t27b::blockers::{cache_row_tests, decode_row, read_cache_tests};
    let tests = vec![("a;b%c".to_string(), true), ("x\ty".to_string(), false), ("plain".to_string(), true)];
    let r = Reference::Fail("1 of 3 tests fail".into());
    let row = cache_row_tests(0xab, Path::new("specs/a.t27"), &r, Some(&tests));
    assert_eq!(row.lines().count(), 1);
    let field = row.trim_end().splitn(3, '\t').nth(2).unwrap();
    assert_eq!(decode_row(field), Some((r.clone(), Some(tests.clone()))));
    // An old binary reads the verdict alone.
    assert_eq!(Reference::decode(field), Some(r.clone()));
    let empty = cache_row_tests(0xcd, Path::new("specs/b.t27"), &Reference::Pass, Some(&[]));
    let field = empty.trim_end().splitn(3, '\t').nth(2).unwrap();
    assert_eq!(decode_row(field), Some((Reference::Pass, Some(vec![]))));
    assert_eq!(decode_row("pass"), Some((Reference::Pass, None)));
    assert_eq!(decode_row("fail\tboom\ttests=?x"), None, "a damaged list is a damaged row");

    let d = scratch("tests");
    let cache = d.join("cache.tsv");
    std::fs::write(&cache, format!("{}00000000000000aa\tspecs/old.t27\tfail\t1 of 2 tests fail\n", row)).unwrap();
    let known = read_cache_tests(&cache);
    assert_eq!(known.get(&0xab), Some(&(r, Some(tests))));
    assert_eq!(known.get(&0xaa), Some(&(Reference::Fail("1 of 2 tests fail".into()), None)));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_timeout_is_never_cacheable() {
    // #6443: a timeout measures the machine, not the spec.
    assert!(!cacheable(&Reference::Timeout));
    assert!(cacheable(&Reference::Pass));
    assert!(cacheable(&Reference::Fail("x".into())));
    assert!(cacheable(&Reference::Blocked("t27c test-report exited Some(1): e".into())));
}

#[test]
fn the_key_closure_follows_use() {
    // #6443: t27c splices imported declarations in, so an import's bytes are
    // part of the verdict. `use a::c;` with a trailing comment, and the dotted
    // form, both resolve; a missing import is skipped.
    let d = scratch("closure");
    let a = d.join("specs").join("a");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::write(a.join("b.t27"), "module b;\nuse a::c;   // note\nuse a.e;\nuse a::gone;\n").unwrap();
    std::fs::write(a.join("c.t27"), "module c;\nuse a::e;\n").unwrap();
    std::fs::write(a.join("e.t27"), "module e;\nuse a::b;\n").unwrap();
    let got = use_closure(&a.join("b.t27"));
    assert_eq!(got, vec![a.join("b.t27"), a.join("c.t27"), a.join("e.t27")]);
    let _ = std::fs::remove_dir_all(&d);
}
