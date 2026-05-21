import { invoke } from "@tauri-apps/api/core";
import {
  AlertCircle,
  Archive,
  Check,
  Loader2,
  RefreshCw,
  RotateCcw,
  Save,
  Trash2,
} from "lucide-react";
import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";

interface HostsBackup {
  filename: string;
  created_at: string;
  size_bytes: number;
}

export function HostsPage({ embedded = false }: { embedded?: boolean }) {
  const [content, setContent] = useState("");
  const [original, setOriginal] = useState("");
  const [backups, setBackups] = useState<HostsBackup[]>([]);
  const [loading, setLoading] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadHosts = async () => {
    setLoading(true);
    setError(null);
    try {
      const text = await invoke<string>("read_hosts");
      setContent(text);
      setOriginal(text);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const loadBackups = async () => {
    try {
      const list = await invoke<HostsBackup[]>("list_hosts_backups");
      setBackups(list);
    } catch {
      setBackups([]);
    }
  };

  useEffect(() => {
    loadHosts();
    loadBackups();
  }, []);

  const handleSave = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke("write_hosts", { content });
      setOriginal(content);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
      loadBackups();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleBackup = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke("backup_hosts");
      loadBackups();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleRestore = async (filename: string) => {
    if (!confirm(`Восстановить резервную копию ${filename}?`)) return;
    setLoading(true);
    setError(null);
    try {
      await invoke("restore_hosts_backup", { filename });
      loadHosts();
      loadBackups();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteBackup = async (filename: string) => {
    if (!confirm(`Удалить резервную копию ${filename}?`)) return;
    try {
      await invoke("delete_hosts_backup", { filename });
      loadBackups();
    } catch (e) {
      setError(String(e));
    }
  };

  const isDirty = content !== original;

  return (
    <div className={cn("space-y-5", embedded ? "" : "p-6 overflow-y-auto h-full")}>
      <div className="flex items-center justify-between">
        <div>
          <h1 className={cn("font-bold text-white", embedded ? "text-lg" : "text-xl")}>
            Hosts-файл
          </h1>
          <p className="text-sm text-zinc-400 mt-1">
            C:\Windows\System32\drivers\etc\hosts
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={loadHosts}
            disabled={loading}
            className="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
            title="Обновить"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
          <button
            onClick={handleBackup}
            disabled={loading}
            className="flex items-center gap-2 px-3 py-2 rounded-lg border border-zinc-700 text-zinc-300 hover:text-white hover:border-zinc-600 text-sm transition-colors"
          >
            <Archive className="w-4 h-4" />
            Создать бэкап
          </button>
          <button
            onClick={handleSave}
            disabled={loading || !isDirty}
            className="flex items-center gap-2 px-3 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 text-white text-sm font-medium transition-colors"
          >
            {saved ? (
              <Check className="w-4 h-4" />
            ) : loading ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Save className="w-4 h-4" />
            )}
            {saved ? "Сохранено" : "Сохранить"}
          </button>
        </div>
      </div>

      {error && (
        <div className="flex items-start gap-3 bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-sm text-red-300">
          <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Editor */}
      <div className="rounded-xl border border-zinc-700 overflow-hidden">
        <div className="bg-zinc-800/80 px-4 py-2 border-b border-zinc-700 flex items-center justify-between">
          <span className="text-xs text-zinc-400 font-mono">hosts</span>
          {isDirty && (
            <span className="text-xs text-amber-400">Есть несохранённые изменения</span>
          )}
        </div>
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          className="w-full h-64 bg-zinc-900 text-zinc-200 text-xs font-mono p-4 resize-none outline-none"
          placeholder="Загрузка..."
          spellCheck={false}
        />
      </div>

      {/* Backups */}
      {backups.length > 0 && (
        <div className="rounded-xl border border-zinc-700 bg-zinc-800/50 p-4">
          <h2 className="text-sm font-semibold text-white mb-3">
            Резервные копии ({backups.length})
          </h2>
          <div className="space-y-2">
            {backups.map((b) => (
              <div
                key={b.filename}
                className="flex items-center justify-between py-2 border-b border-zinc-700/50 last:border-0"
              >
                <div>
                  <p className="text-xs font-mono text-zinc-300">{b.filename}</p>
                  <p className="text-xs text-zinc-500">
                    {b.created_at} · {(b.size_bytes / 1024).toFixed(1)} KB
                  </p>
                </div>
                <div className="flex items-center gap-1">
                  <button
                    onClick={() => handleRestore(b.filename)}
                    className="p-1.5 rounded text-zinc-400 hover:text-emerald-400 hover:bg-emerald-500/10 transition-colors"
                    title="Восстановить"
                  >
                    <RotateCcw className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => handleDeleteBackup(b.filename)}
                    className="p-1.5 rounded text-zinc-400 hover:text-red-400 hover:bg-red-500/10 transition-colors"
                    title="Удалить"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
