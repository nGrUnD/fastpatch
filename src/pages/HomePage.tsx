import { Loader2, Power, Shield } from "lucide-react";
import { useState } from "react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/appStore";

export function HomePage() {
  const {
    activeStrategy,
    loading,
    zapretInstalled,
    zapretInstalling,
    autoDetect,
    stopStrategy,
    installZapret,
    loadActiveStrategy,
    openSettings,
  } = useAppStore();

  const [isConnecting, setIsConnecting] = useState(false);

  const isBusy =
    loading.startStrategy ||
    loading.stopStrategy ||
    loading.installZapret ||
    isConnecting ||
    zapretInstalling;
  const connected = Boolean(activeStrategy);
  const isApexActive = activeStrategy?.name.toUpperCase().includes("APEX") ?? false;

  const handleConnect = async () => {
    setIsConnecting(true);
    try {
      if (!zapretInstalled) {
        await installZapret();
      }
      await autoDetect();
      await loadActiveStrategy();
    } finally {
      setIsConnecting(false);
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-full p-8 overflow-y-auto">
      <div className="w-full max-w-md text-center space-y-8">
        <div className="space-y-2">
          <div
            className={cn(
              "mx-auto w-16 h-16 rounded-2xl flex items-center justify-center",
              connected ? "bg-emerald-500/20" : "bg-zinc-800"
            )}
          >
            <Shield
              className={cn(
                "w-8 h-8",
                connected ? "text-emerald-400" : "text-zinc-500"
              )}
            />
          </div>
          <h1 className="text-2xl font-bold text-white">fastpatch</h1>
          <p className="text-sm text-zinc-400">
            {connected
              ? `Подключено · ${activeStrategy?.name}`
              : "Обход Discord, YouTube и игровых сценариев"}
          </p>
        </div>

        {connected ? (
          <div className="space-y-4">
            <div className="flex items-center justify-center gap-2 text-emerald-300 text-sm">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
              Обход активен
            </div>
            <button
              type="button"
              onClick={() => stopStrategy()}
              disabled={isBusy}
              className="w-full py-4 rounded-xl border border-red-500/40 bg-red-500/10 text-red-200 font-semibold text-lg hover:bg-red-500/20 disabled:opacity-40 transition-colors"
            >
              Отключить
            </button>
          </div>
        ) : (
          <button
            type="button"
            onClick={handleConnect}
            disabled={isBusy}
            className="w-full py-4 rounded-xl bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white font-semibold text-lg shadow-lg shadow-emerald-900/30 transition-colors flex items-center justify-center gap-2"
          >
            {isBusy ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin" />
                {zapretInstalling ? "Установка Zapret 2…" : "Подключение…"}
              </>
            ) : (
              <>
                <Power className="w-5 h-5" />
                Подключить
              </>
            )}
          </button>
        )}

        <p className="text-xs text-zinc-500 leading-relaxed">
          {connected
            ? isApexActive
              ? "Для Apex после смены пресета полностью перезапустите игру."
              : "После смены стратегии перезапустите Discord и YouTube из трея."
            : "Автоматически подберёт рабочий пресет для Discord и YouTube. Apex — во вкладке «Игры»."}
          {" "}
          <button
            type="button"
            onClick={() => openSettings("connection")}
            className="text-zinc-400 hover:text-white underline underline-offset-2"
          >
            Настройки
          </button>
          {" · "}
          <button
            type="button"
            onClick={() => openSettings("games")}
            className="text-zinc-400 hover:text-white underline underline-offset-2"
          >
            Игры / Apex
          </button>
        </p>
      </div>
    </div>
  );
}
