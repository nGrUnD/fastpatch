//! Apex Legends helpers (based on zapret-discord-youtube issues #6503, #6198, #9730).

use super::probe::http_probe;
use crate::paths::{find_data_file, zapret_dir};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const APEX_BAT: &str = "general (APEX).bat";
const APEX_LIST: &str = "list-apex.txt";

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

fn bundled_list_path() -> PathBuf {
    extra_resource_root().join("lists").join(APEX_LIST)
}

fn bundled_bat_path() -> PathBuf {
    extra_resource_root().join(APEX_BAT)
}

/// Copy fastpatch Apex preset into installed zapret folder.
pub fn ensure_apex_assets() -> Result<(), String> {
    let root = zapret_dir();
    if !root.join("bin").join("winws.exe").is_file() {
        return Ok(());
    }

    let lists = root.join("lists");
    std::fs::create_dir_all(&lists).map_err(|e| e.to_string())?;

    let list_src = find_data_file(&format!("resources/zapret-extra/lists/{APEX_LIST}"))
        .or_else(|| {
            let p = bundled_list_path();
            if p.is_file() {
                Some(p)
            } else {
                None
            }
        })
        .ok_or_else(|| format!("Встроенный {APEX_LIST} не найден"))?;

    let list_dst = lists.join(APEX_LIST);
    std::fs::copy(&list_src, &list_dst)
        .map_err(|e| format!("Не удалось скопировать {APEX_LIST}: {e}"))?;

    let bat_src = find_data_file(&format!("resources/zapret-extra/{APEX_BAT}"))
        .or_else(|| {
            let p = bundled_bat_path();
            if p.is_file() {
                Some(p)
            } else {
                None
            }
        })
        .ok_or_else(|| format!("Встроенный {APEX_BAT} не найден"))?;

    let bat_dst = root.join(APEX_BAT);
    std::fs::copy(&bat_src, &bat_dst)
        .map_err(|e| format!("Не удалось скопировать {APEX_BAT}: {e}"))?;

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
            title: "Игровой фильтр".into(),
            body: "Если Apex не заходит в лобби при обходе — выключите Game Filter в настройках и перезапустите стратегию (README zapret, issues #6198, #8724).".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/6198".into()),
        },
        ApexTip {
            title: "IPSet".into(),
            body: "Если лобби всё ещё ломается — попробуйте режим IPSet «Отключён» (empty). Для пресета APEX нужен загруженный ipset-all — обновите список IP в настройках.".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/6503".into()),
        },
        ApexTip {
            title: "Порты UDP 10000–10100".into(),
            body: "В пресете APEX добавлены игровые порты матчмейкинга; без них возможны code:leaf и зависание на загрузке матча.".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/6503".into()),
        },
        ApexTip {
            title: "Стратегия APEX".into(),
            body: "Отдельный .bat с list-apex.txt + ipset-exclude (решение из #6503). Discord/YouTube частично сохраняются.".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/6503".into()),
        },
        ApexTip {
            title: "code:leaf / пинг".into(),
            body: "Часто помогает ALT11 вместо FAKE TLS; иногда нужны свои IP в ipset-all. Проверка в fastpatch — только HTTP до EA, не замена игрового теста.".into(),
            issue_url: Some("https://github.com/Flowseal/zapret-discord-youtube/issues/9730".into()),
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
        "Пресет Apex установлен: list-apex.txt и general (APEX).bat. Игровой фильтр выключен — запустите стратегию APEX и проверьте игру.".into(),
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
