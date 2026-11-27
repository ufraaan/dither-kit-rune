// Injects the registry snippets into the landing page's code panels.
//   node scripts/build-site.mjs
// The page keeps working from file:// because the code is inlined, not fetched.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const registry = JSON.parse(readFileSync(join(root, "registry.json"), "utf8"));
const page = join(root, "www/index.html");
let html = readFileSync(page, "utf8");

for (const item of registry.items) {
  const source = readFileSync(join(root, item.snippet), "utf8").trimEnd();
  const escaped = source
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
  const marker = new RegExp(
    `<!-- code:${item.name} -->[\\s\\S]*?<!-- /code:${item.name} -->`,
  );
  if (!marker.test(html)) {
    console.error(`missing code panel for ${item.name}`);
    process.exit(1);
  }
  html = html.replace(
    marker,
    () => `<!-- code:${item.name} -->${escaped}<!-- /code:${item.name} -->`,
  );
}

writeFileSync(page, html);
console.log(`inlined ${registry.items.length} snippets into www/index.html`);
