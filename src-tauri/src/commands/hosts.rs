use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

pub struct HostsState;

pub(crate) fn hosts_path() -> PathBuf {
    PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
}

fn backup_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("hosts_backups")
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HostsBackup {
    pub filename: String,
    pub created_at: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub fn read_hosts() -> Result<String, String> {
    std::fs::read_to_string(hosts_path())
        .map_err(|e| format!("Не удалось прочитать hosts-файл (требуются права администратора): {e}"))
}

#[tauri::command]
pub fn write_hosts(content: String, _state: State<HostsState>) -> Result<(), String> {
    // Create backup before writing
    let _ = backup_hosts_internal();

    std::fs::write(hosts_path(), content)
        .map_err(|e| format!("Не удалось записать hosts-файл (требуются права администратора): {e}"))
}

#[tauri::command]
pub fn backup_hosts() -> Result<String, String> {
    backup_hosts_internal()
}

pub(crate) fn backup_hosts_internal() -> Result<String, String> {
    let dir = backup_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Не удалось создать папку backup: {e}"))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("hosts.bak.{timestamp}");
    let backup_path = dir.join(&backup_name);

    std::fs::copy(hosts_path(), &backup_path)
        .map_err(|e| format!("Не удалось создать резервную копию: {e}"))?;

    Ok(backup_name)
}

#[tauri::command]
pub fn list_hosts_backups() -> Result<Vec<HostsBackup>, String> {
    let dir = backup_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut backups = Vec::new();
    let entries = std::fs::read_dir(&dir)
        .map_err(|e| format!("Не удалось прочитать папку backup: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "bak").unwrap_or(false)
            || path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("hosts.bak"))
                .unwrap_or(false)
        {
            let meta = std::fs::metadata(&path).ok();
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let created = meta
                .and_then(|m| m.modified().ok())
                .and_then(|t| {
                    let dt: chrono::DateTime<Local> = t.into();
                    Some(dt.format("%d.%m.%Y %H:%M:%S").to_string())
                })
                .unwrap_or_else(|| "неизвестно".to_string());

            backups.push(HostsBackup {
                filename: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                created_at: created,
                size_bytes: size,
            });
        }
    }

    backups.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(backups)
}

#[tauri::command]
pub fn restore_hosts_backup(filename: String) -> Result<(), String> {
    let backup_path = backup_dir().join(&filename);
    if !backup_path.exists() {
        return Err(format!("Резервная копия '{filename}' не найдена"));
    }

    // Backup current hosts before restoring
    let _ = backup_hosts_internal();

    std::fs::copy(&backup_path, hosts_path())
        .map_err(|e| format!("Не удалось восстановить hosts-файл: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_hosts_backup(filename: String) -> Result<(), String> {
    let backup_path = backup_dir().join(&filename);
    std::fs::remove_file(&backup_path)
        .map_err(|e| format!("Не удалось удалить резервную копию: {e}"))
}
