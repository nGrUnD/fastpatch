use std::path::PathBuf;
use tauri::{Manager, Runtime};

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

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            push_dir_candidates(&mut candidates, parent.to_path_buf());
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        push_dir_candidates(&mut candidates, cwd);
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    push_dir_candidates(&mut candidates, manifest.join(".."));
    push_dir_candidates(&mut candidates, manifest.clone());

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

/// Пути поиска icon.ico / icon.png (MSI, resource_dir, dev).
pub fn icon_search_paths<R: Runtime>(app: &tauri::App<R>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(res) = app.path().resource_dir() {
        paths.push(res.join("icon.ico"));
        paths.push(res.join("icons").join("icon.ico"));
        paths.push(res.join("32x32.png"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("icon.ico"));
            paths.push(dir.join("resources").join("icon.ico"));
        }
    }
    let dev_icons = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons");
    paths.push(dev_icons.join("icon.ico"));
    paths.push(dev_icons.join("32x32.png"));
    paths
}

/// Locate a data file (e.g. `strategies.json`) for dev and bundled runs.
pub fn find_data_file(filename: &str) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

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

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest.join("..").join(filename));
    candidates.push(manifest.join(filename));

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

/// Файлы пресета Apex из bundle MSI/NSIS или из репозитория (dev).
pub fn find_zapret_extra_file(relative: &str) -> Option<PathBuf> {
    let rel = relative.replace('\\', "/");
    let file_name = PathBuf::from(&rel)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&rel)
        .to_string();

    let mut candidates: Vec<PathBuf> = Vec::new();

    let rel_suffixes = [
        rel.clone(),
        format!("zapret-extra/{rel}"),
        format!("resources/zapret-extra/{rel}"),
        format!("_up_/resources/zapret-extra/{rel}"),
        file_name,
    ];

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for rp in rel_suffixes {
                candidates.push(dir.join(rp));
            }
        }
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(
        manifest
            .join("..")
            .join("resources")
            .join("zapret-extra")
            .join(&rel),
    );

    candidates.into_iter().find(|p| p.is_file())
}

/// Файлы пресета Apex для Zapret 2 (MSI/NSIS, dev).
pub fn find_zapret2_extra_file(relative: &str) -> Option<PathBuf> {
    let rel = relative.replace('\\', "/");
    let file_name = PathBuf::from(&rel)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&rel)
        .to_string();

    let mut candidates: Vec<PathBuf> = Vec::new();

    let rel_suffixes = [
        rel.clone(),
        format!("zapret2-extra/{rel}"),
        format!("resources/zapret2-extra/{rel}"),
        format!("_up_/resources/zapret2-extra/{rel}"),
        file_name,
    ];

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for rp in rel_suffixes {
                candidates.push(dir.join(rp));
            }
        }
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(
        manifest
            .join("..")
            .join("resources")
            .join("zapret2-extra")
            .join(&rel),
    );

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
            "winws.exe не найден: {}.\n\nУстановите zapret: на главной нажмите «Подключить» или положите папку zapret (с bin/winws.exe) рядом с fastpatch.exe.",
            path.display()
        ))
    }
}

fn zapret2_install_base() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Корень bundle Zapret 2 (zapret2-youtube-discord): `exe/winws2.exe`.
pub fn zapret2_dir() -> PathBuf {
    find_zapret2_dir().unwrap_or_else(|| zapret2_install_base().join("zapret2"))
}

pub fn find_zapret2_dir() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("zapret2"));
            let mut dir = Some(parent.to_path_buf());
            for _ in 0..8 {
                if let Some(ref d) = dir {
                    candidates.push(d.join("zapret2"));
                    dir = d.parent().map(|p| p.to_path_buf());
                } else {
                    break;
                }
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("zapret2"));
    }

    for dir in candidates {
        if dir.join("exe").join("winws2.exe").is_file() {
            return Some(dir);
        }
    }
    None
}

pub fn winws2_path() -> PathBuf {
    zapret2_dir().join("exe").join("winws2.exe")
}

pub fn ensure_current_engine() -> Result<PathBuf, String> {
    use crate::app_prefs::{load, ZapretBackendPref};
    match load().zapret_backend {
        ZapretBackendPref::V2 => ensure_winws2(),
        ZapretBackendPref::V1 => ensure_winws(),
    }
}

pub fn ensure_winws2() -> Result<PathBuf, String> {
    let path = winws2_path();
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "winws2.exe не найден: {}.\n\nНажмите «Подключить» на главной — fastpatch установит Zapret 2.",
            path.display()
        ))
    }
}
