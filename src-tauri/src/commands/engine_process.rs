//! Общие проверки PID winws / winws2 (Windows tasklist).

#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
fn pids_for_image(image: &str) -> Vec<u32> {
    let output = match Command::new("tasklist")
        .args([
            "/FI",
            &format!("IMAGENAME eq {image}"),
            "/FO",
            "CSV",
            "/NH",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let mut pids = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("INFO:") {
            continue;
        }
        let parts: Vec<_> = line.split(',').collect();
        if parts.len() < 2 {
            continue;
        }
        let name = parts[0].trim().trim_matches('"');
        if !name.eq_ignore_ascii_case(image) {
            continue;
        }
        if let Ok(pid) = parts[1].trim().trim_matches('"').parse::<u32>() {
            if pid > 0 {
                pids.push(pid);
            }
        }
    }
    pids
}

#[cfg(not(windows))]
fn pids_for_image(_image: &str) -> Vec<u32> {
    vec![]
}

pub fn find_winws_pids() -> Vec<u32> {
    pids_for_image("winws.exe")
}

pub fn find_winws2_pids() -> Vec<u32> {
    pids_for_image("winws2.exe")
}

pub fn find_winws_pid() -> Option<u32> {
    find_winws_pids().into_iter().next()
}

pub fn find_winws2_pid() -> Option<u32> {
    find_winws2_pids().into_iter().next()
}

/// Процесс с данным PID ещё в списке задач.
pub fn is_pid_running(pid: u32) -> bool {
    #[cfg(windows)]
    {
        if pid == 0 {
            return false;
        }
        let output = match Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            Ok(o) => o,
            Err(_) => return false,
        };
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("INFO:") {
                continue;
            }
            let parts: Vec<_> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(found) = parts[1].trim().trim_matches('"').parse::<u32>() {
                    if found == pid {
                        return true;
                    }
                }
            }
        }
        false
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        false
    }
}

#[cfg(windows)]
pub fn kill_pids(pids: &[u32]) {
    for &pid in pids {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F", "/T"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
}

#[cfg(not(windows))]
pub fn kill_pids(_pids: &[u32]) {}

pub fn kill_image(image: &str) {
    #[cfg(windows)]
    {
        kill_pids(&pids_for_image(image));
        let _ = Command::new("taskkill")
            .args(["/IM", image, "/F", "/T"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = image;
    }
}
