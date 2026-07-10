use arboard::Clipboard;
use std::{sync::atomic::Ordering, thread, time::Duration};
use tauri::{AppHandle, Emitter, Manager};
pub(crate) mod windows;

pub fn start_monitor(app: AppHandle) {
    thread::spawn(move || {
        let mut clipboard = Clipboard::new().ok();
        let mut previous = String::new();
        loop {
            thread::sleep(Duration::from_millis(650));
            if app
                .state::<crate::AppState>()
                .monitoring_paused
                .load(Ordering::Relaxed)
            {
                continue;
            }
            if clipboard.is_none() {
                clipboard = Clipboard::new().ok();
                continue;
            }
            let Some(handle) = clipboard.as_mut() else {
                continue;
            };
            let Ok(text) = handle.get_text() else {
                continue;
            };
            if text.is_empty() || text == previous {
                continue;
            }
            previous.clone_from(&text);
            let context = windows::foreground_context();
            let saved = app
                .state::<crate::AppState>()
                .database
                .lock()
                .ok()
                .and_then(|mut db| db.capture_text(&text, context).ok())
                .is_some();
            if saved {
                let _ = app.emit("clipboard-history-changed", ());
            }
        }
    });
}

pub fn write_text(content: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new()
        .map_err(|_| "The Windows clipboard is temporarily unavailable.".to_string())?;
    clipboard
        .set_text(content.to_owned())
        .map_err(|_| "The Windows clipboard is temporarily unavailable.".to_string())
}
