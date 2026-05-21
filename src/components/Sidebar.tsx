import { invoke } from "@tauri-apps/api/core";
import { Home, Settings, Shield, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { type AppPage, useAppStore } from "@/stores/appStore";

const NAV_ITEMS: { id: AppPage; label: string; icon: React.FC<{ className?: string }> }[] = [
  { id: "home", label: "Главная", icon: Home },
  { id: "settings", label: "Настройки", icon: Settings },
];

export function Sidebar() {
  const { page, setPage, activeStrategy, appInfo } = useAppStore();

  const handleHideToTray = () => {
    invoke("hide_to_tray").catch(console.error);
  };

  return (
    <aside className="w-52 flex-shrink-0 bg-zinc-900 border-r border-zinc-800 flex flex-col h-full">
      {/* Logo */}
      <div className="p-4 border-b border-zinc-800 flex items-center gap-2">
        <Shield className="w-5 h-5 text-emerald-400" />
        <span className="font-bold text-white text-lg">fastpatch</span>
      </div>

      {/* Status indicator */}
      <div className="px-4 py-3 border-b border-zinc-800">
        <div className="flex items-center gap-2">
          <span
            className={cn(
              "w-2 h-2 rounded-full",
              activeStrategy ? "bg-emerald-400 animate-pulse" : "bg-zinc-600"
            )}
          />
          <span className="text-xs text-zinc-400">
            {activeStrategy ? activeStrategy.name : "Отключено"}
            {appInfo && !appInfo.elevated && (
              <span className="block text-amber-400/90 mt-0.5">Нет прав админа</span>
            )}
          </span>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 p-2 space-y-1">
        {NAV_ITEMS.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setPage(id)}
            className={cn(
              "w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors",
              page === id
                ? "bg-emerald-500/20 text-emerald-400"
                : "text-zinc-400 hover:text-white hover:bg-zinc-800"
            )}
          >
            <Icon className="w-4 h-4" />
            {label}
          </button>
        ))}
      </nav>

      {/* Hide to tray */}
      <div className="p-2 border-t border-zinc-800">
        <button
          onClick={handleHideToTray}
          className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800 transition-colors"
        >
          <X className="w-4 h-4" />
          Свернуть в трей
        </button>
      </div>
    </aside>
  );
}
