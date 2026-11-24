#!/usr/bin/env node
// Rebrands this fork from cockpit to flightdeck, so both install and run side by side.
//
//   node scripts/rebrand-flightdeck.mjs paths     git mv every tracked path that names cockpit
//   node scripts/rebrand-flightdeck.mjs content   rewrite the text of every tracked file
//   node scripts/rebrand-flightdeck.mjs all       paths, then content
//   add --dry-run to list what would change without touching anything
//
// Both phases are idempotent: a second run finds nothing left to change.
// Case is kept (cockpit -> flightdeck, Cockpit -> Flightdeck, COCKPIT -> FLIGHTDECK), and the upstream repo and
// website URLs map to this fork's. Files that must keep upstream's name (licence, notices, history, the lock file
// cargo regenerates) are skipped, as are binary files.

import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { pathToFileURL } from "node:url";

const root = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
process.chdir(root);

const SELF = "scripts/rebrand-flightdeck.mjs";

/** Files whose text keeps upstream's name: attribution, history, and what cargo regenerates. */
const KEEP_CONTENT = new Set([
  "LICENSE",
  "THIRD_PARTY_NOTICES.md",
  "CHANGELOG.md",
  "herdr/tui/Cargo.lock",
  // Its first-run adoption of cockpit's config.json names cockpit on purpose; review upstream changes to it by hand.
  "herdr/tui/core/src/files.rs",
  SELF,
]);

/** Exact snippets that name upstream on purpose and survive the rewrite. */
const KEEP_SNIPPETS = ["Upstream: quaresc/cockpit", '<a href="https://github.com/quaresc/cockpit">cockpit</a> (MIT)'];

const BINARY = /\.(png|jpe?g|gif|ico|icns|woff2?|ttf|otf|wasm|mp4|mov|webm|pdf|zip|gz|tar)$/i;

/** Ordered rewrites: upstream URLs first, then the case-preserving word swap. */
const RULES = [
  [/quaresc\.github\.io\/cockpit/g, "gramosbr.github.io/flightdeck"],
  [/quaresc\/cockpit/g, "gramosbr/flightdeck"],
  [/cockpit/g, "flightdeck"],
  [/Cockpit/g, "Flightdeck"],
  [/COCKPIT/g, "FLIGHTDECK"],
];

const dry = process.argv.includes("--dry-run");
const phase = process.argv.slice(2).find((a) => !a.startsWith("--")) ?? "all";

const tracked = () => execFileSync("git", ["ls-files", "-z"], { encoding: "utf8", maxBuffer: 1 << 28 }).split("\0").filter(Boolean);

/** Applies every rule to `text`, leaving KEEP_SNIPPETS as they were. */
export function rebrand(text) {
  const kept = [];
  let out = text;
  for (const snippet of KEEP_SNIPPETS) {
    out = out.split(snippet).join(`\u0000KEEP${kept.push(snippet) - 1}\u0000`);
  }
  for (const [from, to] of RULES) out = out.replace(from, to);
  return out.replace(/\u0000KEEP(\d+)\u0000/g, (_, i) => kept[Number(i)]);
}

function renamePaths() {
  let moved = 0;
  for (const file of tracked()) {
    const next = rebrand(file);
    if (next === file) continue;
    moved += 1;
    if (dry) {
      console.log(`${file} -> ${next}`);
      continue;
    }
    mkdirSync(dirname(next), { recursive: true });
    execFileSync("git", ["mv", file, next]);
  }
  console.log(`paths: ${moved} renamed`);
}

function rewriteContent() {
  let changed = 0;
  for (const file of tracked()) {
    if (KEEP_CONTENT.has(file) || BINARY.test(file)) continue;
    let buf;
    try {
      buf = readFileSync(file);
    } catch {
      continue;
    }
    if (buf.includes(0)) continue;
    const text = buf.toString("utf8");
    const next = rebrand(text);
    if (next === text) continue;
    changed += 1;
    if (dry) console.log(file);
    else writeFileSync(file, next);
  }
  console.log(`content: ${changed} files rewritten`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (phase === "paths" || phase === "all") renamePaths();
  if (phase === "content" || phase === "all") rewriteContent();
}
