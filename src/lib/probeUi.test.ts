import { describe, expect, it } from "vitest";
import { visibleTags } from "@/components/TagBadge";
import { probeHasConnectivity, scoreConnectivityProbe } from "@/lib/probeUi";
import type { TestResult } from "@/stores/appStore";

function result(target: string, success = true, latency_ms = 100): TestResult {
  return { strategy_id: "s", target, success, latency_ms };
}

describe("probeUi", () => {
  it("treats slow probes as unavailable", () => {
    expect(probeHasConnectivity(result("discord", true, 2999))).toBe(true);
    expect(probeHasConnectivity(result("discord", true, 3000))).toBe(false);
    expect(probeHasConnectivity(result("discord", false, 100))).toBe(false);
  });

  it("scores Discord and YouTube by required targets", () => {
    const ok = [
      result("discord_gw"),
      result("discord_upd"),
      result("discord_cdn"),
      result("youtube"),
      result("youtube_gen"),
    ];

    expect(scoreConnectivityProbe(ok, ["discord", "youtube"]).warnings).toEqual([]);

    const broken = ok.filter((r) => r.target !== "discord_upd");
    expect(scoreConnectivityProbe(broken, ["discord", "youtube"]).warnings).toContain(
      "Discord недоступен."
    );
  });

  it("adds a note for game-only UDP checks", () => {
    const score = scoreConnectivityProbe([], ["games"]);
    expect(score.byTag.games).toBe(true);
    expect(score.notes[0]).toContain("HTTP-проверка не выполняется");
  });

  it("hides technical tags and keeps semantic order", () => {
    expect(visibleTags(["preset", "youtube", "recommended", "zapret2", "apex"])).toEqual([
      "recommended",
      "apex",
      "youtube",
    ]);
  });
});
