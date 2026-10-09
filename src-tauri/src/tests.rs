use crate::{database::*, models::*, reports, schema};
use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;

fn at(seconds: i64) -> DateTime<Utc> {
    parse_time("2026-10-09T00:00:00Z").unwrap() + Duration::seconds(seconds)
}
fn console(conn: &Connection) -> i64 {
    add_console(
        conn,
        ConsoleInput {
            id: None,
            name: "PS5".into(),
            hourly_price: 3600,
            controller_price: 1800,
        },
    )
    .unwrap()
}
fn fixture() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    schema::init(&conn).unwrap();
    console(&conn);
    conn
}
fn start(conn: &Connection, count: i64) -> i64 {
    start_session_at(
        conn,
        StartInput {
            console_id: 1,
            controllers: count,
        },
        at(0),
    )
    .unwrap()
}
fn product(conn: &Connection, price: i64, stock: i64) -> i64 {
    save_product(
        conn,
        ProductInput {
            id: None,
            name: "نوشابه".into(),
            price,
            stock,
        },
    )
    .unwrap()
}
fn settle(conn: &Connection, id: i64, seconds: i64) -> Invoice {
    prepare_checkout_at(conn, id, at(seconds)).unwrap();
    confirm_checkout_at(conn, id, at(seconds + 30)).unwrap()
}
#[test]
fn controller_changes_only_charge_future_play() {
    let c = fixture();
    let id = start(&c, 2);
    update_session_controllers_at(
        &c,
        ControllersInput {
            session_id: id,
            controllers: 4,
        },
        at(600),
    )
    .unwrap();
    let invoice = invoice_at(&c, id, at(1200)).unwrap();
    assert_eq!(invoice.game_cost, 1800);
    assert_eq!(invoice.played_ms, 1_200_000);
    assert_eq!(invoice.session.start_time, iso(at(0)));
}
#[test]
fn console_edits_do_not_change_active_rates_or_identity() {
    let c = fixture();
    let id = start(&c, 3);
    update_console(
        &c,
        ConsoleInput {
            id: Some(1),
            name: "Renamed".into(),
            hourly_price: 7200,
            controller_price: 3600,
        },
    )
    .unwrap();
    let old = settle(&c, id, 600);
    assert_eq!(old.game_cost, 900);
    assert_eq!(old.session.console_name, "PS5");
    let next = start_session_at(
        &c,
        StartInput {
            console_id: 1,
            controllers: 3,
        },
        at(700),
    )
    .unwrap();
    assert_eq!(invoice_at(&c, next, at(1300)).unwrap().game_cost, 1800);
}
#[test]
fn pause_resume_keeps_original_start_and_excludes_pause() {
    let c = fixture();
    let id = start(&c, 2);
    pause_session_at(&c, id, at(300)).unwrap();
    assert!(pause_session_at(&c, id, at(400)).is_err());
    update_session_controllers_at(
        &c,
        ControllersInput {
            session_id: id,
            controllers: 4,
        },
        at(450),
    )
    .unwrap();
    resume_session_at(&c, id, at(600)).unwrap();
    let i = invoice_at(&c, id, at(900)).unwrap();
    assert_eq!(i.game_cost, 900);
    assert_eq!(i.played_ms, 600_000);
    assert_eq!(i.session.paused_ms, 300_000);
    assert_eq!(i.session.start_time, iso(at(0)));
    assert!(resume_session_at(&c, id, at(1000)).is_err());
}
#[test]
fn checkout_freezes_exact_total_and_rejects_mutations() {
    let c = fixture();
    let id = start(&c, 4);
    let p = product(&c, 75, 10);
    add_product_to_session(
        &c,
        AddProductInput {
            session_id: id,
            product_id: p,
            quantity: 2,
        },
    )
    .unwrap();
    let shown = prepare_checkout_at(&c, id, at(600)).unwrap();
    assert_eq!(shown.total, 1350);
    assert_eq!(
        prepare_checkout_at(&c, id, at(1200)).unwrap().total,
        shown.total
    );
    assert!(update_session_controllers_at(
        &c,
        ControllersInput {
            session_id: id,
            controllers: 1
        },
        at(1200)
    )
    .is_err());
    assert!(add_product_to_session(
        &c,
        AddProductInput {
            session_id: id,
            product_id: p,
            quantity: 1
        }
    )
    .is_err());
    assert!(pause_session_at(&c, id, at(1200)).is_err());
    let paid = confirm_checkout_at(&c, id, at(1800)).unwrap();
    assert_eq!(paid.total, shown.total);
    assert_eq!(paid.played_ms, shown.played_ms);
    assert_eq!(paid.session.end_time, Some(iso(at(600))));
    assert_eq!(paid.session.paid_at, Some(iso(at(1800))));
    assert!(confirm_checkout_at(&c, id, at(1900)).is_err());
}
#[test]
fn cancelled_checkout_resumes_active_play_and_excludes_preview_time() {
    let c = fixture();
    let id = start(&c, 2);
    prepare_checkout_at(&c, id, at(300)).unwrap();
    cancel_checkout_at(&c, id, at(600)).unwrap();
    let i = invoice_at(&c, id, at(900)).unwrap();
    assert_eq!(i.game_cost, 600);
    assert_eq!(i.played_ms, 600_000);
    assert_eq!(i.session.paused_ms, 300_000);
    assert_eq!(i.session.status, "active");
}
#[test]
fn cancelled_checkout_restores_paused_state() {
    let c = fixture();
    let id = start(&c, 2);
    pause_session_at(&c, id, at(300)).unwrap();
    prepare_checkout_at(&c, id, at(400)).unwrap();
    cancel_checkout_at(&c, id, at(600)).unwrap();
    assert_eq!(get_session(&c, id).unwrap().status, "paused");
    resume_session_at(&c, id, at(900)).unwrap();
    let i = invoice_at(&c, id, at(1200)).unwrap();
    assert_eq!(i.game_cost, 600);
    assert_eq!(i.session.paused_ms, 600_000);
}
#[test]
fn rounding_happens_once_for_all_segments() {
    let c = fixture();
    update_console(
        &c,
        ConsoleInput {
            id: Some(1),
            name: "PS5".into(),
            hourly_price: 1800,
            controller_price: 0,
        },
    )
    .unwrap();
    let id = start(&c, 1);
    update_session_controllers_at(
        &c,
        ControllersInput {
            session_id: id,
            controllers: 2,
        },
        at(1),
    )
    .unwrap();
    update_session_controllers_at(
        &c,
        ControllersInput {
            session_id: id,
            controllers: 1,
        },
        at(2),
    )
    .unwrap();
    assert_eq!(invoice_at(&c, id, at(3)).unwrap().game_cost, 2);
}
#[test]
fn inventory_quantity_edits_are_atomic_and_keep_sale_price() {
    let c = fixture();
    let id = start(&c, 2);
    let p = product(&c, 100, 4);
    add_product_to_session(
        &c,
        AddProductInput {
            session_id: id,
            product_id: p,
            quantity: 2,
        },
    )
    .unwrap();
    let item = session_items(&c, id).unwrap()[0].id;
    assert_eq!(get_products(&c).unwrap()[0].stock, 2);
    assert!(update_item_quantity(
        &c,
        ItemQuantityInput {
            item_id: item,
            quantity: 5
        }
    )
    .is_err());
    assert_eq!(session_items(&c, id).unwrap()[0].quantity, 2);
    assert_eq!(get_products(&c).unwrap()[0].stock, 2);
    save_product(
        &c,
        ProductInput {
            id: Some(p),
            name: "New name".into(),
            price: 200,
            stock: 2,
        },
    )
    .unwrap();
    update_item_quantity(
        &c,
        ItemQuantityInput {
            item_id: item,
            quantity: 3,
        },
    )
    .unwrap();
    let item_data = session_items(&c, id).unwrap().remove(0);
    assert_eq!(item_data.total, 300);
    assert_eq!(item_data.name, "نوشابه");
    assert_eq!(get_products(&c).unwrap()[0].stock, 1);
    delete_session_item(&c, item).unwrap();
    assert_eq!(get_products(&c).unwrap()[0].stock, 4);
    assert!(session_items(&c, id).unwrap().is_empty());
}
#[test]
fn insufficient_stock_and_archived_products_cannot_be_sold() {
    let c = fixture();
    let id = start(&c, 2);
    let p = product(&c, 100, 1);
    assert!(add_product_to_session(
        &c,
        AddProductInput {
            session_id: id,
            product_id: p,
            quantity: 2
        }
    )
    .is_err());
    assert!(session_items(&c, id).unwrap().is_empty());
    assert_eq!(get_products(&c).unwrap()[0].stock, 1);
    archive_product(&c, p).unwrap();
    assert!(add_product_to_session(
        &c,
        AddProductInput {
            session_id: id,
            product_id: p,
            quantity: 1
        }
    )
    .is_err());
}
#[test]
fn console_archival_preserves_receipts_and_rejects_occupied_console() {
    let c = fixture();
    let id = start(&c, 2);
    assert!(archive_console(&c, 1).is_err());
    settle(&c, id, 60);
    archive_console(&c, 1).unwrap();
    assert!(get_consoles(&c, false).unwrap().is_empty());
    assert_eq!(
        invoice_at(&c, id, at(1000)).unwrap().session.console_name,
        "PS5"
    );
    assert!(start_session_at(
        &c,
        StartInput {
            console_id: 1,
            controllers: 2
        },
        at(1000)
    )
    .is_err());
}
#[test]
fn occupied_console_cannot_start_another_session() {
    let c = fixture();
    let id = start(&c, 2);
    assert!(start_session_at(
        &c,
        StartInput {
            console_id: 1,
            controllers: 2
        },
        at(1)
    )
    .is_err());
    prepare_checkout_at(&c, id, at(60)).unwrap();
    assert!(start_session_at(
        &c,
        StartInput {
            console_id: 1,
            controllers: 2
        },
        at(61)
    )
    .is_err());
}
#[test]
fn history_filters_totals_pagination_and_tehran_days() {
    let c = fixture();
    for n in 0..27 {
        let start = at(n * 120);
        let id = start_session_at(
            &c,
            StartInput {
                console_id: 1,
                controllers: 2,
            },
            start,
        )
        .unwrap();
        prepare_checkout_at(&c, id, start + Duration::seconds(60)).unwrap();
        confirm_checkout_at(&c, id, start + Duration::seconds(65)).unwrap();
    }
    let first = reports::get_history(&c, HistoryFilter::default()).unwrap();
    assert_eq!(first.entries.len(), 25);
    assert_eq!(first.pages, 2);
    assert_eq!(first.summary.total_revenue, 27 * 60);
    assert_eq!(first.consoles[0].sessions, 27);
    assert_eq!(first.days[0].day, "2026-10-09");
    let second = reports::get_history(
        &c,
        HistoryFilter {
            page: Some(2),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(second.entries.len(), 2);
    let filtered = reports::get_history(
        &c,
        HistoryFilter {
            from: Some(iso(at(0))),
            to: Some(iso(at(120))),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(filtered.summary.sessions, 1);
    let none = reports::get_history(
        &c,
        HistoryFilter {
            search: Some("absent".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(none.summary.sessions, 0);
    let id = start_session_at(
        &c,
        StartInput {
            console_id: 1,
            controllers: 2,
        },
        at(23 * 3600),
    )
    .unwrap();
    settle(&c, id, 23 * 3600 + 60);
    assert_eq!(
        reports::get_history(&c, HistoryFilter::default())
            .unwrap()
            .days[0]
            .day,
        "2026-10-10"
    );
}
#[test]
fn csv_exports_every_filtered_row_and_escapes_formulas() {
    let c = fixture();
    update_console(
        &c,
        ConsoleInput {
            id: Some(1),
            name: "=1+1,\"quoted\"".into(),
            hourly_price: 3600,
            controller_price: 1800,
        },
    )
    .unwrap();
    settle(&c, start(&c, 2), 60);
    let csv = reports::export_history(&c, HistoryFilter::default()).unwrap();
    assert!(csv.starts_with('\u{feff}'));
    assert!(csv.contains("\"'=1+1,\"\"quoted\"\"\""));
}
#[test]
fn invalid_inputs_are_rejected_without_writes() {
    let c = fixture();
    assert!(add_console(
        &c,
        ConsoleInput {
            id: None,
            name: " ".into(),
            hourly_price: 1,
            controller_price: 1
        }
    )
    .is_err());
    assert!(save_product(
        &c,
        ProductInput {
            id: None,
            name: "Snack".into(),
            price: -1,
            stock: 1
        }
    )
    .is_err());
    assert!(save_product(
        &c,
        ProductInput {
            id: None,
            name: "Snack".into(),
            price: 1,
            stock: -1
        }
    )
    .is_err());
    assert!(start_session_at(
        &c,
        StartInput {
            console_id: 1,
            controllers: 5
        },
        at(0)
    )
    .is_err());
    let kind: String = c
        .query_row("SELECT typeof(hourlyPrice) FROM consoles", [], |r| r.get(0))
        .unwrap();
    assert_eq!(kind, "integer");
}
fn old_schema(c: &Connection) {
    c.execute_batch("CREATE TABLE consoles (id INTEGER PRIMARY KEY,name TEXT,hourlyPrice REAL,controllerPrice REAL);
        CREATE TABLE sessions (id INTEGER PRIMARY KEY,consoleId INTEGER,startTime TEXT,pauseTime TEXT,endTime TEXT,controllers INTEGER,status TEXT,shopItems TEXT,totalPrice REAL);
        INSERT INTO consoles VALUES (1,'Legacy',100.5,20);
        INSERT INTO sessions VALUES (1,1,'2026-10-09T00:00:00Z',NULL,'2026-10-09T01:00:00Z',2,'ended','[{\"name\":\"Snack\",\"price\":50}]',150.5);
        INSERT INTO sessions VALUES (2,1,'2026-10-09T02:00:00Z',NULL,NULL,2,'active','[]',NULL);").unwrap();
}
#[test]
fn legacy_migration_preserves_totals_snapshots_and_is_idempotent() {
    let c = Connection::open_in_memory().unwrap();
    old_schema(&c);
    schema::init(&c).unwrap();
    schema::init(&c).unwrap();
    let old = invoice_at(&c, 1, at(9999)).unwrap();
    assert_eq!(old.total, 151);
    assert_eq!(old.shop_cost, 50);
    assert_eq!(old.game_cost, 101);
    assert!(old.session.legacy);
    assert_eq!(old.items.len(), 1);
    assert_eq!(get_session(&c, 2).unwrap().hourly_price, 101);
    let violations: i64 = c
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(violations, 0);
}
#[test]
fn legacy_duplicate_sessions_survive_but_new_duplicates_are_blocked() {
    let c = Connection::open_in_memory().unwrap();
    old_schema(&c);
    c.execute(
        "INSERT INTO sessions VALUES (3,1,'2026-10-09T02:00:00Z',NULL,NULL,2,'active','[]',NULL)",
        [],
    )
    .unwrap();
    schema::init(&c).unwrap();
    assert_eq!(dashboard(&c).unwrap()[0].sessions.len(), 2);
    assert!(start_session_at(
        &c,
        StartInput {
            console_id: 1,
            controllers: 2
        },
        at(9000)
    )
    .is_err());
}
#[test]
fn failed_migration_rolls_back_old_schema() {
    let c = Connection::open_in_memory().unwrap();
    old_schema(&c);
    c.execute("UPDATE sessions SET shopItems='bad json' WHERE id=1", [])
        .unwrap();
    assert!(schema::init(&c).is_err());
    let version: i64 = c
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 0);
    let count: i64 = c
        .query_row("SELECT count(*) FROM sessions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 2);
    let name: String = c
        .query_row("SELECT name FROM consoles WHERE id=1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(name, "Legacy");
}
#[test]
fn deleted_legacy_console_gets_archived_identity() {
    let c = Connection::open_in_memory().unwrap();
    old_schema(&c);
    c.execute("DELETE FROM consoles WHERE id=1", []).unwrap();
    schema::init(&c).unwrap();
    assert!(get_console(&c, 1).unwrap().archived);
    assert_eq!(invoice_at(&c, 1, at(9999)).unwrap().total, 151);
}
#[test]
fn wal_migration_copies_without_changing_source_and_keeps_pending_checkout() {
    let dir = std::env::temp_dir().join(format!(
        "gamagement-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let old = dir.join("old.db");
    let new = dir.join("new.db");
    let original = Connection::open(&old).unwrap();
    original.pragma_update(None, "journal_mode", "WAL").unwrap();
    old_schema(&original);
    let migrated = open_database(&new, Some(&old)).unwrap();
    let shown = prepare_checkout_at(&migrated, 2, at(3 * 3600)).unwrap();
    drop(migrated);
    let reopened = open_database(&new, Some(&old)).unwrap();
    assert_eq!(get_session(&reopened, 2).unwrap().status, "checkout");
    assert_eq!(
        confirm_checkout_at(&reopened, 2, at(4 * 3600))
            .unwrap()
            .total,
        shown.total
    );
    let version: i64 = original
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 0);
    drop(original);
    drop(reopened);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn existing_database_upgrade_creates_safety_copy() {
    let dir = std::env::temp_dir().join(format!(
        "gamagement-upgrade-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("gamagement.db");
    let old = Connection::open(&path).unwrap();
    old_schema(&old);
    drop(old);
    let upgraded = open_database(&path, None).unwrap();
    let copy = Connection::open(path.with_extension("pre-v2.db")).unwrap();
    let version: i64 = copy
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 0);
    assert_eq!(invoice_at(&upgraded, 1, at(9999)).unwrap().total, 151);
    drop(copy);
    drop(upgraded);
    std::fs::remove_dir_all(dir).unwrap();
}
