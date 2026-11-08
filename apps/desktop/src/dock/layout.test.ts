import { describe, expect, it } from "vitest";
import {
  type DockLayout,
  defaultLayout,
  findPanel,
  moveTab,
  parseLayout,
  resizeStacks,
  showPanel,
} from "./layout";

const tabs = (layout: DockLayout, area: "left" | "right" | "bottom") =>
  layout.areas[area].map((s) => s.tabs);

describe("dock layout", () => {
  it("starts with the default arrangement", () => {
    const layout = defaultLayout();
    expect(tabs(layout, "right")).toEqual([
      ["tool", "camera"],
      ["colour"],
      ["layers", "xsheet"],
    ]);
    expect(tabs(layout, "bottom")).toEqual([["timeline"]]);
    expect(tabs(layout, "left")).toEqual([]);
  });

  it("moves a tab into another stack and makes it active there", () => {
    const layout = moveTab(defaultLayout(), "colour", {
      kind: "tabs",
      area: "right",
      stack: "right-layers",
      index: 0,
    });
    expect(tabs(layout, "right")).toEqual([
      ["tool", "camera"],
      ["colour", "layers", "xsheet"],
    ]);
    expect(findPanel(layout, "colour")?.stack.active).toBe("colour");
  });

  it("reorders within a stack, allowing for the tab's own place", () => {
    let layout = moveTab(defaultLayout(), "colour", {
      kind: "tabs",
      area: "right",
      stack: "right-layers",
      index: 1,
    });
    expect(tabs(layout, "right")[1]).toEqual(["layers", "colour", "xsheet"]);
    layout = moveTab(layout, "layers", {
      kind: "tabs",
      area: "right",
      stack: "right-layers",
      index: 2,
    });
    expect(tabs(layout, "right")[1]).toEqual(["colour", "layers", "xsheet"]);
  });

  it("splits a stack, sharing its space", () => {
    const layout = moveTab(defaultLayout(), "timeline", {
      kind: "split",
      area: "right",
      stack: "right-tool",
      after: true,
    });
    expect(tabs(layout, "right")).toEqual([
      ["tool", "camera"],
      ["timeline"],
      ["colour"],
      ["layers", "xsheet"],
    ]);
    expect(layout.areas.right[0]?.weight).toBeCloseTo(0.7);
    expect(layout.areas.right[1]?.weight).toBeCloseTo(0.7);
    expect(tabs(layout, "bottom")).toEqual([]);
  });

  it("opens an empty area", () => {
    const layout = moveTab(defaultLayout(), "layers", {
      kind: "area",
      area: "left",
    });
    expect(tabs(layout, "left")).toEqual([["layers"]]);
  });

  it("leaves a lone tab dropped on its own stack where it is", () => {
    const layout = defaultLayout();
    expect(
      moveTab(layout, "colour", {
        kind: "split",
        area: "right",
        stack: "right-colour",
        after: false,
      }),
    ).toBe(layout);
  });

  it("shows a panel that isn't in the layout on the right", () => {
    const missing = moveTab(defaultLayout(), "timeline", {
      kind: "tabs",
      area: "right",
      stack: "right-tool",
      index: 1,
    });
    expect(findPanel(missing, "timeline")?.area).toBe("right");
    const removed: DockLayout = {
      ...missing,
      areas: {
        ...missing.areas,
        right: missing.areas.right.map((s) => ({
          ...s,
          tabs: s.tabs.filter((t) => t !== "timeline"),
          active: s.tabs[0] === "timeline" ? "tool" : s.active,
        })),
      },
    };
    expect(findPanel(removed, "timeline")).toBeNull();
    expect(findPanel(showPanel(removed, "timeline"), "timeline")?.area).toBe(
      "right",
    );
  });

  it("resizes neighbouring stacks without making either too small", () => {
    const layout = resizeStacks(defaultLayout(), "right", 0, 10_000, 600);
    const [tool, colour] = layout.areas.right;
    const total = 1.4 + 0.6 + 1;
    expect((colour?.weight ?? 0) / total).toBeCloseTo(80 / 600);
    expect((tool?.weight ?? 0) + (colour?.weight ?? 0)).toBeCloseTo(2);
  });

  it("reads stored layouts, dropping anything unusable", () => {
    expect(parseLayout(null)).toEqual(defaultLayout());
    expect(parseLayout("nonsense")).toEqual(defaultLayout());
    const stored = JSON.stringify({
      areas: {
        left: [{ id: "a", tabs: ["layers", "gone", "layers"], weight: -1 }],
        right: [{ id: "b", tabs: ["tool", "colour"], active: "colour" }],
        bottom: "nope",
      },
      sizes: { left: 9999, right: "wide" },
    });
    const layout = parseLayout(stored);
    // The exposure sheet, missing from what was stored, joins its default neighbour.
    expect(tabs(layout, "left")).toEqual([["layers", "xsheet"]]);
    expect(layout.areas.left[0]?.weight).toBe(1);
    expect(layout.areas.right[0]?.active).toBe("colour");
    // Panels missing from what was stored go back to their default places.
    expect(tabs(layout, "bottom")).toEqual([["timeline"]]);
    expect(layout.sizes.left).toBe(520);
    expect(layout.sizes.right).toBe(272);
  });

  it("adds panels a stored layout doesn't know about beside their default neighbours", () => {
    const stored = JSON.stringify({
      version: 2,
      areas: {
        right: [
          { id: "r", tabs: ["tool", "colour", "layers"], active: "tool" },
        ],
        bottom: [{ id: "b", tabs: ["timeline"] }],
      },
    });
    const layout = parseLayout(stored);
    expect(tabs(layout, "right")).toEqual([
      ["tool", "colour", "layers", "xsheet", "camera"],
    ]);
    expect(layout.areas.right[0]?.active).toBe("tool");
  });

  it("moves the exposure sheet out of layouts stored before it moved", () => {
    const stored = JSON.stringify({
      areas: {
        right: [
          { id: "r", tabs: ["tool"] },
          { id: "c", tabs: ["colour"] },
          { id: "l", tabs: ["layers"] },
        ],
        bottom: [{ id: "b", tabs: ["timeline", "xsheet"], active: "xsheet" }],
      },
    });
    const layout = parseLayout(stored);
    expect(tabs(layout, "right")).toEqual([
      ["tool", "camera"],
      ["colour"],
      ["layers", "xsheet"],
    ]);
    expect(tabs(layout, "bottom")).toEqual([["timeline"]]);
    expect(layout.version).toBe(2);
  });
});
