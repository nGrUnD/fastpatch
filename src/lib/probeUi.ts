import type { ApexProbeResult, TestResult } from "@/stores/appStore";

/** Автоскан / автоподбор в Rust (AUTODETECT_TIMEOUT_MS). */
export const PROBE_SLOW_MS = 3000;

/** Полный тест карточки стратегии в Rust (PROBE_TIMEOUT_MS). */
export const PROBE_TIMEOUT_MS = 5000;

type ProbeLike = Pick<TestResult, "success" | "latency_ms" | "error"> | ApexProbeResult;

export const PROBE_LABELS: Record<string, string> = {
  discord: "Discord",
  discord_gw: "Discord GW",
  discord_cdn: "Discord CDN",
  discord_upd: "Discord upd",
  youtube: "YouTube",
  youtube_gen: "YouTube 204",
  cloudflare: "Cloudflare CDN",
  ea_web: "EA",
  ea_accounts: "EA accounts",
  origin: "Origin",
  apex_site: "Apex",
  ea_cdn: "EA CDN",
};

const APEX_PROBE_IDS = ["ea_web", "ea_accounts", "origin", "apex_site", "ea_cdn"] as const;

export function probeLabel(target: string): string {
  return PROBE_LABELS[target] ?? target;
}

/** Нет ответа: провал на бэкенде или задержка ≥ 3 с (как в автоскане). */
export function probeHasConnectivity(r: ProbeLike): boolean {
  if (!r.success) return false;
  const ms = r.latency_ms ?? 0;
  return ms > 0 && ms < PROBE_SLOW_MS;
}

export function probeIsSlow(r: ProbeLike): boolean {
  return r.latency_ms != null && r.latency_ms >= PROBE_SLOW_MS;
}

export function probeIsTimeout(r: ProbeLike): boolean {
  return r.latency_ms != null && r.latency_ms >= PROBE_TIMEOUT_MS;
}

export function probeFailed(r: ProbeLike): boolean {
  return !probeHasConnectivity(r);
}

export function probeBadgeClass(r: ProbeLike): string {
  if (probeFailed(r)) {
    return "bg-red-500/15 text-red-300 border-red-500/30";
  }
  if (r.error) {
    return "bg-amber-500/15 text-amber-200 border-amber-500/30";
  }
  return "bg-emerald-500/15 text-emerald-300 border-emerald-500/30";
}

export function probeLatencyLabel(r: ProbeLike): string {
  if (r.latency_ms == null) return "";
  if (r.latency_ms >= PROBE_SLOW_MS) return "нет ответа";
  return `${r.latency_ms}ms`;
}

export function scoreMediaProbe(results: TestResult[]): {
  discordOk: boolean;
  youtubeOk: boolean;
} {
  const { byTag } = scoreConnectivityProbe(results, ["discord", "youtube"]);
  return {
    discordOk: byTag.discord ?? true,
    youtubeOk: byTag.youtube ?? true,
  };
}

/** Сводка по тегам по результатам проверки связи. */
export function scoreConnectivityProbe(
  results: TestResult[],
  strategyTags: string[]
): {
  byTag: Partial<Record<string, boolean>>;
  warnings: string[];
  notes: string[];
} {
  const ok = (target: string) =>
    results.some((r) => r.target === target && probeHasConnectivity(r));

  const hasTarget = (pred: (t: string) => boolean) => results.some((r) => pred(r.target));

  const byTag: Partial<Record<string, boolean>> = {};
  const warnings: string[] = [];
  const notes: string[] = [];

  const tagLabels: Record<string, string> = {
    discord: "Discord",
    youtube: "YouTube",
    cloudflare: "Cloudflare CDN",
    apex: "Apex / EA",
    games: "Игры (UDP)",
  };

  if (
    strategyTags.includes("discord") ||
    hasTarget((t) => t.startsWith("discord"))
  ) {
    byTag.discord =
      ok("discord_gw") &&
      ok("discord_upd") &&
      (ok("discord") || ok("discord_cdn"));
    if (!byTag.discord) warnings.push(`${tagLabels.discord} недоступен.`);
  }

  if (
    strategyTags.includes("youtube") ||
    hasTarget((t) => t.startsWith("youtube"))
  ) {
    byTag.youtube = ok("youtube") && ok("youtube_gen");
    if (!byTag.youtube) warnings.push(`${tagLabels.youtube} недоступен.`);
  }

  if (strategyTags.includes("cloudflare") || hasTarget((t) => t === "cloudflare")) {
    byTag.cloudflare = ok("cloudflare");
    if (!byTag.cloudflare) warnings.push(`${tagLabels.cloudflare} недоступен.`);
  }

  if (strategyTags.includes("apex") || hasTarget((t) => APEX_PROBE_IDS.includes(t as (typeof APEX_PROBE_IDS)[number]))) {
    byTag.apex = APEX_PROBE_IDS.every((t) => ok(t));
    if (!byTag.apex) warnings.push(`${tagLabels.apex} — часть серверов недоступна.`);
  }

  if (strategyTags.includes("games")) {
    byTag.games = true;
    notes.push(`${tagLabels.games}: HTTP-проверка не выполняется, тестируйте в игре.`);
  }

  return { byTag, warnings, notes };
}
