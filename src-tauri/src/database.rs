use crate::models::*;
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection};
use std::{path::Path, sync::Mutex};

pub struct Database(pub Mutex<Connection>);
pub fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn iso(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Millis, true)
}
pub fn parse_time(time: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(time)
        .map(|t| t.with_timezone(&Utc))
        .map_err(err)
}

pub fn open_database(destination: &Path, legacy: Option<&Path>) -> AppResult<Connection> {
    if !destination.exists() {
        if let Some(source) = legacy.filter(|p| p.is_file() && *p != destination) {
            let pending = destination.with_extension("migration");
            let migration = (|| {
                let source =
                    Connection::open_with_flags(source, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                        .map_err(err)?;
                let mut copy = Connection::open(&pending).map_err(err)?;
                backup(&source, &mut copy)?;
                crate::schema::init(&copy)?;
                copy.close().map_err(|(_, e)| err(e))?;
                std::fs::rename(&pending, destination).map_err(err)
            })();
            if migration.is_err() {
                let _ = std::fs::remove_file(&pending);
            }
            migration?;
        }
    }
    let conn = Connection::open(destination).map_err(err)?;
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(err)?;
    let has_sessions: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='sessions')",
            [],
            |r| r.get(0),
        )
        .map_err(err)?;
    let safety_copy = destination.with_extension("pre-v2.db");
    if version < 2 && has_sessions && !safety_copy.exists() {
        let mut copy = Connection::open(&safety_copy).map_err(err)?;
        backup(&conn, &mut copy)?;
    }
    crate::schema::init(&conn)?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(err)?;
    Ok(conn)
}

fn backup(source: &Connection, destination: &mut Connection) -> AppResult<()> {
    {
        let backup = rusqlite::backup::Backup::new(source, destination).map_err(err)?;
        backup
            .run_to_completion(100, std::time::Duration::from_millis(10), None)
            .map_err(err)?;
    }
    let check: String = destination
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(err)?;
    if check == "ok" {
        Ok(())
    } else {
        Err("Database integrity check failed".into())
    }
}

fn name(value: &str) -> AppResult<()> {
    if value.trim().is_empty() || value.chars().count() > 100 {
        Err("نام باید بین ۱ و ۱۰۰ نویسه باشد".into())
    } else {
        Ok(())
    }
}
fn price(value: i64) -> AppResult<()> {
    if (0..=1_000_000_000).contains(&value) {
        Ok(())
    } else {
        Err("قیمت باید عدد صحیح بین صفر و یک میلیارد تومان باشد".into())
    }
}
fn quantity(value: i64) -> AppResult<()> {
    if (1..=100_000).contains(&value) {
        Ok(())
    } else {
        Err("تعداد نامعتبر است".into())
    }
}
fn controllers(value: i64) -> AppResult<()> {
    if (1..=4).contains(&value) {
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
const CONSOLE_COLUMNS: &str = "id,name,hourlyPrice,controllerPrice,archived";
fn console_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Console> {
    Ok(Console {
        id: r.get(0)?,
        name: r.get(1)?,
        hourly_price: r.get(2)?,
        controller_price: r.get(3)?,
        archived: r.get(4)?,
    })
}
pub fn get_console(conn: &Connection, id: i64) -> AppResult<Console> {
    conn.query_row(
        &format!("SELECT {CONSOLE_COLUMNS} FROM consoles WHERE id=?"),
        [id],
        console_row,
    )
    .map_err(err)
}
pub fn get_consoles(conn: &Connection, archived: bool) -> AppResult<Vec<Console>> {
    let mut q = conn
        .prepare(&format!(
            "SELECT {CONSOLE_COLUMNS} FROM consoles WHERE (? OR archived=0) ORDER BY id"
        ))
        .map_err(err)?;
    let rows = q.query_map([archived], console_row).map_err(err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(err)
}
pub fn add_console(conn: &Connection, data: ConsoleInput) -> AppResult<i64> {
    name(&data.name)?;
    price(data.hourly_price)?;
    price(data.controller_price)?;
    conn.execute(
        "INSERT INTO consoles (name,hourlyPrice,controllerPrice) VALUES (?,?,?)",
        params![data.name.trim(), data.hourly_price, data.controller_price],
    )
    .map_err(err)?;
    Ok(conn.last_insert_rowid())
}
pub fn update_console(conn: &Connection, data: ConsoleInput) -> AppResult<()> {
    name(&data.name)?;
    price(data.hourly_price)?;
    price(data.controller_price)?;
    changed(
        conn.execute(
            "UPDATE consoles SET name=?,hourlyPrice=?,controllerPrice=? WHERE id=? AND archived=0",
            params![
                data.name.trim(),
                data.hourly_price,
                data.controller_price,
                data.id.ok_or("شناسه کنسول الزامی است")?
            ],
        )
        .map_err(err)?,
    )
}
pub fn archive_console(conn: &Connection, id: i64) -> AppResult<()> {
    let tx = conn.unchecked_transaction().map_err(err)?;
    let occupied: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE consoleId=? AND endTime IS NULL)",
            [id],
            |r| r.get(0),
        )
        .map_err(err)?;
    if occupied {
        return Err("ابتدا جلسه‌های باز این کنسول را تسویه کنید".into());
    }
    changed(
        tx.execute(
            "UPDATE consoles SET archived=1 WHERE id=? AND archived=0",
            [id],
        )
        .map_err(err)?,
    )?;
    tx.commit().map_err(err)
}
const SESSION_COLUMNS: &str = "id,consoleId,consoleName,startTime,pauseTime,endTime,paidAt,controllers,status,hourlyPrice,controllerPrice,pausedMs,checkoutAt,checkoutPreviousStatus,gameCost,shopCost,totalPrice,legacy";
fn session_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: r.get(0)?,
        console_id: r.get(1)?,
        console_name: r.get(2)?,
        start_time: r.get(3)?,
        pause_time: r.get(4)?,
        end_time: r.get(5)?,
        paid_at: r.get(6)?,
        controllers: r.get(7)?,
        status: r.get(8)?,
        hourly_price: r.get(9)?,
        controller_price: r.get(10)?,
        paused_ms: r.get(11)?,
        checkout_at: r.get(12)?,
        checkout_previous_status: r.get(13)?,
        game_cost: r.get(14)?,
        shop_cost: r.get(15)?,
        total_price: r.get(16)?,
        legacy: r.get(17)?,
    })
}
pub fn get_session(conn: &Connection, id: i64) -> AppResult<Session> {
    conn.query_row(
        &format!("SELECT {SESSION_COLUMNS} FROM sessions WHERE id=?"),
        [id],
        session_row,
    )
    .map_err(err)
}
fn editable_session(conn: &Connection, id: i64) -> AppResult<Session> {
    let s = get_session(conn, id)?;
    if matches!(s.status.as_str(), "active" | "paused") {
        Ok(s)
    } else {
        Err("جلسه در حال تسویه یا بسته شده است".into())
    }
}
pub fn start_session_at(conn: &Connection, data: StartInput, now: DateTime<Utc>) -> AppResult<i64> {
    controllers(data.controllers)?;
    let tx = conn.unchecked_transaction().map_err(err)?;
    let c = get_console(&tx, data.console_id)?;
    if c.archived {
        return Err("کنسول بایگانی شده است".into());
    }
    tx.execute("INSERT INTO sessions (consoleId,consoleName,startTime,controllers,status,hourlyPrice,controllerPrice) VALUES (?,?,?,?,'active',?,?)",params![c.id,c.name,iso(now),data.controllers,c.hourly_price,c.controller_price]).map_err(err)?;
    let id = tx.last_insert_rowid();
    open_segment(&tx, id, data.controllers, now)?;
    tx.commit().map_err(err)?;
    Ok(id)
}
fn close_segment(conn: &Connection, id: i64, now: DateTime<Utc>) -> AppResult<()> {
    conn.execute(
        "UPDATE session_segments SET endTime=? WHERE sessionId=? AND endTime IS NULL",
        params![iso(now), id],
    )
    .map_err(err)?;
    Ok(())
}
fn open_segment(conn: &Connection, id: i64, count: i64, now: DateTime<Utc>) -> AppResult<()> {
    conn.execute(
        "INSERT INTO session_segments (sessionId,startTime,controllers) VALUES (?,?,?)",
        params![id, iso(now), count],
    )
    .map_err(err)?;
    Ok(())
}
pub fn pause_session_at(conn: &Connection, id: i64, now: DateTime<Utc>) -> AppResult<()> {
    let tx = conn.unchecked_transaction().map_err(err)?;
    let s = editable_session(&tx, id)?;
    if s.status != "active" {
        return Err("جلسه متوقف است".into());
    }
    close_segment(&tx, id, now)?;
    changed(
        tx.execute(
            "UPDATE sessions SET status='paused',pauseTime=? WHERE id=?",
            params![iso(now), id],
        )
        .map_err(err)?,
    )?;
    tx.commit().map_err(err)
}
pub fn resume_session_at(conn: &Connection, id: i64, now: DateTime<Utc>) -> AppResult<()> {
    let tx = conn.unchecked_transaction().map_err(err)?;
    let s = editable_session(&tx, id)?;
    if s.status != "paused" {
        return Err("جلسه متوقف نیست".into());
    }
    let pause = parse_time(s.pause_time.as_deref().ok_or("زمان توقف ثبت نشده است")?)?;
    let duration = (now - pause).num_milliseconds().max(0);
    changed(
        tx.execute(
            "UPDATE sessions SET status='active',pauseTime=NULL,pausedMs=pausedMs+? WHERE id=?",
            params![duration, id],
        )
        .map_err(err)?,
    )?;
    open_segment(&tx, id, s.controllers, now)?;
    tx.commit().map_err(err)
}
pub fn update_session_controllers_at(
    conn: &Connection,
    data: ControllersInput,
    now: DateTime<Utc>,
) -> AppResult<()> {
    controllers(data.controllers)?;
    let tx = conn.unchecked_transaction().map_err(err)?;
    let s = editable_session(&tx, data.session_id)?;
    if s.controllers != data.controllers {
        if s.status == "active" {
            close_segment(&tx, s.id, now)?;
            open_segment(&tx, s.id, data.controllers, now)?;
        }
        tx.execute(
            "UPDATE sessions SET controllers=? WHERE id=?",
            params![data.controllers, s.id],
        )
        .map_err(err)?;
    }
    tx.commit().map_err(err)
}
pub fn session_items(conn: &Connection, id: i64) -> AppResult<Vec<SessionItem>> {
    let mut q=conn.prepare("SELECT id,sessionId,productId,name,unitPrice,quantity FROM session_items WHERE sessionId=? ORDER BY id").map_err(err)?;
    let rows = q
        .query_map([id], |r| {
            let unit: i64 = r.get(4)?;
            let count: i64 = r.get(5)?;
            Ok(SessionItem {
                id: r.get(0)?,
                session_id: r.get(1)?,
                product_id: r.get(2)?,
                name: r.get(3)?,
                unit_price: unit,
                quantity: count,
                total: unit * count,
            })
        })
        .map_err(err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(err)
}
pub fn invoice_at(conn: &Connection, id: i64, now: DateTime<Utc>) -> AppResult<Invoice> {
    let s = get_session(conn, id)?;
    let mut q=conn.prepare("SELECT startTime,endTime,controllers FROM session_segments WHERE sessionId=? ORDER BY id").map_err(err)?;
    let segments = q
        .query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map_err(err)?;
    let mut played = 0_i64;
    let mut numerator = 0_i128;
    for seg in segments {
        let (start, end, count) = seg.map_err(err)?;
        let end = end.as_deref().map(parse_time).transpose()?.unwrap_or(now);
        let duration = (end - parse_time(&start)?).num_milliseconds().max(0);
        played = played
            .checked_add(duration)
            .ok_or("مدت جلسه بیش از حد مجاز است")?;
        numerator += duration as i128
            * (s.hourly_price as i128 + (count - 2).max(0) as i128 * s.controller_price as i128);
    }
    let items = session_items(conn, id)?;
    let game = i64::try_from((numerator + 1_800_000) / 3_600_000).map_err(err)?;
    let shop = items.iter().try_fold(0_i64, |sum, i| {
        sum.checked_add(i.total).ok_or("مبلغ بیش از حد مجاز است")
    })?;
    let frozen = matches!(s.status.as_str(), "checkout" | "ended");
    let game_cost = if frozen {
        s.game_cost.ok_or("فاکتور ثبت نشده است")?
    } else {
        game
    };
    let shop_cost = if frozen {
        s.shop_cost.ok_or("فاکتور ثبت نشده است")?
    } else {
        shop
    };
    let total = if frozen {
        s.total_price.ok_or("فاکتور ثبت نشده است")?
    } else {
        game_cost
            .checked_add(shop_cost)
            .ok_or("مبلغ بیش از حد مجاز است")?
    };
    if total > 9_000_000_000_000_000 {
        return Err("مبلغ بیش از حد مجاز است".into());
    }
    Ok(Invoice {
        session: s,
        items,
        played_ms: played,
        game_cost,
        shop_cost,
        total,
    })
}
pub fn prepare_checkout_at(conn: &Connection, id: i64, now: DateTime<Utc>) -> AppResult<Invoice> {
    let tx = conn.unchecked_transaction().map_err(err)?;
    let s = get_session(&tx, id)?;
    if s.status == "checkout" {
        return invoice_at(&tx, id, now);
    }
    editable_session(&tx, id)?;
    let invoice = invoice_at(&tx, id, now)?;
    if s.status == "active" {
        close_segment(&tx, id, now)?;
    }
    tx.execute("UPDATE sessions SET status='checkout',checkoutAt=?,checkoutPreviousStatus=?,gameCost=?,shopCost=?,totalPrice=? WHERE id=?",params![iso(now),s.status,invoice.game_cost,invoice.shop_cost,invoice.total,id]).map_err(err)?;
    let result = invoice_at(&tx, id, now)?;
    tx.commit().map_err(err)?;
    Ok(result)
}
pub fn cancel_checkout_at(conn: &Connection, id: i64, now: DateTime<Utc>) -> AppResult<()> {
    let tx = conn.unchecked_transaction().map_err(err)?;
    let s = get_session(&tx, id)?;
    if s.status != "checkout" {
        return Err("جلسه در حال تسویه نیست".into());
    }
    let previous = s
        .checkout_previous_status
        .as_deref()
        .ok_or("وضعیت قبلی ثبت نشده است")?;
    let excluded = if previous == "active" {
        open_segment(&tx, id, s.controllers, now)?;
        (now - parse_time(s.checkout_at.as_deref().ok_or("زمان تسویه ثبت نشده است")?)?)
            .num_milliseconds()
            .max(0)
    } else {
        0
    };
    tx.execute("UPDATE sessions SET status=?,checkoutAt=NULL,checkoutPreviousStatus=NULL,gameCost=NULL,shopCost=NULL,totalPrice=NULL,pausedMs=pausedMs+? WHERE id=?",params![previous,excluded,id]).map_err(err)?;
    tx.commit().map_err(err)
}
pub fn confirm_checkout_at(conn: &Connection, id: i64, now: DateTime<Utc>) -> AppResult<Invoice> {
    let tx = conn.unchecked_transaction().map_err(err)?;
    let s = get_session(&tx, id)?;
    if s.status != "checkout" {
        return Err("ابتدا فاکتور جلسه را باز کنید".into());
    }
    changed(tx.execute("UPDATE sessions SET status='ended',endTime=checkoutAt,paidAt=? WHERE id=? AND status='checkout'",params![iso(now),id]).map_err(err)?)?;
    let invoice = invoice_at(&tx, id, now)?;
    tx.commit().map_err(err)?;
    Ok(invoice)
}
pub fn dashboard(conn: &Connection) -> AppResult<Vec<ConsoleCard>> {
    let now = Utc::now();
    let mut cards = Vec::new();
    for console in get_consoles(conn, true)? {
        let mut q = conn
            .prepare("SELECT id FROM sessions WHERE consoleId=? AND endTime IS NULL ORDER BY id")
            .map_err(err)?;
        let ids = q
            .query_map([console.id], |r| r.get::<_, i64>(0))
            .map_err(err)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(err)?;
        if console.archived && ids.is_empty() {
            continue;
        }
        let sessions = ids
            .into_iter()
            .map(|id| invoice_at(conn, id, now))
            .collect::<AppResult<Vec<_>>>()?;
        cards.push(ConsoleCard { console, sessions });
    }
    Ok(cards)
}

fn product_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Product> {
    Ok(Product {
        id: r.get(0)?,
        name: r.get(1)?,
        price: r.get(2)?,
        stock: r.get(3)?,
        archived: r.get(4)?,
    })
}
pub fn get_products(conn: &Connection) -> AppResult<Vec<Product>> {
    let mut q = conn
        .prepare(
            "SELECT id,name,price,stock,archived FROM products WHERE archived=0 ORDER BY name,id",
        )
        .map_err(err)?;
    let rows = q.query_map([], product_row).map_err(err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(err)
}
pub fn save_product(conn: &Connection, data: ProductInput) -> AppResult<i64> {
    name(&data.name)?;
    price(data.price)?;
    if !(0..=1_000_000).contains(&data.stock) {
        return Err("موجودی نامعتبر است".into());
    }
    if let Some(id) = data.id {
        changed(
            conn.execute(
                "UPDATE products SET name=?,price=?,stock=? WHERE id=? AND archived=0",
                params![data.name.trim(), data.price, data.stock, id],
            )
            .map_err(err)?,
        )?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO products (name,price,stock) VALUES (?,?,?)",
            params![data.name.trim(), data.price, data.stock],
        )
        .map_err(err)?;
        Ok(conn.last_insert_rowid())
    }
}
pub fn archive_product(conn: &Connection, id: i64) -> AppResult<()> {
    changed(
        conn.execute(
            "UPDATE products SET archived=1 WHERE id=? AND archived=0",
            [id],
        )
        .map_err(err)?,
    )
}
pub fn add_product_to_session(conn: &Connection, data: AddProductInput) -> AppResult<()> {
    quantity(data.quantity)?;
    let tx = conn.unchecked_transaction().map_err(err)?;
    editable_session(&tx, data.session_id)?;
    let p = tx
        .query_row(
            "SELECT id,name,price,stock,archived FROM products WHERE id=? AND archived=0",
            [data.product_id],
            product_row,
        )
        .map_err(err)?;
    if p.stock < data.quantity {
        return Err("موجودی محصول کافی نیست".into());
    }
    tx.execute(
        "UPDATE products SET stock=stock-? WHERE id=?",
        params![data.quantity, p.id],
    )
    .map_err(err)?;
    tx.execute("INSERT INTO session_items (sessionId,productId,name,unitPrice,quantity) VALUES (?,?,?,?,?)",params![data.session_id,p.id,p.name,p.price,data.quantity]).map_err(err)?;
    tx.commit().map_err(err)
}
fn item(conn: &Connection, id: i64) -> AppResult<(i64, Option<i64>, i64)> {
    conn.query_row(
        "SELECT sessionId,productId,quantity FROM session_items WHERE id=?",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .map_err(err)
}
pub fn update_item_quantity(conn: &Connection, data: ItemQuantityInput) -> AppResult<()> {
    quantity(data.quantity)?;
    let tx = conn.unchecked_transaction().map_err(err)?;
    let (session, product, old) = item(&tx, data.item_id)?;
    editable_session(&tx, session)?;
    if let Some(product) = product {
        let delta = data.quantity - old;
        let stock: i64 = tx
            .query_row("SELECT stock FROM products WHERE id=?", [product], |r| {
                r.get(0)
            })
            .map_err(err)?;
        if delta > stock {
            return Err("موجودی محصول کافی نیست".into());
        }
        tx.execute(
            "UPDATE products SET stock=stock-? WHERE id=?",
            params![delta, product],
        )
        .map_err(err)?;
    }
    tx.execute(
        "UPDATE session_items SET quantity=? WHERE id=?",
        params![data.quantity, data.item_id],
    )
    .map_err(err)?;
    tx.commit().map_err(err)
}
pub fn delete_session_item(conn: &Connection, id: i64) -> AppResult<()> {
    let tx = conn.unchecked_transaction().map_err(err)?;
    let (session, product, count) = item(&tx, id)?;
    editable_session(&tx, session)?;
    if let Some(product) = product {
        tx.execute(
            "UPDATE products SET stock=stock+? WHERE id=?",
            params![count, product],
        )
        .map_err(err)?;
    }
    tx.execute("DELETE FROM session_items WHERE id=?", [id])
        .map_err(err)?;
    tx.commit().map_err(err)
}
