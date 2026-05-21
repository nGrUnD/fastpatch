import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Bell,
  CheckCircle,
  Download,
  ExternalLink,
  Filter,
  Gamepad2,
  GitFork,
  Globe,
  List,
  Monitor,
  Power,
  RefreshCw,
  Shield,
  Settings2,
  ToggleLeft,
  ToggleRight,
  Wrench,
} from "lucide-react";
import { useState } from "react";
import { ConnectionSettings } from "@/components/settings/ConnectionSettings";
import { cn } from "@/lib/utils";
import { HostsPage } from "@/pages/HostsPage";
import { StrategiesPage } from "@/pages/StrategiesPage";
import { useAppStore } from "@/stores/appStore";

type SettingsTab = "connection" | "strategies" | "hosts" | "system";

const TABS: { id: SettingsTab; label: string }[] = [
  { id: "connection", label: "Подключение" },
  { id: "strategies", label: "Стратегии" },
  { id: "hosts", label: "Hosts" },
  { id: "system", label: "Система" },
];

function Toggle({
  enabled,
  onToggle,
  disabled,
}: {
  enabled: boolean;
  onToggle: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onToggle}
      disabled={disabled}
      className={cn(
        "transition-colors disabled:opacity-40",
        enabled ? "text-emerald-400" : "text-zinc-500"
      )}
    >
      {enabled ? <ToggleRight className="w-8 h-8" /> : <ToggleLeft className="w-8 h-8" />}
    </button>
  );
}

function SettingRow({
  icon: Icon,
  title,
  description,
  children,
}: {
  icon: React.FC<{ className?: string }>;
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between py-4 border-b border-zinc-700/50 last:border-0 gap-4">
      <div className="flex items-start gap-3 min-w-0">
        <div className="p-1.5 rounded-lg bg-zinc-700/50 mt-0.5">
          <Icon className="w-4 h-4 text-zinc-300" />
        </div>
        <div className="min-w-0">
          <p className="text-sm font-medium text-white">{title}</p>
          <p className="text-xs text-zinc-400 mt-0.5">{description}</p>
        </div>
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}

function SelectSetting({
  value,
  options,
  onChange,
  disabled,
}: {
  value: string;
  options: { value: string; label: string }[];
  onChange: (v: string) => void;
  disabled?: boolean;
}) {
  return (
    <select
      value={value}
      disabled={disabled}
      onChange={(e) => onChange(e.target.value)}
      className="text-xs bg-zinc-900 border border-zinc-600 rounded-lg px-2 py-1.5 text-zinc-200 max-w-[160px]"
    >
      {options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  );
}

function SystemSettings() {
  const {
    autostart,
    zapretInstalled,
    zapretSettings,
    isLoading,
    releaseInfo,
    updateCheckError,
    setAutostart,
    setGameFilter,
    setIpsetMode,
    setZapretAutoUpdate,
    updateIpsetList,
    updateZapretHosts,
    checkUpdates,
    installZapret,
    applyUpdate,
    zapretMessage,
  } = useAppStore();

  const [isCheckingUpdates, setIsCheckingUpdates] = useState(false);
  const [isApplying, setIsApplying] = useState(false);
  const [updateMsg, setUpdateMsg] = useState<string | null>(null);

  const handleCheckUpdates = async () => {
    setIsCheckingUpdates(true);
    setUpdateMsg(null);
    try {
      await checkUpdates();
    } finally {
      setIsCheckingUpdates(false);
    }
  };

  const handleApplyUpdate = async () => {
    if (!releaseInfo?.has_update) return;
    setIsApplying(true);
    try {
      const msg = await applyUpdate(releaseInfo.download_url, releaseInfo.tag_name);
      setUpdateMsg(msg);
    } finally {
      setIsApplying(false);
    }
  };

  const gameValue =
    zapretSettings?.game_filter === "disabled" || !zapretSettings?.game_filter
      ? "disabled"
      : zapretSettings.game_filter;

  return (
    <div className="space-y-6">
      <div className="rounded-xl border border-zinc-700 bg-zinc-800/50 divide-y divide-zinc-700/50 px-5">
        <h2 className="text-sm font-semibold text-white pt-4 pb-1">Приложение</h2>
        <SettingRow
          icon={Power}
          title="Автозапуск"
          description="Запускать fastpatch при входе в Windows"
        >
          <Toggle enabled={autostart} onToggle={() => setAutostart(!autostart)} />
        </SettingRow>
        <SettingRow
          icon={Monitor}
          title="Запуск в фоне"
          description="При автозапуске сворачивать в трей"
        >
          <span className="text-xs text-zinc-400 italic">Всегда включено</span>
        </SettingRow>
      </div>

      <div className="rounded-xl border border-zinc-700 bg-zinc-800/50 px-5 pb-2">
        <h2 className="text-sm font-semibold text-white pt-4 pb-2 flex items-center gap-2">
          <Wrench className="w-4 h-4" />
          Zapret
        </h2>
        {!zapretInstalled ? (
          <div className="pb-4 space-y-3">
            <p className="text-xs text-amber-300">Zapret не установлен.</p>
            <button
              type="button"
              onClick={() => installZapret()}
              disabled={isLoading}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-sm text-white"
            >
              <Download className="w-4 h-4" />
              Установить zapret
            </button>
          </div>
        ) : (
          <>
            <SettingRow
              icon={Gamepad2}
              title="Игровой фильтр"
              description={zapretSettings?.game_filter_label ?? "—"}
            >
              <SelectSetting
                value={gameValue}
                disabled={isLoading}
                onChange={setGameFilter}
                options={[
                  { value: "disabled", label: "Выключен" },
                  { value: "all", label: "TCP и UDP" },
                  { value: "tcp", label: "Только TCP" },
                  { value: "udp", label: "Только UDP" },
                ]}
              />
            </SettingRow>
            <SettingRow
              icon={Filter}
              title="IPSet"
              description={zapretSettings?.ipset_label ?? "—"}
            >
              <SelectSetting
                value={zapretSettings?.ipset_mode ?? "any"}
                disabled={isLoading}
                onChange={setIpsetMode}
                options={[
                  { value: "loaded", label: "Загружен" },
                  { value: "none", label: "Отключён" },
                  { value: "any", label: "Любой" },
                ]}
              />
            </SettingRow>
            <SettingRow
              icon={RefreshCw}
              title="Автопроверка обновлений"
              description="При запуске .bat"
            >
              <Toggle
                enabled={zapretSettings?.auto_update_check ?? false}
                disabled={isLoading}
                onToggle={() =>
                  setZapretAutoUpdate(!(zapretSettings?.auto_update_check ?? false))
                }
              />
            </SettingRow>
            <div className="py-4 border-t border-zinc-700/50 space-y-2">
              <p className="text-xs text-zinc-500 uppercase tracking-wide">Обслуживание</p>
              {zapretMessage && (
                <div className="flex items-start gap-2 text-xs text-emerald-300 bg-emerald-500/10 border border-emerald-500/20 rounded-lg px-3 py-2">
                  <CheckCircle className="w-3.5 h-3.5 flex-shrink-0 mt-0.5" />
                  <span>{zapretMessage}</span>
                </div>
              )}
              <div className="flex flex-wrap gap-2">
                <button
                  type="button"
                  onClick={() => updateIpsetList()}
                  disabled={isLoading}
                  className="flex items-center gap-2 px-3 py-2 rounded-lg bg-zinc-700 hover:bg-zinc-600 text-xs text-white disabled:opacity-40"
                >
                  <List className="w-3.5 h-3.5" />
                  IPSet
                </button>
                <button
                  type="button"
                  onClick={() => updateZapretHosts()}
                  disabled={isLoading}
                  className="flex items-center gap-2 px-3 py-2 rounded-lg bg-zinc-700 hover:bg-zinc-600 text-xs text-white disabled:opacity-40"
                >
                  <Globe className="w-3.5 h-3.5" />
                  Hosts
                </button>
                <button
                  type="button"
                  onClick={handleCheckUpdates}
                  disabled={isCheckingUpdates}
                  className="flex items-center gap-2 px-3 py-2 rounded-lg bg-zinc-700 hover:bg-zinc-600 text-xs text-white disabled:opacity-40"
                >
                  <RefreshCw
                    className={cn("w-3.5 h-3.5", isCheckingUpdates && "animate-spin")}
                  />
                  Проверить релиз
                </button>
                <button
                  type="button"
                  onClick={() => installZapret()}
                  disabled={isLoading}
                  className="flex items-center gap-2 px-3 py-2 rounded-lg bg-zinc-700 hover:bg-zinc-600 text-xs text-white disabled:opacity-40"
                >
                  <Download className="w-3.5 h-3.5" />
                  Переустановить
                </button>
              </div>
            </div>
          </>
        )}
      </div>

      {releaseInfo && (
        <div className="rounded-xl border border-zinc-700 bg-zinc-800/50 p-5">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold text-white flex items-center gap-2">
              <Bell className="w-4 h-4 text-zinc-400" />
              Версия zapret
            </h2>
            <button
              type="button"
              onClick={handleCheckUpdates}
              disabled={isCheckingUpdates}
              className="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-700"
            >
              <RefreshCw className={cn("w-4 h-4", isCheckingUpdates && "animate-spin")} />
            </button>
          </div>
          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-zinc-400">Установлена</span>
              <span className="font-mono text-zinc-200">{releaseInfo.current_version}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-400">Последняя</span>
              <span
                className={cn(
                  "font-mono",
                  releaseInfo.has_update
                    ? "text-emerald-400"
                    : releaseInfo.tag_name
                      ? "text-zinc-200"
                      : "text-zinc-500"
                )}
              >
                {releaseInfo.tag_name || "—"}
              </span>
            </div>
          </div>
          {updateCheckError && (
            <p className="mt-2 text-xs text-red-300">{updateCheckError}</p>
          )}
          {releaseInfo.has_update && !updateMsg && (
            <button
              type="button"
              onClick={handleApplyUpdate}
              disabled={isApplying}
              className="w-full mt-3 flex items-center justify-center gap-2 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-sm text-white disabled:opacity-40"
            >
              {isApplying ? (
                <RefreshCw className="w-4 h-4 animate-spin" />
              ) : (
                <Download className="w-4 h-4" />
              )}
              Обновить zapret
            </button>
          )}
          {updateMsg && (
            <p className="mt-2 text-xs text-emerald-300">{updateMsg}</p>
          )}
        </div>
      )}

      <div className="rounded-xl border border-zinc-700 bg-zinc-800/50 p-5 space-y-4">
        <div>
          <h2 className="text-sm font-semibold text-white mb-3">fastpatch</h2>
          <div className="flex flex-col gap-2">
            <button
              type="button"
              onClick={() =>
                openUrl("https://github.com/nGrUnD/fastpatch/releases").catch(console.error)
              }
              className="flex items-center gap-2 text-xs text-zinc-400 hover:text-white"
            >
              <ExternalLink className="w-3.5 h-3.5" />
              Релизы fastpatch
            </button>
            <button
              type="button"
              onClick={() =>
                openUrl("https://github.com/nGrUnD/fastpatch").catch(console.error)
              }
              className="flex items-center gap-2 text-xs text-zinc-400 hover:text-white"
            >
              <Shield className="w-3.5 h-3.5 text-emerald-400/80" />
              GitHub — nGrUnD/fastpatch
            </button>
          </div>
        </div>
        <div className="border-t border-zinc-700/80 pt-4">
          <h3 className="text-xs font-medium text-zinc-500 mb-2">Zapret</h3>
          <div className="flex flex-col gap-2">
            <button
              type="button"
              onClick={() =>
                openUrl("https://github.com/flowseal/zapret-discord-youtube/releases").catch(
                  console.error
                )
              }
              className="flex items-center gap-2 text-xs text-zinc-400 hover:text-white"
            >
              <ExternalLink className="w-3.5 h-3.5" />
              Релизы zapret-discord-youtube
            </button>
            <button
              type="button"
              onClick={() =>
                openUrl("https://github.com/flowseal/zapret-discord-youtube").catch(
                  console.error
                )
              }
              className="flex items-center gap-2 text-xs text-zinc-400 hover:text-white"
            >
              <GitFork className="w-3.5 h-3.5" />
              GitHub — flowseal/zapret-discord-youtube
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export function SettingsPage() {
  const [tab, setTab] = useState<SettingsTab>("connection");

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="p-6 pb-0 shrink-0">
        <h1 className="text-xl font-bold text-white flex items-center gap-2">
          <Settings2 className="w-5 h-5 text-zinc-400" />
          Настройки
        </h1>
        <div className="flex gap-1 mt-4 border-b border-zinc-800">
          {TABS.map((t) => (
            <button
              key={t.id}
              type="button"
              onClick={() => setTab(t.id)}
              className={cn(
                "px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors",
                tab === t.id
                  ? "border-emerald-500 text-emerald-400"
                  : "border-transparent text-zinc-500 hover:text-zinc-300"
              )}
            >
              {t.label}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {tab === "connection" && (
          <ConnectionSettings onOpenStrategies={() => setTab("strategies")} />
        )}
        {tab === "strategies" && <StrategiesPage embedded />}
        {tab === "hosts" && <HostsPage embedded />}
        {tab === "system" && <SystemSettings />}
      </div>
    </div>
  );
}
