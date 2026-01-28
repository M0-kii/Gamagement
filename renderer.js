let lang = 'fa';

const texts = {
    fa: {
        title: 'گیمجمنت',
        consoles: 'کنسول‌ها',
        addConsole: '+ افزودن کنسول',
        shop: 'آیتم‌های فروشگاه',
        addItem: '+ افزودن آیتم',
        hourly: 'تومان / ساعت',
        controller: 'تومان / کنترلر اضافی / ساعت',
        startSession: 'شروع جلسه',
        pause: 'توقف',
        resume: 'ادامه',
        endSession: 'پایان جلسه',
        addShop: '+ افزودن',
        addedItems: 'آیتم‌های اضافه شده:',
        time: 'زمان:',
        price: 'جمع:',
        edit: 'ویرایش',
        delete: 'حذف',
        cancel: 'انصراف',
        save: 'ذخیره',
        addNewConsole: 'کنسول جدید',
        editConsole: 'ویرایش کنسول',
        addNewItem: 'آیتم جدید',
        editItem: 'ویرایش آیتم'
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
    document.querySelector('#consoles button[onclick*="showAddConsoleModal"]').textContent = t('addConsole');
    document.querySelector('#shop h2').textContent = t('shop');
    document.querySelector('#shop button[onclick*="showAddItemModal"]').textContent = t('addItem');
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
            <input id="controllers_${c.id}" type="number" min="1" value="1" style="width:80px; margin:0.8rem 0;">
            <button class="success" onclick="startSession(${c.id})">${t('startSession')}</button>
            <div id="sessions_${c.id}"></div>`;
        list.appendChild(div);
        loadSessionsForConsole(c.id);
    });
}

async function loadShopItems() {
    const items = await window.electronAPI.getShopItems();
    const list = document.getElementById('itemList');
    list.innerHTML = '';
    items.forEach(item => {
        const div = document.createElement('div');
        div.className = 'card';
        div.innerHTML = `
            <strong>${item.name}</strong>
            <div style="margin:0.8rem 0; font-size:1.4rem;">
                ${formatPrice(item.price)}
            </div>
            <div class="card-actions">
                <button class="primary" onclick="editItem(${item.id})">${t('edit')}</button>
                <button class="danger" onclick="deleteItem(${item.id})">${t('delete')}</button>
            </div>`;
        list.appendChild(div);
    });
    document.querySelectorAll('select[id^="itemSelect_"]').forEach(s => {
        s.innerHTML = items.map(i => `<option value="${i.id}">${i.name} – ${formatPrice(i.price)}</option>`).join('');
    });
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
                <div style="margin-top:1rem; display:flex; gap:0.8rem; flex-wrap:wrap;">
                    <select id="itemSelect_${s.id}"></select>
                    <input id="quantity_${s.id}" type="number" min="1" value="1" style="width:80px;">
                    <button class="success" onclick="addItemToSession(${s.id})">${t('addShop')}</button>
                </div>
            </div>`;
        container.appendChild(div);
        loadShopItemsForSession(s.id);
        if (s.status === 'active') setInterval(() => updateTimeAndPrice(s.id), 1000);
    });
}

async function startSession(consoleId) {
    const controllers = parseInt(document.getElementById(`controllers_${consoleId}`).value) || 1;
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

async function endSession(id) {
    const total = await window.electronAPI.endSession(id);
    alert(`${t('price')} ${formatPrice(total)}`);
    document.getElementById(`session_${id}`).remove();
}

async function addItemToSession(id) {
    const itemId = parseInt(document.getElementById(`itemSelect_${id}`).value);
    const qty = parseInt(document.getElementById(`quantity_${id}`).value) || 1;
    await window.electronAPI.addShopItemToSession({ sessionId: id, itemId, quantity: qty });
    loadShopItemsForSession(id);
    updateTimeAndPrice(id);
}

async function loadShopItemsForSession(id) {
    const items = await window.electronAPI.getShopItems();
    document.getElementById(`itemSelect_${id}`).innerHTML = items.map(i => `<option value="${i.id}">${i.name} – ${formatPrice(i.price)}</option>`).join('');

    const session = await window.electronAPI.getSession(id);
    const shopDiv = document.getElementById(`shop_${id}`);
    const shop = JSON.parse(session.shopItems || '[]');
    shopDiv.innerHTML = shop.length
        ? `<p>${t('addedItems')}</p>` + (await Promise.all(shop.map(async s => {
            const price = await window.electronAPI.getItemPrice(s.itemId);
            const item = items.find(i => i.id === s.itemId);
            return `<div>${s.quantity}× ${item?.name || 'نامشخص'} (${formatPrice(price)})</div>`;
        }))).join('')
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
    const shopCost = await calculateShopCostClient(JSON.parse(s.shopItems || '[]'));

    const total = timeCost + ctrlCost + shopCost;
    document.getElementById(`price_${id}`).textContent = formatPrice(total);
}

async function calculateShopCostClient(items) {
    let sum = 0;
    for (let item of items) {
        const p = await window.electronAPI.getItemPrice(item.itemId);
        sum += p * item.quantity;
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

function showItemModal(isEdit = false, id = '', name = '', price = '') {
    const title = isEdit ? t('editItem') : t('addNewItem');
    const btn = isEdit ? t('save') : t('addItem').replace('+ ', '');
    document.getElementById('itemModalTitle').textContent = title;
    document.getElementById('itemSaveBtn').textContent = btn;
    document.getElementById('itemName').value = name;
    document.getElementById('itemPrice').value = price;
    document.getElementById('itemEditId').value = id;
    document.getElementById('itemModal').style.display = 'flex';
}

function hideItemModal() {
    document.getElementById('itemModal').style.display = 'none';
}

function showAddItemModal() { showItemModal(false); }

async function editItem(id) {
    const i = await window.electronAPI.getShopItem(id);
    showItemModal(true, id, i.name, i.price);
}

async function saveItem() {
    try {
        const id = document.getElementById('itemEditId').value;
        const name = document.getElementById('itemName').value.trim();
        const price = parseFloat(document.getElementById('itemPrice').value);
        if (!name || isNaN(price) || price <= 0) return alert('ورودی نامعتبر');
        if (id) {
            await window.electronAPI.updateShopItem({ id: Number(id), name, price });
        } else {
            await window.electronAPI.addShopItem({ name, price });
        }
        hideItemModal();
        loadShopItems();
    } catch (err) {
        alert(err.message.includes('already exists') ? 'این نام قبلاً استفاده شده است!' : 'خطا در ذخیره');
    }
}

async function deleteConsole(id) {
    if (!confirm(t('delete'))) return;
    await window.electronAPI.deleteConsole(id);
    loadConsoles();
}

async function deleteItem(id) {
    if (!confirm(t('delete'))) return;
    await window.electronAPI.deleteShopItem(id);
    loadShopItems();
}

window.addEventListener('DOMContentLoaded', () => {
    updateUI();
    loadConsoles();
    loadShopItems();
});