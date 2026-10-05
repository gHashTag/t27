//! `t27b corpus --reference-cache` (#6332): rows written by concurrent
//! workers never interleave, and the key follows the t27c binary's content,
//! not its mtime.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use t27b::blockers::{binary_stamp, cache_row, read_cache, CacheWriter, Reference};

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
