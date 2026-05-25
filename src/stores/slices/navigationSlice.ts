export type AppPage = "home" | "settings";
export type SettingsTab = "connection" | "strategies" | "games" | "advanced";

export interface NavigationState {
  page: AppPage;
  settingsTab: SettingsTab;
}

export const initialNavigationState: NavigationState = {
  page: "home",
  settingsTab: "connection",
};
