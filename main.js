const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const sqlite3 = require('sqlite3').verbose();
const path = require('path');
let db;
const dbPath = path.join(app.getPath('userData'), 'gamecenter.db');
function createWindow() {
    const win = new BrowserWindow({
        width: 1200,
        height: 800,
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            contextIsolation: true,
            enableRemoteModule: false,
            nodeIntegration: false
        }
    });

    win.removeMenu();

    win.loadFile('index.html');

    win.on('close', (e) => {
        e.preventDefault();
        dialog.showMessageBox(win, {
            type: 'question',
            buttons: ['بله', 'خیر'],
            title: 'تایید',
            message: 'آیا مطمئن هستید که می‌خواهید برنامه را ببندید؟'
        }).then(({ response }) => {
            if (response === 0) {
                win.destroy();
            }
        });
    });
}

app.whenReady().then(() => {
    initDB();
    createWindow();
    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });
});
app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
});
function initDB() {
    db = new sqlite3.Database(dbPath, (err) => {
        if (err) console.error(err);
    });
    db.serialize(() => {
        db.run(`CREATE TABLE IF NOT EXISTS consoles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            hourlyPrice REAL NOT NULL,
            controllerPrice REAL NOT NULL
        )`);
        db.run(`CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            consoleId INTEGER,
            startTime TEXT,
            pauseTime TEXT,
            endTime TEXT,
            controllers INTEGER,
            status TEXT,
            shopItems TEXT,
            totalPrice REAL
        )`);
    });
}
ipcMain.handle('addConsole', async (event, { name, hourlyPrice, controllerPrice }) => {
    return new Promise((resolve, reject) => {
        db.run(`INSERT INTO consoles (name, hourlyPrice, controllerPrice) VALUES (?, ?, ?)`,
            [name, hourlyPrice, controllerPrice], function(err) {
                if (err) reject(err);
                resolve(this.lastID);
            });
    });
});
ipcMain.handle('updateConsole', async (event, { id, name, hourlyPrice, controllerPrice }) => {
    return new Promise((resolve, reject) => {
        db.run(
            `UPDATE consoles SET name = ?, hourlyPrice = ?, controllerPrice = ? WHERE id = ?`,
            [name, hourlyPrice, controllerPrice, id],
            (err) => err ? reject(err) : resolve()
        );
    });
});
ipcMain.handle('deleteConsole', async (event, id) => {
    return new Promise((resolve, reject) => {
        db.run(`DELETE FROM consoles WHERE id = ?`, [id], (err) => {
            if (err) reject(err);
            else resolve();
        });
    });
});
ipcMain.handle('getConsoles', async () => {
    return new Promise((resolve, reject) => {
        db.all(`SELECT * FROM consoles`, (err, rows) => {
            if (err) reject(err);
            resolve(rows);
        });
    });
});
ipcMain.handle('getConsole', async (event, consoleId) => {
    return new Promise((resolve, reject) => {
        db.get(`SELECT * FROM consoles WHERE id = ?`, [consoleId], (err, row) => {
            if (err) reject(err);
            resolve(row);
        });
    });
});
ipcMain.handle('startSession', async (event, { consoleId, controllers }) => {
    return new Promise((resolve, reject) => {
        db.run(`INSERT INTO sessions (consoleId, startTime, status, controllers, shopItems) VALUES (?, ?, 'active', ?, '[]')`,
            [consoleId, new Date().toISOString(), controllers], function(err) {
                if (err) reject(err);
                resolve(this.lastID);
            });
    });
});
ipcMain.handle('pauseSession', async (event, sessionId) => {
    return new Promise((resolve, reject) => {
        db.run(`UPDATE sessions SET pauseTime = ?, status = 'paused' WHERE id = ?`,
            [new Date().toISOString(), sessionId], (err) => {
                if (err) reject(err);
                resolve();
            });
    });
});
ipcMain.handle('resumeSession', async (event, sessionId) => {
    return new Promise((resolve, reject) => {
        db.run(`UPDATE sessions SET pauseTime = NULL, status = 'active' WHERE id = ?`,
            [sessionId], (err) => {
                if (err) reject(err);
                resolve();
            });
    });
});
ipcMain.handle('endSession', async (event, sessionId) => {
    const session = await new Promise((res, rej) =>
        db.get(`SELECT * FROM sessions WHERE id = ?`, [sessionId], (e, r) => e ? rej(e) : res(r))
    );
    const cons = await new Promise((res, rej) =>
        db.get(`SELECT * FROM consoles WHERE id = ?`, [session.consoleId], (e, r) => e ? rej(e) : res(r))
    );
    const timePlayed = calculateTimePlayed(session);
    const hours = timePlayed / 3600000;
    const timeCost = hours * cons.hourlyPrice;
    const payingControllers = Math.max(0, session.controllers - 2);
    const controllerCost = payingControllers * cons.controllerPrice * hours;
    const gameCost = timeCost + controllerCost;
    const shopCost = calculateShopCost(JSON.parse(session.shopItems || '[]'));
    const total = gameCost + shopCost;
    await new Promise((res, rej) =>
        db.run(
            `UPDATE sessions SET endTime = ?, totalPrice = ?, status = 'ended' WHERE id = ?`,
            [new Date().toISOString(), total, sessionId],
            (e) => e ? rej(e) : res()
        )
    );
    return { gameCost, shopCost, total };
});
function calculateTimePlayed(session) {
    const start = new Date(session.startTime);
    const end = session.endTime ? new Date(session.endTime) : new Date();
    const pause = session.pauseTime ? new Date(session.pauseTime) : null;
    let time = end - start;
    if (pause && session.status === 'paused') time = pause - start;
    return time;
}
function calculateShopCost(items) {
    let total = 0;
    for (let item of items) {
        total += item.price;
    }
    return total;
}
ipcMain.handle('getSessionsForConsole', async (event, consoleId) => {
    return new Promise((resolve, reject) => {
        db.all(`SELECT * FROM sessions WHERE consoleId = ? AND endTime IS NULL`, [consoleId], (err, rows) => {
            if (err) reject(err);
            resolve(rows);
        });
    });
});
ipcMain.handle('getSession', async (event, sessionId) => {
    return new Promise((resolve, reject) => {
        db.get(`SELECT * FROM sessions WHERE id = ?`, [sessionId], (err, row) => {
            if (err) reject(err);
            resolve(row);
        });
    });
});
ipcMain.handle('addShopItemToSession', async (event, { sessionId, name, price }) => {
    return new Promise((resolve, reject) => {
        db.get(`SELECT shopItems FROM sessions WHERE id = ?`, [sessionId], (err, row) => {
            if (err) reject(err);
            let items = JSON.parse(row.shopItems || '[]');
            items.push({ name, price });
            db.run(`UPDATE sessions SET shopItems = ? WHERE id = ?`,
                [JSON.stringify(items), sessionId], (err) => {
                    if (err) reject(err);
                    resolve();
                });
        });
    });
});