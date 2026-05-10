//! Hardware integration test — requires DLC10 cable + Wukong V1 board.
//! Run with `cargo test -p dlc10 -- --ignored flash_jedec_id`.

#[test]
#[ignore]
fn flash_jedec_id() {
    let mut cable = dlc10::Dlc10::open().expect("open dlc10");
    let id = cable.read_flash_id().expect("read JEDEC id");
    eprintln!("JEDEC ID: {:02X} {:02X} {:02X}", id[0], id[1], id[2]);
    assert_ne!(id, [0xFF, 0xFF, 0xFF], "all-ones means flash absent or bridge dead");
    assert_ne!(id, [0x00, 0x00, 0x00]);
}
