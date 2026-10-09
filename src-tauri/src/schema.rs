use crate::models::AppResult;
use rusqlite::{params, Connection};
use serde::Deserialize;

const SCHEMA: &str = "
CREATE TABLE consoles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    hourlyPrice INTEGER NOT NULL CHECK(hourlyPrice >= 0),
    controllerPrice INTEGER NOT NULL CHECK(controllerPrice >= 0),
    archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0,1))
);
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    consoleId INTEGER NOT NULL REFERENCES consoles(id),
    consoleName TEXT NOT NULL,
    startTime TEXT NOT NULL,
    pauseTime TEXT,
    endTime TEXT,
    paidAt TEXT,
    controllers INTEGER NOT NULL CHECK(controllers BETWEEN 1 AND 4),
    status TEXT NOT NULL CHECK(status IN ('active','paused','checkout','ended')),
    hourlyPrice INTEGER NOT NULL CHECK(hourlyPrice >= 0),
    controllerPrice INTEGER NOT NULL CHECK(controllerPrice >= 0),
    pausedMs INTEGER NOT NULL DEFAULT 0 CHECK(pausedMs >= 0),
    checkoutAt TEXT,
    checkoutPreviousStatus TEXT,
    gameCost INTEGER,
    shopCost INTEGER,
    totalPrice INTEGER,
    legacy INTEGER NOT NULL DEFAULT 0 CHECK(legacy IN (0,1))
);
CREATE TABLE session_segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sessionId INTEGER NOT NULL REFERENCES sessions(id),
    startTime TEXT NOT NULL,
    endTime TEXT,
    controllers INTEGER NOT NULL CHECK(controllers BETWEEN 1 AND 4)
);
CREATE UNIQUE INDEX one_running_segment ON session_segments(sessionId) WHERE endTime IS NULL;
CREATE INDEX sessions_console_open ON sessions(consoleId, endTime);
CREATE INDEX sessions_paid ON sessions(paidAt) WHERE status = 'ended';
CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    price INTEGER NOT NULL CHECK(price >= 0),
    stock INTEGER NOT NULL CHECK(stock >= 0),
    archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0,1))
);
CREATE TABLE session_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sessionId INTEGER NOT NULL REFERENCES sessions(id),
    productId INTEGER REFERENCES products(id),
    name TEXT NOT NULL,
    unitPrice INTEGER NOT NULL CHECK(unitPrice >= 0),
    quantity INTEGER NOT NULL CHECK(quantity > 0)
);
CREATE INDEX session_items_session ON session_items(sessionId);
CREATE TRIGGER console_available BEFORE INSERT ON sessions
WHEN EXISTS(SELECT 1 FROM sessions WHERE consoleId = NEW.consoleId AND endTime IS NULL)
BEGIN SELECT RAISE(ABORT, 'این کنسول جلسه باز دارد'); END;
PRAGMA user_version = 2;
";

#[derive(Deserialize)]
struct OldItem {
    name: String,
    price: f64,
}

fn money(value: f64) -> AppResult<i64> {
    if !value.is_finite() || !(0.0..=9_000_000_000_000.0).contains(&value) {
        return Err("Legacy database contains an invalid price".into());
    }
    Ok(value.round() as i64)
}

pub fn init(conn: &Connection) -> AppResult<()> {
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "foreign_keys", true)
        .map_err(|e| e.to_string())?;
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if version == 2 {
        return Ok(());
    }
    if version > 2 {
        return Err("Database was created by a newer version of Gamagement".into());
    }
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'sessions')",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    if !exists {
        tx.execute_batch(SCHEMA).map_err(|e| e.to_string())?;
        return tx.commit().map_err(|e| e.to_string());
    }
    let consoles = {
        let mut q = tx
            .prepare("SELECT id, name, hourlyPrice, controllerPrice FROM consoles")
            .map_err(|e| e.to_string())?;
        let rows = q
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, f64>(2)?,
                    r.get::<_, f64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?
    };
    let sessions = {
        let mut q = tx.prepare("SELECT id, consoleId, startTime, pauseTime, endTime, controllers, status, shopItems, totalPrice FROM sessions ORDER BY id").map_err(|e| e.to_string())?;
        let rows = q
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, Option<String>>(7)?
                        .unwrap_or_else(|| "[]".into()),
                    r.get::<_, Option<f64>>(8)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?
    };
    tx.execute_batch("DROP TABLE sessions; DROP TABLE consoles;")
        .map_err(|e| e.to_string())?;
    // Existing installations may have several open sessions on the same console.
    tx.execute_batch(SCHEMA).map_err(|e| e.to_string())?;
    tx.execute_batch("DROP TRIGGER console_available;")
        .map_err(|e| e.to_string())?;
    for (id, name, hourly, controller) in consoles {
        tx.execute(
            "INSERT INTO consoles (id,name,hourlyPrice,controllerPrice) VALUES (?,?,?,?)",
            params![id, name, money(hourly)?, money(controller)?],
        )
        .map_err(|e| e.to_string())?;
    }
    for (id, console_id, start, pause, end, controllers, status, json, old_total) in sessions {
        let cons: Option<(String, i64, i64)> = tx
            .query_row(
                "SELECT name,hourlyPrice,controllerPrice FROM consoles WHERE id=?",
                [console_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let (name, hourly, controller) =
            cons.unwrap_or((format!("کنسول بایگانی‌شده #{console_id}"), 0, 0));
        tx.execute("INSERT OR IGNORE INTO consoles (id,name,hourlyPrice,controllerPrice,archived) VALUES (?,?,?,?,1)", params![console_id,name,hourly,controller]).map_err(|e| e.to_string())?;
        let items: Vec<OldItem> =
            serde_json::from_str(&json).map_err(|e| format!("Session {id}: {e}"))?;
        let shop = items.iter().try_fold(0_i64, |sum, item| {
            sum.checked_add(money(item.price)?)
                .ok_or_else(|| "Legacy shop total overflow".to_string())
        })?;
        let ended = end.is_some();
        let total = old_total.map(money).transpose()?.unwrap_or(shop);
        let state = if ended {
            "ended"
        } else if status == "paused" {
            "paused"
        } else {
            "active"
        };
        tx.execute("INSERT INTO sessions (id,consoleId,consoleName,startTime,pauseTime,endTime,paidAt,controllers,status,hourlyPrice,controllerPrice,gameCost,shopCost,totalPrice,legacy) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,1)",
            params![id,console_id,name,start,pause,end,end,controllers,state,hourly,controller,if ended {Some((total-shop).max(0))} else {None},if ended {Some(shop.min(total))} else {None},if ended {Some(total)} else {None}]
        ).map_err(|e| e.to_string())?;
        let stop = if ended {
            end.as_deref()
        } else if state == "paused" {
            pause.as_deref()
        } else {
            None
        };
        tx.execute("INSERT INTO session_segments (sessionId,startTime,endTime,controllers) VALUES (?,?,?,?)",params![id,start,stop,controllers]).map_err(|e| e.to_string())?;
        for item in items {
            tx.execute(
                "INSERT INTO session_items (sessionId,name,unitPrice,quantity) VALUES (?,?,?,1)",
                params![id, item.name, money(item.price)?],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    tx.execute_batch("CREATE TRIGGER console_available BEFORE INSERT ON sessions WHEN EXISTS(SELECT 1 FROM sessions WHERE consoleId=NEW.consoleId AND endTime IS NULL) BEGIN SELECT RAISE(ABORT, 'این کنسول جلسه باز دارد'); END;").map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

use rusqlite::OptionalExtension;
