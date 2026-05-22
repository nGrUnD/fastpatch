mod app_prefs;
mod commands;
mod launch;
mod paths;
#[cfg(windows)]
mod win_autostart;
#[cfg(windows)]
mod win_process;

use commands::hosts::{
    backup_hosts, delete_hosts_backup, list_hosts_backups, read_hosts, restore_hosts_backup,
    write_hosts, HostsState,
};
use commands::apex::{get_apex_status, setup_apex_preset, test_apex_connectivity};
use commands::autostart_connect::try_autostart_connect;
use commands::strategy::{
    add_custom_strategy, auto_detect_apex_strategy, auto_detect_strategy, get_active_strategy,
    cancel_strategy_scan, get_strategies, get_zapret_status, scan_all_strategies, start_strategy,
    stop_strategy, test_media_connectivity, test_strategy, ProcessState, ScanCancelState,
};
use commands::updater::{apply_update, check_for_updates, get_current_version, install_zapret};
use commands::zapret_config::{
    get_zapret_settings, set_auto_update_check, set_game_filter, set_ipset_mode,
    update_ipset_list, update_zapret_hosts_file,
};

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

#[tauri::command]
fn get_autostart_enabled() -> bool {
    #[cfg(windows)]
    {
        return crate::win_autostart::is_enabled();
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[tauri::command]
fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        if enabled {
            crate::win_autostart::enable(&exe)
        } else {
            crate::win_autostart::disable()
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        Err("Автозапуск доступен только в Windows".into())
    }
}

#[tauri::command]
fn get_app_prefs() -> app_prefs::AppPrefs {
    app_prefs::load()
}

#[tauri::command]
fn set_auto_connect_on_autostart(enabled: bool) -> Result<(), String> {
    app_prefs::set_auto_connect_on_autostart(enabled)
}

#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[derive(serde::Serialize)]
struct AppInfo {
    elevated: bool,
    /// Debug-сборка ожидает Vite на localhost:1420 (только `pnpm tauri dev`).
    is_dev_build: bool,
    /// Старт из задачи планировщика (`--from-autostart`).
    from_autostart: bool,
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        elevated: is_app_elevated(),
        is_dev_build: cfg!(debug_assertions),
        from_autostart: launch::autostart_session(),
    }
}

#[tauri::command]
fn is_app_elevated() -> bool {
    #[cfg(windows)]
    {
        return crate::win_process::is_elevated();
    }
    #[cfg(not(windows))]
    false
}

/// Перезапуск с UAC (release). В dev — подсказка запустить `pnpm tauri dev` из admin-терминала.
#[tauri::command]
fn relaunch_as_admin() -> Result<(), String> {
    #[cfg(windows)]
    {
        if cfg!(debug_assertions) {
            return Err(
                "Режим разработки: закройте это окно, откройте PowerShell от имени администратора,\n\
                 перейдите в папку fastpatch и выполните: pnpm tauri dev"
                    .into(),
            );
        }
        crate::win_process::relaunch_as_admin()
    }
    #[cfg(not(windows))]
    {
        Err("Права администратора требуются только на Windows".into())
    }
}

#[tauri::command]
fn hide_to_tray(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// Иконка из `src-tauri/icons/` (в dev `default_window_icon()` часто остаётся дефолтной Tauri).
fn load_app_icon() -> tauri::Result<Image<'static>> {
    let icons = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons");
    #[cfg(windows)]
    let path = icons.join("icon.ico");
    #[cfg(not(windows))]
    let path = icons.join("32x32.png");
    if path.is_file() {
        return Image::from_path(&path);
    }
    Image::from_path(icons.join("32x32.png"))
}

fn setup_tray<R: Runtime>(app: &tauri::App<R>, icon: Image<'static>) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "Открыть", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Скрыть в трей", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &hide_item, &separator, &quit_item])?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("fastpatch")
        .on_menu_event(|app, event: MenuEvent| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.set_focus();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(ProcessState(Arc::new(Mutex::new(None))))
        .manage(ScanCancelState(Arc::new(AtomicBool::new(false))))
        .manage(HostsState)
        .setup(|app| {
            let from_autostart = launch::autostart_session();

            #[cfg(windows)]
            crate::win_autostart::remove_legacy_run_entries();

            // Автозапуск: задача планировщика уже с правами админа — не показываем UAC повторно.
            #[cfg(all(windows, not(debug_assertions)))]
            {
                if from_autostart {
                    if !crate::win_process::is_elevated() {
                        eprintln!(
                            "[fastpatch] Автозапуск без прав администратора. \
                             Отключите и снова включите автозапуск в настройках (один раз подтвердите UAC)."
                        );
                    }
                } else {
                    crate::win_process::ensure_app_elevated()?;
                }
            }
            #[cfg(all(windows, debug_assertions))]
            if !crate::win_process::is_elevated() {
                eprintln!(
                    "\n[fastpatch] Для winws запустите терминал от имени администратора,\n\
                     затем: pnpm tauri dev\n"
                );
            }

            let icon = load_app_icon()?;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(icon.clone());
            }
            setup_tray(app, icon)?;

            if launch::minimized() || from_autostart {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // strategy
            get_zapret_status,
            get_strategies,
            get_active_strategy,
            start_strategy,
            stop_strategy,
            test_strategy,
            test_media_connectivity,
            auto_detect_strategy,
            scan_all_strategies,
            cancel_strategy_scan,
            add_custom_strategy,
            auto_detect_apex_strategy,
            get_apex_status,
            setup_apex_preset,
            test_apex_connectivity,
            // hosts
            read_hosts,
            write_hosts,
            backup_hosts,
            list_hosts_backups,
            restore_hosts_backup,
            delete_hosts_backup,
            // updater
            check_for_updates,
            install_zapret,
            apply_update,
            get_current_version,
            get_zapret_settings,
            set_game_filter,
            set_ipset_mode,
            set_auto_update_check,
            update_ipset_list,
            update_zapret_hosts_file,
            // window / autostart
            get_autostart_enabled,
            set_autostart_enabled,
            get_app_prefs,
            set_auto_connect_on_autostart,
            try_autostart_connect,
            show_window,
            hide_to_tray,
            is_app_elevated,
            get_app_info,
            relaunch_as_admin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
