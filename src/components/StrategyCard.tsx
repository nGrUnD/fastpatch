import { CheckCircle, HelpCircle, Loader2, Play, Square, TestTube, XCircle } from "lucide-react";
import { useState } from "react";
import { probeBadgeClass, probeFailed, probeLabel, probeLatencyLabel } from "@/lib/probeUi";
import { cn } from "@/lib/utils";
import {
  type Strategy,
  type StrategyScanEntry,
  type TestResult,
  useAppStore,
} from "@/stores/appStore";
import { TagBadge } from "./TagBadge";

interface StrategyCardProps {
  strategy: Strategy;
  isActive: boolean;
  scanEntry?: StrategyScanEntry;
}

export function StrategyCard({ strategy, isActive, scanEntry }: StrategyCardProps) {
  const {
    startStrategy,
    stopStrategy,
    testStrategy,
    isLoading,
    isScanning,
    zapretInstalled,
    activeStrategy,
  } = useAppStore();
  const [testResults, setTestResults] = useState<TestResult[] | null>(null);
  const [isTesting, setIsTesting] = useState(false);

  const scanned = Boolean(scanEntry);
  const nonWorking = scanned && !scanEntry!.works;
  const displayResults = testResults ?? scanEntry?.results ?? null;
  const anotherActive =
    Boolean(activeStrategy) && activeStrategy?.id !== strategy.id;
  const actionsDisabled =
    isTesting || isLoading || isScanning || !zapretInstalled || nonWorking;

  const handleStart = () => startStrategy(strategy.id);
  const handleStop = () => stopStrategy();

  const handleTest = async () => {
    setIsTesting(true);
    setTestResults(null);
    try {
      const results = await testStrategy(strategy.id);
      setTestResults(results);
    } finally {
      setIsTesting(false);
    }
  };

  return (
    <div
      className={cn(
        "rounded-xl border p-4 transition-all",
        nonWorking && "opacity-75",
        isActive
          ? "border-emerald-500/50 bg-emerald-500/5"
          : "border-zinc-700/50 bg-zinc-800/50 hover:border-zinc-600"
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1 flex-wrap">
            {isActive && (
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            )}
            <h3 className="font-semibold text-white text-sm truncate">{strategy.name}</h3>
            {nonWorking && (
              <span className="text-[10px] font-medium uppercase tracking-wide text-red-300/90 bg-red-500/10 border border-red-500/30 px-1.5 py-0.5 rounded">
                не работает
              </span>
            )}
            {scanned && scanEntry!.works && !isActive && (
              <span className="text-[10px] font-medium text-emerald-400/90 bg-emerald-500/10 border border-emerald-500/20 px-1.5 py-0.5 rounded">
                OK
              </span>
            )}
          </div>
          <p className="text-xs text-zinc-400 mb-2">{strategy.description}</p>
          <div className="flex flex-wrap gap-1">
            {strategy.tags.map((tag) => (
              <TagBadge key={tag} tag={tag} />
            ))}
          </div>
        </div>

        <div className="flex items-center gap-1 flex-shrink-0">
          <button
            type="button"
            onClick={handleTest}
            disabled={actionsDisabled}
            title={nonWorking ? "Не работает" : "Тест"}
            className="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-700 disabled:opacity-40 disabled:pointer-events-none transition-colors"
          >
            {isTesting ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <TestTube className="w-4 h-4" />
            )}
          </button>

          {isActive ? (
            <button
              type="button"
              onClick={handleStop}
              disabled={isLoading}
              title="Остановить"
              className="p-1.5 rounded-lg text-red-400 hover:text-red-300 hover:bg-red-500/10 disabled:opacity-40 transition-colors"
            >
              <Square className="w-4 h-4" />
            </button>
          ) : (
            <button
              type="button"
              onClick={handleStart}
              disabled={actionsDisabled}
              title={
                nonWorking
                  ? "Не работает"
                  : anotherActive
                    ? `Сменить с «${activeStrategy?.name}» (winws перезапустится)`
                    : "Запустить"
              }
              className="p-1.5 rounded-lg text-emerald-400 hover:text-emerald-300 hover:bg-emerald-500/10 disabled:opacity-40 disabled:pointer-events-none transition-colors"
            >
              <Play className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>

      {displayResults && (
        <div className="mt-3 pt-3 border-t border-zinc-700/50">
          <div className="flex flex-wrap gap-2">
            {displayResults.map((r) => (
              <div
                key={r.target}
                title={r.error ?? undefined}
                className={cn(
                  "flex items-center gap-1 text-xs px-2 py-0.5 rounded-full border",
                  probeBadgeClass(r)
                )}
              >
                  {probeFailed(r) ? (
                    <XCircle className="w-3 h-3" />
                  ) : r.error ? (
                    <HelpCircle className="w-3 h-3" />
                  ) : (
                    <CheckCircle className="w-3 h-3" />
                  )}
                  {probeLabel(r.target)}
                  {r.latency_ms !== undefined && (
                    <span className="opacity-70">{probeLatencyLabel(r)}</span>
                  )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
