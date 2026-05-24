//! Apex Legends helpers (based on zapret-discord-youtube issues #6503, #6198, #9730).

use super::probe::http_probe;
use crate::paths::{find_data_file, zapret_dir};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const LEGACY_APEX_BAT: &str = "general (APEX).bat";
const APEX_BAT: &str = "general (ALT11 APEX).bat";
const APEX_LIST: &str = "list-apex.txt";
const APEX_LIST_EXTRA: &str = "list-apex-extra.txt";
const IPSET_EXCLUDE_APEX_EA: &str = "ipset-exclude-apex-ea.txt";

/// HTTP endpoints to probe EA / Apex connectivity (not in-game UDP).
pub const APEX_PROBE_TARGETS: &[(&str, &str)] = &[
    ("ea_web", "https://www.ea.com"),
    ("ea_accounts", "https://accounts.ea.com"),
    ("origin", "https://www.origin.com"),
    ("apex_site", "https://www.apexlegends.com"),
    ("ea_cdn", "https://eaassets-a.akamaihd.net"),
];

fn extra_resource_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("resources").join("zapret-extra")
}

fn copy_bundled(relative: &str, dest: &Path) -> Result<(), String> {
    let src = find_data_file(&format!("resources/zapret-extra/{relative}"))
        .or_else(|| {
            let p = extra_resource_root().join(relative);
            if p.is_file() {
                Some(p)
            } else {
                None
            }
        })
        .ok_or_else(|| format!("Встроенный файл не найден: {relative}"))?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::copy(&src, dest).map_err(|e| format!("Не удалось скопировать {relative}: {e}"))?;
    Ok(())
}

/// Copy fastpatch Apex presets into installed zapret folder.
pub fn ensure_apex_assets() -> Result<(), String> {
    let root = zapret_dir();
    if !root.join("bin").join("winws.exe").is_file() {
        return Ok(());
    }

    let lists = root.join("lists");
    std::fs::create_dir_all(&lists).map_err(|e| e.to_string())?;

    copy_bundled(&format!("lists/{APEX_LIST}"), &lists.join(APEX_LIST))?;
    copy_bundled(
        &format!("lists/{APEX_LIST_EXTRA}"),
        &lists.join(APEX_LIST_EXTRA),
    )?;
    copy_bundled(
        &format!("lists/{IPSET_EXCLUDE_APEX_EA}"),
        &lists.join(IPSET_EXCLUDE_APEX_EA),
    )?;
    copy_bundled(APEX_BAT, &root.join(APEX_BAT))?;

    let legacy = root.join(LEGACY_APEX_BAT);
    if legacy.is_file() {
        let _ = std::fs::remove_file(&legacy);
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApexTip {
    pub title: String,
    pub body: String,
    pub issue_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApexStatus {
    pub zapret_installed: bool,
    pub list_installed: bool,
    pub bat_installed: bool,
    pub strategy_available: bool,
    pub game_filter: String,
    pub tips: Vec<ApexTip>,
}

pub fn apex_tips() -> Vec<ApexTip> {
    vec![
        ApexTip {
            title: "Стратегия ALT11 APEX".into(),
            body: "Единственный пресет Apex: ALT11 + list-apex, ipset-exclude EA/Respawn (155.133/16), игровой UDP без ipset-all. Установите пресет и подключите из панели Apex.".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/6503".into()),
        },
        ApexTip {
            title: "Игровой фильтр".into(),
            body: "Для Apex держите Game Filter выключенным — иначе лобби и возврат после матча часто ломаются (#6198, #8724).".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/6198".into()),
        },
        ApexTip {
            title: "code:leaf / матч".into(),
            body: "Порты матчмейкинга и list-apex уже в пресете. При code:leaf обновите пресет Apex и переподключите стратегию.".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/9730".into()),
        },
        ApexTip {
            title: "После матча".into(),
            body: "Если не возвращает в лобби — «Установить пресет» и снова ALT11 APEX. Игровой фильтр — выкл.".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/9529".into()),
        },
    ]
}

fn read_game_filter_mode(root: &Path) -> String {
    let flag = root.join("utils").join("game_filter.enabled");
    if !flag.is_file() {
        return "disabled".into();
    }
    std::fs::read_to_string(&flag)
        .unwrap_or_default()
        .trim()
        .to_lowercase()
}

#[tauri::command]
pub fn get_apex_status() -> ApexStatus {
    let root = zapret_dir();
    let installed = root.join("bin").join("winws.exe").is_file();
    let list_installed = root.join("lists").join(APEX_LIST).is_file();
    let bat_installed = root.join(APEX_BAT).is_file();

    ApexStatus {
        zapret_installed: installed,
        list_installed,
        bat_installed,
        strategy_available: bat_installed && installed,
        game_filter: read_game_filter_mode(&root),
        tips: apex_tips(),
    }
}

/// Install Apex files and apply recommended zapret settings for Apex.
#[tauri::command]
pub fn setup_apex_preset() -> Result<String, String> {
    ensure_apex_assets()?;

    // README / issues: Game Filter often breaks Apex lobby when other sites work
    let flag = zapret_dir().join("utils").join("game_filter.enabled");
    if flag.is_file() {
        std::fs::remove_file(&flag).map_err(|e| e.to_string())?;
    }

    Ok(
        "Пресет Apex: ALT11 APEX (list-apex, ipset-exclude EA). Игровой фильтр выключен. Подключите стратегию из панели или списка.".into(),
    )
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApexProbeResult {
    pub target: String,
    pub success: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

pub async fn probe_apex_targets(timeout_secs: u64) -> Vec<ApexProbeResult> {
    let mut handles = Vec::new();
    for (tag, url) in APEX_PROBE_TARGETS {
        let tag = tag.to_string();
        let url = url.to_string();
        handles.push(tokio::spawn(async move {
            let (success, latency_ms, error) = http_probe(&url, timeout_secs).await;
            ApexProbeResult {
                target: tag,
                success,
                latency_ms,
                error,
            }
        }));
    }

    let mut out = Vec::new();
    for h in handles {
        if let Ok(r) = h.await {
            out.push(r);
        }
    }
    out
}

#[tauri::command]
pub async fn test_apex_connectivity() -> Result<Vec<ApexProbeResult>, String> {
    Ok(probe_apex_targets(3).await)
}
