use tauri::{AppHandle, Emitter, LogicalSize, Manager, Size};
use tauri_plugin_global_shortcut::{Builder, GlobalShortcutExt, ShortcutState};

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                show_compact_palette(app);
            }
        })
        .build()
}

pub fn register_default(app: &AppHandle) -> Result<(), tauri_plugin_global_shortcut::Error> {
    app.global_shortcut().register("Ctrl+Shift+V")
}

pub fn show_compact_palette(app: &AppHandle) {
    unsafe {
        use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::{GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, GUITHREADINFO}};
        let previous = GetForegroundWindow();
        if let Ok(mut slot) = app.state::<crate::AppState>().previous_window.lock() { *slot = previous as isize; }
        let mut info: GUITHREADINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;
        let thread = GetWindowThreadProcessId(previous as HWND, std::ptr::null_mut());
        if GetGUIThreadInfo(thread, &mut info) != 0 {
            if let Ok(mut slot) = app.state::<crate::AppState>().previous_focus.lock() { *slot = info.hwndFocus as isize; }
        }
    }
    show(app, 620.0, 520.0, true);
}

pub fn show_full_workspace(app: &AppHandle) {
    show(app, 980.0, 650.0, false);
}

fn show(app: &AppHandle, width: f64, height: f64, compact: bool) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_size(Size::Logical(LogicalSize::new(width, height)));
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("palette-opened", compact);
    }
}
