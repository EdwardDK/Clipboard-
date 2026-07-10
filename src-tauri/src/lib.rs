mod clipboard;
mod commands;
mod database;
mod search;
mod security;
mod shortcuts;
mod tray;

use std::sync::{atomic::AtomicBool, Mutex};
use tauri::WindowEvent;

pub struct AppState {
    pub database: Mutex<database::Database>,
    pub monitoring_paused: AtomicBool,
    pub previous_window: Mutex<isize>,
    pub previous_focus: Mutex<isize>,
}

pub fn run() {
    let database = match database::Database::open() {
        Ok(database) => database,
        Err(error) => {
            eprintln!("Clipboard+ could not initialize its local database: {error}");
            return;
        }
    };
    let state = AppState {
        database: Mutex::new(database),
        monitoring_paused: AtomicBool::new(false),
        previous_window: Mutex::new(0),
        previous_focus: Mutex::new(0),
    };

    let app = tauri::Builder::default()
        .manage(state)
        .plugin(shortcuts::plugin())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            tray::install(app.handle())?;
            shortcuts::register_default(app.handle())?;
            clipboard::start_monitor(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_clipboard_items,
            commands::copy_clipboard_item,
            commands::paste_clipboard_item,
            commands::delete_clipboard_item,
            commands::clear_clipboard_history,
            commands::set_item_pinned,
            commands::reorder_pinned_items,
            commands::assign_item_group,
            commands::update_clipboard_item,
            commands::list_groups,
            commands::create_group,
            commands::get_retention_settings,
            commands::update_retention_settings,
            commands::set_monitoring_paused,
            commands::is_monitoring_paused,
        ])
        .build(tauri::generate_context!());
    match app {
        Ok(app) => app.run(|_, _| {}),
        Err(error) => eprintln!("Clipboard+ could not start: {error}"),
    }
}
