//! Лог ошибок старта (для MSI на чужих ПК без консоли).

use std::io::Write;
use std::path::PathBuf;

pub fn log(message: &str) {
    let Some(path) = log_path() else {
        eprintln!("[fastpatch] {message}");
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "{message}");
    }
}

fn log_path() -> Option<PathBuf> {
    let profile = std::env::var("USERPROFILE").ok()?;
    Some(
        PathBuf::from(profile)
            .join("AppData")
            .join("Roaming")
            .join("fastpatch")
            .join("startup.log"),
    )
}
