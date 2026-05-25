//! Автоподключение последней стратегии после автозапуска Windows.

use super::strategy::{
    get_active_strategy_inner, start_strategy_inner, ActiveStrategy, ProcessState,
};
use super::zapret_backend::ZapretBackend;
use crate::app_prefs;
use crate::launch;
use crate::paths::ensure_current_engine;
use std::thread;
use std::time::Duration;
use tauri::State;

const ATTEMPTS: u32 = 6;
const RETRY_MS: u64 = 2500;

fn saved_strategy_exists(id: &str) -> Result<bool, String> {
    let strategies = match super::zapret_backend::current() {
        ZapretBackend::V2 => super::preset_loader::load_presets()?,
        ZapretBackend::V1 => super::strategy_loader::load_strategies()?,
    };
    Ok(strategies.iter().any(|s| s.id == id))
}

/// Сессия запущена из автозапуска (планировщик или `--minimized` без ручного старта).
pub fn is_autostart_session() -> bool {
    launch::from_autostart() || launch::minimized()
}

#[tauri::command]
pub fn try_autostart_connect(state: State<ProcessState>) -> Result<Option<ActiveStrategy>, String> {
    if !is_autostart_session() {
        return Ok(None);
    }

    let prefs = app_prefs::load();
    if !prefs.auto_connect_on_autostart {
        return Ok(None);
    }

    let id = prefs
        .last_strategy_id
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            "Нет сохранённой стратегии. Подключитесь вручную один раз — затем автоподключение запомнит выбор.".to_string()
        })?;

    if !saved_strategy_exists(&id)? {
        return Err(format!(
            "Сохранённая стратегия «{id}» недоступна для текущего ядра. Подключите нужную стратегию вручную один раз."
        ));
    }

    let mut last_err = String::from("неизвестная ошибка");

    for attempt in 1..=ATTEMPTS {
        if attempt > 1 {
            thread::sleep(Duration::from_millis(RETRY_MS));
        } else {
            thread::sleep(Duration::from_millis(800));
        }

        if let Err(e) = ensure_current_engine() {
            last_err = e;
            eprintln!("[fastpatch] автоподключение попытка {attempt}/{ATTEMPTS}: {last_err}");
            continue;
        }

        match start_strategy_inner(&id, &state) {
            Ok(()) => {
                if let Some(active) = get_active_strategy_inner(&state) {
                    eprintln!(
                        "[fastpatch] автоподключение: «{}» (попытка {attempt})",
                        active.name
                    );
                    return Ok(Some(active));
                }
                last_err = "winws не появился после запуска стратегии".into();
            }
            Err(e) => {
                last_err = e;
                eprintln!("[fastpatch] автоподключение попытка {attempt}/{ATTEMPTS}: {last_err}");
            }
        }
    }

    Err(format!(
        "Не удалось подключить «{id}» после автозапуска: {last_err}"
    ))
}
