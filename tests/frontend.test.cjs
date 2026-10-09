const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const root = path.resolve(__dirname, "..");
const source = (file) => fs.readFileSync(path.join(root, "src", file), "utf8");
function formatters() {
  const window = {};
  vm.runInNewContext(source("format.js"), { window, Intl, Date });
  return window.formatters;
}

test("every frontend IPC command is registered in Tauri and preserves named arguments", async () => {
  const calls = [];
  const window = {
    __TAURI__: { core: { invoke: async (command, args) => { calls.push({ command, args }); return 42; } } },
  };
  vm.runInNewContext(source("api.js"), { window });
  const main = fs.readFileSync(path.join(root, "src-tauri/src/main.rs"), "utf8");
  const handlers = main.match(/generate_handler!\[([\s\S]*?)\]/)[1].split(",").map((s) => s.trim());
  const data = { sessionId: 7, productId: 1, quantity: 2 };
  const cases = [
    ["addConsole", "add_console", data, { data }],
    ["updateConsole", "update_console", data, { data }],
    ["archiveConsole", "archive_console", 7, { id: 7 }],
    ["getConsoles", "get_consoles", true, { includeArchived: true }],
    ["getDashboard", "get_dashboard", undefined, {}],
    ["startSession", "start_session", data, { data }],
    ["pauseSession", "pause_session", 7, { id: 7 }],
    ["resumeSession", "resume_session", 7, { id: 7 }],
    ["getInvoice", "get_invoice", 7, { id: 7 }],
    ["prepareCheckout", "prepare_checkout", 7, { id: 7 }],
    ["cancelCheckout", "cancel_checkout", 7, { id: 7 }],
    ["confirmCheckout", "confirm_checkout", 7, { id: 7 }],
    ["updateSessionControllers", "update_session_controllers", data, { data }],
    ["addSessionItem", "add_session_item", data, { data }],
    ["deleteSessionItem", "delete_session_item", 7, { id: 7 }],
    ["clearHistory", "clear_history", undefined, {}],
    ["getHistory", "get_history", data, { filter: data }],
    ["exportHistory", "export_history", data, { filter: data }],
    ["confirm", "ask_confirmation", "Confirm?", { message: "Confirm?" }],
  ];
  for (const [method, command, argument, args] of cases) {
    assert.ok(handlers.includes(command), command);
    assert.equal(await window.appAPI[method](argument), 42);
    assert.deepEqual(JSON.parse(JSON.stringify(calls.at(-1))), { command, args });
  }
  assert.equal(cases.length, Object.keys(window.appAPI).length);
});

test("Jalali dates accept Persian digits and produce inclusive Tehran date boundaries", () => {
  const f = formatters();
  assert.equal(f.jalaliDate(new Date("2026-10-09T10:00:00Z")), "1405/07/17");
  assert.equal(f.dateBoundary("۱۴۰۵/۰۷/۱۷"), "2026-10-08T20:30:00.000Z");
  assert.equal(f.dateBoundary("1405/07/17", true), "2026-10-09T20:30:00.000Z");
  assert.equal(f.dateBoundary(""), null);
  assert.throws(() => f.dateBoundary("1405/12/30"));
  assert.throws(() => f.dateBoundary("1405/13/01"));
  assert.throws(() => f.dateBoundary("not a date"));
});

test("calendar conversion handles Nowruz and a leap year", () => {
  const f = formatters();
  assert.equal(f.dateBoundary("1404/01/01"), "2025-03-20T20:30:00.000Z");
  assert.equal(f.dateBoundary("1403/12/30"), "2025-03-19T20:30:00.000Z");
  assert.equal(f.duration(3_661_999), "01:01:01");
  assert.equal(f.escape('<img src=x onerror="bad">'), "&lt;img src=x onerror=&quot;bad&quot;&gt;");
});

function renderer(api = {}) {
  const listeners = {};
  const elements = new Map();
  const element = (id) => {
    if (!elements.has(id)) elements.set(id, {
      value: id === "consoleFilter" ? "all" : "", innerHTML: "", textContent: "", hidden: false,
      classList: { toggle() {} }, querySelector: () => null,
      setAttribute() {}, addEventListener() {}, showModal() { this.open = true; },
      close() { this.open = false; },
    });
    return elements.get(id);
  };
  const document = {
    documentElement: { dataset: { theme: "dark" } },
    getElementById: element, querySelector: () => null,
    addEventListener: (name, handler) => { listeners[name] = handler; },
  };
  const window = { localStorage: new Map(), formatters: formatters(), appAPI: api, addEventListener() {}, print() {} };
  window.localStorage.setItem = (key, value) => window.localStorage.set(key, value);
  const context = vm.createContext({ window, document, console, setInterval() {}, Intl, Date });
  vm.runInContext(source("renderer.js"), context);
  return { context, element, listeners, call: (code) => vm.runInContext(code, context) };
}

test("dashboard hides start controls on occupied consoles and escapes stored names", () => {
  const r = renderer();
  r.call(`state.cards = [{console:{id:1,name:'<script>bad</script>',hourlyPrice:100,controllerPrice:20,archived:false},sessions:[]}]; renderDashboard();`);
  assert.match(r.element("consoleList").innerHTML, /&lt;script&gt;/);
  assert.match(r.element("consoleList").innerHTML, /data-action="startSession"/);
  r.call(`state.cards[0].sessions = [{session:{id:7,status:"checkout",startTime:"2026-10-09T00:00:00Z",controllers:2},playedMs:1000,gameCost:10,items:[]}]; renderDashboard();`);
  assert.doesNotMatch(r.element("consoleList").innerHTML, /data-action="startSession"/);
  assert.match(r.element("consoleList").innerHTML, /ادامه تسویه/);
  assert.doesNotMatch(r.element("consoleList").innerHTML, /data-action="showSale"/);
});

test("receipt displays the backend frozen amount and confirm displays the saved amount", async () => {
  const invoice = { session: { id: 7, status: "checkout", consoleName: "PS5", startTime: "2026-10-09T00:00:00Z", checkoutAt: "2026-10-09T01:00:00Z" }, items: [], playedMs: 3600000, gameCost: 100, shopCost: 0, total: 100 };
  let confirmed = false;
  const r = renderer({
    prepareCheckout: async (id) => { assert.equal(id, 7); return invoice; },
    confirmCheckout: async (id) => { assert.equal(id, 7); confirmed = true; return { ...invoice, session: { ...invoice.session, status: "ended", endTime: invoice.session.checkoutAt } }; },
    getDashboard: async () => [],
  });
  await r.call("checkout(7)");
  assert.match(r.element("receiptContent").innerHTML, /۱۰۰ تومان/);
  assert.match(r.element("receiptButtons").innerHTML, /confirmCheckout/);
  await r.call("confirmCheckout()");
  assert.equal(confirmed, true);
  assert.match(r.element("receiptContent").innerHTML, /پرداخت‌شده/);
  assert.doesNotMatch(r.element("receiptButtons").innerHTML, /confirmCheckout/);
});

test("failed actions show errors and release the busy state", async () => {
  const r = renderer({ pauseSession: async () => { throw new Error("Session changed"); } });
  const control = { disabled: false };
  r.context.control = control;
  await r.call('runAction("pauseSession", [7], control)');
  assert.equal(control.disabled, false);
  assert.equal(r.call("state.busy"), false);
  assert.match(r.element("notification").textContent, /Session changed/);
});

test("all static and dynamic actions exist and packaged assets resolve", () => {
  require("../scripts/build-frontend.cjs");
  const r = renderer();
  const names = r.call("Object.keys(actions)");
  const html = source("index.html");
  for (const match of (html + source("renderer.js")).matchAll(/data-(?:action|submit)="(\w+)"/g)) {
    assert.ok(names.includes(match[1]), match[1]);
  }
  assert.doesNotMatch(html + source("renderer.js"), /onclick=/);
  assert.ok(html.indexOf('src="api.js"') < html.indexOf('src="renderer.js"'));
  assert.ok(html.indexOf('src="format.js"') < html.indexOf('src="renderer.js"'));
  for (const match of html.matchAll(/(?:src|href)="([^"]+)"/g)) {
    if (match[1].startsWith("#")) continue;
    assert.ok(fs.existsSync(path.join(root, "dist", match[1].replace(/^\//, ""))), match[1]);
  }
  for (const match of source("styles.css").matchAll(/url\(['"]?([^'")]+)['"]?\)/g)) {
    assert.ok(fs.existsSync(path.resolve(root, "dist", match[1])), match[1]);
  }
});

test("session purchases submit only a local name and price", async () => {
  let saved;
  const r = renderer({
    addSessionItem: async (data) => { saved = JSON.parse(JSON.stringify(data)); },
    getDashboard: async () => [],
  });
  r.call("showSale(7)");
  r.element("saleName").value = " نوشابه ";
  r.element("salePrice").value = "۷۰۰۰";
  await r.call("saveSale()");
  assert.deepEqual(saved, { sessionId: 7, name: "نوشابه", price: 7000 });
  assert.equal(r.element("saleDialog").open, false);
  assert.doesNotMatch(source("index.html"), /saleProduct|saleQuantity|productStock|view-products/);
});

test("appearance switching updates its label and persists the choice", () => {
  const r = renderer();
  r.call("toggleTheme()");
  assert.equal(r.call("document.documentElement.dataset.theme"), "light");
  assert.equal(r.call('window.localStorage.get("gamagement-theme")'), "light");
  assert.equal(r.element("themeLabel").textContent, "حالت تیره");
  r.call("toggleTheme()");
  assert.equal(r.call("document.documentElement.dataset.theme"), "dark");
  assert.equal(r.call('window.localStorage.get("gamagement-theme")'), "dark");
});

test("history deletion requires confirmation and refreshes reports from page one", async () => {
  let approved = false, removed = 0, filter;
  const r = renderer({
    confirm: async () => approved,
    clearHistory: async () => { removed++; return 3; },
    getHistory: async (data) => {
      filter = data;
      return { summary: { sessions: 0, gameRevenue: 0, shopRevenue: 0, totalRevenue: 0 }, entries: [], days: [], consoles: [], page: 1, pages: 1 };
    },
  });
  await r.call("clearHistory()");
  assert.equal(removed, 0);
  approved = true;
  r.call("state.filter.page = 5");
  await r.call("clearHistory()");
  assert.equal(removed, 1);
  assert.equal(filter.page, 1);
  assert.match(r.element("notification").textContent, /۳ فاکتور/);
});
