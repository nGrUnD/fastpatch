import { Loader2, Shield } from "lucide-react";

export function LoadingScreen() {
  return (
    <div className="flex h-screen w-screen flex-col items-center justify-center bg-zinc-950 text-white select-none">
      <div className="flex flex-col items-center gap-6">
        <div className="flex h-20 w-20 items-center justify-center rounded-2xl bg-zinc-800/80 ring-1 ring-zinc-700">
          <Shield className="h-10 w-10 text-emerald-400" />
        </div>
        <div className="text-center space-y-2">
          <h1 className="text-2xl font-bold tracking-tight">fastpatch</h1>
          <p className="text-sm text-zinc-500">Подготовка…</p>
        </div>
        <Loader2 className="h-6 w-6 animate-spin text-emerald-400/80" aria-hidden />
      </div>
    </div>
  );
}
