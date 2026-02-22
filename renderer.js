let lang = "fa";
const texts = {
  fa: {
    title: "Gamagement",
    consoles: "مدیریت کنسول‌ها",
    addConsole: "+ افزودن کنسول",
    hourly: "تومان / ساعت",
    controller: "کنترلر اضافی (ساعت)",
    startSession: "شروع بازی",
    pause: "توقف موقت",
    resume: "ادامه بازی",
    endSession: "پایان و تسویه",
    addShop: "+ افزودن آیتم فروشگاه",
    addedItems: "آیتم‌ها:",
    time: "زمان:",
    edit: "ویرایش",
    delete: "حذف",
  },
};

const icons = {
  edit: `<svg viewBox="0 0 24 24"><path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/></svg>`,
  delete: `<svg viewBox="0 0 24 24"><path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/></svg>`,
};

function t(key) {
  return texts[lang][key] || key;
}

function formatPrice(value) {
  return Math.round(Number(value)).toLocaleString("fa-IR") + " تومان";
}

async function loadConsoles() {
  const consoles = await window.electronAPI.getConsoles();
  const list = document.getElementById("consoleList");
  list.innerHTML = "";

  consoles.forEach((c) => {
    const div = document.createElement("div");
    div.className = "card";
    div.innerHTML = `
            <div class="card-actions">
                <button class="btn-icon edit-icon" title="${t("edit")}" onclick="editConsole(${c.id})">${icons.edit}</button>
                <button class="btn-icon delete-icon" title="${t("delete")}" onclick="deleteConsole(${c.id})">${icons.delete}</button>
            </div>
            <div class="card-content">
                <h3>${c.name}</h3>
                <div class="price-info">💰 ${formatPrice(c.hourlyPrice)} ${t("hourly")}</div>
                <div class="price-info">🎮 ${formatPrice(c.controllerPrice)} ${t("controller")}</div>
                
                <div class="controller-selector">
                    <span style="font-size:0.85rem">تعداد دسته:</span>
                    <div style="display:flex; align-items:center; gap:8px">
                        <button class="selector-btn" onclick="changeControllers(${c.id}, -1)">-</button>
                        <span id="controllers_${c.id}" style="font-weight:bold; min-width:15px; text-align:center">1</span>
                        <button class="selector-btn" onclick="changeControllers(${c.id}, 1)">+</button>
                    </div>
                </div>

                <button class="btn-success full-width" onclick="startSession(${c.id})">${t("startSession")}</button>
                <div id="sessions_${c.id}"></div>
            </div>
        `;
    list.appendChild(div);
    loadSessionsForConsole(c.id);
  });
}

function changeControllers(id, delta) {
  const span = document.getElementById(`controllers_${id}`);
  let val = parseInt(span.textContent) + delta;
  if (val < 1) val = 1;
  if (val > 4) val = 4; // Added max limit
  span.textContent = val;
}

async function loadSessionsForConsole(consoleId) {
  const sessions = await window.electronAPI.getSessionsForConsole(consoleId);
  const container = document.getElementById(`sessions_${consoleId}`);
  container.innerHTML = "";

  sessions.forEach((s) => {
    const div = document.createElement("div");
    div.id = `session_${s.id}`;
    div.className = `session-block ${s.status === "paused" ? "paused" : ""}`;

    div.innerHTML = `
            <div style="display:flex; justify-content:space-between; align-items:center">
                <span style="font-size:0.8rem; color:var(--text-muted)">
                    <span class="status-dot ${s.status === "active" ? "pulse" : ""}"></span>
                    جلسه #${s.id}
                </span>
                <div style="display:flex; align-items:center; gap:8px">
                    <button class="selector-btn" onclick="updateSessionControllers(${s.id}, -1)">-</button>
                    <span id="session_controllers_${s.id}" style="font-weight:bold; min-width:15px; text-align:center">${s.controllers}</span>
                    <button class="selector-btn" onclick="updateSessionControllers(${s.id}, 1)">+</button>
                    <span style="font-size:0.75rem; color:var(--text-muted)">دسته</span>
                </div>
            </div>
            <div class="time" id="time_${s.id}">۰۰:۰۰:۰۰</div>
            <div class="price-display" id="price_${s.id}">${formatPrice(0)}</div>
            
            <div style="display:flex; gap:8px; margin-top:15px">
                <button class="btn-warning" id="pauseBtn_${s.id}" onclick="pauseSession(${s.id})" style="display: ${s.status === "active" ? "block" : "none"}">⏸ ${t("pause")}</button>
                <button class="btn-success" id="resumeBtn_${s.id}" onclick="resumeSession(${s.id})" style="display: ${s.status === "paused" ? "block" : "none"}">▶ ${t("resume")}</button>
                <button class="btn-danger" onclick="endSession(${s.id})">🏁 ${t("endSession")}</button>
            </div>

            
            <div id="shop_${s.id}" style="margin-top:12px; font-size:0.8rem; color:var(--text-muted); border-top:1px solid rgba(255,255,255,0.05); padding-top:8px"></div>
            <button class="btn-primary" style="margin-top:10px; font-size:0.8rem; width:100%" onclick="showSessionItemModal(${s.id})">${t("addShop")}</button>
        `;
    container.appendChild(div);

    loadShopItemsForSession(s.id);
    if (s.status === "active")
      setInterval(() => updateTimeAndPrice(s.id), 1000);
    else updateTimeAndPrice(s.id);
  });
}

async function startSession(consoleId) {
  const controllers =
    parseInt(document.getElementById(`controllers_${consoleId}`).textContent) ||
    1;
  await window.electronAPI.startSession({ consoleId, controllers });
  loadSessionsForConsole(consoleId);
}

async function pauseSession(id) {
  await window.electronAPI.pauseSession(id);
  document.querySelector(`#session_${id}`).classList.add("paused");
  document.getElementById(`pauseBtn_${id}`).style.display = "none";
  document.getElementById(`resumeBtn_${id}`).style.display = "block";
}

async function resumeSession(id) {
  await window.electronAPI.resumeSession(id);
  document.querySelector(`#session_${id}`).classList.remove("paused");
  document.getElementById(`resumeBtn_${id}`).style.display = "none";
  document.getElementById(`pauseBtn_${id}`).style.display = "block";
}

let endingSessionId;
async function endSession(id) {
  endingSessionId = id;
  const s = await window.electronAPI.getSession(id);
  const c = await window.electronAPI.getConsole(s.consoleId);
  const now = s.status === "paused" ? new Date(s.pauseTime) : new Date();
  const timeMs = now.getTime() - new Date(s.startTime).getTime();
  const hours = timeMs / 3600000;
  const timeCost = hours * c.hourlyPrice;
  const payingControllers = Math.max(0, s.controllers - 2);
  const ctrlCost = payingControllers * c.controllerPrice * hours;
  const shopCost = calculateShopCostClient(JSON.parse(s.shopItems || "[]"));
  const total = timeCost + ctrlCost + shopCost;

  document.getElementById("gamePrice").textContent = formatPrice(
    timeCost + ctrlCost,
  );
  document.getElementById("shopPrice").textContent = formatPrice(shopCost);
  document.getElementById("totalPrice").textContent = formatPrice(total);
  document.getElementById("endSessionModal").style.display = "flex";
}

async function confirmEndSession() {
  await window.electronAPI.endSession(endingSessionId);
  document.getElementById(`session_${endingSessionId}`).remove();
  hideEndSessionModal();
}

function hideEndSessionModal() {
  document.getElementById("endSessionModal").style.display = "none";
}

async function updateTimeAndPrice(id) {
  const s = await window.electronAPI.getSession(id);
  if (!s) return;
  const c = await window.electronAPI.getConsole(s.consoleId);
  const now =
    s.status === "active" ? Date.now() : new Date(s.pauseTime).getTime();
  const timeMs = now - new Date(s.startTime).getTime();

  const h = Math.floor(timeMs / 3600000);
  const m = Math.floor((timeMs % 3600000) / 60000);
  const sec = Math.floor((timeMs % 60000) / 1000);

  const timeEl = document.getElementById(`time_${id}`);
  if (timeEl)
    timeEl.textContent = `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}:${sec.toString().padStart(2, "0")}`;

  const hours = timeMs / 3600000;
  const total =
    hours * c.hourlyPrice +
    Math.max(0, s.controllers - 2) * c.controllerPrice * hours +
    calculateShopCostClient(JSON.parse(s.shopItems || "[]"));

  const priceEl = document.getElementById(`price_${id}`);
  if (priceEl) priceEl.textContent = formatPrice(total);
}

function calculateShopCostClient(items) {
  return items.reduce((sum, item) => sum + item.price, 0);
}

function showConsoleModal(
  isEdit = false,
  id = "",
  name = "",
  h = "",
  ctrl = "",
) {
  document.getElementById("consoleModalTitle").textContent = isEdit
    ? "ویرایش کنسول"
    : "افزودن کنسول";
  document.getElementById("consoleName").value = name;
  document.getElementById("hourlyPrice").value = h;
  document.getElementById("controllerPrice").value = ctrl;
  document.getElementById("consoleEditId").value = id;
  document.getElementById("consoleModal").style.display = "flex";
}
function hideConsoleModal() {
  document.getElementById("consoleModal").style.display = "none";
}
function showAddConsoleModal() {
  showConsoleModal(false);
}
async function editConsole(id) {
  const c = await window.electronAPI.getConsole(id);
  showConsoleModal(true, id, c.name, c.hourlyPrice, c.controllerPrice);
}
async function saveConsole() {
  const id = document.getElementById("consoleEditId").value;
  const name = document.getElementById("consoleName").value.trim();
  const h = parseFloat(document.getElementById("hourlyPrice").value);
  const ctrl = parseFloat(document.getElementById("controllerPrice").value);
  if (!name || isNaN(h)) return alert("ورودی نامعتبر");
  if (id)
    await window.electronAPI.updateConsole({
      id: Number(id),
      name,
      hourlyPrice: h,
      controllerPrice: ctrl,
    });
  else
    await window.electronAPI.addConsole({
      name,
      hourlyPrice: h,
      controllerPrice: ctrl,
    });
  hideConsoleModal();
  loadConsoles();
}
async function deleteConsole(id) {
  if (!confirm("آیا مطمئن هستید؟")) return;
  await window.electronAPI.deleteConsole(id);
  loadConsoles();
}
function showSessionItemModal(id) {
  document.getElementById("sessionIdForItem").value = id;
  document.getElementById("sessionItemName").value = "";
  document.getElementById("sessionItemPrice").value = "";
  document.getElementById("sessionItemModal").style.display = "flex";
}
function hideSessionItemModal() {
  document.getElementById("sessionItemModal").style.display = "none";
}
async function saveSessionItem() {
  const sessionId = parseInt(document.getElementById("sessionIdForItem").value);
  const name = document.getElementById("sessionItemName").value.trim();
  const price = parseFloat(document.getElementById("sessionItemPrice").value);
  if (!name || isNaN(price)) return;
  await window.electronAPI.addShopItemToSession({ sessionId, name, price });
  hideSessionItemModal();
  loadShopItemsForSession(sessionId);
  updateTimeAndPrice(sessionId);
}
async function loadShopItemsForSession(id) {
  const session = await window.electronAPI.getSession(id);
  const shopDiv = document.getElementById(`shop_${id}`);
  const items = JSON.parse(session.shopItems || "[]");
  shopDiv.innerHTML = "";
  if (items.length > 0) {
    items.forEach((item, index) => {
      const row = document.createElement("div");
      row.className = "shop-item-row";
      row.innerHTML = `
                <div class="shop-item-info">
                    <span>${item.name}</span>
                    <span class="shop-item-price">${formatPrice(item.price)}</span>
                </div>
                <div class="shop-item-actions">
                    <button class="shop-action-btn edit-text" onclick="editShopItem(${id}, ${index})">ویرایش</button>
                    <button class="shop-action-btn delete-text" onclick="deleteShopItem(${id}, ${index})">حذف</button>
                </div>
            `;
      shopDiv.appendChild(row);
    });
  }
}

async function updateSessionControllers(id, delta) {
  const span = document.getElementById(`session_controllers_${id}`);
  let val = parseInt(span.textContent) + delta;
  if (val < 1) val = 1;
  if (val > 4) val = 4;
  span.textContent = val;
  await window.electronAPI.updateSessionControllers({ sessionId: id, controllers: val });
  updateTimeAndPrice(id);
}

async function editShopItem(sessionId, index) {
  const session = await window.electronAPI.getSession(sessionId);
  const items = JSON.parse(session.shopItems || "[]");
  const item = items[index];
  if (item) {
    document.getElementById("sessionItemEditName").value = item.name;
    document.getElementById("sessionItemEditPrice").value = item.price;
    document.getElementById("editItemSessionId").value = sessionId;
    document.getElementById("editItemIndex").value = index;
    document.getElementById("sessionItemEditModal").style.display = "flex";
  }
}

function hideSessionItemEditModal() {
  document.getElementById("sessionItemEditModal").style.display = "none";
}

async function saveSessionItemEdit() {
  const sessionId = parseInt(document.getElementById("editItemSessionId").value);
  const index = parseInt(document.getElementById("editItemIndex").value);
  const name = document.getElementById("sessionItemEditName").value.trim();
  const price = parseFloat(document.getElementById("sessionItemEditPrice").value);
  if (!name || isNaN(price)) return;
  await window.electronAPI.updateShopItem({ sessionId, itemIndex: index, name, price });
  hideSessionItemEditModal();
  loadShopItemsForSession(sessionId);
  updateTimeAndPrice(sessionId);
}

async function deleteShopItem(sessionId, index) {
  if (!confirm("آیا از حذف این آیتم مطمئن هستید؟")) return;
  await window.electronAPI.deleteShopItem({ sessionId, itemIndex: index });
  loadShopItemsForSession(sessionId);
  updateTimeAndPrice(sessionId);
}


window.addEventListener("DOMContentLoaded", () => {
  loadConsoles();
});
