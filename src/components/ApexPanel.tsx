import {
  AlertCircle,
  CheckCircle,
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
    zapretBackend,
    loading,
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

  const isV2 = zapretBackend === "v2";
  const apexStrategy = strategies.find((s) => s.tags.includes("apex"));
  const apexStrategyLabel = apexStrategy?.name ?? (isV2 ? "Apex Legends" : "ALT11 APEX");

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
  const presetInstalled = isV2
    ? apexStatus?.preset_v2_installed
    : apexStatus?.bat_installed;
  const strategyFound = Boolean(apexStrategy);
  const apexActive =
    (activeStrategy &&
      strategies.find((s) => s.id === activeStrategy.id)?.tags.includes("apex")) ||
    activeStrategy?.name.toUpperCase().includes("APEX");

  return (
    <div className="rounded-xl border border-orange-500/30 bg-orange-500/5 p-5 space-y-4">
      <div className="flex items-start gap-3">
        <div className="p-2 rounded-lg bg-orange-500/20">
          <Gamepad2 className="w-5 h-5 text-orange-300" />
        </div>
        <div className="min-w-0 flex-1">
          <h2 className="text-sm font-semibold text-white">Apex Legends</h2>
          <p className="text-xs text-zinc-400 mt-0.5">
            Отдельный игровой профиль. Default v5 для Apex не используйте: он ломает вход и
            лобби.
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

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 text-xs">
        <div className="rounded-lg border border-zinc-700 bg-zinc-900/60 px-3 py-2">
          <p className="text-zinc-500">Пресет</p>
          <p className={presetInstalled ? "text-emerald-300" : "text-amber-300"}>
            {presetInstalled ? "Установлен" : "Нужно установить"}
          </p>
        </div>
        <div className="rounded-lg border border-zinc-700 bg-zinc-900/60 px-3 py-2">
          <p className="text-zinc-500">Стратегия</p>
          <p className={strategyFound ? "text-emerald-300" : "text-amber-300"}>
            {strategyFound ? apexStrategyLabel : "Не найдена"}
          </p>
        </div>
        <div className="rounded-lg border border-zinc-700 bg-zinc-900/60 px-3 py-2">
          <p className="text-zinc-500">Проверка EA</p>
          <p className={probeResults ? "text-zinc-200" : "text-zinc-500"}>
            {probeResults ? `${okCount}/${probeTotal} отвечают` : "Не запускалась"}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-2">
        <div className="rounded-lg border border-orange-500/25 bg-zinc-950/40 p-3 space-y-2">
          <p className="text-[11px] uppercase tracking-wide text-orange-300">Шаг 1</p>
          <p className="text-xs text-zinc-300">Установить или обновить файлы Apex.</p>
          <button
            onClick={handleSetup}
            disabled={loading.apexSetup || !zapretInstalled}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-orange-600/80 text-xs text-white hover:bg-orange-600 disabled:opacity-40"
          >
            <Wrench className="w-3.5 h-3.5" />
            Пресет Apex
          </button>
        </div>
        <div className="rounded-lg border border-zinc-700 bg-zinc-950/40 p-3 space-y-2">
          <p className="text-[11px] uppercase tracking-wide text-zinc-400">Шаг 2</p>
          <p className="text-xs text-zinc-300">Запустить выбранную стратегию.</p>
          <button
            onClick={handleStartApex}
            disabled={loading.startStrategy || !zapretInstalled}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 border border-zinc-600 text-xs text-white hover:bg-zinc-700 disabled:opacity-40"
          >
            <Play className="w-3.5 h-3.5" />
            Подключить
          </button>
        </div>
        <div className="rounded-lg border border-zinc-700 bg-zinc-950/40 p-3 space-y-2">
          <p className="text-[11px] uppercase tracking-wide text-zinc-400">Шаг 3</p>
          <p className="text-xs text-zinc-300">Проверить доступность EA-сервисов.</p>
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
        </div>
        <div className="rounded-lg border border-zinc-700 bg-zinc-950/40 p-3 space-y-2">
          <p className="text-[11px] uppercase tracking-wide text-zinc-400">Шаг 4</p>
          <p className="text-xs text-zinc-300">Запустить Apex заново после смены пресета.</p>
          <span className="inline-flex px-3 py-1.5 rounded-lg border border-zinc-700 bg-zinc-900 text-xs text-zinc-300">
            Запустите игру
          </span>
        </div>
      </div>

      <button
        onClick={handleAutoApex}
        disabled={loading.apexDetect || !zapretInstalled}
        className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800 border border-zinc-600 text-xs text-zinc-200 hover:bg-zinc-700 disabled:opacity-40"
      >
        Подобрать Apex автоматически
      </button>

      {apexActive && (
        <p className="text-xs text-emerald-300 flex items-center gap-1">
          <CheckCircle className="w-3.5 h-3.5" />
          Активна стратегия {activeStrategy?.name ?? apexStrategyLabel}
        </p>
      )}

      {!zapretInstalled && (
        <p className="text-xs text-amber-300 flex items-center gap-1">
          <AlertCircle className="w-3.5 h-3.5" />
          Сначала установите выбранное ядро во вкладке «Подключение».
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

      <p className="text-xs text-zinc-500">
        HTTP-проверка EA помогает поймать блокировку сайтов EA, но не гарантирует вход в
        лобби. После обновления пресета полностью перезапустите Apex.
      </p>
    </div>
  );
}
