import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("styles.css", () => {
  it("tells Tailwind to scan the components that apps import from node_modules", () => {
    const css = readFileSync(new URL("./styles.css", import.meta.url), "utf8");
    // Without this, every class used only inside @celframe/ui is missing from the apps.
    expect(css).toMatch(/@source\s+"\.\/";/);
  });

  it("leaves the page background transparent so the native canvas shows through", () => {
    const css = readFileSync(new URL("./styles.css", import.meta.url), "utf8");
    const body = /\bbody\s*\{([^}]*)\}/.exec(css)?.[1] ?? "";
    expect(body).not.toMatch(/background\s*:/);
  });
});
