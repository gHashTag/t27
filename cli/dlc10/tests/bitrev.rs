use dlc10::{bitrev, BIT_REV_TABLE};

#[test]
fn table_is_self_inverse() {
    for b in 0u8..=255 {
        assert_eq!(BIT_REV_TABLE[BIT_REV_TABLE[b as usize] as usize], b);
    }
}

#[test]
fn bitrev_idempotent_twice() {
    let data: Vec<u8> = (0..32u8).collect();
    let r = bitrev(&data);
    let rr = bitrev(&r);
    assert_eq!(rr, data);
}
