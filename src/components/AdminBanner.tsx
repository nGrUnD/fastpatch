import { Loader2, ShieldAlert, ShieldCheck } from "lucide-react";
import { useState } from "react";
import { useAppStore } from "@/stores/appStore";

export function AdminBanner() {
  const { appInfo, relaunchAsAdmin } = useAppStore();
  const [relaunching, setRelaunching] = useState(false);

  if (!appInfo) return null;

  if (appInfo.elevated) {
    return (
      <div className="mx-6 mt-4 flex items-center gap-2 rounded-lg border border-emerald-500/20 bg-emerald-500/5 px-3 py-2 text-xs text-emerald-300/90 shrink-0">
        <ShieldCheck className="w-3.5 h-3.5 flex-shrink-0" />
        Запущено от имени администратора
      </div>
    );
  }

  const handleRelaunch = async () => {
    setRelaunching(true);
    try {
      await relaunchAsAdmin();
    } finally {
      setRelaunching(false);
    }
  };

  return (
    <div className="mx-6 mt-4 rounded-xl border border-amber-500/40 bg-amber-500/10 p-4 shrink-0">
      <div className="flex items-start gap-3">
        <ShieldAlert className="w-5 h-5 text-amber-300 flex-shrink-0 mt-0.5" />
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-amber-100">
            Нужны права администратора
          </p>
          <p className="text-xs text-amber-200/70 mt-1">
            WinDivert и zapret (winws.exe) работают только с повышенными правами.
            Подтвердите один запрос UAC — дальше стратегии запускаются без лишних окон.
          </p>
          {appInfo.is_dev_build && (
            <p className="text-xs text-amber-200/60 mt-2">
              Режим разработки: кнопка ниже не подходит для{" "}
              <code className="text-amber-100/80">target\debug\fastpatch.exe</code>.
              Откройте PowerShell от администратора и выполните{" "}
              <code className="text-amber-100/80">pnpm tauri dev</code>.
              Для обычного запуска установите MSI из{" "}
              <code className="text-amber-100/80">pnpm tauri build</code>.
            </p>
          )}
          <button
            type="button"
            onClick={handleRelaunch}
            disabled={relaunching}
            className="mt-3 flex items-center gap-2 px-4 py-2 rounded-lg bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white text-sm font-medium transition-colors"
          >
            {relaunching ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <ShieldAlert className="w-4 h-4" />
            )}
            Запустить от имени администратора
          </button>
        </div>
      </div>
    </div>
  );
}
