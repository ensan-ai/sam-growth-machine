import { describe, expect, it } from "vitest";
import { newestFirst, nextFeedScrollTop } from "./chrono";

describe("newestFirst", () => {
  it("puts the newest timestamp at the top", () => {
    const items = [
      { id: "old", createdAt: "2026-01-01T10:00:00Z" },
      { id: "new", createdAt: "2026-01-01T12:00:00Z" },
      { id: "mid", createdAt: "2026-01-01T11:00:00Z" },
    ];
    expect(newestFirst(items, (item) => item.createdAt).map((item) => item.id)).toEqual(["new", "mid", "old"]);
  });
});

describe("nextFeedScrollTop", () => {
  it("keeps the newest item visible when the viewer is at the top", () => {
    expect(nextFeedScrollTop(true, { height: 400, top: 0 }, 520)).toBe(0);
  });

  it("does not jump to the top when the viewer is inspecting older history", () => {
    expect(nextFeedScrollTop(false, { height: 400, top: 180 }, 520)).toBe(300);
  });
});
