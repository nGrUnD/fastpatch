import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import { splitInvokeError } from "@/lib/appErrors";
import { anyLoading, idleLoading, type LoadingState } from "@/stores/slices/loadingSlice";
import {
  initialNavigationState,
  type AppPage,
  type SettingsTab,
} from "@/stores/slices/navigationSlice";

export type { AppPage, SettingsTab } from "@/stores/slices/navigationSlice";
export type { LoadingState } from "@/stores/slices/loadingSlice";

export interface Strategy {
  id: string;
  name: string;
  description: string;
  tags: string[];
  source_bat: string;
  args: string;
}

export interface ActiveStrategy {
  id: string;
  name: string;
  pid?: number;
}

export interface TestResult {
  strategy_id: string;
  target: string;
  success: boolean;
  latency_ms?: number;
  error?: string;
}

export interface StrategyScanEntry {
  strategy_id: string;
  results: TestResult[];
  works: boolean;
}

export interface ScanAllResult {
  entries: StrategyScanEntry[];
  restored_previous: boolean;
  previous_name: string | null;
  cancelled: boolean;
}

export interface ScanProgress {
  current: number;
  total: number;
  current_id: string | null;
  current_name: string | null;
  elapsed_ms: number;
  avg_ms_per_strategy?: number;
  eta_ms?: number;
  finished: boolean;
}

export interface ReleaseInfo {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  download_url: string;
  current_version: string;
  has_update: boolean;
}

export type ZapretBackendPref = "v2" | "v1";

export interface ZapretStatus {
  installed: boolean;
  winws_path: string;
  zapret_dir: string;
  backend: ZapretBackendPref;
  v1_installed: boolean;
  v2_installed: boolean;
}

export interface ZapretSettings {
  game_filter: string;
  game_filter_label: string;
  ipset_mode: string;
  ipset_label: string;
  auto_update_check: boolean;
}

export interface ApexProbeResult {
  target: string;
  success: boolean;
  latency_ms?: number;
  error?: string;
}

export interface ApexTip {
  title: string;
  body: string;
  issue_url?: string;
}

export interface AppInfo {
  elevated: boolean;
  is_dev_build: boolean;
  from_autostart: boolean;
}

export interface AppPrefs {
  last_strategy_id: string | null;
  auto_connect_on_autostart: boolean;
  zapret_backend: ZapretBackendPref;
}

export interface ApexStatus {
  zapret_installed: boolean;
  v1_installed: boolean;
  v2_installed: boolean;
  list_installed: boolean;
  bat_installed: boolean;
  preset_v2_installed: boolean;
  strategy_available: boolean;
  game_filter: string;
  tips: ApexTip[];
}

interface AppState {
  page: AppPage;
  settingsTab: SettingsTab;
  appReady: boolean;
  strategies: Strategy[];
  strategyScan: Record<string, StrategyScanEntry>;
  scanProgress: ScanProgress | null;
  isScanning: boolean;
  winwsSessionHint: string | null;
  winwsBusyHint: string | null;
  activeStrategy: ActiveStrategy | null;
  isLoading: boolean;
  loading: LoadingState;
  error: string | null;
  releaseInfo: ReleaseInfo | null;
  updateCheckError: string | null;
  autostart: boolean;
  autoConnectOnAutostart: boolean;
  lastStrategyId: string | null;
  zapretInstalled: boolean;
  zapretInstalling: boolean;
  zapretMessage: string | null;
  zapretSettings: ZapretSettings | null;
  appInfo: AppInfo | null;
  zapretBackend: ZapretBackendPref;

  setPage: (page: AppPage) => void;
  setSettingsTab: (tab: SettingsTab) => void;
  openSettings: (tab?: SettingsTab) => void;
  bootstrapApp: () => Promise<void>;
  loadAppInfo: () => Promise<void>;
  relaunchAsAdmin: () => Promise<void>;
  loadZapretStatus: () => Promise<void>;
  loadZapretSettings: () => Promise<void>;
  installZapret: () => Promise<string | null>;
  setGameFilter: (mode: string) => Promise<void>;
  setIpsetMode: (mode: string) => Promise<void>;
  setZapretAutoUpdate: (enabled: boolean) => Promise<void>;
  updateIpsetList: () => Promise<void>;
  updateZapretHosts: () => Promise<void>;
  apexStatus: ApexStatus | null;
  loadApexStatus: () => Promise<void>;
  setupApexPreset: () => Promise<string | null>;
  testApexConnectivity: () => Promise<ApexProbeResult[]>;
  autoDetectApex: () => Promise<string | null>;
  loadStrategies: () => Promise<void>;
  loadActiveStrategy: () => Promise<void>;
  startStrategy: (id: string) => Promise<void>;
  stopStrategy: () => Promise<void>;
  killWinws: () => Promise<void>;
  testStrategy: (id: string) => Promise<TestResult[]>;
  scanAllStrategies: () => Promise<ScanAllResult>;
  loadScanProgress: () => Promise<void>;
  cancelStrategyScan: () => void;
  addCustomStrategy: (displayName: string, content: string) => Promise<Strategy>;
  testMediaConnectivity: () => Promise<TestResult[]>;
  autoDetect: () => Promise<string | null>;
  loadLocalVersion: () => Promise<void>;
  checkUpdates: () => Promise<void>;
  applyUpdate: (downloadUrl: string, tagName: string) => Promise<string>;
  loadAutostart: () => Promise<void>;
  setAutostart: (enabled: boolean) => Promise<void>;
  loadAppPrefs: () => Promise<void>;
  setAutoConnectOnAutostart: (enabled: boolean) => Promise<void>;
  setZapretBackend: (backend: ZapretBackendPref) => Promise<void>;
  clearError: () => void;
  clearWinwsBusyHint: () => void;
}

function setLoadingFlag(
  set: (partial: Partial<AppState>) => void,
  get: () => AppState,
  key: keyof LoadingState,
  value: boolean
) {
  const loading = { ...get().loading, [key]: value };
  set({ loading, isLoading: anyLoading(loading) });
}

export const useAppStore = create<AppState>((set, get) => ({
  ...initialNavigationState,
  appReady: false,
  strategies: [],
  strategyScan: {},
  scanProgress: null,
  isScanning: false,
  winwsSessionHint: null,
  winwsBusyHint: null,
  activeStrategy: null,
  isLoading: false,
  loading: idleLoading,
  error: null,
  releaseInfo: null,
  updateCheckError: null,
  autostart: false,
  autoConnectOnAutostart: true,
  lastStrategyId: null,
  zapretInstalled: false,
  zapretInstalling: false,
  zapretMessage: null,
  zapretSettings: null,
  apexStatus: null,
  appInfo: null,
  zapretBackend: "v2",

  setPage: (page) => set({ page }),
  setSettingsTab: (tab) => set({ settingsTab: tab }),
  openSettings: (tab = "connection") => set({ page: "settings", settingsTab: tab }),

  bootstrapApp: async () => {
    set({ appReady: false });
    const s = get();
    await Promise.all([
      s.loadAppInfo(),
      s.loadZapretStatus(),
      s.loadStrategies(),
      s.loadActiveStrategy(),
      s.loadZapretSettings(),
      s.loadApexStatus(),
      s.loadAutostart(),
      s.loadAppPrefs(),
      s.loadLocalVersion(),
    ]);
    const { appInfo, autoConnectOnAutostart } = get();
    if (appInfo?.from_autostart && autoConnectOnAutostart) {
      set({ isLoading: true, winwsSessionHint: "Автоподключение после автозапуска…" });
      try {
        const active = await invoke<ActiveStrategy | null>("try_autostart_connect");
        if (active) {
          set({ activeStrategy: active, zapretInstalled: true, error: null });
        } else {
          await get().loadActiveStrategy();
        }
      } catch (e) {
        set({ error: String(e) });
        await get().loadActiveStrategy();
      } finally {
        set({ isLoading: false, winwsSessionHint: null });
      }
    }
    set({ appReady: true });
  },

  loadAppInfo: async () => {
    try {
      const appInfo = await invoke<AppInfo>("get_app_info");
      set({ appInfo });
    } catch {
      set({ appInfo: null });
    }
  },

  relaunchAsAdmin: async () => {
    set({ error: null });
    try {
      await invoke("relaunch_as_admin");
    } catch (e) {
      set({ error: String(e) });
    }
  },

  installZapret: async () => {
    if (get().zapretInstalling) return null;
    setLoadingFlag(set, get, "installZapret", true);
    set({ zapretInstalling: true, error: null, zapretMessage: "Скачивание zapret с GitHub…" });
    try {
      const msg = await invoke<string>("install_zapret");
      set({ zapretMessage: msg, zapretInstalled: true });
      await get().loadZapretStatus();
      await get().loadStrategies();
      if (get().zapretBackend === "v1") {
        await get().loadZapretSettings();
      }
      await get().loadLocalVersion();
      return msg;
    } catch (e) {
      const err = String(e);
      set({ error: err, zapretMessage: null });
      return null;
    } finally {
      set({ zapretInstalling: false });
      setLoadingFlag(set, get, "installZapret", false);
    }
  },

  loadZapretStatus: async () => {
    try {
      const status = await invoke<ZapretStatus>("get_zapret_status");
      set({
        zapretInstalled: status.installed,
        zapretBackend: status.backend ?? get().zapretBackend,
      });
    } catch {
      set({ zapretInstalled: false });
    }
  },

  loadApexStatus: async () => {
    try {
      const apexStatus = await invoke<ApexStatus>("get_apex_status");
      set({ apexStatus });
    } catch {
      set({ apexStatus: null });
    }
  },

  setupApexPreset: async () => {
    setLoadingFlag(set, get, "apexSetup", true);
    set({ error: null, zapretMessage: null });
    try {
      const msg = await invoke<string>("setup_apex_preset");
      set({ zapretMessage: msg });
      await get().loadApexStatus();
      await get().loadStrategies();
      await get().loadZapretSettings();
      return msg;
    } catch (e) {
      set({ error: String(e) });
      return null;
    } finally {
      setLoadingFlag(set, get, "apexSetup", false);
    }
  },

  testApexConnectivity: async () => {
    try {
      return await invoke<ApexProbeResult[]>("test_apex_connectivity");
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  autoDetectApex: async () => {
    setLoadingFlag(set, get, "apexDetect", true);
    set({ error: null });
    try {
      const id = await invoke<string | null>("auto_detect_apex_strategy");
      if (id) {
        await get().loadActiveStrategy();
        set({ lastStrategyId: id });
      } else {
        set({
          zapretMessage:
            "ALT11 APEX не прошла проверку. Установите пресет Apex и подключите вручную.",
        });
      }
      return id;
    } catch (e) {
      set({ error: String(e) });
      return null;
    } finally {
      setLoadingFlag(set, get, "apexDetect", false);
    }
  },

  loadStrategies: async () => {
    try {
      const strategies = await invoke<Strategy[]>("get_strategies");
      set({ strategies });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadActiveStrategy: async () => {
    try {
      const active = await invoke<ActiveStrategy | null>("get_active_strategy");
      set({ activeStrategy: active });
    } catch {
      set({ activeStrategy: null });
    }
  },

  startStrategy: async (id) => {
    setLoadingFlag(set, get, "startStrategy", true);
    set({ error: null, winwsSessionHint: null, winwsBusyHint: null });
    try {
      await invoke("start_strategy", { id });
      await get().loadActiveStrategy();
      set({ zapretInstalled: true, lastStrategyId: id, winwsBusyHint: null });
    } catch (e) {
      set(splitInvokeError(e));
    } finally {
      setLoadingFlag(set, get, "startStrategy", false);
    }
  },

  killWinws: async () => {
    setLoadingFlag(set, get, "killWinws", true);
    set({ error: null });
    try {
      await invoke("kill_winws");
      await get().loadActiveStrategy();
      set({
        winwsBusyHint: null,
        error: null,
        activeStrategy: null,
      });
    } catch (e) {
      set(splitInvokeError(e));
    } finally {
      setLoadingFlag(set, get, "killWinws", false);
    }
  },

  stopStrategy: async () => {
    setLoadingFlag(set, get, "stopStrategy", true);
    set({ error: null });
    try {
      await invoke("stop_strategy");
      set({ activeStrategy: null });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      setLoadingFlag(set, get, "stopStrategy", false);
    }
  },

  testStrategy: async (id) => {
    try {
      return await invoke<TestResult[]>("test_strategy", { id });
    } catch (e) {
      set(splitInvokeError(e));
      throw e;
    }
  },

  scanAllStrategies: async () => {
    const prev = get().activeStrategy;
    set({
      isScanning: true,
      scanProgress: null,
      error: null,
      winwsSessionHint: prev
        ? `winws один на систему: на время скана отключим «${prev.name}», затем восстановим.`
        : null,
    });
    let progressTimer: ReturnType<typeof setInterval> | null = null;
    const pollProgress = () => {
      get().loadScanProgress().catch(console.error);
    };
    pollProgress();
    progressTimer = setInterval(pollProgress, 1000);
    try {
      const result = await invoke<ScanAllResult>("scan_all_strategies");
      const strategyScan = Object.fromEntries(
        result.entries.map((e) => [e.strategy_id, e])
      ) as Record<string, StrategyScanEntry>;
      let hint: string | null = null;
      if (result.cancelled) {
        hint = result.previous_name
          ? result.restored_previous
            ? `Скан остановлен. Снова подключено: ${result.previous_name}`
            : `Скан остановлен. Не удалось восстановить «${result.previous_name}».`
          : "Скан остановлен.";
      } else if (result.previous_name) {
        hint = result.restored_previous
          ? `Снова подключено: ${result.previous_name}`
          : `Не удалось восстановить «${result.previous_name}» — выберите стратегию вручную.`;
      }
      set({ strategyScan, winwsSessionHint: hint });
      await get().loadActiveStrategy();
      await get().loadScanProgress();
      return result;
    } catch (e) {
      set({ ...splitInvokeError(e), winwsSessionHint: null });
      throw e;
    } finally {
      if (progressTimer) clearInterval(progressTimer);
      set({ isScanning: false });
      await get().loadScanProgress();
    }
  },

  loadScanProgress: async () => {
    try {
      const scanProgress = await invoke<ScanProgress | null>("get_scan_progress");
      set({ scanProgress });
    } catch {
      set({ scanProgress: null });
    }
  },

  cancelStrategyScan: () => {
    invoke("cancel_strategy_scan").catch(console.error);
  },

  addCustomStrategy: async (displayName, content) => {
    setLoadingFlag(set, get, "settings", true);
    set({ error: null });
    try {
      const strategy = await invoke<Strategy>("add_custom_strategy", {
        displayName,
        content,
      });
      await get().loadStrategies();
      return strategy;
    } catch (e) {
      const msg = String(e);
      set({ error: msg });
      throw e;
    } finally {
      setLoadingFlag(set, get, "settings", false);
    }
  },

  testMediaConnectivity: async () => {
    try {
      return await invoke<TestResult[]>("test_media_connectivity");
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  autoDetect: async () => {
    const prev = get().activeStrategy;
    setLoadingFlag(set, get, "startStrategy", true);
    set({
      error: null,
      winwsSessionHint: prev
        ? `Подбор стратегии: временно отключим «${prev.name}».`
        : null,
    });
    try {
      const id = await invoke<string | null>("auto_detect_strategy");
      if (id) {
        await get().loadActiveStrategy();
        set({ lastStrategyId: id, winwsSessionHint: null });
      } else {
        set({
          error:
            get().zapretBackend === "v2"
              ? "Рабочий пресет не найден (Discord и YouTube < 3 с). Откройте «Расширенные» и выберите пресет вручную."
              : "Рабочая стратегия не найдена (нужны Discord и YouTube < 3 с). Попробуйте ALT9 или ALT11 вручную.",
          winwsSessionHint: prev
            ? get().activeStrategy
              ? `Восстановлено: ${prev.name}`
              : `Не удалось восстановить «${prev.name}».`
            : null,
        });
        await get().loadActiveStrategy();
      }
      return id;
    } catch (e) {
      set({ ...splitInvokeError(e), winwsSessionHint: null });
      return null;
    } finally {
      setLoadingFlag(set, get, "startStrategy", false);
    }
  },

  loadLocalVersion: async () => {
    try {
      const current = await invoke<string>("get_current_version");
      set({
        releaseInfo: {
          tag_name: "",
          name: "",
          body: "",
          published_at: "",
          download_url: "",
          current_version: current,
          has_update: false,
        },
        updateCheckError: null,
      });
    } catch {
      // локальная версия недоступна — не показываем ошибку
    }
  },

  checkUpdates: async () => {
    setLoadingFlag(set, get, "updates", true);
    set({ updateCheckError: null });
    try {
      const info = await invoke<ReleaseInfo>("check_for_updates");
      set({ releaseInfo: info, updateCheckError: null });
    } catch (e) {
      set({ updateCheckError: String(e) });
    } finally {
      setLoadingFlag(set, get, "updates", false);
    }
  },

  applyUpdate: async (downloadUrl, tagName) => {
    setLoadingFlag(set, get, "updates", true);
    set({ error: null });
    try {
      const msg = await invoke<string>("apply_update", {
        downloadUrl,
        tagName,
      });
      await get().loadZapretStatus();
      return msg;
    } catch (e) {
      const msg = String(e);
      set({ error: msg });
      throw e;
    } finally {
      setLoadingFlag(set, get, "updates", false);
    }
  },

  loadAutostart: async () => {
    try {
      const enabled = await invoke<boolean>("get_autostart_enabled");
      set({ autostart: enabled });
    } catch {
      set({ autostart: false });
    }
  },

  setAutostart: async (enabled) => {
    try {
      await invoke("set_autostart_enabled", { enabled });
      set({ autostart: enabled });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadAppPrefs: async () => {
    try {
      const prefs = await invoke<AppPrefs>("get_app_prefs");
      set({
        autoConnectOnAutostart: prefs.auto_connect_on_autostart,
        lastStrategyId: prefs.last_strategy_id,
        zapretBackend: prefs.zapret_backend ?? "v2",
      });
    } catch {
      set({ autoConnectOnAutostart: true, lastStrategyId: null, zapretBackend: "v2" });
    }
  },

  setZapretBackend: async (backend) => {
    setLoadingFlag(set, get, "settings", true);
    set({ error: null, activeStrategy: null });
    try {
      await invoke("set_zapret_backend", { backend });
      set({ zapretBackend: backend });
      await get().loadZapretStatus();
      await get().loadStrategies();
      await get().loadActiveStrategy();
      if (backend === "v1") {
        await get().loadZapretSettings();
        await get().loadApexStatus();
      }
    } catch (e) {
      set(splitInvokeError(e));
    } finally {
      setLoadingFlag(set, get, "settings", false);
    }
  },

  setAutoConnectOnAutostart: async (enabled) => {
    try {
      await invoke("set_auto_connect_on_autostart", { enabled });
      set({ autoConnectOnAutostart: enabled });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadZapretSettings: async () => {
    try {
      const settings = await invoke<ZapretSettings>("get_zapret_settings");
      set({ zapretSettings: settings });
    } catch {
      set({ zapretSettings: null });
    }
  },

  setGameFilter: async (mode) => {
    try {
      const msg = await invoke<string>("set_game_filter", { mode });
      set({ zapretMessage: msg });
      await get().loadZapretSettings();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setIpsetMode: async (mode) => {
    try {
      const msg = await invoke<string>("set_ipset_mode", { mode });
      set({ zapretMessage: msg });
      await get().loadZapretSettings();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setZapretAutoUpdate: async (enabled) => {
    try {
      const msg = await invoke<string>("set_auto_update_check", { enabled });
      set({ zapretMessage: msg });
      await get().loadZapretSettings();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateIpsetList: async () => {
    setLoadingFlag(set, get, "hosts", true);
    set({ error: null });
    try {
      const msg = await invoke<string>("update_ipset_list");
      set({ zapretMessage: msg });
      await get().loadZapretSettings();
    } catch (e) {
      set({ error: String(e) });
    } finally {
      setLoadingFlag(set, get, "hosts", false);
    }
  },

  updateZapretHosts: async () => {
    setLoadingFlag(set, get, "hosts", true);
    set({ error: null, zapretMessage: null });
    try {
      const msg = await invoke<string>("update_zapret_hosts_file");
      set({ zapretMessage: msg, error: null });
    } catch (e) {
      set({ error: String(e), zapretMessage: null });
    } finally {
      setLoadingFlag(set, get, "hosts", false);
    }
  },

  clearError: () => set({ error: null }),
  clearWinwsBusyHint: () => set({ winwsBusyHint: null }),
}));
