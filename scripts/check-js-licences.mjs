// Fails if any installed JS package has a licence outside the allowlist in CLEANROOM.md.
// Keep this list in sync with deny.toml.
import { execSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const ALLOWED = new Set([
  "Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "BSL-1.0",
  "CC0-1.0",
  "GPL-3.0-or-later",
  "ISC",
  "LGPL-2.1-or-later",
  "LGPL-3.0-or-later",
  "MIT",
  "MPL-2.0",
  "Unicode-3.0",
  "Zlib",
]);

/** Handles SPDX "OR" and "AND" expressions, including one level of parentheses. */
export function isAllowed(expression) {
  const text = expression.trim();
  const orParts = splitTopLevel(text, " OR ");
  if (orParts.length > 1) return orParts.some(isAllowed);
  const andParts = splitTopLevel(text, " AND ");
  if (andParts.length > 1) return andParts.every(isAllowed);
  if (text.startsWith("(") && text.endsWith(")"))
    return isAllowed(text.slice(1, -1));
  return ALLOWED.has(text);
}

function splitTopLevel(text, separator) {
  const parts = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < text.length; i++) {
    if (text[i] === "(") depth++;
    if (text[i] === ")") depth--;
    if (depth === 0 && text.startsWith(separator, i)) {
      parts.push(text.slice(start, i));
      start = i + separator.length;
    }
  }
  parts.push(text.slice(start));
  return parts;
}

function main() {
  const listing = JSON.parse(
    execSync("pnpm licenses list --json", { encoding: "utf8" }),
  );
  const rejected = Object.entries(listing).filter(
    ([licence]) => !isAllowed(licence),
  );
  const total = Object.values(listing).flat().length;
  for (const [licence, packages] of rejected) {
    console.error(`${licence}: ${packages.map((p) => p.name).join(", ")}`);
  }
  if (rejected.length > 0) {
    console.error("These licences aren't on the allowlist. See CLEANROOM.md.");
    process.exit(1);
  }
  console.log(`${total} JS packages checked, all licences allowed.`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) main();
