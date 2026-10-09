#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;

use database::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

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
fn delete_console(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_console(&conn, id)
}

#[tauri::command(async)]
fn get_console(db: State<'_, Database>, id: i64) -> AppResult<Console> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_console(&conn, id)
}

#[tauri::command(async)]
fn get_consoles(db: State<'_, Database>) -> AppResult<Vec<Console>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_consoles(&conn)
}

#[tauri::command(async)]
fn start_session(db: State<'_, Database>, data: StartInput) -> AppResult<i64> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::start_session(&conn, data)
}

#[tauri::command(async)]
fn pause_session(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::pause_session(&conn, id)
}

#[tauri::command(async)]
fn resume_session(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::resume_session_at(&conn, id, chrono::Utc::now())
}

#[tauri::command(async)]
fn end_session(db: State<'_, Database>, id: i64) -> AppResult<Invoice> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::end_session(&conn, id)
}

#[tauri::command(async)]
fn get_session(db: State<'_, Database>, id: i64) -> AppResult<Session> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_session(&conn, id)
}

#[tauri::command(async)]
fn get_sessions_for_console(db: State<'_, Database>, console_id: i64) -> AppResult<Vec<Session>> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_sessions_for_console(&conn, console_id)
}

#[tauri::command(async)]
fn add_shop_item_to_session(db: State<'_, Database>, data: ShopInput) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::save_shop_item(&conn, data, false)
}

#[tauri::command(async)]
fn update_shop_item(db: State<'_, Database>, data: ShopInput) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::save_shop_item(&conn, data, true)
}

#[tauri::command(async)]
fn delete_shop_item(db: State<'_, Database>, data: DeleteItemInput) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_shop_item(&conn, data)
}

#[tauri::command(async)]
fn update_session_controllers(db: State<'_, Database>, data: ControllersInput) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_session_controllers(&conn, data)
}

#[tauri::command]
async fn ask_confirmation(app: tauri::AppHandle, message: String) -> AppResult<bool> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .title("تایید")
            .buttons(MessageDialogButtons::OkCancelCustom(
                "بله".into(),
                "خیر".into(),
            ))
            .blocking_show()
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn show_message(app: tauri::AppHandle, message: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .message(message)
            .title("Gamagement")
            .blocking_show();
    })
    .await
    .map_err(|e| e.to_string())
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
            // Electron stores userData under APPDATA; Tauri uses local app data.
            let directory = app.path().app_local_data_dir()?.join("data");
            std::fs::create_dir_all(&directory)?;
            let legacy = legacy_database_path();
            let db = open_database(&directory.join("gamagement.db"), legacy.as_deref())
                .map_err(std::io::Error::other)?;
            app.manage(Database(std::sync::Mutex::new(db)));
            Ok(())
        })
        .on_window_event({
            let asking = Arc::new(AtomicBool::new(false));
            move |window, event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if asking.swap(true, Ordering::SeqCst) {
                        return;
                    }
                    let asking = asking.clone();
                    let window = window.clone();
                    window
                        .app_handle()
                        .dialog()
                        .message("آیا مطمئن هستید که می‌خواهید برنامه را ببندید؟")
                        .title("تایید")
                        .buttons(MessageDialogButtons::OkCancelCustom(
                            "بله".into(),
                            "خیر".into(),
                        ))
                        .show(move |confirmed| {
                            asking.store(false, Ordering::SeqCst);
                            if confirmed {
                                let _ = window.destroy();
                            }
                        });
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            add_console,
            update_console,
            delete_console,
            get_console,
            get_consoles,
            start_session,
            pause_session,
            resume_session,
            end_session,
            get_session,
            get_sessions_for_console,
            add_shop_item_to_session,
            update_shop_item,
            delete_shop_item,
            update_session_controllers,
            ask_confirmation,
            show_message
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run Gamagement");
}
