import { Info, X } from "lucide-react";
import { useAppStore } from "@/stores/appStore";

export function WinwsSessionBanner() {
  const { winwsSessionHint, isScanning, loading } = useAppStore();

  if (!winwsSessionHint) return null;

  const busy = isScanning || loading.startStrategy || loading.installZapret;

  return (
    <div
      className={`flex items-start gap-2 px-4 py-2 text-xs border-b shrink-0 ${
        busy
          ? "bg-amber-500/10 border-amber-500/30 text-amber-100"
          : "bg-sky-500/10 border-sky-500/30 text-sky-100"
      }`}
    >
      <Info className="w-3.5 h-3.5 flex-shrink-0 mt-0.5" />
      <p className="flex-1 leading-relaxed">{winwsSessionHint}</p>
      {!busy && (
        <button
          type="button"
          onClick={() => useAppStore.setState({ winwsSessionHint: null })}
          className="p-0.5 rounded text-current/60 hover:text-white"
          aria-label="Закрыть"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      )}
    </div>
  );
}
