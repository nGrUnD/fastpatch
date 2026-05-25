import { useEffect } from "react";
import { LoadingScreen } from "@/components/LoadingScreen";
import { Sidebar } from "@/components/Sidebar";
import { StatusBannerStack } from "@/components/StatusBannerStack";
import { HomePage } from "@/pages/HomePage";
import { SettingsPage } from "@/pages/SettingsPage";
import { useAppStore } from "@/stores/appStore";

export default function App() {
  const { page, appReady, bootstrapApp } = useAppStore();

  useEffect(() => {
    bootstrapApp();
  }, [bootstrapApp]);

  if (!appReady) {
    return <LoadingScreen />;
  }

  return (
    <div className="flex h-screen w-screen bg-zinc-950 text-white overflow-hidden select-none">
      <Sidebar />
      <main className="flex-1 flex flex-col overflow-hidden">
        <StatusBannerStack />
        <div className="flex-1 overflow-hidden">
          {page === "home" && <HomePage />}
          {page === "settings" && <SettingsPage />}
        </div>
      </main>
    </div>
  );
}
