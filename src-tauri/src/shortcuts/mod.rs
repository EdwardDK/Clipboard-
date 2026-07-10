use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Builder, GlobalShortcutExt, ShortcutState};

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                show_palette(app);
            }
        })
        .build()
}

pub fn register_default(app: &AppHandle) -> Result<(), tauri_plugin_global_shortcut::Error> {
    app.global_shortcut().register("Ctrl+Shift+V")
}

pub fn show_palette(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("palette-opened", ());
    }
}
