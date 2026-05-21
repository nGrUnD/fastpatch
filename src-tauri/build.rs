fn main() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let strategies = manifest_dir.join("../strategies.json");
    if strategies.exists() {
        println!("cargo:rerun-if-changed={}", strategies.display());
    }
    let icons_dir = manifest_dir.join("icons");
    for name in [
        "icon.ico",
        "icon.icns",
        "icon.png",
        "32x32.png",
        "128x128.png",
        "128x128@2x.png",
    ] {
        let path = icons_dir.join(name);
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    tauri_build::build()
}
