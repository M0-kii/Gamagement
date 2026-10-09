#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod models;
mod reports;
mod schema;
#[cfg(test)]
mod tests;

use database::Database;
use models::*;
use tauri::{Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command(async)]
fn add_console(db: State<'_, Database>, data: ConsoleInput) -> AppResult<i64> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::add_console(&conn, data)
}

#[tauri::command(async)]
fn update_console(db: State<'_, Database>, data: ConsoleInput) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_console(&conn, data)
}

#[tauri::command(async)]
fn archive_console(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::archive_console(&conn, id)
}

#[tauri::command(async)]
fn get_consoles(
    db: State<'_, Database>,
    include_archived: Option<bool>,
) -> AppResult<Vec<Console>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_consoles(&conn, include_archived.unwrap_or(false))
}

#[tauri::command(async)]
fn get_dashboard(db: State<'_, Database>) -> AppResult<Vec<ConsoleCard>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::dashboard(&conn)
}

#[tauri::command(async)]
fn start_session(db: State<'_, Database>, data: StartInput) -> AppResult<i64> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::start_session_at(&conn, data, chrono::Utc::now())
}

#[tauri::command(async)]
fn pause_session(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::pause_session_at(&conn, id, chrono::Utc::now())
}

#[tauri::command(async)]
fn resume_session(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::resume_session_at(&conn, id, chrono::Utc::now())
}

#[tauri::command(async)]
fn get_invoice(db: State<'_, Database>, id: i64) -> AppResult<Invoice> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::invoice_at(&conn, id, chrono::Utc::now())
}

#[tauri::command(async)]
fn prepare_checkout(db: State<'_, Database>, id: i64) -> AppResult<Invoice> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::prepare_checkout_at(&conn, id, chrono::Utc::now())
}

#[tauri::command(async)]
fn cancel_checkout(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::cancel_checkout_at(&conn, id, chrono::Utc::now())
}

#[tauri::command(async)]
fn confirm_checkout(db: State<'_, Database>, id: i64) -> AppResult<Invoice> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::confirm_checkout_at(&conn, id, chrono::Utc::now())
}

#[tauri::command(async)]
fn get_products(db: State<'_, Database>) -> AppResult<Vec<Product>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_products(&conn)
}

#[tauri::command(async)]
fn save_product(db: State<'_, Database>, data: ProductInput) -> AppResult<i64> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::save_product(&conn, data)
}

#[tauri::command(async)]
fn archive_product(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::archive_product(&conn, id)
}

#[tauri::command(async)]
fn add_product_to_session(db: State<'_, Database>, data: AddProductInput) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::add_product_to_session(&conn, data)
}

#[tauri::command(async)]
fn add_session_item(db: State<'_, Database>, data: SessionItemInput) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::add_session_item(&conn, data)
}

#[tauri::command(async)]
fn update_item_quantity(db: State<'_, Database>, data: ItemQuantityInput) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_item_quantity(&conn, data)
}

#[tauri::command(async)]
fn delete_session_item(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_session_item(&conn, id)
}

#[tauri::command(async)]
fn clear_history(db: State<'_, Database>) -> AppResult<usize> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::clear_history(&conn)
}

#[tauri::command(async)]
fn get_history(db: State<'_, Database>, filter: HistoryFilter) -> AppResult<HistoryPage> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    reports::get_history(&conn, filter)
}

#[tauri::command(async)]
fn update_session_controllers(db: State<'_, Database>, data: ControllersInput) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_session_controllers_at(&conn, data, chrono::Utc::now())
}

#[tauri::command]
async fn export_history(
    app: tauri::AppHandle,
    db: State<'_, Database>,
    filter: HistoryFilter,
) -> AppResult<bool> {
    let csv = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        reports::export_history(&conn, filter)?
    };
    tauri::async_runtime::spawn_blocking(move || {
        let file = app
            .dialog()
            .file()
            .add_filter("CSV", &["csv"])
            .set_file_name("gamagement-sessions.csv")
            .blocking_save_file();
        match file {
            Some(file) => {
                let path = file.into_path().map_err(|e| e.to_string())?;
                std::fs::write(path, csv).map_err(|e| e.to_string())?;
                Ok(true)
            }
            None => Ok(false),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
fn minimize_window(window: tauri::Window) -> AppResult<()> {
    window.minimize().map_err(database::err)
}
#[tauri::command]
fn is_window_maximized(window: tauri::Window) -> AppResult<bool> {
    window.is_maximized().map_err(database::err)
}
#[tauri::command]
fn toggle_maximize_window(window: tauri::Window) -> AppResult<bool> {
    if window.is_maximized().map_err(database::err)? {
        window.unmaximize().map_err(database::err)?;
        Ok(false)
    } else {
        window.maximize().map_err(database::err)?;
        Ok(true)
    }
}
#[tauri::command]
fn start_window_dragging(window: tauri::Window) -> AppResult<()> {
    window.start_dragging().map_err(database::err)
}
#[tauri::command]
fn close_window(window: tauri::Window) -> AppResult<()> {
    window.destroy().map_err(database::err)
}

fn legacy_database_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(std::path::PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|p| std::path::PathBuf::from(p).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| std::path::PathBuf::from(p).join(".config")));
    base.map(|p| p.join("dev.m0-kii.gamagement").join("gamagement.db"))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let directory = app.path().app_local_data_dir()?.join("data");
            std::fs::create_dir_all(&directory)?;
            let legacy = legacy_database_path();
            let db = database::open_database(&directory.join("gamagement.db"), legacy.as_deref())
                .map_err(std::io::Error::other)?;
            app.manage(Database(std::sync::Mutex::new(db)));
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.emit("window-close-requested", ());
            }
            tauri::WindowEvent::Resized(_) => {
                if let Ok(maximized) = window.is_maximized() {
                    let _ = window.emit("window-maximized-changed", maximized);
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            add_console,
            update_console,
            archive_console,
            get_consoles,
            get_dashboard,
            update_session_controllers,
            start_session,
            pause_session,
            resume_session,
            get_invoice,
            prepare_checkout,
            cancel_checkout,
            confirm_checkout,
            get_products,
            save_product,
            archive_product,
            add_product_to_session,
            add_session_item,
            update_item_quantity,
            delete_session_item,
            get_history,
            clear_history,
            export_history,
            minimize_window,
            toggle_maximize_window,
            is_window_maximized,
            start_window_dragging,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run Gamagement");
}
