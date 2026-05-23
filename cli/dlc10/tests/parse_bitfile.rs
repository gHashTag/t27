use dlc10::{bitrev, parse_bitfile};

#[test]
fn parses_synthetic_bit() {
    let payload: Vec<u8> = (0..64u8).collect();
    let mut buf = vec![0u8; 8];
    buf.push(0x65);
    buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    buf.extend_from_slice(&payload);
    let parsed = parse_bitfile(&buf).expect("parse");
    assert_eq!(parsed, bitrev(&payload));
}

#[test]
fn rejects_short_buffer() {
    assert!(parse_bitfile(&[0u8; 4]).is_err());
}
