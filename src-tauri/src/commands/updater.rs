use crate::paths::{winws_path, zapret_dir};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

const REPO_OWNER: &str = "flowseal";
const REPO_NAME: &str = "zapret-discord-youtube";
const VERSION_FILE: &str = "zapret_version.txt";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub download_url: String,
    pub current_version: String,
    pub has_update: bool,
}

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

fn version_file_path() -> PathBuf {
    zapret_dir().join(VERSION_FILE)
}

fn read_current_version() -> String {
    std::fs::read_to_string(version_file_path())
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn http_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("fastpatch/0.1")
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| e.to_string())
}

async fn fetch_latest_release() -> Result<GithubRelease, String> {
    let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest");
    let client = http_client(15)?;
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
        return Err(format!(
            "GitHub API вернул {status}. Проверьте интернет или повторите позже."
        ));
    }

    serde_json::from_str(&body).map_err(|e| {
        format!(
            "Ошибка парсинга ответа GitHub: {e}. Ответ: {}",
            body.chars().take(200).collect::<String>()
        )
    })
}

fn pick_zip_url(release: &GithubRelease) -> Result<String, String> {
    release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".zip"))
        .map(|a| a.browser_download_url.clone())
        .ok_or_else(|| {
            format!(
                "В релизе {} нет .zip архива. Доступно: {}",
                release.tag_name,
                release
                    .assets
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

fn release_to_info(release: GithubRelease) -> ReleaseInfo {
    let download_url = pick_zip_url(&release).unwrap_or_default();
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

/// If every entry lives under one top-level folder (e.g. `zapret-1.9.8c/...`), return it.
fn detect_strip_prefix<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>) -> Option<PathBuf> {
    let mut root: Option<String> = None;

    for i in 0..archive.len() {
        let file = archive.by_index(i).ok()?;
        let name = file.enclosed_name()?;
        let mut components = name.components();
        let first = match components.next() {
            Some(Component::Normal(s)) => s.to_string_lossy().to_string(),
            _ => return None,
        };
        if components.next().is_none() {
            // File at archive root — no shared wrapper folder
            return None;
        }
        match &root {
            None => root = Some(first),
            Some(r) if *r == first => {}
            _ => return None,
        }
    }

    root.map(PathBuf::from)
}

fn map_zip_entry_path(enclosed: &Path, target_dir: &Path, strip_prefix: Option<&Path>) -> PathBuf {
    let relative = if let Some(prefix) = strip_prefix {
        enclosed.strip_prefix(prefix).unwrap_or(enclosed)
    } else {
        enclosed
    };
    target_dir.join(relative)
}

async fn download_bytes(url: &str) -> Result<Vec<u8>, String> {
    let client = http_client(180)?;
    let bytes = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Ошибка скачивания: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Ошибка чтения данных: {e}"))?;
    Ok(bytes.to_vec())
}

fn extract_zip(bytes: &[u8], target_dir: &Path, full_install: bool) -> Result<usize, String> {
    std::fs::create_dir_all(target_dir)
        .map_err(|e| format!("Не удалось создать папку {}: {e}", target_dir.display()))?;

    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Ошибка открытия архива: {e}"))?;

    let strip_prefix = detect_strip_prefix(&mut archive);
    let mut written = 0usize;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Ошибка чтения файла в архиве: {e}"))?;

        let enclosed = match file.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };

        let outpath = map_zip_entry_path(&enclosed, target_dir, strip_prefix.as_deref());

        if !full_install {
            let outpath_str = outpath.to_string_lossy();
            let is_bat = outpath.extension().map(|e| e == "bat").unwrap_or(false);
            let is_bin = outpath_str.contains("bin\\") || outpath_str.contains("bin/");
            let is_list = outpath.extension().map(|e| e == "txt").unwrap_or(false);
            let filename = outpath
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if filename.contains("-user") || filename == "hosts" {
                continue;
            }
            if !is_bat && !is_bin && !is_list {
                continue;
            }
        }

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("Ошибка создания директории: {e}"))?;
            continue;
        }

        if let Some(parent) = outpath.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Ошибка создания директории: {e}"))?;
        }

        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| format!("Ошибка чтения содержимого файла: {e}"))?;
        std::fs::write(&outpath, &content)
            .map_err(|e| format!("Ошибка записи файла {}: {e}", outpath.display()))?;
        written += 1;
    }

    Ok(written)
}

fn verify_install() -> Result<(), String> {
    let winws = winws_path();
    if winws.is_file() {
        Ok(())
    } else {
        Err(format!(
            "После установки winws.exe не найден: {}. Попробуйте переустановить.",
            winws.display()
        ))
    }
}

#[tauri::command]
pub async fn check_for_updates() -> Result<ReleaseInfo, String> {
    match crate::commands::zapret_backend::current() {
        crate::commands::zapret_backend::ZapretBackend::V2 => {
            crate::commands::zapret2_updater::check_for_updates_v2().await
        }
        crate::commands::zapret_backend::ZapretBackend::V1 => {
            let release = fetch_latest_release().await?;
            Ok(release_to_info(release))
        }
    }
}

/// Full zapret 1 install from GitHub (first run / repair).
pub async fn install_zapret_v1() -> Result<String, String> {
    let release = fetch_latest_release().await?;
    let download_url = pick_zip_url(&release)?;
    let tag_name = release.tag_name.clone();
    let target_dir = zapret_dir();

    let bytes = download_bytes(&download_url).await?;
    let written = extract_zip(&bytes, &target_dir, true)?;
    verify_install()?;
    let _ = crate::commands::apex::ensure_apex_assets();

    std::fs::write(version_file_path(), &tag_name)
        .map_err(|e| format!("Ошибка сохранения версии: {e}"))?;

    Ok(format!(
        "Zapret 1 {} установлен в {} ({} файлов). Пресет Apex добавлен.",
        tag_name,
        target_dir.display(),
        written
    ))
}

/// Установка ядра по настройке (Zapret 2 по умолчанию).
#[tauri::command]
pub async fn install_zapret() -> Result<String, String> {
    match crate::commands::zapret_backend::current() {
        crate::commands::zapret_backend::ZapretBackend::V2 => {
            crate::commands::zapret2_updater::install_zapret2_inner().await
        }
        crate::commands::zapret_backend::ZapretBackend::V1 => install_zapret_v1().await,
    }
}

#[tauri::command]
pub async fn apply_update(download_url: String, tag_name: String) -> Result<String, String> {
    if matches!(
        crate::commands::zapret_backend::current(),
        crate::commands::zapret_backend::ZapretBackend::V2
    ) {
        return crate::commands::zapret2_updater::apply_update_v2(download_url, tag_name).await;
    }

    if download_url.is_empty() {
        return install_zapret().await;
    }

    let target_dir = zapret_dir();
    let bytes = download_bytes(&download_url).await?;
    let written = extract_zip(&bytes, &target_dir, false)?;
    verify_install()?;
    let _ = crate::commands::apex::ensure_apex_assets();

    std::fs::write(version_file_path(), &tag_name)
        .map_err(|e| format!("Ошибка сохранения версии: {e}"))?;

    Ok(format!(
        "Обновление {} применено. Обновлено файлов: {written}",
        tag_name
    ))
}

#[tauri::command]
pub fn get_current_version() -> String {
    match crate::commands::zapret_backend::current() {
        crate::commands::zapret_backend::ZapretBackend::V2 => {
            crate::commands::zapret2_updater::read_current_version()
        }
        crate::commands::zapret_backend::ZapretBackend::V1 => read_current_version(),
    }
}
