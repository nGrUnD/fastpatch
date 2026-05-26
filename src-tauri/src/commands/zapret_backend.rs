//! Выбор ядра Zapret 1 (winws) / Zapret 2 (winws2).

use crate::app_prefs::{load, set_zapret_backend, ZapretBackendPref};
use crate::commands::engine_process::{find_winws2_pid, find_winws_pid};
use crate::commands::strategy_runner::{
    stop_all_winws, stop_all_winws_and_wait, winws_busy_message,
};
use crate::commands::zapret2_runner::{
    stop_all_winws2, stop_all_winws2_and_wait, winws_busy_message as winws2_busy_message,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZapretBackend {
    V1,
    V2,
}

impl From<ZapretBackendPref> for ZapretBackend {
    fn from(p: ZapretBackendPref) -> Self {
        match p {
            ZapretBackendPref::V1 => Self::V1,
            ZapretBackendPref::V2 => Self::V2,
        }
    }
}

impl From<ZapretBackend> for ZapretBackendPref {
    fn from(b: ZapretBackend) -> Self {
        match b {
            ZapretBackend::V1 => Self::V1,
            ZapretBackend::V2 => Self::V2,
        }
    }
}

pub fn current() -> ZapretBackend {
    load().zapret_backend.into()
}

pub fn set_current(backend: ZapretBackend) -> Result<(), String> {
    kill_all_processes_and_wait(6000);
    set_zapret_backend(backend.into())
}

/// Остановить оба движка (v1 и v2 не должны работать одновременно).
pub fn kill_all_processes() {
    stop_all_winws();
    stop_all_winws2();
}

pub fn kill_all_processes_and_wait(max_wait_ms: u64) {
    kill_all_processes();
    stop_all_winws_and_wait(max_wait_ms);
    stop_all_winws2_and_wait(max_wait_ms);
}

/// Перед запуском: завершить старые процессы и убедиться, что winws/winws2 не занят.
pub fn preflight_engine_spawn() -> Result<(), String> {
    kill_all_processes_and_wait(6000);
    match current() {
        ZapretBackend::V2 => {
            if let Some(pid) = find_winws2_pid() {
                return Err(winws2_busy_message(&format!(
                    "не удалось остановить процесс {pid} — zapret, возможно, запущен вне fastpatch"
                )));
            }
        }
        ZapretBackend::V1 => {
            if let Some(pid) = find_winws_pid() {
                return Err(winws_busy_message(&format!(
                    "не удалось остановить процесс {pid} — zapret, возможно, запущен вне fastpatch"
                )));
            }
        }
    }
    Ok(())
}
