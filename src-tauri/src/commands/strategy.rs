use super::apex::APEX_PROBE_TARGETS;
use super::probe::{
    http_probe, merge_probe_targets, run_probes_with_limit,
    score_probe_hits_with_timeout, AUTODETECT_PROBE_TARGETS, AUTODETECT_TIMEOUT_MS,
    AUTODETECT_TIMEOUT_SECS, PROBE_TIMEOUT_MS, PROBE_TIMEOUT_SECS,
};
use super::strategy_loader;
use super::strategy_runner::{
    find_winws_pid, run_zapret_preamble, spawn_strategy_bat_with_options, stop_all_winws_and_wait,
    SpawnOptions, WINWS_BUSY_PREFIX, WINWS_FAST_WARMUP_MS,
};
use crate::paths::zapret_dir;
use super::Strategy;
use crate::paths::ensure_winws;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::State;

/// Флаг отмены автоскана (см. `cancel_strategy_scan`).
pub struct ScanCancelState(pub Arc<AtomicBool>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategyScanEntry {
    pub strategy_id: String,
    pub results: Vec<TestResult>,
    pub works: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestResult {
    pub strategy_id: String,
    pub target: String,
    pub success: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveStrategy {
    pub id: String,
    pub name: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZapretStatus {
    pub installed: bool,
    pub winws_path: String,
    pub zapret_dir: String,
}

#[derive(Clone)]
pub struct ProcessState(pub Arc<Mutex<Option<(String, String, u32)>>>);

const AUTO_DETECT_WARMUP_MS: u64 = 1500;
const TEST_WARMUP_MS: u64 = 2500;
/// Быстрый прогон при автоскане (меньше целей, короче ожидание).
const SCAN_WARMUP_MS: u64 = 1000;

fn probe_targets_for(strategy: &Strategy) -> Vec<(&'static str, &'static str)> {
    if strategy.tags.iter().any(|t| t == "apex") {
        return merge_probe_targets(true, APEX_PROBE_TARGETS);
    }
    let discord = strategy.tags.iter().any(|t| t == "discord")
        || strategy.source_bat.to_lowercase().contains("general");
    merge_probe_targets(discord, &[])
}

#[derive(Clone)]
struct SessionSnapshot {
    id: String,
    name: String,
}

fn stop_child(state: &ProcessState) {
    let mut guard = state.0.lock().unwrap();
    guard.take();
    stop_all_winws_and_wait(4000);
}

fn pause_session(state: &ProcessState) -> Option<SessionSnapshot> {
    let snap = state.0.lock().unwrap().as_ref().map(|(id, name, _)| SessionSnapshot {
        id: id.clone(),
        name: name.clone(),
    });
    stop_child(state);
    snap
}

fn resume_session(
    state: &ProcessState,
    snap: Option<SessionSnapshot>,
    strategies: &[Strategy],
    opts: SpawnOptions,
) -> bool {
    let Some(s) = snap else {
        return false;
    };
    let Some(prev) = strategies.iter().find(|x| x.id == s.id) else {
        return false;
    };
    match spawn_strategy_with_options(prev, opts) {
        Ok(pid) => {
            let mut guard = state.0.lock().unwrap();
            *guard = Some((s.id, s.name, pid));
            true
        }
        Err(e) => {
            eprintln!("[fastpatch] не удалось восстановить {}: {e}", prev.name);
            false
        }
    }
}

fn spawn_strategy(strategy: &Strategy) -> Result<u32, String> {
    spawn_strategy_with_options(strategy, SpawnOptions::default())
}

fn spawn_strategy_with_options(strategy: &Strategy, opts: SpawnOptions) -> Result<u32, String> {
    ensure_winws()?;
    spawn_strategy_bat_with_options(&strategy.source_bat, opts)
}

fn strategies_for_autodetect(strategies: &[Strategy]) -> Vec<&Strategy> {
    let mut list: Vec<&Strategy> = strategies.iter().collect();
    list.sort_by(|a, b| {
        strategy_loader::autodetect_priority(&a.id, &a.source_bat)
            .cmp(&strategy_loader::autodetect_priority(&b.id, &b.source_bat))
    });
    list
}

#[tauri::command]
pub fn get_zapret_status() -> ZapretStatus {
    let dir = crate::paths::zapret_dir();
    let winws = crate::paths::winws_path();
    ZapretStatus {
        installed: winws.is_file(),
        winws_path: winws.display().to_string(),
        zapret_dir: dir.display().to_string(),
    }
}

#[tauri::command]
pub fn get_strategies() -> Result<Vec<Strategy>, String> {
    strategy_loader::load_strategies()
}

pub fn get_active_strategy_inner(state: &ProcessState) -> Option<ActiveStrategy> {
    let mut guard = state.0.lock().unwrap();
    let Some((id, name, _)) = guard.as_ref() else {
        return None;
    };
    let live_pid = find_winws_pid();
    if live_pid.is_none() {
        guard.take();
        return None;
    }
    Some(ActiveStrategy {
        id: id.clone(),
        name: name.clone(),
        pid: live_pid,
    })
}

#[tauri::command]
pub fn get_active_strategy(state: State<ProcessState>) -> Option<ActiveStrategy> {
    get_active_strategy_inner(&state)
}

pub fn start_strategy_inner(id: &str, state: &ProcessState) -> Result<(), String> {
    let strategies = get_strategies()?;
    let strategy = strategies
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Стратегия '{id}' не найдена"))?;

    stop_child(state);

    let pid = spawn_strategy(strategy)?;
    let name = strategy.name.clone();
    crate::app_prefs::set_last_strategy_id(id);
    let mut guard = state.0.lock().unwrap();
    *guard = Some((id.to_string(), name, pid));
    Ok(())
}

#[tauri::command]
pub fn start_strategy(id: String, state: State<ProcessState>) -> Result<(), String> {
    start_strategy_inner(&id, &state)
}

#[tauri::command]
pub fn stop_strategy(state: State<ProcessState>) -> Result<(), String> {
    stop_child(&state);
    Ok(())
}

/// Принудительно завершить все winws.exe (если мешают подключению стратегии).
#[tauri::command]
pub fn kill_winws(state: State<ProcessState>) -> Result<(), String> {
    stop_child(&state);
    stop_all_winws_and_wait(8000);
    if let Some(pid) = find_winws_pid() {
        return Err(format!(
            "{WINWS_BUSY_PREFIX}winws.exe всё ещё работает (PID {pid}). \
             Закройте zapret вручную или перезапустите fastpatch от имени администратора."
        ));
    }
    Ok(())
}

async fn probe_strategy_targets(
    id: &str,
    targets: &[(&str, &str)],
    timeout_secs: u64,
    max_latency_ms: u64,
) -> Vec<TestResult> {
    let mut handles = Vec::new();
    for (tag, url) in targets {
        let sid = id.to_string();
        let tag = tag.to_string();
        let url = url.to_string();
        handles.push(tokio::spawn(async move {
            let (reachable, latency_ms, error) = http_probe(&url, timeout_secs).await;
            TestResult {
                strategy_id: sid,
                target: tag,
                success: super::probe::probe_is_ok_with_limit(reachable, latency_ms, max_latency_ms),
                latency_ms,
                error,
            }
        }));
    }
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(r) => results.push(r),
            Err(e) => results.push(TestResult {
                strategy_id: id.to_string(),
                target: "error".into(),
                success: false,
                latency_ms: None,
                error: Some(e.to_string()),
            }),
        }
    }
    results
}

fn scan_results_work(results: &[TestResult]) -> bool {
    results.iter().any(|r| r.success)
}

#[tauri::command]
pub async fn test_strategy(id: String, state: State<'_, ProcessState>) -> Result<Vec<TestResult>, String> {
    ensure_winws()?;
    let strategies = get_strategies()?;
    let strategy = strategies
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Стратегия '{id}' не найдена"))?;

    let snap = state.0.lock().unwrap().as_ref().map(|(sid, name, _)| SessionSnapshot {
        id: sid.clone(),
        name: name.clone(),
    });
    let testing_active = snap.as_ref().map(|s| s.id == id).unwrap_or(false);

    if testing_active && find_winws_pid().is_some() {
        tokio::time::sleep(Duration::from_millis(1500)).await;
    } else {
        stop_child(&state);
        let pid = spawn_strategy(strategy)?;
        {
            let mut guard = state.0.lock().unwrap();
            *guard = Some((id.clone(), strategy.name.clone(), pid));
        }
        tokio::time::sleep(Duration::from_millis(TEST_WARMUP_MS)).await;
    }

    let targets = probe_targets_for(strategy);
    let targets_refs: Vec<(&str, &str)> = targets.iter().map(|(a, b)| (*a, *b)).collect();
    let results =
        probe_strategy_targets(&id, &targets_refs, PROBE_TIMEOUT_SECS, PROBE_TIMEOUT_MS).await;

    if !testing_active || snap.as_ref().map(|s| s.id.as_str()) != Some(id.as_str()) {
        stop_child(&state);
        if snap.as_ref().map(|s| s.id.as_str()) != Some(id.as_str()) {
            resume_session(&state, snap, &strategies, SpawnOptions::default());
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn auto_detect_strategy(state: State<'_, ProcessState>) -> Result<Option<String>, String> {
    ensure_winws()?;
    let strategies = get_strategies()?;
    let ordered = strategies_for_autodetect(&strategies);

    let paused = pause_session(&state);
    let mut first = true;

    for strategy in ordered {
        let opts = SpawnOptions {
            run_preamble: first,
            warmup_ms: if first { AUTO_DETECT_WARMUP_MS } else { WINWS_FAST_WARMUP_MS },
            stop_wait_ms: if first { 5000 } else { 3500 },
        };
        first = false;
        let pid = match spawn_strategy_with_options(strategy, opts) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[fastpatch] skip {}: {e}", strategy.source_bat);
                stop_all_winws_and_wait(3500);
                continue;
            }
        };
        {
            let mut guard = state.0.lock().unwrap();
            *guard = Some((strategy.id.clone(), strategy.name.clone(), pid));
        }

        tokio::time::sleep(Duration::from_millis(AUTO_DETECT_WARMUP_MS)).await;

        let hits = run_probes_with_limit(
            AUTODETECT_PROBE_TARGETS,
            AUTODETECT_TIMEOUT_SECS,
            AUTODETECT_TIMEOUT_MS,
        )
        .await;
        let score = score_probe_hits_with_timeout(&hits, AUTODETECT_TIMEOUT_MS);

        eprintln!(
            "[fastpatch] autodetect {}: discord {}/3 req={} youtube={} avg={}ms",
            strategy.name,
            score.discord_ok,
            score.discord_required_ok,
            score.youtube_ok,
            score.avg_latency_ms
        );

        if score.passes_autodetect() {
            crate::app_prefs::set_last_strategy_id(&strategy.id);
            return Ok(Some(strategy.id.clone()));
        }

        stop_child(&state);
    }

    if paused.is_some() {
        resume_session(&state, paused, &strategies, SpawnOptions::default());
    }

    Ok(None)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanAllResult {
    pub entries: Vec<StrategyScanEntry>,
    pub restored_previous: bool,
    pub previous_name: Option<String>,
    pub cancelled: bool,
}

fn scan_is_cancelled(cancel: &ScanCancelState) -> bool {
    cancel.0.load(Ordering::Relaxed)
}

async fn sleep_scan_ms(ms: u64, cancel: &ScanCancelState) -> bool {
    let mut left = ms;
    while left > 0 {
        if scan_is_cancelled(cancel) {
            return false;
        }
        let chunk = left.min(100);
        tokio::time::sleep(Duration::from_millis(chunk)).await;
        left = left.saturating_sub(chunk);
    }
    !scan_is_cancelled(cancel)
}

#[tauri::command]
pub fn cancel_strategy_scan(cancel: State<'_, ScanCancelState>) {
    cancel.0.store(true, Ordering::Relaxed);
}

/// Прогон всех стратегий с быстрыми пробами (результаты для UI, без автозапуска).
#[tauri::command]
pub async fn scan_all_strategies(
    state: State<'_, ProcessState>,
    cancel: State<'_, ScanCancelState>,
) -> Result<ScanAllResult, String> {
    ensure_winws()?;
    cancel.0.store(false, Ordering::Relaxed);
    let strategies = get_strategies()?;
    let paused = pause_session(&state);
    let previous_name = paused.as_ref().map(|s| s.name.clone());

    // После «Подключить» winws ещё держит WinDivert — даём ОС время и один раз грузим списки.
    stop_all_winws_and_wait(6000);
    run_zapret_preamble(&zapret_dir(), true);
    if !sleep_scan_ms(600, &cancel).await {
        let restored_previous = resume_session(&state, paused, &strategies, SpawnOptions::default());
        return Ok(ScanAllResult {
            entries: vec![],
            restored_previous,
            previous_name,
            cancelled: true,
        });
    }

    let scan_targets: Vec<(&str, &str)> = AUTODETECT_PROBE_TARGETS.to_vec();
    let mut entries = Vec::with_capacity(strategies.len());
    let mut cancelled = false;
    let mut scan_index = 0usize;

    for strategy in &strategies {
        if scan_is_cancelled(&cancel) {
            cancelled = true;
            stop_child(&state);
            break;
        }
        let opts = SpawnOptions::for_scan_iteration(scan_index == 0);
        scan_index += 1;
        let pid = match spawn_strategy_with_options(strategy, opts) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[fastpatch] scan skip {}: {e}", strategy.source_bat);
                stop_all_winws_and_wait(4000);
                let err = TestResult {
                    strategy_id: strategy.id.clone(),
                    target: "spawn".into(),
                    success: false,
                    latency_ms: None,
                    error: Some(e),
                };
                entries.push(StrategyScanEntry {
                    strategy_id: strategy.id.clone(),
                    results: vec![err],
                    works: false,
                });
                continue;
            }
        };
        {
            let mut guard = state.0.lock().unwrap();
            *guard = Some((strategy.id.clone(), strategy.name.clone(), pid));
        }

        if !sleep_scan_ms(SCAN_WARMUP_MS, &cancel).await {
            cancelled = true;
            stop_child(&state);
            break;
        }

        let results = probe_strategy_targets(
            &strategy.id,
            &scan_targets,
            AUTODETECT_TIMEOUT_SECS,
            AUTODETECT_TIMEOUT_MS,
        )
        .await;
        let works = scan_results_work(&results);
        entries.push(StrategyScanEntry {
            strategy_id: strategy.id.clone(),
            results,
            works,
        });

        stop_child(&state);

        if scan_is_cancelled(&cancel) {
            cancelled = true;
            break;
        }
    }

    let restored_previous = resume_session(
        &state,
        paused,
        &strategies,
        SpawnOptions::default(),
    );

    Ok(ScanAllResult {
        entries,
        restored_previous,
        previous_name,
        cancelled,
    })
}

#[tauri::command]
pub fn add_custom_strategy(display_name: String, content: String) -> Result<Strategy, String> {
    strategy_loader::save_custom_strategy(&display_name, &content)
}

/// Проверка связи по тегам активной стратегии (Discord, YouTube, Cloudflare, EA/Apex…).
#[tauri::command]
pub async fn test_media_connectivity(
    state: State<'_, ProcessState>,
) -> Result<Vec<TestResult>, String> {
    ensure_winws()?;
    let active_id = {
        let guard = state.0.lock().unwrap();
        guard.as_ref().map(|(id, _, _)| id.clone())
    };
    let id = active_id.ok_or("Сначала запустите стратегию")?;

    if find_winws_pid().is_none() {
        return Err("winws.exe не запущен. Перезапустите стратегию.".into());
    }

    let strategies = get_strategies()?;
    let strategy = strategies
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Стратегия '{id}' не найдена"))?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    let targets = probe_targets_for(strategy);
    let targets_refs: Vec<(&str, &str)> = targets.iter().map(|(a, b)| (*a, *b)).collect();
    let results =
        probe_strategy_targets(&id, &targets_refs, PROBE_TIMEOUT_SECS, PROBE_TIMEOUT_MS).await;
    Ok(results)
}

/// Auto-detect among strategies tagged for Apex (faster than full scan).
#[tauri::command]
pub async fn auto_detect_apex_strategy(state: State<'_, ProcessState>) -> Result<Option<String>, String> {
    ensure_winws()?;
    let strategies = get_strategies()?;
    let apex_only: Vec<_> = strategies
        .iter()
        .filter(|s| s.tags.iter().any(|t| t == "apex"))
        .collect();

    if apex_only.is_empty() {
        return Err(
            "Нет стратегий с тегом Apex. Нажмите «Установить пресет Apex» в настройках или на главной.".into(),
        );
    }

    let paused = pause_session(&state);
    let mut first = true;

    for strategy in apex_only {
        let opts = SpawnOptions {
            run_preamble: first,
            warmup_ms: if first { AUTO_DETECT_WARMUP_MS } else { WINWS_FAST_WARMUP_MS },
            stop_wait_ms: if first { 5000 } else { 3500 },
        };
        first = false;
        let pid = match spawn_strategy_with_options(strategy, opts) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[fastpatch] apex skip {}: {e}", strategy.source_bat);
                stop_all_winws_and_wait(3500);
                continue;
            }
        };
        {
            let mut guard = state.0.lock().unwrap();
            *guard = Some((strategy.id.clone(), strategy.name.clone(), pid));
        }

        tokio::time::sleep(Duration::from_millis(AUTO_DETECT_WARMUP_MS)).await;

        let hits = run_probes_with_limit(
            AUTODETECT_PROBE_TARGETS,
            AUTODETECT_TIMEOUT_SECS,
            AUTODETECT_TIMEOUT_MS,
        )
        .await;
        let score = score_probe_hits_with_timeout(&hits, AUTODETECT_TIMEOUT_MS);
        if score.passes_autodetect() {
            crate::app_prefs::set_last_strategy_id(&strategy.id);
            return Ok(Some(strategy.id.clone()));
        }

        stop_child(&state);
    }

    if paused.is_some() {
        resume_session(&state, paused, &strategies, SpawnOptions::default());
    }

    Ok(None)
}
