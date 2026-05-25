//! Apex Legends helpers (based on zapret-discord-youtube issues #6503, #6198, #9730).

use super::probe::http_probe;
use crate::app_prefs::{load, ZapretBackendPref};
use crate::paths::{
    find_zapret2_extra_file, find_zapret_extra_file, zapret2_dir, zapret_dir,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

const LEGACY_APEX_BAT: &str = "general (APEX).bat";
const APEX_BAT: &str = "general (ALT11 APEX).bat";
const APEX_PRESET_V2: &str = "Apex Legends.txt";
const APEX_LIST: &str = "list-apex.txt";
const APEX_LIST_EXTRA: &str = "list-apex-extra.txt";
const APEX_LIST_NODESYNC: &str = "list-apex-nodesync.txt";
const IPSET_EXCLUDE_APEX_EA: &str = "ipset-exclude-apex-ea.txt";

/// HTTP endpoints to probe EA / Apex connectivity (not in-game UDP).
pub const APEX_PROBE_TARGETS: &[(&str, &str)] = &[
    ("ea_web", "https://www.ea.com"),
    ("ea_accounts", "https://accounts.ea.com"),
    ("origin", "https://www.origin.com"),
    ("apex_site", "https://www.apexlegends.com"),
    ("ea_cdn", "https://eaassets-a.akamaihd.net"),
];

fn embedded_content(relative: &str) -> Option<&'static str> {
    match relative.replace('\\', "/").as_str() {
        "lists/list-apex.txt" => Some(include_str!(
            "../../../resources/zapret-extra/lists/list-apex.txt"
        )),
        "lists/list-apex-extra.txt" => Some(include_str!(
            "../../../resources/zapret-extra/lists/list-apex-extra.txt"
        )),
        "lists/ipset-exclude-apex-ea.txt" => Some(include_str!(
            "../../../resources/zapret-extra/lists/ipset-exclude-apex-ea.txt"
        )),
        "lists/list-apex-nodesync.txt" => Some(include_str!(
            "../../../resources/zapret-extra/lists/list-apex-nodesync.txt"
        )),
        "general (ALT11 APEX).bat" => Some(include_str!(
            "../../../resources/zapret-extra/general (ALT11 APEX).bat"
        )),
        "presets/Apex Legends.txt" => Some(include_str!(
            "../../../resources/zapret2-extra/presets/Apex Legends.txt"
        )),
        _ => None,
    }
}

fn copy_bundled(relative: &str, dest: &Path, find_file: fn(&str) -> Option<std::path::PathBuf>) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    if let Some(src) = find_file(relative) {
        std::fs::copy(&src, dest)
            .map_err(|e| format!("Не удалось скопировать {relative} из {}: {e}", src.display()))?;
        return Ok(());
    }

    if let Some(content) = embedded_content(relative) {
        std::fs::write(dest, content)
            .map_err(|e| format!("Не удалось записать {relative}: {e}"))?;
        return Ok(());
    }

    let hint = std::env::current_exe()
        .map(|p| format!(" (exe: {})", p.display()))
        .unwrap_or_default();
    Err(format!(
        "Встроенный файл не найден: {relative}{hint}. Переустановите fastpatch."
    ))
}

fn copy_apex_lists_to(lists: &Path) -> Result<(), String> {
    std::fs::create_dir_all(lists).map_err(|e| e.to_string())?;
    copy_bundled(
        &format!("lists/{APEX_LIST}"),
        &lists.join(APEX_LIST),
        find_zapret_extra_file,
    )?;
    copy_bundled(
        &format!("lists/{APEX_LIST_EXTRA}"),
        &lists.join(APEX_LIST_EXTRA),
        find_zapret_extra_file,
    )?;
    copy_bundled(
        &format!("lists/{IPSET_EXCLUDE_APEX_EA}"),
        &lists.join(IPSET_EXCLUDE_APEX_EA),
        find_zapret_extra_file,
    )?;
    copy_bundled(
        &format!("lists/{APEX_LIST_NODESYNC}"),
        &lists.join(APEX_LIST_NODESYNC),
        find_zapret_extra_file,
    )?;
    Ok(())
}

/// Copy fastpatch Apex assets into installed Zapret 1 folder.
pub fn ensure_apex_assets() -> Result<(), String> {
    let root = zapret_dir();
    if !root.join("bin").join("winws.exe").is_file() {
        return Ok(());
    }

    copy_apex_lists_to(&root.join("lists"))?;
    copy_bundled(APEX_BAT, &root.join(APEX_BAT), find_zapret_extra_file)?;

    let legacy = root.join(LEGACY_APEX_BAT);
    if legacy.is_file() {
        let _ = std::fs::remove_file(&legacy);
    }

    Ok(())
}

/// Copy fastpatch Apex preset and lists into installed Zapret 2 folder.
pub fn ensure_apex_assets_v2() -> Result<(), String> {
    let root = zapret2_dir();
    if !root.join("exe").join("winws2.exe").is_file() {
        return Ok(());
    }

    copy_apex_lists_to(&root.join("lists"))?;
    let presets = root.join("presets");
    std::fs::create_dir_all(&presets).map_err(|e| e.to_string())?;
    copy_bundled(
        &format!("presets/{APEX_PRESET_V2}"),
        &presets.join(APEX_PRESET_V2),
        find_zapret2_extra_file,
    )?;

    Ok(())
}

/// Установить списки и пресеты Apex для всех установленных ядер.
pub fn ensure_apex_assets_all() -> Result<(), String> {
    let _ = ensure_apex_assets();
    let _ = ensure_apex_assets_v2();
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
    pub v1_installed: bool,
    pub v2_installed: bool,
    pub list_installed: bool,
    pub bat_installed: bool,
    pub preset_v2_installed: bool,
    pub strategy_available: bool,
    pub game_filter: String,
    pub tips: Vec<ApexTip>,
}

pub fn apex_tips() -> Vec<ApexTip> {
    vec![
        ApexTip {
            title: "Пресет Apex Legends".into(),
            body: "Для Apex используйте только пресет Apex Legends, не Default v5. Игровой UDP и Respawn (stryder) без DPI. После «Установить пресет» переподключите стратегию.".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/6503".into()),
        },
        ApexTip {
            title: "Игровой фильтр (Zapret 1)".into(),
            body: "Для Apex на Zapret 1 держите Game Filter выключенным — иначе лобби и возврат после матча часто ломаются (#6198).".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/6198".into()),
        },
        ApexTip {
            title: "code:leaf / матч".into(),
            body: "Порты матчмейкинга и list-apex уже в пресете. При code:leaf обновите пресет Apex и переподключите стратегию.".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/9730".into()),
        },
        ApexTip {
            title: "После матча".into(),
            body: "Если не возвращает в лобби — «Установить пресет» и снова Apex Legends / ALT11 APEX.".into(),
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
    let v1_root = zapret_dir();
    let v2_root = zapret2_dir();
    let v1_installed = v1_root.join("bin").join("winws.exe").is_file();
    let v2_installed = v2_root.join("exe").join("winws2.exe").is_file();
    let list_v1 = v1_root.join("lists").join(APEX_LIST).is_file();
    let list_v2 = v2_root.join("lists").join(APEX_LIST).is_file();
    let bat_installed = v1_root.join(APEX_BAT).is_file();
    let preset_v2_installed = v2_root.join("presets").join(APEX_PRESET_V2).is_file();

    let backend = load().zapret_backend;
    let (zapret_installed, list_installed, strategy_available) = match backend {
        ZapretBackendPref::V2 => (v2_installed, list_v2, preset_v2_installed && v2_installed),
        ZapretBackendPref::V1 => (v1_installed, list_v1, bat_installed && v1_installed),
    };

    ApexStatus {
        zapret_installed,
        v1_installed,
        v2_installed,
        list_installed,
        bat_installed,
        preset_v2_installed,
        strategy_available,
        game_filter: read_game_filter_mode(&v1_root),
        tips: apex_tips(),
    }
}

/// Install Apex files for installed Zapret engines.
#[tauri::command]
pub fn setup_apex_preset() -> Result<String, String> {
    ensure_apex_assets_all()?;

    let backend = load().zapret_backend;
    if backend == ZapretBackendPref::V1 {
        let flag = zapret_dir().join("utils").join("game_filter.enabled");
        if flag.is_file() {
            std::fs::remove_file(&flag).map_err(|e| e.to_string())?;
        }
    }

    let msg = match backend {
        ZapretBackendPref::V2 => {
            "Пресет Apex Legends установлен в zapret2 (list-apex, ipset-exclude EA). Подключите из панели Apex или списка стратегий.".into()
        }
        ZapretBackendPref::V1 => {
            "Пресет Apex: ALT11 APEX (list-apex, ipset-exclude EA). Игровой фильтр выключен. Подключите стратегию из панели или списка.".into()
        }
    };
    Ok(msg)
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
