import { cn } from "@/lib/utils";

const TAG_COLORS: Record<string, string> = {
  discord: "bg-indigo-500/20 text-indigo-300 border-indigo-500/30",
  youtube: "bg-red-500/20 text-red-300 border-red-500/30",
  cloudflare: "bg-sky-500/20 text-sky-300 border-sky-500/30",
  games: "bg-purple-500/20 text-purple-300 border-purple-500/30",
  apex: "bg-orange-500/20 text-orange-300 border-orange-500/30",
  general: "bg-zinc-500/20 text-zinc-300 border-zinc-500/30",
};

const TAG_LABELS: Record<string, string> = {
  discord: "Discord",
  youtube: "YouTube",
  cloudflare: "Cloudflare CDN",
  games: "Игры",
  apex: "Apex Legends",
  general: "Общий",
};

interface TagBadgeProps {
  tag: string;
  active?: boolean;
  onClick?: () => void;
  className?: string;
  count?: number;
}

export function TagBadge({ tag, active, onClick, className, count }: TagBadgeProps) {
  const color = TAG_COLORS[tag] ?? "bg-zinc-600/20 text-zinc-400 border-zinc-600/30";
  return (
    <span
      onClick={onClick}
      className={cn(
        "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium border",
        color,
        onClick && "cursor-pointer select-none",
        active && "ring-1 ring-white/20",
        className
      )}
    >
      {TAG_LABELS[tag] ?? tag}
      {count !== undefined && (
        <span className="rounded-full px-1 py-0.5 text-[10px] font-semibold bg-black/20">
          {count}
        </span>
      )}
    </span>
  );
}

export const ALL_TAGS = ["discord", "youtube", "cloudflare", "games", "apex", "general"];
