//! Настройки fastpatch (последняя стратегия, автоподключение при автозапуске).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const PREFS_FILE: &str = "prefs.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZapretBackendPref {
    #[serde(rename = "v2")]
    V2,
    #[serde(rename = "v1")]
    V1,
}

impl Default for ZapretBackendPref {
    fn default() -> Self {
        Self::V2
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPrefs {
    #[serde(default)]
    pub last_strategy_id: Option<String>,
    /// Подключить `last_strategy_id` после автозапуска Windows.
    #[serde(default = "default_auto_connect")]
    pub auto_connect_on_autostart: bool,
    /// Ядро обхода: Zapret 2 (по умолчанию) или Zapret 1 (запасной).
    #[serde(default)]
    pub zapret_backend: ZapretBackendPref,
}

fn default_auto_connect() -> bool {
    true
}

impl Default for AppPrefs {
    fn default() -> Self {
        Self {
            last_strategy_id: None,
            auto_connect_on_autostart: true,
            zapret_backend: ZapretBackendPref::V2,
        }
    }
}

/// Всегда профиль пользователя (не systemprofile при UAC / планировщике).
fn user_prefs_path() -> Result<PathBuf, String> {
    let profile = std::env::var("USERPROFILE")
        .map_err(|_| "Не удалось определить USERPROFILE".to_string())?;
    Ok(PathBuf::from(profile)
        .join("AppData")
        .join("Roaming")
        .join("fastpatch")
        .join(PREFS_FILE))
}

fn legacy_prefs_path() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(|a| PathBuf::from(a).join("fastpatch").join(PREFS_FILE))
}

fn read_prefs_file(path: &PathBuf) -> Option<AppPrefs> {
    if !path.is_file() {
        return None;
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

pub fn load() -> AppPrefs {
    let mut prefs = user_prefs_path()
        .ok()
        .and_then(|p| read_prefs_file(&p))
        .unwrap_or_default();

    // Старые сборки могли писать в APPDATA elevated-процесса — подхватываем last_strategy_id.
    if prefs.last_strategy_id.is_none() {
        if let Some(legacy) = legacy_prefs_path().and_then(|p| read_prefs_file(&p)) {
            if legacy.last_strategy_id.is_some() {
                prefs.last_strategy_id = legacy.last_strategy_id;
                let _ = save(&prefs);
            }
        }
    }

    prefs
}

fn save(prefs: &AppPrefs) -> Result<(), String> {
    let path = user_prefs_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn set_last_strategy_id(id: &str) {
    let mut prefs = load();
    prefs.last_strategy_id = Some(id.to_string());
    let _ = save(&prefs);
}

pub fn set_auto_connect_on_autostart(enabled: bool) -> Result<(), String> {
    let mut prefs = load();
    prefs.auto_connect_on_autostart = enabled;
    save(&prefs)
}

pub fn set_zapret_backend(backend: ZapretBackendPref) -> Result<(), String> {
    let mut prefs = load();
    prefs.zapret_backend = backend;
    save(&prefs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_get_safe_defaults() {
        let prefs: AppPrefs = serde_json::from_str(r#"{"last_strategy_id":"default-v5"}"#)
            .expect("prefs json should parse");

        assert_eq!(prefs.last_strategy_id.as_deref(), Some("default-v5"));
        assert!(prefs.auto_connect_on_autostart);
        assert_eq!(prefs.zapret_backend, ZapretBackendPref::V2);
    }

    #[test]
    fn backend_pref_serializes_as_version_slug() {
        let prefs = AppPrefs {
            last_strategy_id: None,
            auto_connect_on_autostart: false,
            zapret_backend: ZapretBackendPref::V1,
        };
        let json = serde_json::to_string(&prefs).expect("prefs should serialize");
        assert!(json.contains(r#""zapret_backend":"v1""#));
    }
}
