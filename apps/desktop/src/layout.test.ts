import { describe, expect, it } from "vitest";
import { SIDE_PANEL_MAX, SIDE_PANEL_MIN, readLayout } from "./store";

describe("remembered layout", () => {
  it("falls back to the default when nothing usable is stored", () => {
    for (const stored of [null, "nope", "[]", '{"sidePanelWidth":"wide"}']) {
      const layout = readLayout(stored);
      expect(layout.collapsedPanels).toEqual([]);
      expect(layout.sidePanelWidth).toBeGreaterThanOrEqual(SIDE_PANEL_MIN);
    }
  });

  it("keeps the width in range and drops bad panel names", () => {
    const layout = readLayout(
      JSON.stringify({ sidePanelWidth: 9999, collapsedPanels: ["colour", 3] }),
    );
    expect(layout).toEqual({
      sidePanelWidth: SIDE_PANEL_MAX,
      collapsedPanels: ["colour"],
    });
  });
});
