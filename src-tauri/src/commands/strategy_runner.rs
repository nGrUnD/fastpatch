//! Launch/stop winws: преамбула service.bat скрыто, winws без окна консоли.

use crate::commands::strategy_loader;
use crate::paths::zapret_dir;
use crate::win_process::spawn_winws;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub const WINWS_WARMUP_MS: u64 = 4000;
pub const WINWS_FAST_WARMUP_MS: u64 = 800;

/// Префикс для UI: жёлтое предупреждение + кнопка «Снять задачу».
pub const WINWS_BUSY_PREFIX: &str = "WINWS_BUSY:";

pub fn winws_busy_message(detail: &str) -> String {
    format!(
        "{WINWS_BUSY_PREFIX}winws.exe уже запущен ({detail}). \
         Нажмите «Снять задачу» ниже и снова подключите нужную стратегию."
    )
}

#[derive(Clone, Copy)]
pub struct SpawnOptions {
    pub run_preamble: bool,
    pub warmup_ms: u64,
    /// Сколько ждать завершения предыдущего winws.exe перед запуском.
    pub stop_wait_ms: u64,
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self {
            run_preamble: true,
            warmup_ms: WINWS_WARMUP_MS,
            stop_wait_ms: 4000,
        }
    }
}

impl SpawnOptions {
    /// Быстрый цикл автоскана: без повторной преамбулы, короче ожидание stop/warmup.
    pub fn for_scan_iteration(long_warmup: bool) -> Self {
        Self {
            run_preamble: false,
            warmup_ms: if long_warmup { 2200 } else { 1200 },
            stop_wait_ms: 3500,
        }
    }
}

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn hidden_cmd(root: &Path, args: &[&str]) -> Result<(), String> {
    #[cfg(windows)]
    {
        let status = Command::new("cmd")
            .arg("/C")
            .args(args)
            .current_dir(root)
            .env("NO_UPDATE_CHECK", "1")
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| format!("cmd: {e}"))?;
        if !status.success() {
            return Err(format!("cmd завершился с кодом {:?}", status.code()));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (root, args);
        Err("Только Windows".into())
    }
}

/// То же, что в начале general*.bat, без `start` (не открывает консоль).
pub fn run_zapret_preamble(root: &Path, minimal: bool) {
    let hooks: &[&str] = if minimal {
        &[
            "call service.bat load_game_filter",
            "call service.bat load_user_lists",
        ]
    } else {
        &[
            "call service.bat status_zapret",
            "call service.bat load_game_filter",
            "call service.bat load_user_lists",
        ]
    };
    for hook in hooks {
        if let Err(e) = hidden_cmd(root, &[hook]) {
            eprintln!("[fastpatch] {hook}: {e}");
        }
    }
}

/// Stop every winws.exe instance (as in test zapret.ps1).
pub fn stop_all_winws() {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/IM", "winws.exe", "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
}

/// winws — один экземпляр; ждём завершения перед следующим запуском.
pub fn stop_all_winws_and_wait(max_wait_ms: u64) {
    #[cfg(windows)]
    {
        stop_all_winws();
        let step = 100u64;
        let mut waited = 0u64;
        while find_winws_pid().is_some() && waited < max_wait_ms {
            thread::sleep(Duration::from_millis(step));
            waited += step;
            if waited % 500 == 0 {
                stop_all_winws();
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = max_wait_ms;
    }
}

pub fn find_winws_pid() -> Option<u32> {
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq winws.exe", "/FO", "CSV", "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.contains("Image Name") {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let pid_str = parts[1].trim().trim_matches('"');
                if let Ok(pid) = pid_str.parse::<u32>() {
                    return Some(pid);
                }
            }
        }
    }
    None
}

/// Запуск стратегии: service.bat скрыто + winws.exe без консоли (не через `start` в .bat).
pub fn spawn_strategy_bat_with_options(source_bat: &str, opts: SpawnOptions) -> Result<u32, String> {
    let root = zapret_dir();
    let bat_path = root.join(source_bat);
    if !bat_path.is_file() {
        return Err(format!(
            "Файл стратегии не найден: {}. Переустановите zapret.",
            bat_path.display()
        ));
    }

    let winws_exe = root.join("bin").join("winws.exe");
    if !winws_exe.is_file() {
        return Err(format!("winws.exe не найден: {}", winws_exe.display()));
    }

    stop_all_winws_and_wait(opts.stop_wait_ms);

    #[cfg(windows)]
    {
        if !crate::win_process::is_elevated() {
            return Err(
                "Нет прав администратора. Запустите fastpatch от имени администратора.".into(),
            );
        }

        if let Some(pid) = find_winws_pid() {
            return Err(winws_busy_message(&format!(
                "не удалось остановить процесс {pid} — возможно, zapret запущен вне fastpatch"
            )));
        }

        if opts.run_preamble {
            run_zapret_preamble(&root, false);
        }

        let argv = strategy_loader::winws_argv_from_bat(&root, source_bat)?;
        if argv.is_empty() {
            return Err(format!("В {source_bat} не найдены аргументы winws.exe"));
        }

        let bin_dir = root.join("bin");
        let spawn_err = spawn_winws(&winws_exe, &argv, &bin_dir).err();
        if let Some(e) = spawn_err {
            if find_winws_pid().is_some() {
                return Err(winws_busy_message(&e));
            }
            return Err(e);
        }

        thread::sleep(Duration::from_millis(opts.warmup_ms));

        let pid = find_winws_pid().ok_or_else(|| {
            winws_busy_message(&format!(
                "после {source_bat} процесс не появился — WinDivert может быть занят"
            ))
        })?;

        Ok(pid)
    }

    #[cfg(not(windows))]
    {
        let _ = (source_bat, opts);
        Err("Запуск стратегий поддерживается только на Windows".into())
    }
}
