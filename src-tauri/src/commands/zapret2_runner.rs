//! Запуск/остановка winws2.exe (Zapret 2).

use crate::commands::apex;
use crate::commands::engine_process::{find_winws2_pids, is_pid_running, kill_image};
use crate::commands::strategy_runner::{SpawnOptions, WINWS_BUSY_PREFIX};
use std::process::Command;
use crate::paths::{winws2_path, zapret2_dir};
use crate::startup_log;
use crate::win_process::spawn_winws2;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Относительный путь, как в zapret-console.bat (`@"utils\preset-active.txt"`).
const ACTIVE_PRESET_ARG: &str = "@utils\\preset-active.txt";
const WINWS2_WARMUP_MS: u64 = 2000;

pub const WINWS2_IMAGE: &str = "winws2.exe";

pub fn winws_busy_message(detail: &str) -> String {
    format!(
        "{WINWS_BUSY_PREFIX}winws2.exe уже запущен ({detail}). \
         Нажмите «Снять задачу» ниже и снова подключите нужную стратегию."
    )
}

pub fn winws_start_failed_message(detail: &str) -> String {
    format!(
        "winws2.exe не запустился ({detail}). \
         Попробуйте «Снять задачу», пресет Default v5, или zapret2\\service.bat → диагностика WinDivert."
    )
}

fn winws2_log_path(root: &Path) -> PathBuf {
    root.join("utils").join("fastpatch-winws2.log")
}

fn log_path_hint(root: &Path) -> String {
    let path = winws2_log_path(root);
    if path.is_file() {
        format!("\n\nЛог: {}", path.display())
    } else {
        String::new()
    }
}

fn tail_log(path: &Path, max_lines: usize) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let lines: Vec<_> = text.lines().rev().take(max_lines).collect();
    if lines.is_empty() {
        return None;
    }
    Some(lines.into_iter().rev().collect::<Vec<_>>().join("\n"))
}

/// Снять зависшие службы WinDivert/Monkey (как пункт «r» в zapret-console.bat).
#[cfg(windows)]
fn cleanup_stale_windivert_services() {
    for service in ["WinDivert", "WinDivert14", "Monkey", "Monkey14"] {
        let ok = Command::new("sc")
            .args(["query", service])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !ok {
            continue;
        }
        let _ = Command::new("net")
            .args(["stop", service])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        let _ = Command::new("sc")
            .args(["delete", service])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
}

#[cfg(not(windows))]
fn cleanup_stale_windivert_services() {}

fn child_exit_detail(child: &mut Child, preset_label: &str, root: &Path) -> String {
    let code = child
        .try_wait()
        .ok()
        .flatten()
        .and_then(|s| s.code())
        .unwrap_or(-1);
    let log_path = winws2_log_path(root);
    let tail = tail_log(&log_path, 8)
        .map(|t| format!("\n\nПоследние строки лога:\n{t}"))
        .unwrap_or_default();
    let hint = log_path_hint(root);
    startup_log::log(&format!(
        "winws2 exit code={code} preset={preset_label} log={}",
        log_path.display()
    ));
    format!(
        "после {preset_label} завершился с кодом {code}{tail}{hint}"
    )
}

pub fn stop_all_winws2() {
    kill_image(WINWS2_IMAGE);
}

pub fn stop_all_winws2_and_wait(max_wait_ms: u64) {
    stop_all_winws2();
    let step = 200u64;
    let mut waited = 0u64;
    while find_winws2_pid().is_some() && waited < max_wait_ms {
        thread::sleep(Duration::from_millis(step));
        waited += step;
        if waited % 2000 == 0 {
            stop_all_winws2();
        }
    }
}

pub fn find_winws2_pid() -> Option<u32> {
    find_winws2_pids().into_iter().next()
}

/// `preset_rel` — путь относительно корня zapret2, например `presets/Default v5.txt`.
pub fn spawn_preset_with_options(preset_rel: &str, opts: SpawnOptions) -> Result<u32, String> {
    let root = zapret2_dir();
    let preset_path = root.join(preset_rel.replace('/', "\\"));
    if !preset_path.is_file() {
        return Err(format!(
            "Пресет не найден: {}. Переустановите Zapret 2.",
            preset_path.display()
        ));
    }

    let winws2_exe = winws2_path();
    if !winws2_exe.is_file() {
        return Err(format!("winws2.exe не найден: {}", winws2_exe.display()));
    }

    if preset_rel.contains("Apex") {
        apex::ensure_apex_assets_v2().map_err(|e| {
            format!("Не удалось подготовить файлы Apex: {e}. Настройки → Игры → «Установить пресет».")
        })?;
    }

    crate::commands::zapret_backend::kill_all_processes_and_wait(opts.stop_wait_ms);
    cleanup_stale_windivert_services();
    thread::sleep(Duration::from_millis(500));

    #[cfg(windows)]
    {
        if !crate::win_process::is_elevated() {
            return Err(
                "Нет прав администратора. Запустите fastpatch от имени администратора.".into(),
            );
        }

        if let Some(pid) = find_winws2_pid() {
            return Err(winws_busy_message(&format!(
                "не удалось остановить процесс {pid} — zapret, возможно, запущен вне fastpatch"
            )));
        }

        let active = root.join("utils").join("preset-active.txt");
        if let Some(parent) = active.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::copy(&preset_path, &active)
            .map_err(|e| format!("Не удалось активировать пресет: {e}"))?;

        let state = root.join("utils").join("current_preset.txt");
        if let Some(name) = preset_path.file_stem().and_then(|s| s.to_str()) {
            let _ = std::fs::write(&state, name);
        }

        let preset_label = preset_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| preset_rel.to_string());

        let log_path = winws2_log_path(&root);
        let mut child = match spawn_winws2(
            &winws2_exe,
            ACTIVE_PRESET_ARG,
            &root,
            Some(&log_path),
        ) {
            Ok(child) => child,
            Err(e) => {
                if find_winws2_pid().is_some() {
                    return Err(winws_busy_message(&e));
                }
                return Err(winws_start_failed_message(&format!(
                    "{e}{}",
                    log_path_hint(&root)
                )));
            }
        };

        let pid = child.id();
        let warmup = if opts.warmup_ms > 0 {
            opts.warmup_ms.max(WINWS2_WARMUP_MS)
        } else {
            WINWS2_WARMUP_MS
        };
        thread::sleep(Duration::from_millis(warmup));

        if let Ok(Some(_)) = child.try_wait() {
            return Err(winws_start_failed_message(&child_exit_detail(
                &mut child,
                &preset_label,
                &root,
            )));
        }

        if is_pid_running(pid) {
            return Ok(pid);
        }

        if find_winws2_pid().is_some() {
            return Err(winws_busy_message(
                "обнаружен другой экземпляр winws2 после запуска",
            ));
        }

        Err(winws_start_failed_message(&child_exit_detail(
            &mut child,
            &preset_label,
            &root,
        )))
    }

    #[cfg(not(windows))]
    {
        let _ = (preset_rel, opts, preset_path);
        Err("Zapret 2 поддерживается только на Windows".into())
    }
}
