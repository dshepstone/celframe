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

// Allowed only for tools that never ship in the app. See CLEANROOM.md.
const DEV_ONLY_ALLOWED = new Set(["BlueOak-1.0.0"]);

/** Handles SPDX "OR" and "AND" expressions, including one level of parentheses. */
export function isAllowed(expression, extra = new Set()) {
  const text = expression.trim();
  const orParts = splitTopLevel(text, " OR ");
  if (orParts.length > 1) return orParts.some((part) => isAllowed(part, extra));
  const andParts = splitTopLevel(text, " AND ");
  if (andParts.length > 1)
    return andParts.every((part) => isAllowed(part, extra));
  if (text.startsWith("(") && text.endsWith(")"))
    return isAllowed(text.slice(1, -1), extra);
  return ALLOWED.has(text) || extra.has(text);
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

export function rejected(listing, extra = new Set()) {
  return Object.entries(listing).filter(
    ([licence]) => !isAllowed(licence, extra),
  );
}

function list(flags) {
  const output = execSync(`pnpm -r licenses list --json ${flags}`, {
    encoding: "utf8",
  });
  return output.trim().startsWith("{") ? JSON.parse(output) : {};
}

function main() {
  const shipped = list("--prod");
  const all = list("");
  const failures = [
    ...rejected(shipped).map((entry) => ["shipped", ...entry]),
    ...rejected(all, DEV_ONLY_ALLOWED).map((entry) => ["dev", ...entry]),
  ];
  for (const [group, licence, packages] of failures) {
    console.error(
      `${group}: ${licence}: ${packages.map((p) => p.name).join(", ")}`,
    );
  }
  if (failures.length > 0) {
    console.error("These licences aren't allowed. See CLEANROOM.md.");
    process.exit(1);
  }
  const count = (listing) => Object.values(listing).flat().length;
  console.log(
    `Licences OK: ${count(shipped)} shipped packages, ${count(all)} in total.`,
  );
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) main();
