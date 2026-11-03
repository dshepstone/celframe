import assert from "node:assert/strict";
import { test } from "node:test";
import { isAllowed, rejected } from "./check-js-licences.mjs";

test("accepts a listed licence", () => assert.equal(isAllowed("MIT"), true));
test("rejects an unlisted licence", () =>
  assert.equal(isAllowed("BlueOak-1.0.0"), false));
test("rejects an unknown licence", () =>
  assert.equal(isAllowed("Unknown"), false));
test("OR needs one allowed option", () => {
  assert.equal(isAllowed("(MIT OR CC-BY-3.0)"), true);
  assert.equal(isAllowed("CC-BY-3.0 OR Proprietary"), false);
});
test("AND needs every part allowed", () => {
  assert.equal(isAllowed("MIT AND Zlib"), true);
  assert.equal(isAllowed("MIT AND CC-BY-3.0"), false);
});
test("handles nested groups", () => {
  assert.equal(isAllowed("(MIT AND Zlib) OR CC-BY-3.0"), true);
  assert.equal(isAllowed("(MIT OR CC-BY-3.0) AND BlueOak-1.0.0"), false);
});
test("the dev-only licence passes only when allowed for that group", () => {
  const listing = { "BlueOak-1.0.0": [{ name: "tool" }] };
  assert.equal(rejected(listing).length, 1);
  assert.equal(rejected(listing, new Set(["BlueOak-1.0.0"])).length, 0);
});
