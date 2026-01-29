let lang = 'fa';
const texts = {
    fa: {
        title: 'گیمجمنت',
        consoles: 'کنسول‌ها',
        addConsole: '+ افزودن کنسول',
        hourly: 'تومان / ساعت',
        controller: 'تومان / کنترلر اضافی / ساعت',
        startSession: 'شروع جلسه',
        pause: 'توقف',
        resume: 'ادامه',
        endSession: 'پایان جلسه',
        addShop: '+ افزودن آیتم',
        addedItems: 'آیتم‌های اضافه شده:',
        time: 'زمان:',
        price: 'جمع:',
        edit: 'ویرایش',
        delete: 'حذف',
        cancel: 'انصراف',
        save: 'ذخیره',
        addNewConsole: 'کنسول جدید',
        editConsole: 'ویرایش کنسول'
    }
};
function t(key) {
    return texts[lang][key] || key;
}
function formatPrice(value) {
    return Math.round(Number(value)).toLocaleString('fa-IR') + ' تومان';
}
function updateUI() {
    document.querySelector('h1').textContent = t('title');
    document.querySelector('#consoles h2').textContent = t('consoles');
    document.querySelector('#addConsoleBtn').textContent = t('addConsole');
}
async function loadConsoles() {
    const consoles = await window.electronAPI.getConsoles();
    const list = document.getElementById('consoleList');
    list.innerHTML = '';
    consoles.forEach(c => {
        const div = document.createElement('div');
        div.className = 'card';
        div.innerHTML = `
            <h3>${c.name}</h3>
            <p>${formatPrice(c.hourlyPrice)} ${t('hourly')}</p>
            <p>${formatPrice(c.controllerPrice)} ${t('controller')}</p>
            <div class="card-actions">
                <button class="primary" onclick="editConsole(${c.id})">${t('edit')}</button>
                <button class="danger" onclick="deleteConsole(${c.id})">${t('delete')}</button>
            </div>
            <div class="controller-selector" style="margin:0.8rem 0; display:flex; gap:0.5rem; align-items:center;">
                <button onclick="changeControllers(${c.id}, -1)">-</button>
                <span id="controllers_${c.id}">1</span>
                <button onclick="changeControllers(${c.id}, 1)">+</button>
            </div>
            <button class="success" onclick="startSession(${c.id})">${t('startSession')}</button>
            <div id="sessions_${c.id}"></div>`;
        list.appendChild(div);
        loadSessionsForConsole(c.id);
    });
}
function changeControllers(id, delta) {
    const span = document.getElementById(`controllers_${id}`);
    let val = parseInt(span.textContent) + delta;
    if (val < 1) val = 1;
    span.textContent = val;
}
async function loadSessionsForConsole(consoleId) {
    const sessions = await window.electronAPI.getSessionsForConsole(consoleId);
    const container = document.getElementById(`sessions_${consoleId}`);
    container.innerHTML = '';
    sessions.forEach(s => {
        const div = document.createElement('div');
        div.id = `session_${s.id}`;
        div.innerHTML = `
            <div class="session-block">
                <p>جلسه #${s.id} • ${s.controllers} کنترلر</p>
                <div class="time" id="time_${s.id}">${t('time')} ۰۰:۰۰:۰۰</div>
                <div class="price" id="price_${s.id}">${formatPrice(0)}</div>
                <button class="warning" onclick="pauseSession(${s.id})">${t('pause')}</button>
                <button class="success" onclick="resumeSession(${s.id})" style="display:none;">${t('resume')}</button>
                <button class="danger" onclick="endSession(${s.id})">${t('endSession')}</button>
                <div id="shop_${s.id}" style="margin-top:1rem;"></div>
                <button class="success" style="margin-top:1rem;" onclick="showSessionItemModal(${s.id})">${t('addShop')}</button>
            </div>`;
        container.appendChild(div);
        loadShopItemsForSession(s.id);
        if (s.status === 'active') setInterval(() => updateTimeAndPrice(s.id), 1000);
    });
}
async function startSession(consoleId) {
    const controllers = parseInt(document.getElementById(`controllers_${consoleId}`).textContent) || 1;
    await window.electronAPI.startSession({ consoleId, controllers });
    loadSessionsForConsole(consoleId);
}
async function pauseSession(id) {
    await window.electronAPI.pauseSession(id);
    document.querySelector(`#session_${id} .session-block`).classList.add('paused');
    document.querySelector(`#session_${id} button[onclick="pauseSession(${id})"]`).style.display = 'none';
    document.querySelector(`#session_${id} button[onclick="resumeSession(${id})"]`).style.display = 'inline-block';
}
async function resumeSession(id) {
    await window.electronAPI.resumeSession(id);
    document.querySelector(`#session_${id} .session-block`).classList.remove('paused');
    document.querySelector(`#session_${id} button[onclick="resumeSession(${id})"]`).style.display = 'none';
    document.querySelector(`#session_${id} button[onclick="pauseSession(${id})"]`).style.display = 'inline-block';
}
let endingSessionId;
async function endSession(id) {
    endingSessionId = id;
    const data = await window.electronAPI.endSession(id);
    document.getElementById('gamePrice').textContent = formatPrice(data.gameCost);
    document.getElementById('shopPrice').textContent = formatPrice(data.shopCost);
    document.getElementById('totalPrice').textContent = formatPrice(data.total);
    document.getElementById('endSessionModal').style.display = 'flex';
}
function hideEndSessionModal() {
    document.getElementById('endSessionModal').style.display = 'none';
    document.getElementById(`session_${endingSessionId}`).remove();
}
function showSessionItemModal(id) {
    document.getElementById('sessionIdForItem').value = id;
    document.getElementById('sessionItemModal').style.display = 'flex';
}
function hideSessionItemModal() {
    document.getElementById('sessionItemModal').style.display = 'none';
}
async function saveSessionItem() {
    const sessionId = parseInt(document.getElementById('sessionIdForItem').value);
    const name = document.getElementById('sessionItemName').value.trim();
    const price = parseFloat(document.getElementById('sessionItemPrice').value);
    if (!name || isNaN(price) || price <= 0) return alert('ورودی نامعتبر');
    await window.electronAPI.addShopItemToSession({ sessionId, name, price });
    hideSessionItemModal();
    loadShopItemsForSession(sessionId);
    updateTimeAndPrice(sessionId);
}
async function loadShopItemsForSession(id) {
    const session = await window.electronAPI.getSession(id);
    const shopDiv = document.getElementById(`shop_${id}`);
    const shop = JSON.parse(session.shopItems || '[]');
    shopDiv.innerHTML = shop.length
        ? `<p>${t('addedItems')}</p>` + shop.map(s => `<div>${s.name} (${formatPrice(s.price)})</div>`).join('')
        : '';
}
async function updateTimeAndPrice(id) {
    const s = await window.electronAPI.getSession(id);
    if (s.status !== 'active') return;
    const c = await window.electronAPI.getConsole(s.consoleId);
    const timeMs = Date.now() - new Date(s.startTime).getTime();
    const h = Math.floor(timeMs / 3600000);
    const m = Math.floor((timeMs % 3600000) / 60000);
    const sec = Math.floor((timeMs % 60000) / 1000);
    document.getElementById(`time_${id}`).textContent =
        `${t('time')} ${h}:${m.toString().padStart(2,'0')}:${sec.toString().padStart(2,'0')}`;
    const hours = timeMs / 3600000;
    const timeCost = hours * c.hourlyPrice;
    const payingControllers = Math.max(0, s.controllers - 2);
    const ctrlCost = payingControllers * c.controllerPrice * hours;
    const shopCost = calculateShopCostClient(JSON.parse(s.shopItems || '[]'));
    const total = timeCost + ctrlCost + shopCost;
    document.getElementById(`price_${id}`).textContent = formatPrice(total);
}
function calculateShopCostClient(items) {
    let sum = 0;
    for (let item of items) {
        sum += item.price;
    }
    return sum;
}
function showConsoleModal(isEdit = false, id = '', name = '', h = '', ctrl = '') {
    const title = isEdit ? t('editConsole') : t('addNewConsole');
    const btn = isEdit ? t('save') : t('addConsole').replace('+ ', '');
    document.getElementById('consoleModalTitle').textContent = title;
    document.getElementById('consoleSaveBtn').textContent = btn;
    document.getElementById('consoleName').value = name;
    document.getElementById('hourlyPrice').value = h;
    document.getElementById('controllerPrice').value = ctrl;
    document.getElementById('consoleEditId').value = id;
    document.getElementById('consoleModal').style.display = 'flex';
}
function hideConsoleModal() {
    document.getElementById('consoleModal').style.display = 'none';
}
function showAddConsoleModal() { showConsoleModal(false); }
async function editConsole(id) {
    const c = await window.electronAPI.getConsole(id);
    showConsoleModal(true, id, c.name, c.hourlyPrice, c.controllerPrice);
}
async function saveConsole() {
    const id = document.getElementById('consoleEditId').value;
    const name = document.getElementById('consoleName').value.trim();
    const h = parseFloat(document.getElementById('hourlyPrice').value);
    const ctrl = parseFloat(document.getElementById('controllerPrice').value);
    if (!name || isNaN(h) || isNaN(ctrl)) return alert('ورودی نامعتبر');
    if (id) {
        await window.electronAPI.updateConsole({ id: Number(id), name, hourlyPrice: h, controllerPrice: ctrl });
    } else {
        await window.electronAPI.addConsole({ name, hourlyPrice: h, controllerPrice: ctrl });
    }
    hideConsoleModal();
    loadConsoles();
}
async function deleteConsole(id) {
    if (!confirm(t('delete'))) return;
    await window.electronAPI.deleteConsole(id);
    loadConsoles();
}
window.addEventListener('DOMContentLoaded', () => {
    updateUI();
    loadConsoles();
});