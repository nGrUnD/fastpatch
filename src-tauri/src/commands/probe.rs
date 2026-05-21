//! HTTP connectivity checks (aligned with zapret utils/targets.txt).

use std::time::Duration;

pub const PROBE_TIMEOUT_SECS: u64 = 5;
pub const PROBE_TIMEOUT_MS: u64 = PROBE_TIMEOUT_SECS * 1000;

/// Быстрее полного теста на карточке стратегии.
pub const AUTODETECT_TIMEOUT_SECS: u64 = 3;
pub const AUTODETECT_TIMEOUT_MS: u64 = AUTODETECT_TIMEOUT_SECS * 1000;

/// Только Discord + YouTube (без cloudflare и лишних хостов).
pub const AUTODETECT_PROBE_TARGETS: &[(&str, &str)] = &[
    ("discord_gw", "https://gateway.discord.gg"),
    ("discord_upd", "https://updates.discord.com"),
    ("discord", "https://discord.com"),
    ("youtube", "https://www.youtube.com"),
    ("youtube_gen", "https://www.youtube.com/generate_204"),
];

/// Ответ считается валидным только если TLS/HTTP успели до таймаута (latency < max_ms).
pub fn probe_is_ok_with_limit(reachable: bool, latency_ms: Option<u64>, max_ms: u64) -> bool {
    reachable && latency_ms.is_some_and(|ms| ms < max_ms)
}

/// Как в test zapret.ps1: успех = реальный HTTP-ответ до таймаута (не 5s timeout).
pub async fn http_probe(url: &str, timeout_secs: u64) -> (bool, Option<u64>, Option<String>) {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .redirect(reqwest::redirect::Policy::limited(5))
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(e) => return (false, None, Some(e.to_string())),
    };

    let start = std::time::Instant::now();
    match client.get(url).send().await {
        Ok(resp) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            let code = resp.status().as_u16();
            let http_ok = (200..400).contains(&code) || code == 204;
            (
                true,
                Some(latency_ms),
                if http_ok {
                    None
                } else {
                    Some(format!("HTTP {code}"))
                },
            )
        }
        Err(e) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            let msg = if e.is_timeout() {
                format!("таймаут (>{timeout_secs} с)")
            } else {
                e.to_string()
            };
            (false, Some(latency_ms), Some(msg))
        }
    }
}

/// Цели из zapret utils/targets.txt (Discord — полный набор для клиента).
pub const DISCORD_PROBE_TARGETS: &[(&str, &str)] = &[
    ("discord", "https://discord.com"),
    ("discord_gw", "https://gateway.discord.gg"),
    ("discord_cdn", "https://cdn.discordapp.com"),
    ("discord_upd", "https://updates.discord.com"),
];

/// Обязательны для автоподбора (без них клиент Discord часто не стартует).
pub const DISCORD_REQUIRED_TARGETS: &[&str] = &["discord_gw", "discord_upd"];

pub const GENERAL_PROBE_TARGETS: &[(&str, &str)] = &[
    ("youtube", "https://www.youtube.com"),
    ("youtube_gen", "https://www.youtube.com/generate_204"),
    ("cloudflare", "https://www.cloudflare.com"),
];

pub fn merge_probe_targets(
    discord: bool,
    extra: &[(&'static str, &'static str)],
) -> Vec<(&'static str, &'static str)> {
    let mut out: Vec<(&'static str, &'static str)> = Vec::new();
    if discord {
        out.extend_from_slice(DISCORD_PROBE_TARGETS);
    }
    out.extend_from_slice(GENERAL_PROBE_TARGETS);
    out.extend_from_slice(extra);
    out
}

#[derive(Debug, Clone)]
pub struct ProbeHit {
    pub tag: String,
    pub ok: bool,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct StrategyProbeScore {
    pub discord_required_ok: bool,
    pub discord_ok: usize,
    pub youtube_ok: bool,
    pub avg_latency_ms: u64,
}

impl StrategyProbeScore {
    /// Discord (gw + updates) и YouTube — достаточно для автоподбора.
    pub fn passes_autodetect(&self) -> bool {
        self.discord_required_ok && self.discord_ok >= 2 && self.youtube_ok
    }
}

pub async fn run_probes_with_limit(
    targets: &[(&str, &str)],
    timeout_secs: u64,
    max_latency_ms: u64,
) -> Vec<ProbeHit> {
    let mut handles = Vec::new();
    for (tag, url) in targets {
        let tag = tag.to_string();
        let url = url.to_string();
        handles.push(tokio::spawn(async move {
            let (reachable, latency_ms, _) = http_probe(&url, timeout_secs).await;
            let ok = probe_is_ok_with_limit(reachable, latency_ms, max_latency_ms);
            ProbeHit {
                tag,
                ok,
                latency_ms,
            }
        }));
    }
    let mut out = Vec::new();
    for h in handles {
        if let Ok(hit) = h.await {
            out.push(hit);
        }
    }
    out
}

pub fn score_probe_hits_with_timeout(hits: &[ProbeHit], timeout_ms: u64) -> StrategyProbeScore {
    let mut discord_ok = 0usize;
    let mut lat_sum = 0u64;
    let mut lat_n = 0u64;
    let mut required_ok = true;
    let mut youtube_ok = false;
    let mut youtube_gen_ok = false;

    for h in hits {
        if h.ok {
            if let Some(ms) = h.latency_ms {
                lat_sum += ms;
                lat_n += 1;
            }
            if h.tag.starts_with("discord") {
                discord_ok += 1;
            }
            if h.tag == "youtube" {
                youtube_ok = true;
            }
            if h.tag == "youtube_gen" {
                youtube_gen_ok = true;
            }
        }
        if DISCORD_REQUIRED_TARGETS.contains(&h.tag.as_str()) && !h.ok {
            required_ok = false;
        }
    }

    StrategyProbeScore {
        discord_required_ok: required_ok,
        discord_ok,
        youtube_ok: youtube_ok && youtube_gen_ok,
        avg_latency_ms: if lat_n > 0 { lat_sum / lat_n } else { timeout_ms },
    }
}
