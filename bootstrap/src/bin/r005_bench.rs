use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("baseline");
    fs::create_dir_all("gen/rust").unwrap();

    let weights = 3 * 27 * 27;

    let bytes: Vec<u8> = match mode {
        "compressed" => (0..weights)
            .flat_map(|i| (i as u16).to_le_bytes())
            .collect(),
        _ => (0..weights)
            .flat_map(|i| (i as i64).to_le_bytes())
            .collect(),
    };

    let path = match mode {
        "compressed" => "gen/rust/toy_lm_compressed.trib",
        _ => "gen/rust/toy_lm_baseline.trib",
    };

    fs::write(path, &bytes).unwrap();
    println!("artifact_size: {} bytes", bytes.len());
    println!("path: {}", path);
}
