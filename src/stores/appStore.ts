import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

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

export interface ReleaseInfo {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  download_url: string;
  current_version: string;
  has_update: boolean;
}

export interface ZapretStatus {
  installed: boolean;
  winws_path: string;
  zapret_dir: string;
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
}

export interface ApexStatus {
  zapret_installed: boolean;
  list_installed: boolean;
  bat_installed: boolean;
  strategy_available: boolean;
  game_filter: string;
  tips: ApexTip[];
}

export type AppPage = "home" | "settings";

interface AppState {
  page: AppPage;
  appReady: boolean;
  strategies: Strategy[];
  strategyScan: Record<string, StrategyScanEntry>;
  isScanning: boolean;
  winwsSessionHint: string | null;
  activeStrategy: ActiveStrategy | null;
  isLoading: boolean;
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

  setPage: (page: AppPage) => void;
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
  testStrategy: (id: string) => Promise<TestResult[]>;
  scanAllStrategies: () => Promise<ScanAllResult>;
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
  clearError: () => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  page: "home",
  appReady: false,
  strategies: [],
  strategyScan: {},
  isScanning: false,
  winwsSessionHint: null,
  activeStrategy: null,
  isLoading: false,
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

  setPage: (page) => set({ page }),

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
    set({ zapretInstalling: true, error: null, zapretMessage: "Скачивание zapret с GitHub…" });
    try {
      const msg = await invoke<string>("install_zapret");
      set({ zapretMessage: msg, zapretInstalled: true });
      await get().loadZapretStatus();
      await get().loadStrategies();
      await get().loadZapretSettings();
      await get().loadLocalVersion();
      return msg;
    } catch (e) {
      const err = String(e);
      set({ error: err, zapretMessage: null });
      return null;
    } finally {
      set({ zapretInstalling: false });
    }
  },

  loadZapretStatus: async () => {
    try {
      const status = await invoke<ZapretStatus>("get_zapret_status");
      set({ zapretInstalled: status.installed });
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
    set({ isLoading: true, error: null, zapretMessage: null });
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
      set({ isLoading: false });
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
    set({ isLoading: true, error: null });
    try {
      const id = await invoke<string | null>("auto_detect_apex_strategy");
      if (id) {
        await get().loadActiveStrategy();
        set({ lastStrategyId: id });
      } else {
        set({
          zapretMessage:
            "Среди Apex-стратегий рабочая не найдена. Попробуйте general (APEX) или ALT11 вручную.",
        });
      }
      return id;
    } catch (e) {
      set({ error: String(e) });
      return null;
    } finally {
      set({ isLoading: false });
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
    set({ isLoading: true, error: null, winwsSessionHint: null });
    try {
      await invoke("start_strategy", { id });
      await get().loadActiveStrategy();
      set({ zapretInstalled: true, lastStrategyId: id });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  stopStrategy: async () => {
    set({ isLoading: true, error: null });
    try {
      await invoke("stop_strategy");
      set({ activeStrategy: null });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  testStrategy: async (id) => {
    try {
      return await invoke<TestResult[]>("test_strategy", { id });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  scanAllStrategies: async () => {
    const prev = get().activeStrategy;
    set({
      isScanning: true,
      error: null,
      winwsSessionHint: prev
        ? `winws один на систему: на время скана отключим «${prev.name}», затем восстановим.`
        : null,
    });
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
      return result;
    } catch (e) {
      set({ error: String(e), winwsSessionHint: null });
      throw e;
    } finally {
      set({ isScanning: false });
    }
  },

  cancelStrategyScan: () => {
    invoke("cancel_strategy_scan").catch(console.error);
  },

  addCustomStrategy: async (displayName, content) => {
    set({ isLoading: true, error: null });
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
      set({ isLoading: false });
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
    set({
      isLoading: true,
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
            "Рабочая стратегия не найдена (нужны Discord и YouTube < 3 с). Попробуйте ALT9 или ALT11 вручную.",
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
      set({ error: String(e), winwsSessionHint: null });
      return null;
    } finally {
      set({ isLoading: false });
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
    set({ isLoading: true, updateCheckError: null });
    try {
      const info = await invoke<ReleaseInfo>("check_for_updates");
      set({ releaseInfo: info, updateCheckError: null });
    } catch (e) {
      set({ updateCheckError: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  applyUpdate: async (downloadUrl, tagName) => {
    set({ isLoading: true, error: null });
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
      set({ isLoading: false });
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
      });
    } catch {
      set({ autoConnectOnAutostart: true, lastStrategyId: null });
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
    set({ isLoading: true, error: null });
    try {
      const msg = await invoke<string>("update_ipset_list");
      set({ zapretMessage: msg });
      await get().loadZapretSettings();
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  updateZapretHosts: async () => {
    set({ isLoading: true, error: null, zapretMessage: null });
    try {
      const msg = await invoke<string>("update_zapret_hosts_file");
      set({ zapretMessage: msg, error: null });
    } catch (e) {
      set({ error: String(e), zapretMessage: null });
    } finally {
      set({ isLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
