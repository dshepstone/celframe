import assert from "node:assert/strict";
import { test } from "node:test";
import { containsBannedName } from "./check-clean-room.mjs";

const decode = (encoded) => Buffer.from(encoded, "base64").toString("utf8");

test("finds banned names in any case and spacing", () => {
  assert.equal(containsBannedName(decode("VE9PTiAgQm9vbQ==")), true);
  assert.equal(containsBannedName(decode("dG9vbmJvb20=")), true);
  assert.equal(containsBannedName(decode("SEFSTU9OWQ==")), true);
});
test("allows ordinary text", () => {
  assert.equal(containsBannedName("cartoon timeline boom mic"), false);
});
