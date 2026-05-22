//! Windows: автозапуск через Планировщик заданий (высокий приоритет, без UAC при входе).

use crate::win_process::{is_elevated, ps_escape};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub const TASK_NAME: &str = "fastpatch";

fn run_powershell(script: &str) -> Result<String, String> {
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("PowerShell: {e}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        let msg = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        return Err(if msg.is_empty() {
            format!("PowerShell завершился с кодом {:?}", out.status.code())
        } else {
            msg
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Убрать старый автозапуск из реестра Run (tauri-plugin-autostart / Discord-раньше нас не мешает).
pub fn remove_legacy_run_entries() {
    let names = ["fastpatch", "com.fastpatch.app", "Fastpatch"];
    for name in names {
        let n = ps_escape(name);
        let _ = run_powershell(&format!(
            r#"$p = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'; if (Get-ItemProperty -Path $p -Name '{n}' -ErrorAction SilentlyContinue) {{ Remove-ItemProperty -Path $p -Name '{n}' -Force }}"#
        ));
    }
}

pub fn is_enabled() -> bool {
    let tn = ps_escape(TASK_NAME);
    run_powershell(&format!(
        "if (Get-ScheduledTask -TaskName '{tn}' -ErrorAction SilentlyContinue) {{ 'yes' }} else {{ 'no' }}"
    ))
    .map(|s| s.eq_ignore_ascii_case("yes"))
    .unwrap_or(false)
}

/// Создать задачу: вход в систему, наивысшие права (один UAC при включении), приоритет 0, раньше типичного автозапуска Discord.
pub fn enable(exe: &Path) -> Result<(), String> {
    if !is_elevated() {
        return Err(
            "Для включения автозапуска нужны права администратора.\n\
             Запустите fastpatch вручную, подтвердите UAC один раз, затем снова включите автозапуск в настройках."
                .into(),
        );
    }

    remove_legacy_run_entries();

    let exe_s = ps_escape(&exe.display().to_string());
    let tn = ps_escape(TASK_NAME);

    let script = format!(
        r#"
$exe = '{exe_s}'
$action = New-ScheduledTaskAction -Execute $exe -Argument '--minimized --from-autostart'
$trigger = New-ScheduledTaskTrigger -AtLogOn
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew -Priority 0
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Highest
Register-ScheduledTask -TaskName '{tn}' -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Force | Out-Null
"#
    );

    run_powershell(&script)?;
    Ok(())
}

pub fn disable() -> Result<(), String> {
    remove_legacy_run_entries();
    let tn = ps_escape(TASK_NAME);
    let _ = run_powershell(&format!(
        "Unregister-ScheduledTask -TaskName '{tn}' -Confirm:$false -ErrorAction SilentlyContinue"
    ));
    Ok(())
}
