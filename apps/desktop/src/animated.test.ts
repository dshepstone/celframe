import { describe, expect, it } from "vitest";
import { valueAt } from "./animated";

describe("valueAt", () => {
  it("matches the core's interpolation", () => {
    const value = {
      value: 1,
      keys: [
        { frame: 10, value: 0, interpolation: "linear" as const },
        { frame: 20, value: 1, interpolation: "ease" as const },
        { frame: 30, value: 0, interpolation: "step" as const },
        { frame: 40, value: 0.5, interpolation: "linear" as const },
      ],
    };
    expect(valueAt(value, 0)).toBe(0);
    expect(valueAt(value, 15)).toBe(0.5);
    expect(valueAt(value, 22)).toBeCloseTo(0.896);
    expect(valueAt(value, 39)).toBe(0);
    expect(valueAt(value, 99)).toBe(0.5);
    expect(valueAt({ value: 0.3, keys: [] }, 5)).toBe(0.3);
  });
});
