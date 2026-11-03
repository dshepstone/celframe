// Fails if a banned proprietary product name appears in tracked files.
// The patterns are encoded so that this file doesn't contain the names itself.
import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const ENCODED_PATTERNS = ["dG9vblxzKmJvb20=", "aGFybW9ueQ=="];

const PATTERNS = ENCODED_PATTERNS.map(
  (encoded) => new RegExp(Buffer.from(encoded, "base64").toString("utf8"), "i"),
);

// Lockfiles list third-party package names, which we don't choose the spelling of.
const SKIPPED = new Set(["Cargo.lock", "pnpm-lock.yaml"]);

export function containsBannedName(text) {
  return PATTERNS.some((pattern) => pattern.test(text));
}

function main() {
  const files = execSync("git ls-files", { encoding: "utf8" })
    .split("\n")
    .filter((file) => file && !SKIPPED.has(file));
  const hits = files.filter(
    (file) =>
      containsBannedName(file) ||
      containsBannedName(readFileSync(file, "utf8")),
  );
  if (hits.length > 0) {
    console.error(`Banned product name found in:\n${hits.join("\n")}`);
    console.error("See CLEANROOM.md.");
    process.exit(1);
  }
  console.log(`${files.length} files checked, no banned names.`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) main();
