//! Аргументы командной строки при старте.

pub fn args() -> Vec<String> {
    std::env::args().collect()
}

/// Запуск из задачи планировщика (автозапуск с повышенными правами, без UAC).
pub fn from_autostart() -> bool {
    args().iter().any(|a| a == "--from-autostart")
}

pub fn minimized() -> bool {
    args().iter().any(|a| a == "--minimized")
}

/// Автозапуск: планировщик передаёт `--from-autostart` и/или `--minimized`.
pub fn autostart_session() -> bool {
    from_autostart() || minimized()
}
