//! Hardware integration test — requires a DLC10 cable plugged in.
//! Run with `cargo test -p dlc10 -- --ignored idcode_xc7a100t`.

#[test]
#[ignore]
fn idcode_xc7a100t() {
    let mut cable = dlc10::Dlc10::open().expect("open dlc10");
    let id = cable.read_idcode().expect("read idcode");
    assert_eq!(id, 0x13631093, "expected XC7A100T IDCODE");
}
