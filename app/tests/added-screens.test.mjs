// Screens a layout adds: a child of the screens' stack whose block props carry `screen: {label, icon}` joins the
// layout's screens after flightdeck's own, so the sidebar, the palette, `flightdeck.go:` and the keys sheet all find it.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createContext, SourceTextModule, SyntheticModule } from "node:vm";
import { test } from "node:test";

const MODS = new URL("../mods/flightdeck/", import.meta.url);

/** screens.js in a VM, Herdr's natives stubbed: `shared` names the layout, `tree` is what layout() returns. */
async function load({ name = "flightdeck", tree = null, layoutThrows = false } = {}) {
  const shared = { layout: name };
  const context = createContext({ console });
  const natives = {
    herdr: {
      run: () => null,
      shared: (key) => shared[key] ?? null,
      setShared: (key, value) => (shared[key] = value),
      layout: () => {
        if (layoutThrows) throw new Error("no layout");
        return tree;
      },
      focused: () => null,
      agents: () => [],
      exec: async () => ({ code: 0, stdout: "", stderr: "" }),
    },
  };
  const modules = new Map();
  async function link(spec) {
    if (modules.has(spec)) return modules.get(spec);
    const native = natives[spec];
    const m = native
      ? new SyntheticModule(Object.keys(native), function () {
          for (const [k, v] of Object.entries(native)) this.setExport(k, v);
        }, { context })
      : new SourceTextModule(await readFile(new URL(spec, MODS), "utf8"), { context, identifier: spec });
    modules.set(spec, m);
    return m;
  }
  const entry = await link("./screens.js");
  await entry.link(link);
  await entry.evaluate();
  return entry.namespace;
}

const page = (id, props) => ({ id, type: "block", content: { mod: "acme-ops", props } });
const flightdeckTree = (...extra) => ({
  id: "root",
  children: [{ id: "center", type: "stack", children: [{ id: "home" }, { id: "work" }, { id: "inbox" }, ...extra] }],
});

test("a stack child with screen props is a screen of the layout, after flightdeck's own", async () => {
  const s = await load({ tree: flightdeckTree(page("ops", { view: "page", screen: { label: "Ops", icon: "activity" } })) });
  const list = s.screens().list;
  assert.deepEqual(JSON.parse(JSON.stringify(list.at(-1))), { child: "ops", label: "Ops", icon: "activity", command: "flightdeck.go:ops", added: true });
  assert.equal(list.filter((x) => x.added).length, 1);
  assert.equal(list.slice(0, 2).map((x) => x.child).join(), "home,work");
});

test("`flightdeck.go:` and the keys sheet see the added screen through screensOf for the shown layout", async () => {
  const s = await load({ name: "flightdeck-ops", tree: flightdeckTree(page("ops", { screen: { label: "Ops" } })) });
  assert.ok(s.screensOf("flightdeck").list.some((x) => x.child === "ops" && x.icon === "layout"));
  assert.ok(!s.screensOf("flightdeck-chat").list.some((x) => x.child === "ops"));
});

test("children without screen props, flightdeck's own screens, and other stacks add nothing", async () => {
  const tree = flightdeckTree(page("plain", { view: "page" }), page("inbox", { screen: { label: "Mine" } }), { id: "loose" });
  tree.children.push({ id: "elsewhere", type: "stack", children: [page("x", { screen: { label: "X" } })] });
  const s = await load({ tree });
  assert.equal(s.screens().list.filter((x) => x.added).length, 0);
  assert.equal(s.screens().list.filter((x) => x.child === "inbox").length, 1);
});

test("without a layout to read, the screens are flightdeck's own", async () => {
  const s = await load({ layoutThrows: true });
  assert.equal(s.screens().list.some((x) => x.added), false);
  assert.ok(s.screens().list.length > 0);
});
