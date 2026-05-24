import { AlertTriangle, X } from "lucide-react";
import { useAppStore } from "@/stores/appStore";

export function WinwsBusyBanner() {
  const { winwsBusyHint, isLoading, killWinws, clearWinwsBusyHint } = useAppStore();

  if (!winwsBusyHint) return null;

  return (
    <div className="mx-6 mt-4 flex items-start gap-3 bg-amber-500/10 border border-amber-500/35 rounded-xl p-4 text-sm text-amber-100 shrink-0">
      <AlertTriangle className="w-4 h-4 mt-0.5 flex-shrink-0 text-amber-300" />
      <div className="flex-1 min-w-0 space-y-3">
        <p className="whitespace-pre-wrap leading-relaxed">{winwsBusyHint}</p>
        <button
          type="button"
          onClick={() => killWinws()}
          disabled={isLoading}
          className="inline-flex items-center px-3 py-1.5 rounded-lg bg-amber-600/90 text-xs font-medium text-white hover:bg-amber-600 disabled:opacity-50"
        >
          Снять задачу
        </button>
      </div>
      <button
        type="button"
        onClick={clearWinwsBusyHint}
        className="text-amber-300/80 hover:text-amber-100 flex-shrink-0"
        aria-label="Закрыть"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}
