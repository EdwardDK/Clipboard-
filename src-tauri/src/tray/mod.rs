use std::sync::atomic::Ordering;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Clipboard+", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause monitoring", true, None::<&str>)?;
    let clear = MenuItem::with_id(app, "clear", "Clear history", true, None::<&str>)?;
    let settings = MenuItem::with_id(
        app,
        "settings",
        "Settings (Milestone 2)",
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &pause, &clear, &settings, &quit])?;
    TrayIconBuilder::with_id("clipboard-plus-tray")
        .menu(&menu)
        .tooltip("Clipboard+")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => crate::shortcuts::show_full_workspace(app),
            "pause" => {
                let state = app.state::<crate::AppState>();
                state.monitoring_paused.fetch_xor(true, Ordering::Relaxed);
            }
            "clear" => {
                if let Ok(mut database) = app.state::<crate::AppState>().database.lock() {
                    let _ = database.clear();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::shortcuts::show_full_workspace(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
