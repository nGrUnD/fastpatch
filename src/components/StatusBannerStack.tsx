import {
  AlertCircle,
  AlertTriangle,
  CheckCircle,
  Download,
  Info,
  Loader2,
  ShieldAlert,
  ShieldCheck,
  X,
} from "lucide-react";
import { useState } from "react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/appStore";

export function StatusBannerStack() {
  const {
    appInfo,
    clearError,
    clearWinwsBusyHint,
    error,
    installZapret,
    killWinws,
    loading,
    page,
    relaunchAsAdmin,
    winwsBusyHint,
    winwsSessionHint,
    zapretBackend,
    zapretInstalled,
    zapretInstalling,
    zapretMessage,
  } = useAppStore();
  const [relaunching, setRelaunching] = useState(false);

  const showAdminOk = appInfo && appInfo.elevated && !(appInfo.from_autostart && appInfo.elevated);
  const showAdminWarn = appInfo && !appInfo.elevated;
  const showInstall = !zapretInstalled && page === "home";
  const hasItems =
    error ||
    winwsBusyHint ||
    winwsSessionHint ||
    showAdminOk ||
    showAdminWarn ||
    showInstall;

  if (!hasItems) return null;

  const handleRelaunch = async () => {
    setRelaunching(true);
    try {
      await relaunchAsAdmin();
    } finally {
      setRelaunching(false);
    }
  };

  return (
    <div className="mx-6 mt-4 shrink-0 rounded-xl border border-zinc-700/80 bg-zinc-900/95 p-3 space-y-2 shadow-lg shadow-black/20">
      {error && (
        <div className="flex items-start gap-2 text-sm text-red-300">
          <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
          <p className="flex-1 whitespace-pre-wrap">{error}</p>
          <button type="button" onClick={clearError} className="text-red-400 hover:text-red-200">
            <X className="w-4 h-4" />
          </button>
        </div>
      )}

      {winwsBusyHint && (
        <div className="flex items-start gap-2 text-sm text-amber-100">
          <AlertTriangle className="w-4 h-4 mt-0.5 flex-shrink-0 text-amber-300" />
          <div className="flex-1 min-w-0">
            <p className="whitespace-pre-wrap">{winwsBusyHint}</p>
            <button
              type="button"
              onClick={() => killWinws()}
              disabled={loading.killWinws}
              className="mt-2 inline-flex items-center px-3 py-1.5 rounded-lg bg-amber-600/90 text-xs font-medium text-white hover:bg-amber-600 disabled:opacity-50"
            >
              Снять задачу
            </button>
          </div>
          <button
            type="button"
            onClick={clearWinwsBusyHint}
            className="text-amber-300/80 hover:text-amber-100"
            aria-label="Закрыть"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      )}

      {winwsSessionHint && (
        <div
          className={cn(
            "flex items-start gap-2 text-xs",
            loading.startStrategy || loading.installZapret ? "text-amber-100" : "text-sky-100"
          )}
        >
          <Info className="w-3.5 h-3.5 mt-0.5 flex-shrink-0" />
          <p className="flex-1">{winwsSessionHint}</p>
        </div>
      )}

      {showAdminOk && (
        <div className="flex items-center gap-2 text-xs text-emerald-300/90">
          <ShieldCheck className="w-3.5 h-3.5 flex-shrink-0" />
          Запущено от имени администратора
        </div>
      )}

      {showAdminWarn && (
        <div className="flex items-start gap-3 text-amber-100">
          <ShieldAlert className="w-5 h-5 text-amber-300 flex-shrink-0 mt-0.5" />
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium">Нужны права администратора</p>
            <p className="text-xs text-amber-200/70 mt-1">
              WinDivert и zapret работают только с повышенными правами.
            </p>
            {appInfo?.is_dev_build && (
              <p className="text-xs text-amber-200/60 mt-2">
                Dev-режим: откройте PowerShell от администратора и выполните{" "}
                <code className="text-amber-100/80">pnpm tauri dev</code>.
              </p>
            )}
            <button
              type="button"
              onClick={handleRelaunch}
              disabled={relaunching}
              className="mt-2 inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white text-xs font-medium"
            >
              {relaunching ? (
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
              ) : (
                <ShieldAlert className="w-3.5 h-3.5" />
              )}
              Запустить от имени администратора
            </button>
          </div>
        </div>
      )}

      {showInstall && (
        <div className="flex items-start gap-2 text-sm text-amber-200">
          <Download className="w-4 h-4 mt-0.5 flex-shrink-0" />
          <div className="flex-1 min-w-0">
            <p className="font-medium">Установка {zapretBackend === "v2" ? "Zapret 2" : "Zapret 1"}</p>
            <p className="text-xs text-amber-200/70 mt-1">
              При первом подключении fastpatch скачает нужный bundle с GitHub.
            </p>
            {zapretMessage && (
              <p className="mt-2 inline-flex items-center gap-1 text-xs text-emerald-300">
                <CheckCircle className="w-3.5 h-3.5" />
                {zapretMessage}
              </p>
            )}
            {zapretInstalling ? (
              <p className="mt-2 inline-flex items-center gap-2 text-xs">
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                Скачивание и распаковка...
              </p>
            ) : (
              <button
                type="button"
                onClick={() => installZapret()}
                disabled={loading.installZapret}
                className="mt-2 inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white text-xs font-medium"
              >
                <Download className="w-3.5 h-3.5" />
                Установить
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
