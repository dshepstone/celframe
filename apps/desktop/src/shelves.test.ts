import { describe, expect, it } from "vitest";
import {
  DEFAULT_SHELVES,
  moveAction,
  nextShelfName,
  parseShelves,
} from "./shelves";

const known = new Set(["undo", "redo", "fit"]);

describe("shelves", () => {
  it("falls back to the defaults when nothing usable is stored", () => {
    for (const stored of [null, "not json", "{}", "[]", '[{"id":1}]']) {
      expect(parseShelves(stored, known)).toBe(DEFAULT_SHELVES);
    }
  });

  it("drops unknown and repeated actions", () => {
    const stored = JSON.stringify([
      { id: "a", name: "Mine", actions: ["undo", "gone", "undo", 7, "fit"] },
    ]);
    expect(parseShelves(stored, known)).toEqual([
      { id: "a", name: "Mine", actions: ["undo", "fit"] },
    ]);
  });

  it("names new shelves without clashing", () => {
    expect(nextShelfName([{ id: "x", name: "Shelf 2", actions: [] }])).toBe(
      "Shelf 3",
    );
  });

  it("moves buttons within the shelf only", () => {
    expect(moveAction(["a", "b", "c"], 0, 1)).toEqual(["b", "a", "c"]);
    expect(moveAction(["a", "b", "c"], 2, 1)).toEqual(["a", "b", "c"]);
    expect(moveAction(["a", "b", "c"], 0, -1)).toEqual(["a", "b", "c"]);
  });
});
