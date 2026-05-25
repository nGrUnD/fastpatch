import { CheckCircle, Download, Loader2 } from "lucide-react";
import { useAppStore } from "@/stores/appStore";

export function ZapretBanner() {
  const { zapretInstalled, zapretInstalling, zapretMessage, installZapret } = useAppStore();

  const { page } = useAppStore();
  if (zapretInstalled || page !== "home") return null;

  return (
    <div className="mx-6 mt-4 rounded-xl border border-amber-500/40 bg-amber-500/10 p-4 shrink-0">
      <p className="text-sm font-medium text-amber-200">Установка Zapret 2</p>
      <p className="text-xs text-amber-200/70 mt-1">
        При «Подключить» скачивается bundle zapret2-youtube-discord с GitHub (~2 МБ).
        Нужны интернет и права администратора.
      </p>

      {zapretMessage && (
        <div className="mt-2 flex items-center gap-2 text-xs text-emerald-300 bg-emerald-500/10 rounded-lg px-3 py-2">
          <CheckCircle className="w-3.5 h-3.5 flex-shrink-0" />
          {zapretMessage}
        </div>
      )}

      {zapretInstalling && (
        <div className="mt-3 flex items-center gap-2 text-xs text-amber-200/90">
          <Loader2 className="w-4 h-4 animate-spin" />
          Скачивание и распаковка… Это может занять до минуты.
        </div>
      )}

      {!zapretInstalling && (
        <button
          onClick={() => installZapret()}
          className="mt-3 flex items-center gap-2 px-4 py-2 rounded-lg bg-amber-600 hover:bg-amber-500 text-white text-sm font-medium transition-colors"
        >
          <Download className="w-4 h-4" />
          Установить Zapret 2
        </button>
      )}
    </div>
  );
}
