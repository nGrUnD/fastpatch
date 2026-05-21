import { AlertCircle, X } from "lucide-react";
import { useAppStore } from "@/stores/appStore";

export function ErrorBanner() {
  const { error, clearError } = useAppStore();
  if (!error) return null;

  return (
    <div className="mx-6 mt-4 flex items-start gap-3 bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-sm text-red-300 shrink-0">
      <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
      <span className="flex-1 whitespace-pre-wrap">{error}</span>
      <button onClick={clearError} className="text-red-400 hover:text-red-200">
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}
