//! Windows: single UAC at app start; winws inherits admin token.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn ps_escape(s: &str) -> String {
    s.replace('\'', "''")
}

pub fn is_elevated() -> bool {
    let script = "([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)";
    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Relaunch fastpatch elevated (one UAC), then exit current process.
pub fn relaunch_as_admin() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_s = ps_escape(&exe.display().to_string());
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg_list: String = args
        .iter()
        .map(|a| format!("'{}'", ps_escape(a)))
        .collect::<Vec<_>>()
        .join(",");

    let script = if arg_list.is_empty() {
        format!("Start-Process -FilePath '{exe_s}' -Verb RunAs")
    } else {
        format!("Start-Process -FilePath '{exe_s}' -ArgumentList {arg_list} -Verb RunAs")
    };

    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Не удалось перезапустить fastpatch с правами администратора: {e}"))?;

    std::process::exit(0);
}

/// One UAC for the whole app; winws launches without extra prompts (release build only).
#[cfg_attr(debug_assertions, allow(dead_code))]
pub fn ensure_app_elevated() -> Result<(), String> {
    if is_elevated() {
        return Ok(());
    }
    relaunch_as_admin()
}

pub fn spawn_winws(exe: &Path, args: &[String], cwd: &Path) -> Result<u32, String> {
    if !is_elevated() {
        return Err(
            "Нет прав администратора. В fastpatch нажмите «Запустить от имени администратора»."
                .to_string(),
        );
    }

    let mut cmd = Command::new(exe);
    cmd.current_dir(cwd).creation_flags(CREATE_NO_WINDOW);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.spawn()
        .map(|c| c.id())
        .map_err(|e| format!("Не удалось запустить winws.exe: {e}"))
}

