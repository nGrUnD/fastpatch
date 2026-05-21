import { ArrowRight, CheckCircle, Loader2, Stethoscope, Wand2 } from "lucide-react";
import { useState } from "react";
import {
  probeBadgeClass,
  probeLatencyLabel,
  probeLabel,
  scoreConnectivityProbe,
} from "@/lib/probeUi";
import { TagBadge } from "@/components/TagBadge";
import { cn } from "@/lib/utils";
import { ApexPanel } from "@/components/ApexPanel";
import { useAppStore } from "@/stores/appStore";

export function ConnectionSettings({
  onOpenStrategies,
}: {
  onOpenStrategies: () => void;
}) {
  const {
    strategies,
    activeStrategy,
    isLoading,
    zapretInstalled,
    autoDetect,
    startStrategy,
    testMediaConnectivity,
  } = useAppStore();

  const [autoMsg, setAutoMsg] = useState<string | null>(null);
  const [isAutoDetecting, setIsAutoDetecting] = useState(false);
  const [mediaProbe, setMediaProbe] = useState<
    Awaited<ReturnType<typeof testMediaConnectivity>> | null
  >(null);
  const [mediaProbing, setMediaProbing] = useState(false);

  const handleAutoDetect = async () => {
    setIsAutoDetecting(true);
    setAutoMsg(null);
    try {
      const id = await autoDetect();
      if (id) {
        const found = strategies.find((s) => s.id === id);
        setAutoMsg(found?.name ?? id);
        await useAppStore.getState().loadActiveStrategy();
      }
    } finally {
      setIsAutoDetecting(false);
    }
  };

  return (
    <div className="space-y-6">
      <section className="rounded-xl border border-zinc-700 bg-zinc-800/50 p-5">
        <h2 className="text-sm font-semibold text-white mb-1">Автоподбор</h2>
        <p className="text-xs text-zinc-400 mb-4">
          То же, что «Подключить» на главной: Discord + YouTube, первая рабочая стратегия.
        </p>
        <button
          type="button"
          onClick={handleAutoDetect}
          disabled={isAutoDetecting || isLoading || !zapretInstalled}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 text-white text-sm font-medium"
        >
          {isAutoDetecting ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Wand2 className="w-4 h-4" />
          )}
          Подобрать стратегию
        </button>
        {autoMsg && (
          <p className="mt-3 flex items-center gap-2 text-xs text-emerald-300">
            <CheckCircle className="w-3.5 h-3.5" />
            Активна: <strong>{autoMsg}</strong>
          </p>
        )}
      </section>

      {activeStrategy && (
        <section className="rounded-xl border border-zinc-700 bg-zinc-800/50 p-5">
          <h2 className="text-sm font-semibold text-white mb-1">Проверка связи</h2>
          <p className="text-xs text-zinc-400 mb-3">
            По тегам активной стратегии: Discord, YouTube, Cloudflare CDN, EA/Apex и др.
          </p>
          <div className="flex flex-wrap gap-1 mb-3">
            {activeStrategy &&
              strategies
                .find((s) => s.id === activeStrategy.id)
                ?.tags.filter((t) => t !== "general")
                .map((tag) => <TagBadge key={tag} tag={tag} />)}
          </div>
          <button
            type="button"
            onClick={async () => {
              setMediaProbing(true);
              setMediaProbe(null);
              try {
                setMediaProbe(await testMediaConnectivity());
              } finally {
                setMediaProbing(false);
              }
            }}
            disabled={isLoading || mediaProbing}
            className="flex items-center gap-2 px-3 py-2 rounded-lg bg-violet-600/80 hover:bg-violet-600 disabled:opacity-40 text-sm text-white"
          >
            {mediaProbing ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Stethoscope className="w-4 h-4" />
            )}
            Проверить все сервисы
          </button>
          {mediaProbe && (
            <div className="flex flex-wrap gap-2 mt-3">
              {mediaProbe.map((r) => (
                <span
                  key={r.target}
                  title={r.error ?? undefined}
                  className={cn(
                    "text-xs px-2 py-0.5 rounded-full border",
                    probeBadgeClass(r)
                  )}
                >
                  {probeLabel(r.target)}
                  {r.latency_ms != null && ` ${probeLatencyLabel(r)}`}
                </span>
              ))}
            </div>
          )}
          {mediaProbe && (() => {
            const strategy = strategies.find((s) => s.id === activeStrategy.id);
            const { warnings, notes } = scoreConnectivityProbe(
              mediaProbe,
              strategy?.tags ?? []
            );
            return (
              <div className="mt-2 space-y-1">
                {warnings.length === 0 ? (
                  <p className="text-xs text-emerald-300/90">
                    Все проверенные сервисы отвечают.
                  </p>
                ) : (
                  <ul className="text-xs text-amber-300/90 list-disc list-inside">
                    {warnings.map((w) => (
                      <li key={w}>{w}</li>
                    ))}
                  </ul>
                )}
                {notes.map((n) => (
                  <p key={n} className="text-xs text-zinc-500">
                    {n}
                  </p>
                ))}
              </div>
            );
          })()}
        </section>
      )}

      {zapretInstalled && strategies.length > 0 && (
        <section className="rounded-xl border border-zinc-700 bg-zinc-800/50 p-5">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold text-white">Ручной выбор ALT</h2>
            <button
              type="button"
              onClick={onOpenStrategies}
              className="text-xs text-zinc-400 hover:text-white flex items-center gap-1"
            >
              Все стратегии
              <ArrowRight className="w-3 h-3" />
            </button>
          </div>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 max-h-40 overflow-y-auto">
            {strategies
              .filter((s) => /^ALT\d*$/i.test(s.name) || s.name === "ALT")
              .slice(0, 12)
              .map((s) => (
                <button
                  key={s.id}
                  type="button"
                  onClick={() => startStrategy(s.id)}
                  disabled={isLoading}
                  className={cn(
                    "px-3 py-2 rounded-lg border text-xs font-medium transition-colors",
                    activeStrategy?.id === s.id
                      ? "border-emerald-500/50 bg-emerald-500/10 text-emerald-300"
                      : "border-zinc-700 bg-zinc-900 text-zinc-300 hover:border-zinc-600"
                  )}
                >
                  {s.name}
                </button>
              ))}
          </div>
          <p className="text-xs text-zinc-500 mt-2">
            Полный список — вкладка «Стратегии».
          </p>
        </section>
      )}

      {zapretInstalled && <ApexPanel />}
    </div>
  );
}
