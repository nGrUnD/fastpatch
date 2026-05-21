import { FileUp, Loader2, X } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/appStore";

interface AddStrategyModalProps {
  open: boolean;
  onClose: () => void;
  onAdded?: (name: string) => void;
}

export function AddStrategyModal({ open, onClose, onAdded }: AddStrategyModalProps) {
  const { addCustomStrategy, isLoading, zapretInstalled } = useAppStore();
  const [name, setName] = useState("");
  const [content, setContent] = useState("");
  const [localError, setLocalError] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const reset = () => {
    setName("");
    setContent("");
    setLocalError(null);
  };

  const handleClose = () => {
    reset();
    onClose();
  };

  const applyFile = useCallback(async (file: File) => {
    if (!file.name.toLowerCase().endsWith(".bat")) {
      setLocalError("Нужен файл .bat");
      return;
    }
    const text = await file.text();
    setContent(text);
    if (!name.trim()) {
      const stem = file.name.replace(/\.bat$/i, "");
      const inner = stem.match(/general\s*\((.+)\)/i)?.[1] ?? stem;
      setName(inner.trim());
    }
    setLocalError(null);
  }, [name]);

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const file = e.dataTransfer.files[0];
    if (file) await applyFile(file);
  };

  const handleSubmit = async () => {
    setLocalError(null);
    if (!content.trim()) {
      setLocalError("Вставьте содержимое .bat или перетащите файл");
      return;
    }
    try {
      const strategy = await addCustomStrategy(name.trim() || "CUSTOM", content);
      onAdded?.(strategy.name);
      handleClose();
    } catch (e) {
      setLocalError(String(e));
    }
  };

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      onClick={handleClose}
    >
      <div
        className="w-full max-w-lg rounded-xl border border-zinc-700 bg-zinc-900 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-zinc-800 px-5 py-4">
          <h2 className="text-sm font-semibold text-white">Добавить стратегию</h2>
          <button
            type="button"
            onClick={handleClose}
            className="p-1 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-5 space-y-4">
          <div>
            <label className="text-xs text-zinc-400 block mb-1">Название</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Например: MY ALT"
              className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-white placeholder:text-zinc-500 focus:outline-none focus:border-emerald-500/50"
            />
            <p className="text-[10px] text-zinc-500 mt-1">
              Сохранится как general ({name.trim() || "CUSTOM"}).bat
            </p>
          </div>

          <div
            onDragOver={(e) => {
              e.preventDefault();
              setIsDragging(true);
            }}
            onDragLeave={() => setIsDragging(false)}
            onDrop={handleDrop}
            className={cn(
              "rounded-lg border border-dashed p-4 transition-colors",
              isDragging
                ? "border-emerald-500/60 bg-emerald-500/5"
                : "border-zinc-600 bg-zinc-800/30"
            )}
          >
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="Вставьте содержимое .bat или перетащите файл сюда…"
              rows={8}
              className="w-full resize-y rounded-lg bg-transparent text-xs font-mono text-zinc-200 placeholder:text-zinc-500 focus:outline-none"
            />
            <div className="flex items-center justify-between mt-3">
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                className="flex items-center gap-1.5 text-xs text-zinc-400 hover:text-white"
              >
                <FileUp className="w-3.5 h-3.5" />
                Выбрать .bat
              </button>
              <input
                ref={fileInputRef}
                type="file"
                accept=".bat"
                className="hidden"
                onChange={async (e) => {
                  const file = e.target.files?.[0];
                  if (file) await applyFile(file);
                  e.target.value = "";
                }}
              />
            </div>
          </div>

          {localError && <p className="text-xs text-red-300">{localError}</p>}
          {!zapretInstalled && (
            <p className="text-xs text-amber-400/90">Сначала установите zapret на главной.</p>
          )}
        </div>

        <div className="flex justify-end gap-2 border-t border-zinc-800 px-5 py-4">
          <button
            type="button"
            onClick={handleClose}
            className="px-3 py-2 rounded-lg text-sm text-zinc-400 hover:text-white hover:bg-zinc-800"
          >
            Отмена
          </button>
          <button
            type="button"
            onClick={handleSubmit}
            disabled={isLoading || !zapretInstalled}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 text-sm text-white font-medium"
          >
            {isLoading && <Loader2 className="w-4 h-4 animate-spin" />}
            Сохранить
          </button>
        </div>
      </div>
    </div>
  );
}
