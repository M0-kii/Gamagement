const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

test("frontend methods invoke the corresponding Rust commands with named arguments", async () => {
  const calls = [];
  const window = {
    __TAURI__: { core: { invoke: async (command, args) => { calls.push({ command, args }); return 42; } } },
    addEventListener() {},
  };
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../src/api.js"), "utf8"), { window, console });
  const data = { sessionId: 7, name: "Snack", price: 10, itemIndex: 0 };
  const cases = [
    ["addConsole", "add_console", data, { data }],
    ["getConsoles", "get_consoles", undefined, {}],
    ["updateConsole", "update_console", data, { data }],
    ["deleteConsole", "delete_console", 7, { id: 7 }],
    ["getConsole", "get_console", 7, { id: 7 }],
    ["startSession", "start_session", data, { data }],
    ["pauseSession", "pause_session", 7, { id: 7 }],
    ["resumeSession", "resume_session", 7, { id: 7 }],
    ["endSession", "end_session", 7, { id: 7 }],
    ["getSessionsForConsole", "get_sessions_for_console", 7, { consoleId: 7 }],
    ["getSession", "get_session", 7, { id: 7 }],
    ["addShopItemToSession", "add_shop_item_to_session", data, { data }],
    ["updateSessionControllers", "update_session_controllers", data, { data }],
    ["updateShopItem", "update_shop_item", data, { data }],
    ["deleteShopItem", "delete_shop_item", data, { data }],
    ["confirm", "ask_confirmation", "Confirm?", { message: "Confirm?" }],
    ["alert", "show_message", "Error", { message: "Error" }],
  ];
  for (const [method, command, argument, args] of cases) {
    assert.equal(await window.appAPI[method](argument), 42);
    assert.deepEqual(JSON.parse(JSON.stringify(calls.at(-1))), { command, args });
  }
});

test("packaged HTML loads the bridge before renderer and all assets are present", () => {
  require("../scripts/build-frontend.cjs");
  const root = path.join(__dirname, "../dist");
  const html = fs.readFileSync(path.join(root, "index.html"), "utf8");
  assert.ok(html.indexOf('src="api.js"') < html.indexOf('src="renderer.js"'));
  for (const match of html.matchAll(/(?:src|href)="([^"]+)"/g)) {
    assert.ok(fs.existsSync(path.join(root, match[1].replace(/^\//, ""))), match[1]);
  }
  const css = fs.readFileSync(path.join(root, "styles.css"), "utf8");
  for (const match of css.matchAll(/url\(['"]?([^'")]+)['"]?\)/g)) {
    assert.ok(fs.existsSync(path.resolve(root, match[1])), match[1]);
  }
});

test("delegated buttons dispatch session actions and timers resume paused sessions", async () => {
  const listeners = {};
  const timers = [];
  const elements = new Map();
  const element = (id) => {
    if (!elements.has(id)) elements.set(id, { style: {}, textContent: "", classList: { add() {}, remove() {} } });
    return elements.get(id);
  };
  const session = {
    id: 7, consoleId: 1, controllers: 2, status: "paused",
    startTime: "2026-10-08T10:00:00.000Z", pauseTime: "2026-10-08T11:00:00.000Z",
  };
  const api = {
    getConsoles: async () => [],
    getSession: async () => ({ ...session }),
    getConsole: async () => ({ hourlyPrice: 100, controllerPrice: 20 }),
    resumeSession: async (id) => {
      assert.equal(id, 7);
      session.status = "active";
      session.startTime = new Date(Date.now() - 5000).toISOString();
      session.pauseTime = null;
    },
    alert: async () => {},
  };
  const document = {
    addEventListener: (name, handler) => { listeners[name] = handler; },
    getElementById: element,
    querySelector: () => element("session_7"),
    querySelectorAll: () => [{ id: "session_7" }],
  };
  const window = { appAPI: api, addEventListener: (name, handler) => { listeners[name] = handler; } };
  const context = vm.createContext({ document, window, console, setInterval: (handler) => { timers.push(handler); }, Date });
  vm.runInContext(fs.readFileSync(path.join(__dirname, "../src/renderer.js"), "utf8"), context);
  listeners.DOMContentLoaded();
  assert.equal(timers.length, 1);
  await timers[0]();
  assert.equal(element("time_7").textContent, "01:00:00");
  assert.ok(element("price_7").textContent.includes("۱۰۰"));
  const button = { disabled: false, dataset: { action: "resumeSession", args: "[7]" } };
  await listeners.click({ target: { closest: () => button } });
  assert.equal(button.disabled, false);
  assert.equal(element("resumeBtn_7").style.display, "none");
  assert.equal(element("pauseBtn_7").style.display, "block");
  await timers[0]();
  assert.match(element("time_7").textContent, /^00:00:0[56]$/);
  assert.equal(timers.length, 1);

  const html = fs.readFileSync(path.join(__dirname, "../src/index.html"), "utf8");
  const renderer = fs.readFileSync(path.join(__dirname, "../src/renderer.js"), "utf8");
  assert.doesNotMatch(html + renderer, /onclick=/);
  const names = vm.runInContext("Object.keys(actions)", context);
  for (const match of (html + renderer).matchAll(/data-action="(\w+)"/g)) {
    assert.ok(names.includes(match[1]), match[1]);
  }
});
