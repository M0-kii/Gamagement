use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Mutex};

pub type AppResult<T> = Result<T, String>;
pub struct Database(pub Mutex<Connection>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Console {
    pub id: i64,
    pub name: String,
    pub hourly_price: f64,
    pub controller_price: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleInput {
    pub id: Option<i64>,
    pub name: String,
    pub hourly_price: f64,
    pub controller_price: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub console_id: i64,
    pub start_time: String,
    pub pause_time: Option<String>,
    pub end_time: Option<String>,
    pub controllers: i64,
    pub status: String,
    pub shop_items: String,
    pub total_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartInput {
    pub console_id: i64,
    pub controllers: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllersInput {
    pub session_id: i64,
    pub controllers: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShopItem {
    pub name: String,
    pub price: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopInput {
    pub session_id: i64,
    pub name: String,
    pub price: f64,
    pub item_index: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteItemInput {
    pub session_id: i64,
    pub item_index: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub game_cost: f64,
    pub shop_cost: f64,
    pub total: f64,
}

fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}

pub fn init(conn: &Connection) -> AppResult<()> {
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(error)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS consoles (
            id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL,
            hourlyPrice REAL NOT NULL, controllerPrice REAL NOT NULL
        );
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT, consoleId INTEGER,
            startTime TEXT, pauseTime TEXT, endTime TEXT, controllers INTEGER,
            status TEXT, shopItems TEXT, totalPrice REAL
        );
        CREATE INDEX IF NOT EXISTS sessions_console_open ON sessions(consoleId, endTime);",
    )
    .map_err(error)
}

// SQLite's backup API also includes committed data in an Electron WAL file.
// Publish only a finished, validated copy; never modify the original database.
pub fn open_database(destination: &Path, legacy: Option<&Path>) -> AppResult<Connection> {
    if !destination.exists() {
        if let Some(source) = legacy.filter(|p| p.is_file() && *p != destination) {
            let pending = destination.with_extension("migration");
            let migration = (|| {
                let source =
                    Connection::open_with_flags(source, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                        .map_err(error)?;
                let mut copy = Connection::open(&pending).map_err(error)?;
                {
                    let backup =
                        rusqlite::backup::Backup::new(&source, &mut copy).map_err(error)?;
                    backup
                        .run_to_completion(100, std::time::Duration::from_millis(10), None)
                        .map_err(error)?;
                }
                let check: String = copy
                    .query_row("PRAGMA integrity_check", [], |r| r.get(0))
                    .map_err(error)?;
                if check != "ok" {
                    return Err("Legacy database integrity check failed".into());
                }
                init(&copy)?;
                copy.close().map_err(|(_, e)| error(e))?;
                std::fs::rename(&pending, destination).map_err(error)
            })();
            if migration.is_err() {
                let _ = std::fs::remove_file(&pending);
            }
            migration?;
        }
    }
    let conn = Connection::open(destination).map_err(error)?;
    init(&conn)?;
    Ok(conn)
}

fn console_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Console> {
    Ok(Console {
        id: row.get(0)?,
        name: row.get(1)?,
        hourly_price: row.get(2)?,
        controller_price: row.get(3)?,
    })
}
fn session_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        console_id: row.get(1)?,
        start_time: row.get(2)?,
        pause_time: row.get(3)?,
        end_time: row.get(4)?,
        controllers: row.get(5)?,
        status: row.get(6)?,
        shop_items: row
            .get::<_, Option<String>>(7)?
            .unwrap_or_else(|| "[]".into()),
        total_price: row.get(8)?,
    })
}
const SESSION_COLUMNS: &str =
    "id, consoleId, startTime, pauseTime, endTime, controllers, status, shopItems, totalPrice";

pub fn get_console(conn: &Connection, id: i64) -> AppResult<Console> {
    conn.query_row(
        "SELECT id, name, hourlyPrice, controllerPrice FROM consoles WHERE id = ?",
        [id],
        console_row,
    )
    .map_err(error)
}
pub fn get_consoles(conn: &Connection) -> AppResult<Vec<Console>> {
    let mut stmt = conn
        .prepare("SELECT id, name, hourlyPrice, controllerPrice FROM consoles ORDER BY id")
        .map_err(error)?;
    let rows = stmt.query_map([], console_row).map_err(error)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(error)
}
fn validate_price(price: f64) -> AppResult<()> {
    if price.is_finite() && price >= 0.0 {
        Ok(())
    } else {
        Err("قیمت باید عددی نامنفی باشد".into())
    }
}
fn validate_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        Err("نام الزامی است".into())
    } else {
        Ok(())
    }
}
fn validate_controllers(controllers: i64) -> AppResult<()> {
    if (1..=4).contains(&controllers) {
        Ok(())
    } else {
        Err("تعداد دسته باید بین ۱ و ۴ باشد".into())
    }
}
fn changed(rows: usize) -> AppResult<()> {
    if rows == 1 {
        Ok(())
    } else {
        Err("رکورد پیدا نشد یا وضعیت آن تغییر کرده است".into())
    }
}
pub fn add_console(conn: &Connection, data: ConsoleInput) -> AppResult<i64> {
    validate_name(&data.name)?;
    validate_price(data.hourly_price)?;
    validate_price(data.controller_price)?;
    conn.execute(
        "INSERT INTO consoles (name, hourlyPrice, controllerPrice) VALUES (?, ?, ?)",
        params![data.name.trim(), data.hourly_price, data.controller_price],
    )
    .map_err(error)?;
    Ok(conn.last_insert_rowid())
}
pub fn update_console(conn: &Connection, data: ConsoleInput) -> AppResult<()> {
    validate_name(&data.name)?;
    validate_price(data.hourly_price)?;
    validate_price(data.controller_price)?;
    changed(
        conn.execute(
            "UPDATE consoles SET name = ?, hourlyPrice = ?, controllerPrice = ? WHERE id = ?",
            params![
                data.name.trim(),
                data.hourly_price,
                data.controller_price,
                data.id.ok_or("Missing console id")?
            ],
        )
        .map_err(error)?,
    )
}
pub fn delete_console(conn: &Connection, id: i64) -> AppResult<()> {
    let active: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE consoleId = ? AND endTime IS NULL)",
            [id],
            |r| r.get(0),
        )
        .map_err(error)?;
    if active {
        return Err("ابتدا جلسه‌های باز این کنسول را تسویه کنید".into());
    }
    changed(
        conn.execute("DELETE FROM consoles WHERE id = ?", [id])
            .map_err(error)?,
    )
}
pub fn get_session(conn: &Connection, id: i64) -> AppResult<Session> {
    conn.query_row(
        &format!("SELECT {SESSION_COLUMNS} FROM sessions WHERE id = ?"),
        [id],
        session_row,
    )
    .map_err(error)
}
fn open_session(conn: &Connection, id: i64) -> AppResult<Session> {
    let session = get_session(conn, id)?;
    if session.end_time.is_some() || !matches!(session.status.as_str(), "active" | "paused") {
        return Err("جلسه بسته شده است".into());
    }
    Ok(session)
}
pub fn get_sessions_for_console(conn: &Connection, console_id: i64) -> AppResult<Vec<Session>> {
    let mut stmt = conn.prepare(&format!("SELECT {SESSION_COLUMNS} FROM sessions WHERE consoleId = ? AND endTime IS NULL ORDER BY id")).map_err(error)?;
    let rows = stmt.query_map([console_id], session_row).map_err(error)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(error)
}
fn iso(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Millis, true)
}
fn parse_time(time: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(time)
        .map(|t| t.with_timezone(&Utc))
        .map_err(error)
}
pub fn start_session(conn: &Connection, data: StartInput) -> AppResult<i64> {
    validate_controllers(data.controllers)?;
    get_console(conn, data.console_id)?;
    conn.execute("INSERT INTO sessions (consoleId, startTime, status, controllers, shopItems) VALUES (?, ?, 'active', ?, '[]')",
        params![data.console_id, iso(Utc::now()), data.controllers]).map_err(error)?;
    Ok(conn.last_insert_rowid())
}
pub fn pause_session(conn: &Connection, id: i64) -> AppResult<()> {
    changed(conn.execute("UPDATE sessions SET pauseTime = ?, status = 'paused' WHERE id = ? AND status = 'active' AND endTime IS NULL", params![iso(Utc::now()), id]).map_err(error)?)
}
pub fn resume_session_at(conn: &Connection, id: i64, now: DateTime<Utc>) -> AppResult<()> {
    let session = open_session(conn, id)?;
    if session.status != "paused" {
        return Err("جلسه متوقف نیست".into());
    }
    let paused = parse_time(session.pause_time.as_deref().ok_or("Missing pause time")?)?;
    let start = parse_time(&session.start_time)?;
    let new_start = start + (now - paused).max(chrono::Duration::zero());
    changed(
        conn.execute(
            "UPDATE sessions SET startTime = ?, pauseTime = NULL, status = 'active' WHERE id = ?",
            params![iso(new_start), id],
        )
        .map_err(error)?,
    )
}
pub fn calculate_invoice(
    session: &Session,
    console: &Console,
    now: DateTime<Utc>,
) -> AppResult<Invoice> {
    let end = if session.status == "paused" {
        parse_time(session.pause_time.as_deref().ok_or("Missing pause time")?)?
    } else {
        now
    };
    let hours = (end - parse_time(&session.start_time)?)
        .num_milliseconds()
        .max(0) as f64
        / 3_600_000.0;
    let game_cost = hours
        * (console.hourly_price
            + (session.controllers - 2).max(0) as f64 * console.controller_price);
    let items: Vec<ShopItem> = serde_json::from_str(&session.shop_items).map_err(error)?;
    let shop_cost = items.iter().map(|item| item.price).sum::<f64>();
    Ok(Invoice {
        game_cost,
        shop_cost,
        total: game_cost + shop_cost,
    })
}
pub fn end_session(conn: &Connection, id: i64) -> AppResult<Invoice> {
    let session = open_session(conn, id)?;
    let console = get_console(conn, session.console_id)?;
    let now = Utc::now();
    let invoice = calculate_invoice(&session, &console, now)?;
    changed(conn.execute("UPDATE sessions SET endTime = ?, totalPrice = ?, status = 'ended' WHERE id = ? AND endTime IS NULL",
        params![iso(now), invoice.total, id]).map_err(error)?)?;
    Ok(invoice)
}
pub fn update_session_controllers(conn: &Connection, data: ControllersInput) -> AppResult<()> {
    validate_controllers(data.controllers)?;
    open_session(conn, data.session_id)?;
    changed(
        conn.execute(
            "UPDATE sessions SET controllers = ? WHERE id = ?",
            params![data.controllers, data.session_id],
        )
        .map_err(error)?,
    )
}
pub fn save_shop_item(conn: &Connection, data: ShopInput, edit: bool) -> AppResult<()> {
    validate_name(&data.name)?;
    validate_price(data.price)?;
    let session = open_session(conn, data.session_id)?;
    let mut items: Vec<ShopItem> = serde_json::from_str(&session.shop_items).map_err(error)?;
    let item = ShopItem {
        name: data.name.trim().into(),
        price: data.price,
    };
    if edit {
        let slot = items
            .get_mut(data.item_index.ok_or("Missing item index")?)
            .ok_or("Item not found")?;
        *slot = item;
    } else {
        items.push(item);
    }
    write_shop_items(conn, data.session_id, &items)
}
fn write_shop_items(conn: &Connection, id: i64, items: &[ShopItem]) -> AppResult<()> {
    let json = serde_json::to_string(items).map_err(error)?;
    changed(
        conn.execute(
            "UPDATE sessions SET shopItems = ? WHERE id = ?",
            params![json, id],
        )
        .map_err(error)?,
    )
}
pub fn delete_shop_item(conn: &Connection, data: DeleteItemInput) -> AppResult<()> {
    let session = open_session(conn, data.session_id)?;
    let mut items: Vec<ShopItem> = serde_json::from_str(&session.shop_items).map_err(error)?;
    if data.item_index >= items.len() {
        return Err("Item not found".into());
    }
    items.remove(data.item_index);
    write_shop_items(conn, data.session_id, &items)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        add_console(
            &conn,
            ConsoleInput {
                id: None,
                name: "PS5".into(),
                hourly_price: 100.0,
                controller_price: 20.0,
            },
        )
        .unwrap();
        conn
    }
    fn session(conn: &Connection, controllers: i64) -> i64 {
        let id = start_session(
            conn,
            StartInput {
                console_id: 1,
                controllers,
            },
        )
        .unwrap();
        conn.execute(
            "UPDATE sessions SET startTime = '2026-10-08T10:00:00.000Z' WHERE id = ?",
            [id],
        )
        .unwrap();
        id
    }
    #[test]
    fn billing_includes_extra_controllers_and_shop_but_excludes_pause() {
        let conn = fixture();
        let id = session(&conn, 4);
        save_shop_item(
            &conn,
            ShopInput {
                session_id: id,
                name: "Snack".into(),
                price: 50.0,
                item_index: None,
            },
            false,
        )
        .unwrap();
        conn.execute("UPDATE sessions SET status = 'paused', pauseTime = '2026-10-08T11:30:00.000Z' WHERE id = ?", [id]).unwrap();
        let invoice = calculate_invoice(
            &get_session(&conn, id).unwrap(),
            &get_console(&conn, 1).unwrap(),
            parse_time("2026-10-08T13:00:00Z").unwrap(),
        )
        .unwrap();
        assert_eq!(invoice.game_cost, 210.0);
        assert_eq!(invoice.shop_cost, 50.0);
        assert_eq!(invoice.total, 260.0);
    }
    #[test]
    fn resume_preserves_played_duration_and_repeat_resume_fails() {
        let conn = fixture();
        let id = session(&conn, 2);
        conn.execute("UPDATE sessions SET status = 'paused', pauseTime = '2026-10-08T11:00:00.000Z' WHERE id = ?", [id]).unwrap();
        let now = parse_time("2026-10-08T12:00:00Z").unwrap();
        resume_session_at(&conn, id, now).unwrap();
        let s = get_session(&conn, id).unwrap();
        assert_eq!(s.start_time, "2026-10-08T11:00:00.000Z");
        assert!(s.pause_time.is_none());
        assert_eq!(
            calculate_invoice(&s, &get_console(&conn, 1).unwrap(), now)
                .unwrap()
                .total,
            100.0
        );
        assert!(resume_session_at(&conn, id, now).is_err());
    }
    #[test]
    fn checkout_is_persistent_and_cannot_be_repeated() {
        let conn = fixture();
        let id = session(&conn, 1);
        assert!(delete_console(&conn, 1).is_err());
        pause_session(&conn, id).unwrap();
        assert!(pause_session(&conn, id).is_err());
        let invoice = end_session(&conn, id).unwrap();
        let saved = get_session(&conn, id).unwrap();
        assert_eq!(saved.total_price, Some(invoice.total));
        assert_eq!(saved.status, "ended");
        assert!(get_sessions_for_console(&conn, 1).unwrap().is_empty());
        assert!(end_session(&conn, id).is_err());
        assert!(update_session_controllers(
            &conn,
            ControllersInput {
                session_id: id,
                controllers: 2
            }
        )
        .is_err());
        delete_console(&conn, 1).unwrap();
    }
    #[test]
    fn shop_edits_and_validation() {
        let conn = fixture();
        assert!(start_session(
            &conn,
            StartInput {
                console_id: 1,
                controllers: 5
            }
        )
        .is_err());
        assert!(start_session(
            &conn,
            StartInput {
                console_id: 99,
                controllers: 1
            }
        )
        .is_err());
        let id = session(&conn, 1);
        assert!(save_shop_item(
            &conn,
            ShopInput {
                session_id: id,
                name: "Snack".into(),
                price: -1.0,
                item_index: None
            },
            false
        )
        .is_err());
        save_shop_item(
            &conn,
            ShopInput {
                session_id: id,
                name: "Snack".into(),
                price: 50.0,
                item_index: None,
            },
            false,
        )
        .unwrap();
        save_shop_item(
            &conn,
            ShopInput {
                session_id: id,
                name: "Drink".into(),
                price: 30.0,
                item_index: Some(0),
            },
            true,
        )
        .unwrap();
        let items: Vec<ShopItem> =
            serde_json::from_str(&get_session(&conn, id).unwrap().shop_items).unwrap();
        assert_eq!(items[0].name, "Drink");
        assert_eq!(items[0].price, 30.0);
        assert!(delete_shop_item(
            &conn,
            DeleteItemInput {
                session_id: id,
                item_index: 4
            }
        )
        .is_err());
        delete_shop_item(
            &conn,
            DeleteItemInput {
                session_id: id,
                item_index: 0,
            },
        )
        .unwrap();
        assert_eq!(get_session(&conn, id).unwrap().shop_items, "[]");
    }
    #[test]
    fn legacy_migration_copies_data_and_never_overwrites_existing_database() {
        let dir = std::env::temp_dir().join(format!(
            "gamagement-test-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let old = dir.join("electron.db");
        let new = dir.join("tauri.db");
        let original = Connection::open(&old).unwrap();
        original.pragma_update(None, "journal_mode", "WAL").unwrap();
        init(&original).unwrap();
        add_console(
            &original,
            ConsoleInput {
                id: None,
                name: "Legacy".into(),
                hourly_price: 100.0,
                controller_price: 20.0,
            },
        )
        .unwrap();
        let migrated = open_database(&new, Some(&old)).unwrap();
        assert_eq!(get_consoles(&migrated).unwrap()[0].name, "Legacy");
        migrated
            .execute("UPDATE consoles SET name = 'Tauri'", [])
            .unwrap();
        drop(migrated);
        let reopened = open_database(&new, Some(&old)).unwrap();
        assert_eq!(get_consoles(&reopened).unwrap()[0].name, "Tauri");
        assert_eq!(get_consoles(&original).unwrap()[0].name, "Legacy");
        drop(reopened);
        drop(original);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
