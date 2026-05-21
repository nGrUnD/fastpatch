use std::path::PathBuf;

fn push_dir_candidates(candidates: &mut Vec<PathBuf>, dir: PathBuf) {
    candidates.push(dir.join("zapret"));
    candidates.push(dir.clone());
    let mut current = Some(dir);
    for _ in 0..8 {
        if let Some(ref d) = current {
            candidates.push(d.join("zapret"));
            candidates.push(d.clone());
            current = d.parent().map(|p| p.to_path_buf());
        } else {
            break;
        }
    }
}

/// Directory containing `bin/winws.exe`.
pub fn find_zapret_dir() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    push_dir_candidates(&mut candidates, manifest.join(".."));
    push_dir_candidates(&mut candidates, manifest.clone());

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            push_dir_candidates(&mut candidates, parent.to_path_buf());
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        push_dir_candidates(&mut candidates, cwd);
    }

    for dir in candidates {
        let winws = dir.join("bin").join("winws.exe");
        if winws.is_file() {
            return Some(dir);
        }
    }
    None
}

/// Default zapret folder (created on first update if missing).
pub fn zapret_dir() -> PathBuf {
    find_zapret_dir().unwrap_or_else(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("zapret")))
            .unwrap_or_else(|| PathBuf::from("zapret"))
    })
}

pub fn winws_path() -> PathBuf {
    zapret_dir().join("bin").join("winws.exe")
}

/// Locate a data file (e.g. `strategies.json`) for dev and bundled runs.
pub fn find_data_file(filename: &str) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest.join("..").join(filename));
    candidates.push(manifest.join(filename));

    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..8 {
            if let Some(ref d) = dir {
                candidates.push(d.join(filename));
                candidates.push(d.join("resources").join(filename));
                dir = d.parent().map(|p| p.to_path_buf());
            } else {
                break;
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let mut dir: Option<PathBuf> = Some(cwd);
        for _ in 0..5 {
            if let Some(ref d) = dir {
                candidates.push(d.join(filename));
                dir = d.parent().map(|p| p.to_path_buf());
            } else {
                break;
            }
        }
    }

    candidates.into_iter().find(|p| p.is_file())
}

pub fn data_file_or_err(filename: &str) -> Result<PathBuf, String> {
    find_data_file(filename).ok_or_else(|| {
        let hint = std::env::current_exe()
            .map(|p| format!(" (exe: {})", p.display()))
            .unwrap_or_default();
        format!(
            "Файл {filename} не найден{hint}. Положите его рядом с fastpatch.exe или в корень проекта."
        )
    })
}

pub fn ensure_winws() -> Result<PathBuf, String> {
    let path = winws_path();
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "winws.exe не найден: {}.\n\nУстановите zapret: на главной нажмите «Скачать zapret» или положите папку zapret (с bin/winws.exe) рядом с fastpatch.exe.",
            path.display()
        ))
    }
}
