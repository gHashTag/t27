use dlc10::parse_intel_hex;

#[test]
fn parses_small_hex() {
    let txt = "\
:020000041000EA
:04E0000001020304E2
:00000001FF
";
    let recs = parse_intel_hex(txt).expect("parse");
    // Type-0 with rlen=4 at addr 0xE000.
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].0, 0xE000);
    assert_eq!(recs[0].1, vec![0x01, 0x02, 0x03, 0x04]);
}

#[test]
fn eof_terminates() {
    let txt = ":00000001FF\n:04000000DEADBEEFCC\n";
    let recs = parse_intel_hex(txt).expect("parse");
    assert!(recs.is_empty());
}

#[test]
fn rejects_garbage() {
    assert!(parse_intel_hex(":NOTHEX\n").is_err());
}
