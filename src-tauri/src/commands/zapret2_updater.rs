//! Установка bundle [zapret2-youtube-discord](https://github.com/periayellowish469/zapret2-youtube-discord).

use crate::commands::updater::ReleaseInfo;
use crate::paths::zapret2_dir;
use serde::Deserialize;
use std::path::{Path, PathBuf};

const REPO_OWNER: &str = "periayellowish469";
const REPO_NAME: &str = "zapret2-youtube-discord";
const VERSION_FILE: &str = "zapret2_version.txt";

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

fn http_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("fastpatch/0.3")
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| e.to_string())
}

async fn fetch_latest_release() -> Result<GithubRelease, String> {
    let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest");
    let client = http_client(120)?;
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("Не удалось подключиться к GitHub: {e}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Не удалось прочитать ответ GitHub: {e}"))?;

    if !status.is_success() {
        return Err(format!("GitHub API вернул {status}"));
    }

    serde_json::from_str(&body).map_err(|e| format!("Ошибка парсинга ответа GitHub: {e}"))
}

fn pick_7z_url(release: &GithubRelease) -> Result<String, String> {
    release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".7z"))
        .map(|a| a.browser_download_url.clone())
        .ok_or_else(|| {
            format!(
                "В релизе {} нет .7z архива",
                release.tag_name
            )
        })
}

fn version_file_path_in(root: &Path) -> PathBuf {
    root.join(VERSION_FILE)
}

fn version_file_path() -> PathBuf {
    version_file_path_in(&zapret2_dir())
}

pub fn read_current_version() -> String {
    std::fs::read_to_string(version_file_path())
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn release_to_info(release: GithubRelease) -> ReleaseInfo {
    let download_url = pick_7z_url(&release).unwrap_or_default();
    let current_version = read_current_version();
    let has_update =
        current_version != "unknown" && current_version != release.tag_name && !download_url.is_empty();

    ReleaseInfo {
        tag_name: release.tag_name.clone(),
        name: release.name.unwrap_or_else(|| release.tag_name.clone()),
        body: release.body.unwrap_or_default(),
        published_at: release.published_at,
        download_url,
        current_version,
        has_update,
    }
}

async fn download_bytes(url: &str) -> Result<Vec<u8>, String> {
    let client = http_client(300)?;
    let bytes = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Ошибка скачивания Zapret 2: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Ошибка чтения данных: {e}"))?;
    Ok(bytes.to_vec())
}

fn extract_7z(bytes: &[u8], target_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(target_dir)
        .map_err(|e| format!("Не удалось создать {}: {e}", target_dir.display()))?;

    let tmp = std::env::temp_dir().join(format!(
        "fastpatch-zapret2-{}.7z",
        std::process::id()
    ));
    std::fs::write(&tmp, bytes).map_err(|e| format!("Не удалось записать архив: {e}"))?;

    let extract_to = std::env::temp_dir().join(format!(
        "fastpatch-zapret2-extract-{}",
        std::process::id()
    ));
    if extract_to.exists() {
        let _ = std::fs::remove_dir_all(&extract_to);
    }
    std::fs::create_dir_all(&extract_to).map_err(|e| e.to_string())?;

    sevenz_rust::decompress_file(&tmp, &extract_to)
        .map_err(|e| format!("Ошибка распаковки 7z: {e}"))?;
    let _ = std::fs::remove_file(&tmp);

    copy_extracted_tree(&extract_to, target_dir)?;
    let _ = std::fs::remove_dir_all(&extract_to);
    Ok(())
}

fn copy_extracted_tree(src: &Path, dest: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Err("Пустой архив Zapret 2".into());
    }

    let entries: Vec<_> = std::fs::read_dir(src)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();

    let copy_from = if entries.len() == 1 {
        let only = &entries[0];
        if only.path().is_dir() {
            only.path().clone()
        } else {
            src.to_path_buf()
        }
    } else {
        src.to_path_buf()
    };

    copy_dir_recursive(&copy_from, dest)?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::copy(&from, &to).map_err(|e| format!("copy {}: {e}", from.display()))?;
        }
    }
    Ok(())
}

fn verify_install_at(root: &Path) -> Result<(), String> {
    let winws2 = root.join("exe").join("winws2.exe");
    if winws2.is_file() {
        Ok(())
    } else {
        Err(format!(
            "После установки winws2.exe не найден: {}",
            winws2.display()
        ))
    }
}

fn staging_dir_for(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("zapret2");
    target.with_file_name(format!("{name}.staging-{}", std::process::id()))
}

fn backup_dir_for(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("zapret2");
    target.with_file_name(format!("{name}.backup-{}", std::process::id()))
}

fn replace_install_dir(staging: &Path, target: &Path) -> Result<(), String> {
    let backup = backup_dir_for(target);
    if backup.exists() {
        let _ = std::fs::remove_dir_all(&backup);
    }

    if target.exists() {
        std::fs::rename(target, &backup).map_err(|e| {
            format!(
                "Не удалось подготовить замену {}: {e}. Закройте процессы Zapret и повторите.",
                target.display()
            )
        })?;
    }

    match std::fs::rename(staging, target) {
        Ok(()) => {
            let _ = std::fs::remove_dir_all(&backup);
            Ok(())
        }
        Err(e) => {
            if backup.exists() && !target.exists() {
                let _ = std::fs::rename(&backup, target);
            }
            Err(format!(
                "Не удалось заменить Zapret 2 в {}: {e}. Старая установка восстановлена.",
                target.display()
            ))
        }
    }
}

async fn install_zapret2_from_url(url: &str, tag_name: &str) -> Result<String, String> {
    let bytes = download_bytes(url).await?;
    let target = zapret2_dir();
    let staging = staging_dir_for(&target);

    if staging.exists() {
        let _ = std::fs::remove_dir_all(&staging);
    }

    extract_7z(&bytes, &staging)?;
    verify_install_at(&staging)?;
    std::fs::write(version_file_path_in(&staging), tag_name)
        .map_err(|e| format!("Ошибка сохранения версии Zapret 2: {e}"))?;

    crate::commands::zapret_backend::kill_all_processes_and_wait(5000);
    replace_install_dir(&staging, &target)?;
    if let Err(e) = crate::commands::apex::ensure_apex_assets_v2() {
        eprintln!("[fastpatch] ensure_apex_assets_v2: {e}");
    }

    Ok(format!(
        "Zapret 2 установлен ({}). Пресеты в presets/, Apex Legends добавлен.",
        tag_name
    ))
}

pub async fn check_for_updates_v2() -> Result<ReleaseInfo, String> {
    let release = fetch_latest_release().await?;
    Ok(release_to_info(release))
}

pub async fn install_zapret2_inner() -> Result<String, String> {
    let release = fetch_latest_release().await?;
    let url = pick_7z_url(&release)?;
    install_zapret2_from_url(&url, &release.tag_name).await
}

pub async fn apply_update_v2(download_url: String, tag_name: String) -> Result<String, String> {
    if download_url.is_empty() {
        return install_zapret2_inner().await;
    }
    install_zapret2_from_url(&download_url, &tag_name).await
}

pub fn zapret2_installed() -> bool {
    zapret2_dir().join("exe").join("winws2.exe").is_file()
}
