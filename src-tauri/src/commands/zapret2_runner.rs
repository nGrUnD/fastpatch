//! Запуск/остановка winws2.exe (Zapret 2).

use crate::commands::strategy_runner::{SpawnOptions, WINWS_BUSY_PREFIX, WINWS_FAST_WARMUP_MS};
use crate::paths::{winws2_path, zapret2_dir};
use crate::win_process::spawn_winws2;
use std::process::Command;
use std::thread;
use std::time::Duration;

const WINWS2_IMAGE: &str = "winws2.exe";

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn winws_busy_message(detail: &str) -> String {
    format!(
        "{WINWS_BUSY_PREFIX}winws2.exe уже запущен ({detail}). \
         Нажмите «Снять задачу» ниже и снова подключите нужную стратегию."
    )
}

pub fn stop_all_winws2() {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/IM", WINWS2_IMAGE, "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
}

pub fn stop_all_winws2_and_wait(max_wait_ms: u64) {
    #[cfg(windows)]
    {
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
}

pub fn find_winws2_pid() -> Option<u32> {
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {WINWS2_IMAGE}"), "/FO", "CSV", "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("INFO:") {
                continue;
            }
            let first = line.split(',').next()?.trim_matches('"');
            if first.eq_ignore_ascii_case(WINWS2_IMAGE) {
                let parts: Vec<_> = line.split(',').collect();
                if parts.len() >= 2 {
                    return parts[1].trim_matches('"').parse().ok();
                }
            }
        }
    }
    None
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

    crate::commands::zapret_backend::kill_all_processes_and_wait(opts.stop_wait_ms);

    #[cfg(windows)]
    {
        if !crate::win_process::is_elevated() {
            return Err(
                "Нет прав администратора. Запустите fastpatch от имени администратора.".into(),
            );
        }

        if find_winws2_pid().is_some() {
            return Err(winws_busy_message(
                "не удалось остановить предыдущий winws2",
            ));
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

        let at_arg = format!("@{}", active.display());
        if let Err(e) = spawn_winws2(&winws2_exe, &at_arg, &root) {
            if find_winws2_pid().is_some() {
                return Err(winws_busy_message(&e));
            }
            return Err(e);
        }

        let warmup = if opts.warmup_ms > 0 {
            opts.warmup_ms
        } else {
            WINWS_FAST_WARMUP_MS
        };
        thread::sleep(Duration::from_millis(warmup));

        find_winws2_pid().ok_or_else(|| {
            winws_busy_message(&format!(
                "после {} процесс не появился",
                preset_path.file_name().unwrap_or_default().to_string_lossy()
            ))
        })
    }

    #[cfg(not(windows))]
    {
        let _ = (preset_rel, opts, preset_path);
        Err("Zapret 2 поддерживается только на Windows".into())
    }
}
