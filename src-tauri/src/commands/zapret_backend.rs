//! Выбор ядра Zapret 1 (winws) / Zapret 2 (winws2).

use crate::app_prefs::{load, set_zapret_backend, ZapretBackendPref};
use crate::commands::strategy_runner::{stop_all_winws, stop_all_winws_and_wait};
use crate::commands::zapret2_runner::{stop_all_winws2, stop_all_winws2_and_wait};

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
