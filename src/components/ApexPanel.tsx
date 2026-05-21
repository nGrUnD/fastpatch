import { openUrl } from "@tauri-apps/plugin-opener";
import {
  CheckCircle,
  ExternalLink,
  Gamepad2,
  Loader2,
  Play,
  Stethoscope,
  Wrench,
} from "lucide-react";
import { useState } from "react";
import { probeBadgeClass, probeHasConnectivity } from "@/lib/probeUi";
import { cn } from "@/lib/utils";
import { type ApexProbeResult, useAppStore } from "@/stores/appStore";

export function ApexPanel() {
  const {
    strategies,
    activeStrategy,
    zapretInstalled,
    isLoading,
    apexStatus,
    setupApexPreset,
    testApexConnectivity,
    autoDetectApex,
    startStrategy,
    loadApexStatus,
  } = useAppStore();

  const [probeResults, setProbeResults] = useState<ApexProbeResult[] | null>(null);
  const [isProbing, setIsProbing] = useState(false);
  const [localMsg, setLocalMsg] = useState<string | null>(null);

  const apexStrategy = strategies.find(
    (s) => s.id === "apex" || s.name.toUpperCase() === "APEX"
  );

  const handleSetup = async () => {
    setLocalMsg(null);
    const msg = await setupApexPreset();
    if (msg) setLocalMsg(msg);
    await loadApexStatus();
  };

  const handleTest = async () => {
    setIsProbing(true);
    setProbeResults(null);
    try {
      const results = await testApexConnectivity();
      setProbeResults(results);
    } finally {
      setIsProbing(false);
    }
  };

  const handleStartApex = async () => {
    if (!apexStrategy) {
      await handleSetup();
      return;
    }
    await startStrategy(apexStrategy.id);
  };

  const handleAutoApex = async () => {
    setLocalMsg(null);
    const id = await autoDetectApex();
    if (id) {
      await useAppStore.getState().loadActiveStrategy();
      const name = strategies.find((s) => s.id === id)?.name ?? id;
      setLocalMsg(`Подобрана стратегия: ${name}`);
    }
  };

  const okCount = probeResults?.filter((r) => probeHasConnectivity(r)).length ?? 0;
  const probeTotal = probeResults?.length ?? 0;

  return (
    <div className="rounded-xl border border-orange-500/30 bg-orange-500/5 p-5 space-y-4">
      <div className="flex items-start gap-3">
        <div className="p-2 rounded-lg bg-orange-500/20">
          <Gamepad2 className="w-5 h-5 text-orange-300" />
        </div>
        <div className="min-w-0 flex-1">
          <h2 className="text-sm font-semibold text-white">Apex Legends</h2>
          <p className="text-xs text-zinc-400 mt-0.5">
            Пресет из Issues zapret (#6503): отдельный list-apex.txt, порты матчмейкинга,
            ipset-exclude. HTTP-проверка EA — не замена входа в лобби.
          </p>
        </div>
      </div>

      {apexStatus && !apexStatus.strategy_available && zapretInstalled && (
        <p className="text-xs text-amber-300/90">
          Пресет не установлен — нажмите «Установить пресет».
        </p>
      )}

      {localMsg && (
        <p className="text-xs text-emerald-300/90">{localMsg}</p>
      )}

      <div className="flex flex-wrap gap-2">
        <button
          onClick={handleSetup}
          disabled={isLoading || !zapretInstalled}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 border border-zinc-600 text-xs text-white hover:bg-zinc-700 disabled:opacity-40"
        >
          <Wrench className="w-3.5 h-3.5" />
          Установить пресет
        </button>
        <button
          onClick={handleStartApex}
          disabled={isLoading || !zapretInstalled}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-orange-600/80 text-xs text-white hover:bg-orange-600 disabled:opacity-40"
        >
          <Play className="w-3.5 h-3.5" />
          Запустить APEX
        </button>
        <button
          onClick={handleTest}
          disabled={isProbing || !zapretInstalled}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 border border-zinc-600 text-xs text-zinc-200 hover:bg-zinc-700 disabled:opacity-40"
        >
          {isProbing ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <Stethoscope className="w-3.5 h-3.5" />
          )}
          Проверить EA
        </button>
        <button
          onClick={handleAutoApex}
          disabled={isLoading || !zapretInstalled}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 border border-zinc-600 text-xs text-zinc-200 hover:bg-zinc-700 disabled:opacity-40"
        >
          Подобрать (Apex)
        </button>
      </div>

      {activeStrategy?.name.toUpperCase() === "APEX" && (
        <p className="text-xs text-emerald-300 flex items-center gap-1">
          <CheckCircle className="w-3.5 h-3.5" />
          Активна стратегия APEX
        </p>
      )}

      {probeResults && (
        <div className="text-xs space-y-1">
          <p className="text-zinc-400">
            Доступность серверов EA (HTTP): {okCount}/{probeTotal}
          </p>
          <ul className="grid grid-cols-1 sm:grid-cols-2 gap-1">
            {probeResults.map((r) => (
              <li
                key={r.target}
                title={r.error ?? undefined}
                className={cn("px-2 py-1 rounded border", probeBadgeClass(r))}
              >
                {r.target}: {probeHasConnectivity(r) ? "доступен" : "нет ответа"}
                {r.latency_ms != null && ` (${r.latency_ms} ms)`}
                {r.error && (
                  <span className="opacity-70"> — {r.error}</span>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {apexStatus?.tips && apexStatus.tips.length > 0 && (
        <details className="text-xs">
          <summary className="text-zinc-400 cursor-pointer hover:text-white">
            Советы из Issues ({apexStatus.tips.length})
          </summary>
          <ul className="mt-2 space-y-2 max-h-40 overflow-y-auto">
            {apexStatus.tips.map((tip) => (
              <li key={tip.title} className="text-zinc-400">
                <span className="text-zinc-200 font-medium">{tip.title}: </span>
                {tip.body}
                {tip.issue_url && (
                  <button
                    type="button"
                    onClick={() => openUrl(tip.issue_url!)}
                    className="ml-1 inline-flex items-center text-orange-300/80 hover:text-orange-200"
                  >
                    <ExternalLink className="w-3 h-3" />
                  </button>
                )}
              </li>
            ))}
          </ul>
        </details>
      )}
    </div>
  );
}
