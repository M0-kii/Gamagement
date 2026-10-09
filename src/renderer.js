const fmt = window.formatters;
const state = {
  view: "dashboard", cards: [], consoles: [], invoice: null,
  selectedControllers: new Map(), history: null, filter: {}, busy: false, polling: false,
};
const $ = (id) => document.getElementById(id);
const escape = fmt.escape;
const statusNames = { active: "در حال بازی", paused: "متوقف", checkout: "در حال تسویه", available: "آزاد" };

const activeMotion = new WeakMap();
function animateUI(element, frames, duration = 180, delay = 0) {
  if (!element?.animate || document.documentElement.dataset.input === "keyboard"
    || window.matchMedia?.("(prefers-reduced-motion: reduce)").matches) return;
  activeMotion.get(element)?.cancel();
  const animation = element.animate(frames, {
    duration, delay, fill: "backwards", easing: "cubic-bezier(0.22, 1, 0.36, 1)",
  });
  activeMotion.set(element, animation);
  animation.finished.then(() => {
    if (activeMotion.get(element) === animation) activeMotion.delete(element);
  }).catch(() => {});
}
function cardMotionKey(card, selected) {
  return JSON.stringify([card.console, selected, card.sessions.map((i) =>
    [i.session.id, i.session.status, i.session.controllers, i.items])]);
}
function statusBadge(status) {
  const paths = {
    available: '<path d="m3 6 2 2 4-4"/>',
    active: '<path d="m4 2 6 4-6 4Z"/>',
    paused: '<path d="M4 2v8M8 2v8"/>',
    checkout: '<path d="M3 1h6v10l-2-1-1 1-1-1-2 1ZM5 4h2M5 6h2"/>',
  };
  return `<span class="status ${status}"><svg viewBox="0 0 12 12" aria-hidden="true">${paths[status]}</svg>${statusNames[status]}</span>`;
}
function emptyState(title, description, controls = "", kind = "receipt") {
  const shape = kind === "console"
    ? '<rect x="3" y="5" width="18" height="12" rx="3"/><path d="M8 21h8M12 17v4M7 9v4M5 11h4M16 10h.01M18 12h.01"/>'
    : '<path d="M6 3h12v18l-3-2-3 2-3-2-3 2ZM9 8h6M9 12h6"/>';
  return `<div class="empty-state"><div class="empty-symbol"><svg viewBox="0 0 24 24" aria-hidden="true">${shape}</svg></div><h3>${title}</h3><p>${description}</p>${controls ? `<div class="empty-actions">${controls}</div>` : ""}</div>`;
}
function resetConsoleFilter() { $("consoleFilter").value = "all"; renderDashboard(); }
async function resetHistoryFilters() {
  for (const id of ["historyFrom", "historyTo", "historySearch", "historyConsole"]) $(id).value = "";
  await filterHistory();
}
function button(action, text, args = [], style = "btn-secondary", disabled = false) {
  return `<button type="button" class="${style}" data-action="${action}" data-args="${escape(JSON.stringify(args))}" ${disabled ? "disabled" : ""}>${text}</button>`;
}
function notify(message, error = false) {
  $("notification").textContent = message;
  $("notification").classList.toggle("error", error);
  $("notification").hidden = false;
  animateUI($("notification"), [
    { opacity: 0, transform: "translateY(-4px)" },
    { opacity: 1, transform: "translateY(0)" },
  ]);
}
function showDialog(id) {
  const dialog = $(id);
  const error = dialog.querySelector(".form-error");
  if (error) error.textContent = "";
  if (!dialog.open) dialog.showModal();
}
function closeDialog(id) { $(id).close(); }
function integer(id, min = 0, max = 1_000_000_000) {
  const raw = fmt.digits($(id).value.trim());
  const value = raw === "" ? NaN : Number(raw);
  if (!Number.isSafeInteger(value) || value < min || value > max) throw new Error("یک عدد صحیح در بازه مجاز وارد کنید");
  return value;
}
function cardStatus(card) {
  if (card.sessions.some((i) => i.session.status === "active")) return "active";
  if (card.sessions.some((i) => i.session.status === "checkout")) return "checkout";
  return card.sessions.length ? "paused" : "available";
}
function summaryItem(label, value, className = "") {
  return `<div class="stat ${className}"><span>${label}</span><strong>${value}</strong></div>`;
}
async function reloadDashboard() {
  state.cards = await window.appAPI.getDashboard();
  renderDashboard();
}
function renderDashboard() {
  const previous = new Map(Array.from($("consoleList").querySelectorAll?.(".card") || [],
    (card) => [card.dataset.consoleId, card.dataset.motionKey]));
  const counts = { available: 0, active: 0, paused: 0, checkout: 0 };
  for (const card of state.cards) counts[cardStatus(card)]++;
  $("dashboardSummary").innerHTML = Object.entries(counts).map(([key, count]) =>
    summaryItem(statusNames[key], fmt.number(count), key)).join("");
  const filter = $("consoleFilter").value;
  const cards = state.cards.filter((c) => filter === "all" || cardStatus(c) === filter);
  $("consoleList").innerHTML = cards.map((card) => {
    const c = card.console;
    const status = cardStatus(card);
    const selected = state.selectedControllers.get(c.id) || 2;
    return `<article class="card ${status}" data-console-id="${c.id}" data-motion-key="${escape(cardMotionKey(card, selected))}">
      <div class="card-top">${statusBadge(status)}<div class="card-actions">
        ${c.archived ? '<span class="muted">بایگانی</span>' : button("showConsole", "ویرایش", [c.id], "text-button")}
        ${!c.archived && !card.sessions.length ? button("archiveConsole", "بایگانی", [c.id], "text-button danger-text") : ""}
      </div></div>
      <div class="console-symbol" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M7 7h10c3 0 5 8 3 10-1 1-3-2-4-3H8c-1 1-3 4-4 3-2-2 0-10 3-10Z"/><path d="M7 9v4M5 11h4M16 10h.01M18 12h.01"/></svg></div><h3>${escape(c.name)}</h3>
      <p class="rate">${fmt.price(c.hourlyPrice)} <span>در ساعت</span></p>
      <p class="muted">هر دسته اضافه: ${fmt.price(c.controllerPrice)} در ساعت</p>
      ${card.sessions.length ? card.sessions.map(renderSession).join("") : `
        <div class="controller-selector"><span>تعداد دسته</span><div class="counter">
          ${button("changeStartControllers", "−", [c.id, -1], "selector-btn", selected <= 1)}
          <span>${fmt.number(selected)}</span>
          ${button("changeStartControllers", "+", [c.id, 1], "selector-btn", selected >= 4)}
        </div></div>
        ${button("startSession", "شروع بازی", [c.id], "btn-primary full-width")}
      `}
    </article>`;
  }).join("") || (state.cards.length
    ? emptyState("کنسولی با این وضعیت وجود ندارد", "برای دیدن میزهای دیگر، فیلتر وضعیت را بردارید.", button("resetConsoleFilter", "نمایش همه کنسول‌ها"), "console")
    : emptyState("هنوز کنسولی ثبت نشده است", "اولین میز بازی را اضافه کنید تا بتوانید جلسه‌ها را مدیریت کنید.", button("showConsole", "افزودن اولین کنسول", [], "btn-primary"), "console"));
  let entered = 0;
  for (const card of $("consoleList").querySelectorAll?.(".card") || []) {
    const oldKey = previous.get(card.dataset.consoleId);
    if (oldKey === card.dataset.motionKey) continue;
    if (oldKey === undefined) {
      animateUI(card, [
        { opacity: 0, transform: "translateY(6px)" },
        { opacity: 1, transform: "translateY(0)" },
      ], 200, Math.min(entered++ * 25, 75));
    } else {
      animateUI(card, [{ opacity: .65 }, { opacity: 1 }], 160);
      animateUI(card.querySelector(".counter span"), [
        { transform: "scale(1.12)", opacity: .6 }, { transform: "scale(1)", opacity: 1 },
      ], 160);
    }
  }
}
function renderSession(invoice) {
  const s = invoice.session;
  const editable = s.status !== "checkout";
  const items = invoice.items.map((item) => `<li class="shop-item-row">
    <div><strong>${escape(item.name)}</strong>${item.quantity > 1 ? `<small>${fmt.number(item.quantity)} × ${fmt.price(item.unitPrice)}</small>` : ""}</div>
    <div><span>${fmt.price(item.total)}</span><div class="shop-item-actions">
      ${editable ? button("removeItem", "حذف", [item.id], "text-button danger-text") : ""}
    </div></div>
  </li>`).join("");
  return `<section class="session-block ${s.status}" id="session_${s.id}">
    <div class="session-heading"><span>جلسه #${fmt.number(s.id)}</span>${statusBadge(s.status)}</div>
    <p class="hint">شروع: ${fmt.date(s.startTime)}</p>
    <div class="time" dir="ltr" id="time_${s.id}">${fmt.duration(invoice.playedMs)}</div>
    <div class="price-display" id="price_${s.id}">${fmt.price(invoice.gameCost)}</div>
    <p class="hint">هزینه بازی؛ خریدها در فاکتور اضافه می‌شوند</p>
    <div class="controller-selector"><span>دسته</span><div class="counter">
      ${button("changeSessionControllers", "−", [s.id, -1], "selector-btn", !editable || s.controllers <= 1)}
      <span>${fmt.number(s.controllers)}</span>
      ${button("changeSessionControllers", "+", [s.id, 1], "selector-btn", !editable || s.controllers >= 4)}
    </div></div>
    <div class="session-buttons">
      ${s.status === "active" ? button("pauseSession", "توقف موقت", [s.id], "btn-warning") : ""}
      ${s.status === "paused" ? button("resumeSession", "ادامه بازی", [s.id], "btn-success") : ""}
      ${button("checkout", s.status === "checkout" ? "ادامه تسویه" : "پایان و تسویه", [s.id], "btn-primary")}
    </div>
    ${items ? `<ul class="shop-items">${items}</ul>` : ""}
    ${editable ? button("showSale", "افزودن خرید", [s.id], "btn-secondary full-width") : '<p class="hint">مبلغ فاکتور ثابت است. برای ادامه بازی، تسویه را لغو کنید.</p>'}
  </section>`;
}
function changeStartControllers(id, delta) {
  state.selectedControllers.set(id, Math.max(1, Math.min(4, (state.selectedControllers.get(id) || 2) + delta)));
  renderDashboard();
}
async function startSession(id) {
  await window.appAPI.startSession({ consoleId: id, controllers: state.selectedControllers.get(id) || 2 });
  await reloadDashboard();
}
async function pauseSession(id) { await window.appAPI.pauseSession(id); await reloadDashboard(); }
async function resumeSession(id) { await window.appAPI.resumeSession(id); await reloadDashboard(); }
async function changeSessionControllers(id, delta) {
  const invoice = state.cards.flatMap((c) => c.sessions).find((i) => i.session.id === id);
  if (!invoice) return;
  const controllers = Math.max(1, Math.min(4, invoice.session.controllers + delta));
  await window.appAPI.updateSessionControllers({ sessionId: id, controllers });
  await reloadDashboard();
}
function showConsole(id) {
  const c = state.cards.find((c) => c.console.id === id)?.console;
  $("consoleDialogTitle").textContent = c ? "ویرایش کنسول" : "افزودن کنسول";
  $("consoleEditId").value = c?.id || "";
  $("consoleName").value = c?.name || "";
  $("hourlyPrice").value = c?.hourlyPrice ?? "";
  $("controllerPrice").value = c?.controllerPrice ?? 0;
  showDialog("consoleDialog");
}
async function saveConsole() {
  const data = { name: $("consoleName").value.trim(), hourlyPrice: integer("hourlyPrice"), controllerPrice: integer("controllerPrice") };
  if ($("consoleEditId").value) {
    data.id = Number($("consoleEditId").value);
    await window.appAPI.updateConsole(data);
  } else await window.appAPI.addConsole(data);
  closeDialog("consoleDialog");
  await reloadDashboard();
  notify("کنسول ذخیره شد.");
}
async function archiveConsole(id) {
  if (!await window.appAPI.confirm("این کنسول بایگانی شود؟ فاکتورهای قبلی حفظ می‌شوند.", { confirmLabel: "بایگانی کنسول", danger: false })) return;
  await window.appAPI.archiveConsole(id);
  await reloadDashboard();
}
function showSale(id) {
  $("saleSessionId").value = id;
  $("saleName").value = "";
  $("salePrice").value = "";
  showDialog("saleDialog");
}
async function saveSale() {
  await window.appAPI.addSessionItem({
    sessionId: Number($("saleSessionId").value),
    name: $("saleName").value.trim(), price: integer("salePrice"),
  });
  closeDialog("saleDialog");
  await reloadDashboard();
}
async function removeItem(id) {
  if (!await window.appAPI.confirm("این خرید از فاکتور جلسه حذف شود؟", { confirmLabel: "حذف خرید" })) return;
  await window.appAPI.deleteSessionItem(id);
  await reloadDashboard();
}
async function checkout(id) {
  state.invoice = await window.appAPI.prepareCheckout(id);
  renderReceipt();
  showDialog("receiptDialog");
  await reloadDashboard();
}
async function openReceipt(id) {
  state.invoice = await window.appAPI.getInvoice(id);
  renderReceipt();
  showDialog("receiptDialog");
}
function renderReceipt() {
  const i = state.invoice;
  const s = i.session;
  const preview = s.status === "checkout";
  $("receiptContent").innerHTML = `<div class="receipt-heading"><span>AsiaGame</span><span>جلسه #${fmt.number(s.id)}</span></div>
    <h3 id="receiptTitle">${preview ? "تسویه جلسه" : "فاکتور پرداخت‌شده"}</h3>
    <p class="receipt-console">${escape(s.consoleName)}</p>
    <dl class="receipt-meta"><div><dt>شروع</dt><dd>${fmt.date(s.startTime)}</dd></div><div><dt>پایان بازی</dt><dd>${fmt.date(s.endTime || s.checkoutAt)}</dd></div>
    <div><dt>مدت بازی</dt><dd dir="ltr">${fmt.duration(i.playedMs)}</dd></div>${s.paidAt ? `<div><dt>زمان پرداخت</dt><dd>${fmt.date(s.paidAt)}</dd></div>` : ""}</dl>
    ${preview ? '<p class="checkout-note">زمان بازی متوقف شده و این مبلغ تا تأیید ثابت می‌ماند. با بازگشت، وضعیت قبلی جلسه برمی‌گردد.</p>' : ""}
    <div class="invoice-item"><span>هزینه بازی</span><strong>${fmt.price(i.gameCost)}</strong></div>
    ${i.items.length ? `<table class="receipt-items"><thead><tr><th>محصول</th><th>تعداد</th><th>قیمت واحد</th><th>جمع</th></tr></thead><tbody>${i.items.map((item) =>
      `<tr><td>${escape(item.name)}</td><td>${fmt.number(item.quantity)}</td><td>${fmt.price(item.unitPrice)}</td><td>${fmt.price(item.total)}</td></tr>`).join("")}</tbody></table>` : ""}
    <div class="invoice-item"><span>هزینه فروشگاه</span><strong>${fmt.price(i.shopCost)}</strong></div>
    <div class="invoice-total"><span>${preview ? "قابل پرداخت" : "پرداخت‌شده"}</span><strong>${fmt.price(i.total)}</strong></div>
    ${s.legacy ? '<p class="hint">فاکتور منتقل‌شده از نسخه قبلی؛ مبلغ ثبت‌شده حفظ شده است.</p>' : ""}`;
  $("receiptButtons").innerHTML = preview
    ? button("confirmCheckout", "ثبت پرداخت", [], "btn-primary") + button("cancelCheckout", "بازگشت به بازی", [], "btn-secondary")
    : button("printReceipt", "چاپ فاکتور", [], "btn-primary") + button("closeReceipt", "بستن", [], "btn-secondary");
}
async function confirmCheckout() {
  state.invoice = await window.appAPI.confirmCheckout(state.invoice.session.id);
  renderReceipt();
  animateUI($("receiptContent"), [{ opacity: .5 }, { opacity: 1 }], 180);
  await reloadDashboard();
  notify("پرداخت ثبت شد.");
}
async function cancelCheckout() {
  if (state.invoice?.session.status === "checkout") await window.appAPI.cancelCheckout(state.invoice.session.id);
  state.invoice = null;
  closeDialog("receiptDialog");
  await reloadDashboard();
}
function closeReceipt() { state.invoice = null; closeDialog("receiptDialog"); }
function printReceipt() { window.print(); }

function readFilter(page = 1) {
  const from = fmt.dateBoundary($("historyFrom").value);
  const to = fmt.dateBoundary($("historyTo").value, true);
  if (from && to && from >= to) throw new Error("تاریخ پایان باید پس از تاریخ شروع باشد");
  return { from, to, search: $("historySearch").value.trim(), consoleId: $("historyConsole").value ? Number($("historyConsole").value) : null, page };
}
async function loadHistory() {
  state.history = await window.appAPI.getHistory(state.filter);
  renderHistory();
}
async function filterHistory() { state.filter = readFilter(); await loadHistory(); }
async function changePage(delta) {
  if (!state.history) return;
  state.filter.page = Math.max(1, Math.min(state.history.pages, state.history.page + delta));
  await loadHistory();
}
function renderHistory() {
  const h = state.history;
  $("historySummary").innerHTML = summaryItem("جلسه‌های تسویه‌شده", fmt.number(h.summary.sessions))
    + summaryItem("درآمد بازی", fmt.price(h.summary.gameRevenue))
    + summaryItem("درآمد فروشگاه", fmt.price(h.summary.shopRevenue))
    + summaryItem("جمع درآمد", fmt.price(h.summary.totalRevenue), "revenue");
  $("historyList").innerHTML = h.entries.map((e) => `<tr>
    <td>#${fmt.number(e.id)}</td><td>${escape(e.consoleName)}</td><td>${fmt.date(e.paidAt)}</td>
    <td dir="ltr">${fmt.duration(e.playedMs)}</td><td>${fmt.price(e.gameCost)}</td><td>${fmt.price(e.shopCost)}</td><td><strong>${fmt.price(e.total)}</strong></td>
    <td>${button("openReceipt", "مشاهده", [e.id], "text-button")}</td>
  </tr>`).join("") || `<tr><td colspan="8">${emptyState("فاکتوری پیدا نشد", "در این بازه فاکتور تسویه‌شده‌ای وجود ندارد. فیلترها را پاک کنید یا یک جلسه را تسویه کنید.", button("resetHistoryFilters", "نمایش همه سوابق") + button("switchView", "رفتن به کنسول‌ها", ["dashboard"], "text-button"))}</td></tr>`;
  $("pageInfo").textContent = `صفحه ${fmt.number(h.page)} از ${fmt.number(h.pages)}`;
  $("previousPage").disabled = h.page <= 1;
  $("nextPage").disabled = h.page >= h.pages;
  $("dailyReport").innerHTML = h.days.map((d) => `<tr><td>${fmt.jalaliDate(new Date(d.day + "T12:00:00Z"))}</td><td>${fmt.number(d.sessions)}</td><td>${fmt.price(d.gameRevenue)}</td><td>${fmt.price(d.shopRevenue)}</td><td>${fmt.price(d.totalRevenue)}</td></tr>`).join("")
    || `<tr><td colspan="5">${emptyState("درآمدی ثبت نشده است", "گزارش روزانه پس از ثبت پرداخت نمایش داده می‌شود.")}</td></tr>`;
  $("consoleReport").innerHTML = h.consoles.map((c) => `<tr><td>${escape(c.name)}</td><td>${fmt.number(c.sessions)}</td><td dir="ltr">${fmt.duration(c.playedMs)}</td><td>${fmt.price(c.revenue)}</td></tr>`).join("")
    || `<tr><td colspan="4">${emptyState("گزارش کنسولی وجود ندارد", "جلسه‌های تسویه‌شده، مدت استفاده و درآمد هر کنسول را مشخص می‌کنند.", "", "console")}</td></tr>`;
}
async function exportHistory() {
  if (await window.appAPI.exportHistory(readFilter())) notify("فایل گزارش ذخیره شد.");
}
async function switchView(view) {
  const changed = state.view !== view;
  state.view = view;
  for (const name of ["dashboard", "history"]) {
    $("view-" + name).hidden = name !== view;
    $("nav-" + name).setAttribute("aria-current", name === view ? "page" : "false");
  }
  $("notification").hidden = true;
  if (changed) animateUI($("view-" + view), [
    { opacity: 0, transform: "translateY(5px)" },
    { opacity: 1, transform: "translateY(0)" },
  ], 180);
  if (view === "dashboard") await reloadDashboard();
  if (view === "history") {
    state.consoles = await window.appAPI.getConsoles(true);
    const selected = $("historyConsole").value;
    $("historyConsole").innerHTML = '<option value="">همه کنسول‌ها</option>' + state.consoles.map((c) =>
      `<option value="${c.id}">${escape(c.name)}${c.archived ? " (بایگانی)" : ""}</option>`).join("");
    $("historyConsole").value = selected;
    await filterHistory();
  }
}
async function refreshLive() {
  if (state.busy || state.polling || state.view !== "dashboard" || document.querySelector("dialog[open]")) return;
  state.polling = true;
  try {
    const cards = await window.appAPI.getDashboard();
    if (state.busy) return;
    const signature = (data) => JSON.stringify(data.map((c) => [c.console, c.sessions.map((i) => [i.session.id, i.session.status, i.session.controllers, i.items])]));
    const changed = signature(cards) !== signature(state.cards);
    state.cards = cards;
    if (changed) renderDashboard();
    else for (const i of cards.flatMap((c) => c.sessions)) {
      const time = $("time_" + i.session.id);
      const cost = $("price_" + i.session.id);
      if (time) time.textContent = fmt.duration(i.playedMs);
      if (cost) cost.textContent = fmt.price(i.gameCost);
    }
  } catch (error) {
    notify("دریافت وضعیت جلسه‌ها انجام نشد: " + String(error), true);
  } finally { state.polling = false; }
}
function applySidebar(collapsed) {
  document.documentElement.dataset.sidebar = collapsed ? "collapsed" : "expanded";
  const label = collapsed ? "باز کردن نوار کناری" : "جمع کردن نوار کناری";
  const control = $("sidebarToggle");
  control.setAttribute("aria-expanded", String(!collapsed));
  control.setAttribute("aria-label", label);
  control.setAttribute("title", label);
  for (const [id, title] of [["nav-dashboard", "کنسول‌ها"], ["nav-history", "سوابق و گزارش"]]) {
    $(id).setAttribute("title", collapsed ? "" : title);
  }
}
function toggleSidebar() {
  const collapsed = document.documentElement.dataset.sidebar !== "collapsed";
  applySidebar(collapsed);
  try { window.localStorage.setItem("gamagement-sidebar", collapsed ? "collapsed" : "expanded"); } catch {}
}
async function clearHistory() {
  const menu = document.querySelector(".history-menu");
  if (menu) menu.open = false;
  if (!await window.appAPI.confirm("همه جلسه‌های پرداخت‌شده و فاکتورهای آن‌ها برای همیشه حذف شوند؟ این کار قابل بازگشت نیست. جلسه‌های باز حفظ می‌شوند.", { title: "پاک کردن همه سوابق", confirmLabel: "حذف همه سوابق" })) return;
  const removed = await window.appAPI.clearHistory();
  state.filter.page = 1;
  await loadHistory();
  notify(removed ? `${fmt.number(removed)} فاکتور حذف شد.` : "سابقه‌ای برای حذف وجود ندارد.");
}
function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
  $("themeLabel").textContent = theme === "dark" ? "حالت روشن" : "حالت تیره";
  document.querySelector(".theme-button")?.setAttribute("aria-pressed", String(theme === "dark"));
}
function toggleTheme() {
  const theme = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  applyTheme(theme);
  try { window.localStorage.setItem("gamagement-theme", theme); } catch {}
}
const actions = {
  toggleTheme, toggleSidebar, clearHistory, resetConsoleFilter, resetHistoryFilters,
  switchView, showConsole, saveConsole, archiveConsole, changeStartControllers,
  startSession, pauseSession, resumeSession, changeSessionControllers,
  showSale, saveSale, removeItem, checkout, confirmCheckout, cancelCheckout,
  openReceipt, closeReceipt, printReceipt, closeDialog, filterHistory, changePage, exportHistory,
};
async function runAction(action, args = [], control = null, form = null) {
  if (state.busy || !Object.hasOwn(actions, action)) return;
  state.busy = true;
  if (control) control.disabled = true;
  const errorElement = form?.querySelector(".form-error") || document.querySelector("dialog[open] .form-error");
  if (errorElement) errorElement.textContent = "";
  try { await actions[action](...args); }
  catch (error) {
    if (errorElement) errorElement.textContent = String(error);
    else notify(String(error), true);
  } finally {
    state.busy = false;
    if (control) control.disabled = false;
  }
}
document.addEventListener("pointerdown", () => {
  document.documentElement.dataset.input = "pointer";
});
document.addEventListener("click", (event) => {
  const menu = document.querySelector(".history-menu");
  if (menu?.open && !menu.contains(event.target)) menu.open = false;
  const control = event.target.closest("button[data-action]");
  if (control && !control.disabled) runAction(control.dataset.action, JSON.parse(control.dataset.args || "[]"), control);
});
document.addEventListener("submit", (event) => {
  const form = event.target.closest("form[data-submit]");
  if (!form) return;
  event.preventDefault();
  runAction(form.dataset.submit, [], form.querySelector('button[type="submit"]'), form);
});
document.addEventListener("keydown", (event) => {
  document.documentElement.dataset.input = "keyboard";
  if (event.key === "Escape") {
    const menu = document.querySelector(".history-menu");
    if (menu?.open) menu.open = false;
  }
  if (event.key === "Escape" && state.busy && !$("confirmationDialog").open && document.querySelector("dialog[open]")) event.preventDefault();
});
window.addEventListener("DOMContentLoaded", async () => {
  let theme = "dark";
  try { theme = window.localStorage.getItem("gamagement-theme") || theme; } catch {}
  applyTheme(theme === "light" ? "light" : "dark");
  applySidebar(document.documentElement.dataset.sidebar === "collapsed");
  $("historyFrom").value = fmt.jalaliDate().replace(/\/\d{2}$/, "/01");
  $("historyTo").value = fmt.jalaliDate();
  $("consoleFilter").addEventListener("change", renderDashboard);
  $("receiptDialog").addEventListener("cancel", (event) => {
    if (state.invoice?.session.status === "checkout") {
      event.preventDefault();
      runAction("cancelCheckout");
    } else state.invoice = null;
  });
  await runAction("switchView", ["dashboard"]);
  setInterval(refreshLive, 1000);
});
window.addEventListener("unhandledrejection", (event) => {
  event.preventDefault();
  notify(String(event.reason), true);
});
