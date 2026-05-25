import { CheckCircle, Plus, Radar, Square } from "lucide-react";
import { useState } from "react";
import { AddStrategyModal } from "@/components/AddStrategyModal";
import { StrategyCard } from "@/components/StrategyCard";
import { ALL_TAGS, TagBadge, sortTags } from "@/components/TagBadge";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/appStore";

export function StrategiesPage({ embedded = false }: { embedded?: boolean }) {
  const {
    strategies,
    activeStrategy,
    isLoading,
    isScanning,
    zapretInstalled,
    strategyScan,
    scanProgress,
    scanAllStrategies,
    cancelStrategyScan,
    loadStrategies,
  } = useAppStore();
  const [activeTag, setActiveTag] = useState<string | null>(null);
  const [scanMsg, setScanMsg] = useState<string | null>(null);
  const [addOpen, setAddOpen] = useState(false);
  const [addMsg, setAddMsg] = useState<string | null>(null);

  const filtered = activeTag
    ? strategies.filter((s) => s.tags.includes(activeTag))
    : strategies;

  const tagCounts = ALL_TAGS.reduce<Record<string, number>>((acc, tag) => {
    acc[tag] = strategies.filter((s) => s.tags.includes(tag)).length;
    return acc;
  }, {});

  const scanDone = Object.keys(strategyScan).length > 0;
  const workingCount = Object.values(strategyScan).filter((e) => e.works).length;
  const scanPercent =
    scanProgress && scanProgress.total > 0
      ? Math.min(100, Math.round((scanProgress.current / scanProgress.total) * 100))
      : 0;

  const formatDuration = (ms?: number) => {
    if (ms == null) return "—";
    const totalSeconds = Math.max(0, Math.round(ms / 1000));
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    if (minutes === 0) return `${seconds} сек`;
    return `${minutes} мин ${seconds.toString().padStart(2, "0")} сек`;
  };

  const handleAutoScan = async () => {
    setScanMsg(null);
    try {
      const result = await scanAllStrategies();
      const ok = result.entries.filter((e) => e.works).length;
      const total = strategies.length;
      if (result.cancelled) {
        setScanMsg(
          `Остановлено: проверено ${result.entries.length} из ${total}, рабочих ${ok}`
        );
      } else {
        setScanMsg(`Готово: ${ok} из ${result.entries.length} отвечают на сервисы`);
      }
    } catch {
      // error in store
    }
  };

  return (
    <div
      className={cn(
        "space-y-5",
        embedded ? "" : "p-6 overflow-y-auto h-full"
      )}
    >
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className={cn("font-bold text-white", embedded ? "text-lg" : "text-xl")}>
            Стратегии
          </h1>
          <p className="text-sm text-zinc-400 mt-1">
            {activeTag
              ? `${filtered.length} из ${strategies.length} стратегий`
              : `${strategies.length} стратегий доступно`}
            {scanDone && (
              <span className="text-zinc-500">
                {" "}
                · рабочих: {workingCount}
              </span>
            )}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            onClick={() => setAddOpen(true)}
            disabled={!zapretInstalled || isLoading}
            className="flex items-center gap-2 px-3 py-2 rounded-lg border border-zinc-600 hover:border-zinc-500 hover:bg-zinc-800 disabled:opacity-40 text-zinc-200 text-sm transition-colors"
          >
            <Plus className="w-4 h-4" />
            Добавить стратегию
          </button>
          {isScanning ? (
            <button
              type="button"
              onClick={() => cancelStrategyScan()}
              className="flex items-center gap-2 px-3 py-2 rounded-lg bg-red-600/90 hover:bg-red-500 text-white text-sm font-medium transition-colors"
            >
              <Square className="w-4 h-4" />
              Остановить
            </button>
          ) : (
            <button
              type="button"
              onClick={handleAutoScan}
              disabled={isLoading || !zapretInstalled}
              className="flex items-center gap-2 px-3 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 text-white text-sm font-medium transition-colors"
            >
              <Radar className="w-4 h-4" />
              Автоскан
            </button>
          )}
        </div>
      </div>

      {isScanning && (
        <section className="rounded-xl border border-amber-500/30 bg-amber-500/10 p-4 space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div>
              <p className="text-sm font-medium text-amber-100">
                Сканирование стратегий
              </p>
              <p className="text-xs text-amber-100/75">
                {scanProgress?.current ?? 0} / {scanProgress?.total ?? strategies.length}
                {scanProgress?.current_name && ` · сейчас: ${scanProgress.current_name}`}
              </p>
            </div>
            <div className="text-right text-xs text-amber-100/75">
              <p>Прошло: {formatDuration(scanProgress?.elapsed_ms)}</p>
              <p>Осталось: {formatDuration(scanProgress?.eta_ms)}</p>
            </div>
          </div>
          <div className="h-2 rounded-full bg-black/30 overflow-hidden">
            <div
              className="h-full rounded-full bg-amber-400 transition-all"
              style={{ width: `${scanPercent}%` }}
            />
          </div>
          <p className="text-xs text-amber-100/65">
            Нажмите «Остановить», чтобы прервать скан и вернуть прежнее подключение.
          </p>
        </section>
      )}

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => setActiveTag(null)}
          className={cn(
            "flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border transition-colors",
            activeTag === null
              ? "border-white/20 bg-white/10 text-white"
              : "border-zinc-700 text-zinc-400 hover:text-white hover:border-zinc-600"
          )}
        >
          Все
          <span
            className={cn(
              "rounded-full px-1.5 py-0.5 text-[10px] font-semibold",
              activeTag === null ? "bg-white/20 text-white" : "bg-zinc-700 text-zinc-400"
            )}
          >
            {strategies.length}
          </span>
        </button>
        {sortTags(ALL_TAGS).map((tag) => (
          <TagBadge
            key={tag}
            tag={tag}
            active={activeTag === tag}
            count={tagCounts[tag]}
            onClick={() => setActiveTag(activeTag === tag ? null : tag)}
          />
        ))}
      </div>

      {scanMsg && (
        <div className="flex items-center gap-2 text-xs text-emerald-300 bg-emerald-500/10 rounded-lg px-3 py-2">
          <CheckCircle className="w-3.5 h-3.5 flex-shrink-0" />
          {scanMsg}
        </div>
      )}

      {addMsg && (
        <div className="flex items-center gap-2 text-xs text-emerald-300 bg-emerald-500/10 rounded-lg px-3 py-2">
          <CheckCircle className="w-3.5 h-3.5 flex-shrink-0" />
          Добавлена стратегия: <strong>{addMsg}</strong>
        </div>
      )}

      <div className="space-y-3">
        {filtered.map((strategy) => (
          <StrategyCard
            key={strategy.id}
            strategy={strategy}
            isActive={activeStrategy?.id === strategy.id}
            scanEntry={strategyScan[strategy.id]}
          />
        ))}
        {filtered.length === 0 && (
          <div className="flex flex-col items-center justify-center py-12 gap-2">
            <p className="text-sm text-zinc-500">Нет стратегий для выбранного тега</p>
            <button
              type="button"
              onClick={() => setActiveTag(null)}
              className="text-xs text-zinc-400 hover:text-white underline underline-offset-2 transition-colors"
            >
              Сбросить фильтр
            </button>
          </div>
        )}
      </div>

      <AddStrategyModal
        open={addOpen}
        onClose={() => setAddOpen(false)}
        onAdded={(name) => {
          setAddMsg(name);
          loadStrategies();
        }}
      />
    </div>
  );
}
